// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// pyo3 generates errors with _obj and _tmp values
#![allow(clippy::used_underscore_binding)]
// Some arguments get turned into Deref by PyO3 macros, which we can't control.
#![allow(clippy::borrow_deref_ref, clippy::needless_option_as_deref)]
// This was introduced in 1.62, but we can't update the dependency to
// to resolve it until we move to a newer version of python.
#![allow(clippy::format_push_string)]

use pyo3::{
    exceptions::{PyOSError, PyTypeError, PyUnicodeDecodeError, PyValueError},
    prelude::*,
    types::{PyBytes, PySequence, PyString, PyUnicode},
    PyClass,
};
use qirlib::{
    builder::{BuilderExt, ModuleBuilder},
    inkwell::{
        builder::Builder as InkwellBuilder,
        context::Context as InkwellContext,
        module::Module as InkwellModule,
        types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum, FunctionType},
        values::{AnyValue, AnyValueEnum, BasicMetadataValueEnum, CallableValue},
        IntPredicate,
    },
    module,
    qis::BuilderBasicExt,
    types,
};
use std::{
    borrow::Borrow,
    convert::{Into, TryFrom},
    mem::transmute,
};

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<BasicQisBuilder>()?;
    m.add_class::<Builder>()?;
    m.add_class::<SimpleModule>()?;
    m.add_class::<Type>()?;
    m.add_class::<TypeFactory>()?;
    m.add_class::<Value>()?;
    m.add_function(wrap_pyfunction!(bitcode_to_ir, m)?)?;
    m.add("const", wrap_pyfunction!(constant, m)?)?;
    m.add_function(wrap_pyfunction!(ir_to_bitcode, m)?)?;
    Ok(())
}

struct PyIntPredicate(IntPredicate);

impl<'source> FromPyObject<'source> for PyIntPredicate {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        match ob.getattr("name")?.extract()? {
            "EQ" => Ok(Self(IntPredicate::EQ)),
            "NE" => Ok(Self(IntPredicate::NE)),
            "UGT" => Ok(Self(IntPredicate::UGT)),
            "UGE" => Ok(Self(IntPredicate::UGE)),
            "ULT" => Ok(Self(IntPredicate::ULT)),
            "ULE" => Ok(Self(IntPredicate::ULE)),
            "SGT" => Ok(Self(IntPredicate::SGT)),
            "SGE" => Ok(Self(IntPredicate::SGE)),
            "SLT" => Ok(Self(IntPredicate::SLT)),
            "SLE" => Ok(Self(IntPredicate::SLE)),
            _ => Err(PyValueError::new_err("Invalid integer predicate.")),
        }
    }
}

#[pyclass]
#[derive(Eq, PartialEq)]
struct Context(InkwellContext);

#[pyclass(unsendable)]
struct Type {
    ty: AnyTypeEnum<'static>,
    context: Py<Context>,
}

#[pyclass(unsendable)]
struct Module {
    module: InkwellModule<'static>,
    context: Py<Context>,
}

impl Module {
    fn new(py: Python, context: Py<Context>, name: &str) -> Module {
        let module = {
            let context = context.borrow(py);
            let module = context.0.create_module(name);
            unsafe { transmute::<InkwellModule<'_>, InkwellModule<'static>>(module) }
        };
        Module { module, context }
    }
}

#[pyclass]
struct TypeFactory {
    module: Py<Module>,
}

#[pymethods]
impl TypeFactory {
    #[getter]
    fn void(&self, py: Python) -> PyResult<Py<Type>> {
        self.create_type(py, |m| m.get_context().void_type().into())
    }

    #[getter]
    fn bool(&self, py: Python) -> PyResult<Py<Type>> {
        self.create_type(py, |m| m.get_context().bool_type().into())
    }

    fn int(&self, py: Python, width: u32) -> PyResult<Py<Type>> {
        self.create_type(py, |m| m.get_context().custom_width_int_type(width).into())
    }

    #[getter]
    fn double(&self, py: Python) -> PyResult<Py<Type>> {
        self.create_type(py, |m| m.get_context().f64_type().into())
    }

    #[getter]
    fn qubit(&self, py: Python) -> PyResult<Py<Type>> {
        self.create_type(py, |m| types::qubit(m).into())
    }

    #[getter]
    fn result(&self, py: Python) -> PyResult<Py<Type>> {
        self.create_type(py, |m| types::result(m).into())
    }

    #[staticmethod]
    #[allow(clippy::needless_pass_by_value)]
    fn function(py: Python, return_: &Type, params: Vec<Py<Type>>) -> PyResult<Py<Type>> {
        require_same_contexts(
            py,
            params
                .iter()
                .map(|t| t.borrow(py).context.clone())
                .chain([return_.context.clone()]),
        )?;

        let ty = function_type(&return_.ty, params.iter().map(|t| t.borrow(py).ty))
            .ok_or_else(|| PyValueError::new_err("Invalid return or parameter type."))?
            .into();

        let ty = unsafe { transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty) };
        let context = return_.context.clone();
        Py::new(py, Type { ty, context })
    }
}

impl TypeFactory {
    fn create_type(
        &self,
        py: Python,
        f: impl for<'ctx> Fn(&InkwellModule<'ctx>) -> AnyTypeEnum<'ctx>,
    ) -> PyResult<Py<Type>> {
        let module = self.module.borrow(py);
        let context = module.context.clone();
        let ty = {
            let ty = f(&module.module);
            unsafe { transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty) }
        };
        Py::new(py, Type { ty, context })
    }
}

/// A QIR value.
#[pyclass(unsendable)]
#[derive(Clone)]
struct Value {
    value: AnyValueEnum<'static>,
    context: Py<Context>,
}

impl Value {
    unsafe fn new<'ctx>(context: Py<Context>, value: &impl AnyValue<'ctx>) -> Self {
        let value = value.as_any_value_enum();
        let value = transmute::<AnyValueEnum<'_>, AnyValueEnum<'static>>(value);
        Self { value, context }
    }
}

/// Creates a constant QIR value.
///
/// :param Type ty: The type of the value.
/// :param Union[int, float] value: A Python value that will be converted into a QIR value.
/// :returns: The constant QIR value.
/// :rtype: Value
#[pyfunction]
#[pyo3(text_signature = "(ty, value)")]
fn constant(ty: &Type, value: &PyAny) -> PyResult<Value> {
    let context = ty.context.clone();
    let value = extract_constant(&ty.ty, value)?;
    Ok(unsafe { Value::new(context, &value) })
}

/// An instruction builder.
#[pyclass(unsendable)]
struct Builder {
    builder: InkwellBuilder<'static>,
    context: Py<Context>,
    // TODO: In principle, the module could be extracted from the builder.
    // See https://github.com/TheDan64/inkwell/issues/347.
    module: Py<Module>,
}

impl Builder {
    fn new(py: Python, module: Py<Module>) -> Self {
        let context = module.borrow(py).context.clone();
        let builder = {
            let context = context.borrow(py);
            let builder = context.0.create_builder();
            unsafe { transmute::<InkwellBuilder<'_>, InkwellBuilder<'static>>(builder) }
        };

        Self {
            builder,
            context,
            module,
        }
    }
}

#[pymethods]
impl Builder {
    /// Inserts a bitwise logical and instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn and_(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<Value> {
        require_same_contexts(py, [&self.context, &lhs.context, &rhs.context])?;
        let value =
            self.builder
                .build_and(lhs.value.into_int_value(), rhs.value.into_int_value(), "");
        Ok(unsafe { Value::new(self.context.clone(), &value) })
    }

    /// Inserts a bitwise logical or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn or_(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<Value> {
        require_same_contexts(py, [&self.context, &lhs.context, &rhs.context])?;
        let value =
            self.builder
                .build_or(lhs.value.into_int_value(), rhs.value.into_int_value(), "");
        Ok(unsafe { Value::new(self.context.clone(), &value) })
    }

    /// Inserts a bitwise logical exclusive or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn xor(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<Value> {
        require_same_contexts(py, [&self.context, &lhs.context, &rhs.context])?;
        let value =
            self.builder
                .build_xor(lhs.value.into_int_value(), rhs.value.into_int_value(), "");
        Ok(unsafe { Value::new(self.context.clone(), &value) })
    }

    /// Inserts an addition instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The sum.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn add(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<Value> {
        require_same_contexts(py, [&self.context, &lhs.context, &rhs.context])?;
        let value =
            self.builder
                .build_int_add(lhs.value.into_int_value(), rhs.value.into_int_value(), "");
        Ok(unsafe { Value::new(self.context.clone(), &value) })
    }

    /// Inserts a subtraction instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The difference.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn sub(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<Value> {
        require_same_contexts(py, [&self.context, &lhs.context, &rhs.context])?;
        let value =
            self.builder
                .build_int_sub(lhs.value.into_int_value(), rhs.value.into_int_value(), "");
        Ok(unsafe { Value::new(self.context.clone(), &value) })
    }

    /// Inserts a multiplication instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The product.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn mul(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<Value> {
        require_same_contexts(py, [&self.context, &lhs.context, &rhs.context])?;
        let value =
            self.builder
                .build_int_mul(lhs.value.into_int_value(), rhs.value.into_int_value(), "");
        Ok(unsafe { Value::new(self.context.clone(), &value) })
    }

    /// Inserts a shift left instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn shl(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<Value> {
        require_same_contexts(py, [&self.context, &lhs.context, &rhs.context])?;
        let value = self.builder.build_left_shift(
            lhs.value.into_int_value(),
            rhs.value.into_int_value(),
            "",
        );
        Ok(unsafe { Value::new(self.context.clone(), &value) })
    }

    /// Inserts a logical (zero fill) shift right instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn lshr(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<Value> {
        require_same_contexts(py, [&self.context, &lhs.context, &rhs.context])?;
        let value = self.builder.build_right_shift(
            lhs.value.into_int_value(),
            rhs.value.into_int_value(),
            false,
            "",
        );
        Ok(unsafe { Value::new(self.context.clone(), &value) })
    }

    /// Inserts an integer comparison instruction.
    ///
    /// :param IntPredicate pred: The predicate to compare by.
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :return: The boolean result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, pred, lhs, rhs)")]
    #[allow(clippy::needless_pass_by_value)]
    fn icmp(&self, py: Python, pred: PyIntPredicate, lhs: Value, rhs: Value) -> PyResult<Value> {
        require_same_contexts(py, [&self.context, &lhs.context, &rhs.context])?;
        let value = self.builder.build_int_compare(
            pred.0,
            lhs.value.into_int_value(),
            rhs.value.into_int_value(),
            "",
        );
        Ok(unsafe { Value::new(self.context.clone(), &value) })
    }

    /// Inserts a call instruction.
    ///
    /// :param Function function: The function to call.
    /// :param Sequence[Union[Value, bool, int, float]] args: The arguments to the function.
    /// :returns: The return value, or None if the function has a void return type.
    /// :rtype: Optional[Value]
    #[pyo3(text_signature = "(self, function, args)")]
    fn call(&self, py: Python, function: &Value, args: &PySequence) -> PyResult<Option<Value>> {
        require_same_contexts(
            py,
            value_contexts(args.iter()?).chain([self.context.clone(), function.context.clone()]),
        )?;

        let (callable, param_types) = try_callable_value(function.value)
            .ok_or_else(|| PyValueError::new_err("Value is not callable."))?;

        if param_types.len() != args.len()? {
            return Err(PyValueError::new_err(format!(
                "Expected {} arguments, got {}.",
                param_types.len(),
                args.len()?
            )));
        }

        let args = extract_values(param_types, args)?
            .iter()
            .map(|v| any_to_meta(*v).ok_or_else(|| PyValueError::new_err("Invalid argument.")))
            .collect::<PyResult<Vec<_>>>()?;

        let call = self.builder.build_call(callable, &args, "");
        let value = call.try_as_basic_value().left();
        Ok(value.map(|v| unsafe { Value::new(function.context.clone(), &v) }))
    }

    /// Inserts a branch conditioned on a boolean.
    ///
    /// Instructions inserted when ``true`` is called will be inserted into the true branch.
    /// Instructions inserted when ``false`` is called will be inserted into the false branch. The
    /// true and false callables should use this module's builder to build instructions.
    ///
    /// :param Value cond: The boolean condition to branch on.
    /// :param Callable[[], None] true:
    ///     A callable that inserts instructions for the branch where the condition is true.
    /// :param Callable[[], None] false:
    ///     A callable that inserts instructions for the branch where the condition is false.
    #[pyo3(text_signature = "(self, cond, true, false)")]
    fn if_(
        &self,
        py: Python,
        cond: &Value,
        r#true: Option<&PyAny>,
        r#false: Option<&PyAny>,
    ) -> PyResult<()> {
        require_same_contexts(py, [&self.context, &cond.context])?;
        self.builder.try_build_if(
            cond.value.into_int_value(),
            || call_if_some(r#true),
            || call_if_some(r#false),
        )
    }
}

/// A simple module represents an executable QIR program with these restrictions:
///
/// - There is one global qubit register and one global result register. Both are statically
///   allocated with a fixed size.
/// - There is only a single function that runs as the entry point.
///
/// :param str name: The name of the module.
/// :param int num_qubits: The number of statically allocated qubits.
/// :param int num_results: The number of statically allocated results.
#[pyclass(unsendable)]
#[pyo3(text_signature = "(name, num_qubits, num_results)")]
struct SimpleModule {
    module: Py<Module>,
    builder: Py<Builder>,
    types: Py<TypeFactory>,
    num_qubits: u64,
    num_results: u64,
}

#[pymethods]
impl SimpleModule {
    #[new]
    fn new(py: Python, name: &str, num_qubits: u64, num_results: u64) -> PyResult<SimpleModule> {
        let context = Py::new(py, Context(InkwellContext::create()))?;
        let module = Py::new(py, Module::new(py, context, name))?;
        let builder = Py::new(py, Builder::new(py, module.clone()))?;

        {
            let builder = builder.borrow(py);
            let module = module.borrow(py);
            module::simple_init(&module.module, &builder.builder, num_qubits, num_results);
        }

        let types = Py::new(
            py,
            TypeFactory {
                module: module.clone(),
            },
        )?;

        Ok(SimpleModule {
            module,
            builder,
            types,
            num_qubits,
            num_results,
        })
    }

    #[getter]
    fn types(&self) -> Py<TypeFactory> {
        self.types.clone()
    }

    /// The global qubit register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn qubits(&self, py: Python) -> Vec<Value> {
        let builder = self.builder.borrow(py);
        let module = self.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        (0..self.num_qubits)
            .map(|id| unsafe { Value::new(module.context.clone(), &builder.build_qubit(id)) })
            .collect()
    }

    /// The global result register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn results(&self, py: Python) -> Vec<Value> {
        let builder = self.builder.borrow(py);
        let module = self.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        (0..self.num_results)
            .map(|id| unsafe { Value::new(module.context.clone(), &builder.build_result(id)) })
            .collect()
    }

    /// The instruction builder.
    ///
    /// :type: Builder
    #[getter]
    fn builder(&self) -> Py<Builder> {
        self.builder.clone()
    }

    /// Emits the LLVM IR for the module as plain text.
    ///
    /// :rtype: str
    fn ir(&self, py: Python) -> PyResult<String> {
        self.emit(py, |m| m.print_to_string().to_string())
    }

    /// Emits the LLVM bitcode for the module as a sequence of bytes.
    ///
    /// :rtype: bytes
    fn bitcode<'py>(&self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        self.emit(py, |m| {
            PyBytes::new(py, m.write_bitcode_to_memory().as_slice())
        })
    }

    /// Adds a declaration for an externally linked function to the module.
    ///
    /// :param str name: The name of the function.
    /// :param Type ty: The type of the function.
    /// :return: The function value.
    /// :rtype: Function
    #[pyo3(text_signature = "(self, name, ty)")]
    fn add_external_function(&mut self, py: Python, name: &str, ty: &Type) -> PyResult<Value> {
        let module = self.module.borrow(py);
        require_same_contexts(py, [&module.context, &ty.context])?;

        let context = ty.context.clone();
        let ty = ty.ty.into_function_type();
        let function = module.module.add_function(name, ty, None);
        Ok(unsafe { Value::new(context, &function) })
    }
}

impl SimpleModule {
    fn emit<T>(&self, py: Python, f: impl Fn(&InkwellModule) -> T) -> PyResult<T> {
        let module = self.module.borrow(py);
        let builder = self.builder.borrow(py);
        let ret = builder.builder.build_return(None);
        let new_context = InkwellContext::create();
        let new_module = clone_module(&module.module, &new_context)?;
        ret.erase_from_basic_block();
        module::simple_finalize(&new_module).map_err(PyOSError::new_err)?;
        Ok(f(&new_module))
    }
}

/// An instruction builder that generates instructions from the basic quantum instruction set.
///
/// :param Builder builder: The underlying builder used to build QIS instructions.
#[pyclass]
#[pyo3(text_signature = "(builder)")]
struct BasicQisBuilder {
    builder: Py<Builder>,
}

#[pymethods]
impl BasicQisBuilder {
    #[new]
    fn new(builder: Py<Builder>) -> Self {
        BasicQisBuilder { builder }
    }

    /// Inserts a controlled Pauli :math:`X` gate.
    ///
    /// :param Value control: The control qubit.
    /// :param Value target: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, control, target)")]
    fn cx(&self, py: Python, control: &Value, target: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &control.context, &target.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_cx(
            control.value.into_pointer_value(),
            target.value.into_pointer_value(),
        );
        Ok(())
    }

    /// Inserts a controlled Pauli :math:`Z` gate.
    ///
    /// :param Value control: The control qubit.
    /// :param Value target: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, control, target)")]
    fn cz(&self, py: Python, control: &Value, target: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &control.context, &target.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_cz(
            control.value.into_pointer_value(),
            target.value.into_pointer_value(),
        );
        Ok(())
    }

    /// Inserts a Hadamard gate.
    ///
    /// :param qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn h(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_h(qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts a Z-basis measurement operation.
    ///
    /// :param Value qubit: The qubit to measure.
    /// :param Value result: A result where the measurement result will be written to.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit, result)")]
    fn mz(&self, py: Python, qubit: &Value, result: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context, &result.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_mz(
            qubit.value.into_pointer_value(),
            result.value.into_pointer_value(),
        );
        Ok(())
    }

    /// Inserts a reset operation.
    ///
    /// :param Value qubit: The qubit to reset.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn reset(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_reset(qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`x` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rx(&self, py: Python, theta: &PyAny, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(
            py,
            value_contexts([Ok(theta)]).chain([builder.context.clone(), qubit.context.clone()]),
        )?;

        let context = builder.context.borrow(py);
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        let theta = extract_value(&context.0.f64_type(), theta)?;
        builder.build_rx(theta.into_float_value(), qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`y` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn ry(&self, py: Python, theta: &PyAny, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(
            py,
            value_contexts([Ok(theta)]).chain([builder.context.clone(), qubit.context.clone()]),
        )?;

        let context = builder.context.borrow(py);
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        let theta = extract_value(&context.0.f64_type(), theta)?;
        builder.build_ry(theta.into_float_value(), qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`z` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rz(&self, py: Python, theta: &PyAny, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(
            py,
            value_contexts([Ok(theta)]).chain([builder.context.clone(), qubit.context.clone()]),
        )?;

        let context = builder.context.borrow(py);
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        let theta = extract_value(&context.0.f64_type(), theta)?;
        builder.build_rz(theta.into_float_value(), qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts an :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_s(qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts an adjoint :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s_adj(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_s_adj(qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts a :math:`T` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_t(qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts an adjoint :math:`T` gate.
    ///
    /// :param qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t_adj(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_t_adj(qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts a Pauli :math:`X` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn x(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_x(qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts a Pauli :math:`Y` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn y(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_y(qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts a Pauli :math:`Z` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn z(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &qubit.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.build_z(qubit.value.into_pointer_value());
        Ok(())
    }

    /// Inserts a branch conditioned on a measurement result.
    ///
    /// Instructions inserted when ``one`` is called will be inserted into the one branch.
    /// Instructions inserted when ``zero`` is called will be inserted into the zero branch. The one
    /// and zero callables should use this module's builder to build instructions.
    ///
    /// :param Value cond: The result condition to branch on.
    /// :param Callable[[], None] one:
    ///     A callable that inserts instructions for the branch where the result is one.
    /// :param Callable[[], None] zero:
    ///     A callable that inserts instructions for the branch where the result is zero.
    /// :rtype: None
    #[pyo3(text_signature = "(self, cond, one, zero)")]
    fn if_result(
        &self,
        py: Python,
        cond: &Value,
        one: Option<&PyAny>,
        zero: Option<&PyAny>,
    ) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        require_same_contexts(py, [&builder.context, &cond.context])?;
        let module = builder.module.borrow(py);
        let builder = ModuleBuilder::from(&builder.builder, &module.module);
        builder.try_build_if_result(
            cond.value.into_pointer_value(),
            || call_if_some(one),
            || call_if_some(zero),
        )
    }
}

/// Converts the supplied QIR string to its bitcode equivalent.
///
/// :param str ir: The QIR string to convert
/// :param Optional[str] module_name: The name of the QIR module, default is "" if None
/// :param Optional[str] source_file_name: The source file name of the QIR module. Unchanged if None
/// :return: The equivalent bitcode as bytes.
/// :rtype: bytes
#[pyfunction]
#[pyo3(text_signature = "(ir, module_name=None, source_file_name=None)")]
#[allow(clippy::needless_pass_by_value)]
fn ir_to_bitcode<'a>(
    py: Python<'a>,
    ir: &str,
    module_name: Option<&str>,
    source_file_name: Option<&str>,
) -> PyResult<&'a PyBytes> {
    let bitcode =
        module::ir_to_bitcode(ir, module_name, source_file_name).map_err(PyOSError::new_err)?;
    Ok(PyBytes::new(py, &bitcode))
}

/// Converts the supplied bitcode to its QIR string equivalent.
///
/// :param bytes ir: The bitcode bytes to convert
/// :param Optional[str] module_name: The name of the QIR module, default is "" if None
/// :param Optional[str] source_file_name: The source file name of the QIR module. Unchanged if None
/// :return: The equivalent QIR string.
/// :rtype: str
#[pyfunction]
#[pyo3(text_signature = "(bitcode, module_name=None, source_file_name=None)")]
#[allow(clippy::needless_pass_by_value)]
fn bitcode_to_ir<'a>(
    py: Python<'a>,
    bitcode: &PyBytes,
    module_name: Option<&str>,
    source_file_name: Option<&str>,
) -> PyResult<&'a PyString> {
    let ir = module::bitcode_to_ir(bitcode.as_bytes(), module_name, source_file_name)
        .map_err(PyOSError::new_err)?;
    Ok(PyUnicode::new(py, &ir))
}

fn extract_constant<'ctx>(ty: &impl AnyType<'ctx>, ob: &PyAny) -> PyResult<AnyValueEnum<'ctx>> {
    match ty.as_any_type_enum() {
        AnyTypeEnum::IntType(int) => Ok(int.const_int(ob.extract()?, true).into()),
        AnyTypeEnum::FloatType(float) => Ok(float.const_float(ob.extract()?).into()),
        _ => Err(PyTypeError::new_err(
            "Can't convert Python value into this type.",
        )),
    }
}

fn extract_value<'ctx>(ty: &impl AnyType<'ctx>, ob: &PyAny) -> PyResult<AnyValueEnum<'ctx>> {
    match ob.extract::<Value>() {
        Ok(value) => Ok(value.value),
        Err(_) => extract_constant(ty, ob),
    }
}

fn extract_values<'ctx>(
    types: impl IntoIterator<Item = impl AnyType<'ctx>>,
    values: &PySequence,
) -> PyResult<Vec<AnyValueEnum<'ctx>>> {
    values
        .iter()?
        .zip(types)
        .map(|(v, t)| extract_value(&t, v?))
        .collect::<PyResult<Vec<_>>>()
}

fn function_type<'ctx>(
    return_type: &impl AnyType<'ctx>,
    params: impl IntoIterator<Item = AnyTypeEnum<'ctx>>,
) -> Option<FunctionType<'ctx>> {
    let params = params
        .into_iter()
        .map(|ty| BasicTypeEnum::try_from(ty).map(Into::into).ok())
        .collect::<Option<Vec<_>>>()?;

    match return_type.as_any_type_enum() {
        AnyTypeEnum::VoidType(void) => Some(void.fn_type(&params, false)),
        any => BasicTypeEnum::try_from(any)
            .map(|basic| basic.fn_type(&params, false))
            .ok(),
    }
}

fn try_callable_value(value: AnyValueEnum) -> Option<(CallableValue, Vec<BasicTypeEnum>)> {
    match value {
        AnyValueEnum::FunctionValue(f) => {
            Some((CallableValue::from(f), f.get_type().get_param_types()))
        }
        AnyValueEnum::PointerValue(p) => match p.get_type().get_element_type() {
            AnyTypeEnum::FunctionType(ty) => {
                Some((CallableValue::try_from(p).unwrap(), ty.get_param_types()))
            }
            _ => None,
        },
        _ => None,
    }
}

fn any_to_meta(value: AnyValueEnum) -> Option<BasicMetadataValueEnum> {
    match value {
        AnyValueEnum::ArrayValue(a) => Some(BasicMetadataValueEnum::ArrayValue(a)),
        AnyValueEnum::IntValue(i) => Some(BasicMetadataValueEnum::IntValue(i)),
        AnyValueEnum::FloatValue(f) => Some(BasicMetadataValueEnum::FloatValue(f)),
        AnyValueEnum::PointerValue(p) => Some(BasicMetadataValueEnum::PointerValue(p)),
        AnyValueEnum::StructValue(s) => Some(BasicMetadataValueEnum::StructValue(s)),
        AnyValueEnum::VectorValue(v) => Some(BasicMetadataValueEnum::VectorValue(v)),
        AnyValueEnum::MetadataValue(m) => Some(BasicMetadataValueEnum::MetadataValue(m)),
        AnyValueEnum::PhiValue(_)
        | AnyValueEnum::FunctionValue(_)
        | AnyValueEnum::InstructionValue(_) => None,
    }
}

fn call_if_some(f: Option<&PyAny>) -> PyResult<()> {
    match f {
        Some(f) => f.call0().map(|_| ()),
        None => Ok(()),
    }
}

fn is_all_same<T>(py: Python, items: impl IntoIterator<Item = impl Borrow<Py<T>>>) -> bool
where
    T: Eq + PyClass,
{
    let mut items = items.into_iter();
    if let Some(mut prev) = items.next() {
        for item in items {
            if *item.borrow().borrow(py) != *prev.borrow().borrow(py) {
                return false;
            }
            prev = item;
        }
    };
    true
}

fn require_same_contexts(
    py: Python,
    contexts: impl IntoIterator<Item = impl Borrow<Py<Context>>>,
) -> PyResult<()> {
    // then_some is stabilized in Rust 1.62.
    #[allow(clippy::unnecessary_lazy_evaluations)]
    is_all_same(py, contexts)
        .then(|| ())
        .ok_or_else(|| PyValueError::new_err("Some objects come from a different context."))
}

fn value_contexts<'a>(
    obs: impl IntoIterator<Item = PyResult<&'a PyAny>> + 'a,
) -> impl Iterator<Item = Py<Context>> + 'a {
    obs.into_iter()
        .filter_map(|a| Some(a.ok()?.extract::<Value>().ok()?.context))
}

fn clone_module<'ctx>(
    module: &InkwellModule,
    context: &'ctx InkwellContext,
) -> PyResult<InkwellModule<'ctx>> {
    let bitcode = module.write_bitcode_to_memory();
    let new_module = InkwellModule::parse_bitcode_from_buffer(&bitcode, context).map_err(|e| {
        module.verify().err().map_or_else(
            || PyOSError::new_err(e.to_string()),
            |e| PyOSError::new_err(e.to_string()),
        )
    })?;
    let name = module
        .get_name()
        .to_str()
        .map_err(PyUnicodeDecodeError::new_err)?;
    new_module.set_name(name);
    Ok(new_module)
}
