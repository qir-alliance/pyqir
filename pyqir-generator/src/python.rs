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
    exceptions::{PyOSError, PyOverflowError, PyTypeError, PyValueError},
    prelude::*,
    types::{PyBytes, PySequence, PyString, PyUnicode},
    PyObjectProtocol,
};
use qirlib::inkwell::{
    self,
    module::Linkage,
    types::{AnyTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, VoidType},
    values::{AnyValueEnum, BasicMetadataValueEnum},
    AddressSpace,
};
use std::{convert::TryFrom, mem::transmute, ops::Deref};

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Type>()?;
    m.add_class::<Types>()?;
    m.add_class::<SimpleModule>()?;
    m.add_class::<Builder>()?;
    m.add_class::<BasicQisBuilder>()?;
    m.add_class::<Value>()?;
    m.add_function(wrap_pyfunction!(bitcode_to_ir, m)?)?;
    m.add_function(wrap_pyfunction!(ir_to_bitcode, m)?)?;
    m.add("const", wrap_pyfunction!(constant, m)?)?;
    Ok(())
}

#[pyclass]
struct Context(inkwell::context::Context);

impl Deref for Context {
    type Target = inkwell::context::Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[pyclass(unsendable)]
struct Type {
    ty: AnyTypeEnum<'static>,
    context: Py<Context>,
}

#[pyclass]
struct Types {
    module: Py<Module>,
}

#[pymethods]
impl Types {
    #[getter]
    fn void(&self, py: Python) -> PyResult<Py<Type>> {
        let module = self.module.borrow(py);
        let ty = {
            let context = module.context.borrow(py);
            let ty = context.void_type();
            let ty = unsafe { transmute::<VoidType, VoidType<'static>>(ty) };
            AnyTypeEnum::VoidType(ty)
        };
        let context = module.context.clone();
        Py::new(py, Type { ty, context })
    }

    #[getter]
    fn bool(&self, py: Python) -> PyResult<Py<Type>> {
        let module = self.module.borrow(py);
        let ty = {
            let context = module.context.borrow(py);
            let ty = context.bool_type();
            let ty = unsafe { transmute::<IntType, IntType<'static>>(ty) };
            AnyTypeEnum::IntType(ty)
        };
        let context = module.context.clone();
        Py::new(py, Type { ty, context })
    }

    fn integer(&self, py: Python, width: u32) -> PyResult<Py<Type>> {
        let module = self.module.borrow(py);
        let ty = {
            let context = module.context.borrow(py);
            let ty = context.custom_width_int_type(width);
            let ty = unsafe { transmute::<IntType, IntType<'static>>(ty) };
            AnyTypeEnum::IntType(ty)
        };
        let context = module.context.clone();
        Py::new(py, Type { ty, context })
    }

    #[getter]
    fn double(&self, py: Python) -> PyResult<Py<Type>> {
        let module = self.module.borrow(py);
        let ty = {
            let context = module.context.borrow(py);
            let ty = context.f64_type();
            let ty = unsafe { transmute::<FloatType, FloatType<'static>>(ty) };
            AnyTypeEnum::FloatType(ty)
        };
        let context = module.context.clone();
        Py::new(py, Type { ty, context })
    }

    fn function(&self, py: Python, return_: &Type, params: Vec<Py<Type>>) -> PyResult<Py<Type>> {
        let ty = {
            let params = params
                .iter()
                .map(|ty| {
                    let ty = ty.borrow(py);
                    let ty = BasicTypeEnum::try_from(ty.ty)
                        .map_err(|()| PyTypeError::new_err("Invalid parameter type."))?;
                    Ok(ty.into())
                })
                .collect::<PyResult<Vec<_>>>()?;

            let ty = match return_.ty {
                AnyTypeEnum::VoidType(void) => void.fn_type(&params, false),
                ret => BasicTypeEnum::try_from(ret)
                    .expect("Invalid return type.")
                    .fn_type(&params, false),
            };

            unsafe { transmute::<FunctionType, FunctionType<'static>>(ty) }
        };

        // TODO (safety): What if not all types use the same context?
        Py::new(
            py,
            Type {
                ty: AnyTypeEnum::FunctionType(ty),
                context: return_.context.clone(),
            },
        )
    }

    #[getter]
    fn qubit(&self, py: Python) -> PyResult<Py<Type>> {
        let module = self.module.borrow(py);
        let context = module.context.clone();
        let context_ref = module.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };
        let ty = AnyTypeEnum::PointerType(
            qirlib::codegen::types::qubit(&context_ref.0, module).ptr_type(AddressSpace::Generic),
        );
        let ty = unsafe { transmute::<AnyTypeEnum, AnyTypeEnum<'static>>(ty) };
        Py::new(py, Type { ty, context })
    }

    #[getter]
    fn result(&self, py: Python) -> PyResult<Py<Type>> {
        let module = self.module.borrow(py);
        let context = module.context.clone();
        let context_ref = module.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };
        let ty = AnyTypeEnum::PointerType(
            qirlib::codegen::types::result(&context_ref.0, module).ptr_type(AddressSpace::Generic),
        );
        let ty = unsafe { transmute::<AnyTypeEnum, AnyTypeEnum<'static>>(ty) };
        Py::new(py, Type { ty, context })
    }
}

struct PyIntPredicate(inkwell::IntPredicate);

impl<'source> FromPyObject<'source> for PyIntPredicate {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        match ob.getattr("name")?.extract()? {
            "EQ" => Ok(inkwell::IntPredicate::EQ),
            "NE" => Ok(inkwell::IntPredicate::NE),
            "UGT" => Ok(inkwell::IntPredicate::UGT),
            "UGE" => Ok(inkwell::IntPredicate::UGE),
            "ULT" => Ok(inkwell::IntPredicate::ULT),
            "ULE" => Ok(inkwell::IntPredicate::ULE),
            "SGT" => Ok(inkwell::IntPredicate::SGT),
            "SGE" => Ok(inkwell::IntPredicate::SGE),
            "SLT" => Ok(inkwell::IntPredicate::SLT),
            "SLE" => Ok(inkwell::IntPredicate::SLE),
            _ => Err(PyValueError::new_err("Invalid predicate.")),
        }
        .map(Self)
    }
}

/// A QIR value.
#[pyclass(unsendable)]
#[derive(Clone)]
struct Value {
    value: AnyValueEnum<'static>,
    context: Py<Context>,
}

#[pyproto]
impl PyObjectProtocol for Value {
    fn __repr__(&self) -> String {
        format!("{:?}", self.value)
    }
}

impl Value {
    fn new<'ctx>(context: Py<Context>, value: impl inkwell::values::AnyValue<'ctx>) -> Self {
        Self::from_any(context, value.as_any_value_enum())
    }

    fn from_any<'ctx>(context: Py<Context>, value: AnyValueEnum<'ctx>) -> Self {
        let value = unsafe { transmute::<AnyValueEnum, AnyValueEnum<'static>>(value) };
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
#[allow(clippy::needless_pass_by_value)]
fn constant(py: Python, ty: Py<Type>, value: &PyAny) -> PyResult<Value> {
    let context = ty.borrow(py).context.clone();
    let value = extract_value(ty.borrow(py).ty, value)?;
    Ok(Value::new(context, value))
}

#[pyclass(unsendable)]
struct Module {
    module: inkwell::module::Module<'static>,
    context: Py<Context>,
}

impl Module {
    fn new(py: Python, context: Py<Context>, name: &str) -> Module {
        let module = {
            let context = context.borrow(py);
            let module = context.create_module(name);
            unsafe {
                transmute::<inkwell::module::Module, inkwell::module::Module<'static>>(module)
            }
        };

        Module { module, context }
    }
}

/// An instruction builder.
#[pyclass(unsendable)]
struct Builder {
    builder: inkwell::builder::Builder<'static>,
    module: Py<Module>, // TODO: https://github.com/TheDan64/inkwell/issues/347
    context: Py<Context>,
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
    fn and_(&self, lhs: Value, rhs: Value) -> Value {
        let context = lhs.context;
        let lhs = lhs.value.into_int_value();
        let rhs = rhs.value.into_int_value();
        Value::new(context, self.builder.build_and(lhs, rhs, ""))
    }

    /// Inserts a bitwise logical or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn or_(&self, lhs: Value, rhs: Value) -> Value {
        let context = lhs.context;
        let lhs = lhs.value.into_int_value();
        let rhs = rhs.value.into_int_value();
        Value::new(context, self.builder.build_or(lhs, rhs, ""))
    }

    /// Inserts a bitwise logical exclusive or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn xor(&self, lhs: Value, rhs: Value) -> Value {
        let context = lhs.context;
        let lhs = lhs.value.into_int_value();
        let rhs = rhs.value.into_int_value();
        Value::new(context, self.builder.build_xor(lhs, rhs, ""))
    }

    /// Inserts an addition instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The sum.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn add(&self, lhs: Value, rhs: Value) -> Value {
        let context = lhs.context;
        let lhs = lhs.value.into_int_value();
        let rhs = rhs.value.into_int_value();
        Value::new(context, self.builder.build_int_add(lhs, rhs, ""))
    }

    /// Inserts a subtraction instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The difference.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn sub(&self, lhs: Value, rhs: Value) -> Value {
        let context = lhs.context;
        let lhs = lhs.value.into_int_value();
        let rhs = rhs.value.into_int_value();
        Value::new(context, self.builder.build_int_sub(lhs, rhs, ""))
    }

    /// Inserts a multiplication instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The product.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn mul(&self, lhs: Value, rhs: Value) -> Value {
        let context = lhs.context;
        let lhs = lhs.value.into_int_value();
        let rhs = rhs.value.into_int_value();
        Value::new(context, self.builder.build_int_mul(lhs, rhs, ""))
    }

    /// Inserts a shift left instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn shl(&self, lhs: Value, rhs: Value) -> Value {
        let context = lhs.context;
        let lhs = lhs.value.into_int_value();
        let rhs = rhs.value.into_int_value();
        Value::new(context, self.builder.build_left_shift(lhs, rhs, ""))
    }

    /// Inserts a logical (zero fill) shift right instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn lshr(&self, lhs: Value, rhs: Value) -> Value {
        let context = lhs.context;
        let lhs = lhs.value.into_int_value();
        let rhs = rhs.value.into_int_value();
        Value::new(context, self.builder.build_right_shift(lhs, rhs, false, ""))
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
    fn icmp(&self, pred: PyIntPredicate, lhs: Value, rhs: Value) -> Value {
        let context = lhs.context;
        let lhs = lhs.value.into_int_value();
        let rhs = rhs.value.into_int_value();
        Value::new(
            context,
            self.builder.build_int_compare(pred.0, lhs, rhs, ""),
        )
    }

    /// Inserts a call instruction.
    ///
    /// :param Function function: The function to call.
    /// :param Sequence[Union[Value, bool, int, float]] args: The arguments to the function.
    /// :returns: The return value, or None if the function has a void return type.
    /// :rtype: Optional[Value]
    #[pyo3(text_signature = "(self, function, args)")]
    fn call(&self, function: &Value, args: &PySequence) -> PyResult<Option<Value>> {
        let context = function.context.clone();
        let function = match function.value {
            AnyValueEnum::FunctionValue(f) => Ok(f),
            _ => Err(PyValueError::new_err("Not a function value.")),
        }?;

        let ty = function.get_type();
        let param_types = ty.get_param_types();
        let num_params = param_types.len();
        let num_args = args.len()?;

        if num_params != num_args {
            let message = format!("Expected {} arguments, got {}.", num_params, num_args);
            return Err(PyValueError::new_err(message));
        }

        let args = args
            .iter()?
            .zip(&param_types)
            .map(|(arg, ty)| Ok(any_to_meta(extract_value(*ty, arg?)?).unwrap()))
            .collect::<PyResult<Vec<_>>>()?;

        Ok(self
            .builder
            .build_call(function, &args, "")
            .try_as_basic_value()
            .left()
            .map(|v| Value::new(context, v)))
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
    fn if_(&self, cond: Value, r#true: Option<&PyAny>, r#false: Option<&PyAny>) -> PyResult<()> {
        build_if(&self.builder, cond.value.into_int_value(), r#true, r#false)
    }
}

impl Builder {
    fn new(py: Python, context: Py<Context>, module: Py<Module>) -> Self {
        let builder = {
            let context = context.borrow(py);
            unsafe {
                transmute::<inkwell::builder::Builder, inkwell::builder::Builder<'static>>(
                    context.create_builder(),
                )
            }
        };

        Self {
            builder,
            module,
            context,
        }
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
    types: Py<Types>,
    num_qubits: u64,
    num_results: u64,
}

#[pymethods]
impl SimpleModule {
    #[new]
    fn new(py: Python, name: &str, num_qubits: u64, num_results: u64) -> PyResult<SimpleModule> {
        let context = Py::new(py, Context(inkwell::context::Context::create()))?;
        let context_ref = context.borrow(py);

        let module = Module::new(py, context.clone(), name);
        let module_ref = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };
        let entry_point = qirlib::generation::qir::create_entry_point(&context_ref.0, module_ref);
        let module = Py::new(py, module)?;

        let builder = Builder::new(py, context.clone(), module.clone());
        let entry = context_ref.append_basic_block(entry_point, "entry");
        builder.builder.position_at_end(entry);
        let builder = Py::new(py, builder)?;

        let types = Py::new(
            py,
            Types {
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
    fn types(&self) -> Py<Types> {
        self.types.clone()
    }

    /// The global qubit register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn qubits(&self, py: Python) -> Vec<Value> {
        let module = self.module.borrow(py);
        let builder = self.builder.borrow(py);
        let context = module.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };
        let ty = qirlib::codegen::types::qubit(&context.0, module)
            .ptr_type(inkwell::AddressSpace::Generic);

        (0..self.num_qubits)
            .map(|id| {
                let id = qirlib::codegen::basicvalues::u64_to_i64(&context.0, id).into_int_value();
                Value::new(
                    builder.context.clone(),
                    builder.builder.build_int_to_ptr(id, ty, ""),
                )
            })
            .collect()
    }

    /// The global result register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn results(&self, py: Python) -> Vec<Value> {
        let module = self.module.borrow(py);
        let builder = self.builder.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };
        let ty = qirlib::codegen::types::result(&context.0, module)
            .ptr_type(inkwell::AddressSpace::Generic);

        (0..self.num_results)
            .map(|id| {
                let id = qirlib::codegen::basicvalues::u64_to_i64(&context.0, id).into_int_value();
                Value::new(
                    builder.context.clone(),
                    builder.builder.build_int_to_ptr(id, ty, ""),
                )
            })
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
        // TODO: Repeated calls to ir() keep adding returns.
        self.builder.borrow(py).builder.build_return(None);
        let module = self.module.borrow(py);
        module
            .module
            .verify()
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        Ok(module.module.print_to_string().to_string())
    }

    /// Emits the LLVM bitcode for the module as a sequence of bytes.
    ///
    /// :rtype: bytes
    fn bitcode<'a>(&self, py: Python<'a>) -> &'a PyBytes {
        // TODO: Repeated calls to bitcode() keep adding returns.
        self.builder.borrow(py).builder.build_return(None);
        let bitcode = self.module.borrow(py).module.write_bitcode_to_memory();
        PyBytes::new(py, bitcode.as_slice())
    }

    /// Adds a declaration for an externally linked function to the module.
    ///
    /// :param str name: The name of the function.
    /// :param Type ty: The type of the function.
    /// :return: The function value.
    /// :rtype: Function
    #[pyo3(text_signature = "(self, name, ty)")]
    fn add_external_function(&mut self, py: Python, name: &str, ty: Py<Type>) -> Value {
        let ty = ty.borrow(py);
        let context = ty.context.clone();
        let ty = ty.ty.into_function_type();
        let module = self.module.borrow(py);
        let function = module
            .module
            .add_function(name, ty, Some(Linkage::External));

        // TODO: Need to manually wrap in AnyValueEnum::FunctionValue. Going through the AnyValue
        // trait seems to turn the FunctionValue into a PointerValue. Why?
        Value::from_any(context, AnyValueEnum::FunctionValue(function))
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
    fn cx(&self, py: Python, control: Value, target: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            // TODO (safety): Check invariance of Module lifetime.
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let control = any_to_meta(control.value).unwrap();
        let target = any_to_meta(target.value).unwrap();
        let function = qirlib::codegen::qis::cnot_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[control, target]);
    }

    /// Inserts a controlled Pauli :math:`Z` gate.
    ///
    /// :param Value control: The control qubit.
    /// :param Value target: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, control, target)")]
    fn cz(&self, py: Python, control: Value, target: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let control = any_to_meta(control.value).unwrap();
        let target = any_to_meta(target.value).unwrap();
        let function = qirlib::codegen::qis::cz_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[control, target]);
    }

    /// Inserts a Hadamard gate.
    ///
    /// :param qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn h(&self, py: Python, qubit: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::h_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit]);
    }

    /// Inserts a Z-basis measurement operation.
    ///
    /// :param Value qubit: The qubit to measure.
    /// :param Value result: A result where the measurement result will be written to.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit, result)")]
    fn mz(&self, py: Python, qubit: Value, result: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let result = any_to_meta(result.value).unwrap();
        let function = qirlib::codegen::qis::mz_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit, result]);
    }

    /// Inserts a reset operation.
    ///
    /// :param Value qubit: The qubit to reset.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn reset(&self, py: Python, qubit: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::reset_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit]);
    }

    /// Inserts a rotation gate about the :math:`x` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rx(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let theta = any_to_meta(extract_value(context.f64_type(), theta)?).unwrap();
        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::rx_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[theta, qubit]);
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`y` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn ry(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let theta = any_to_meta(extract_value(context.f64_type(), theta)?).unwrap();
        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::ry_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[theta, qubit]);
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`z` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rz(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let theta = any_to_meta(extract_value(context.f64_type(), theta)?).unwrap();
        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::rz_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[theta, qubit]);
        Ok(())
    }

    /// Inserts an :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s(&self, py: Python, qubit: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::s_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit]);
    }

    /// Inserts an adjoint :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s_adj(&self, py: Python, qubit: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::s_adj(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit]);
    }

    /// Inserts a :math:`T` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t(&self, py: Python, qubit: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::t_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit]);
    }

    /// Inserts an adjoint :math:`T` gate.
    ///
    /// :param qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t_adj(&self, py: Python, qubit: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::t_adj(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit]);
    }

    /// Inserts a Pauli :math:`X` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn x(&self, py: Python, qubit: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::x_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit]);
    }

    /// Inserts a Pauli :math:`Y` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn y(&self, py: Python, qubit: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::y_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit]);
    }

    /// Inserts a Pauli :math:`Z` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn z(&self, py: Python, qubit: Value) {
        let builder = self.builder.borrow(py);
        let module = builder.module.borrow(py);
        let context = builder.context.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };

        let qubit = any_to_meta(qubit.value).unwrap();
        let function = qirlib::codegen::qis::z_body(&context.0, module);
        qirlib::codegen::calls::emit_void_call(&builder.builder, function, &[qubit]);
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
        cond: Value,
        one: Option<&PyAny>,
        zero: Option<&PyAny>,
    ) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        let context = builder.context.borrow(py);
        let module = builder.module.borrow(py);
        let module = unsafe {
            transmute::<&inkwell::module::Module, &inkwell::module::Module>(&module.module)
        };
        let read_result = qirlib::codegen::qis::read_result(&context.0, module);
        let cond = qirlib::codegen::calls::emit_call_with_return(
            &builder.builder,
            read_result,
            &[any_to_meta(cond.value).unwrap()],
            "",
        );
        build_if(&builder.builder, cond.into_int_value(), one, zero)
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
    module_name: Option<String>,
    source_file_name: Option<String>,
) -> PyResult<&'a PyBytes> {
    let bitcode = qirlib::generation::ir_to_bitcode(ir, &module_name, &source_file_name)
        .map_err(PyOSError::new_err)?;
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
    module_name: Option<String>,
    source_file_name: Option<String>,
) -> PyResult<&'a PyString> {
    let ir = qirlib::generation::bitcode_to_ir(bitcode.as_bytes(), &module_name, &source_file_name)
        .map_err(PyOSError::new_err)?;
    Ok(PyUnicode::new(py, ir.as_str()))
}

fn extract_value<'ctx>(
    ty: impl inkwell::types::AnyType<'ctx>,
    ob: &PyAny,
) -> PyResult<AnyValueEnum<'ctx>> {
    match ob.extract::<Value>() {
        Ok(value) => Ok(value.value),
        Err(_) => match ty.as_any_type_enum() {
            AnyTypeEnum::IntType(int) => {
                let value = ob.extract()?;
                let value_width = u64::BITS - u64::leading_zeros(value);
                if value_width > int.get_bit_width() {
                    // TODO: LLVM doesn't seem to care. Should we check this?
                    Err(PyOverflowError::new_err(
                        "Constant integer uses more bits than its type has.",
                    ))
                } else {
                    Ok(int.const_int(value, true).into())
                }
            }
            AnyTypeEnum::FloatType(float) => Ok(float.const_float(ob.extract()?).into()),
            _ => Err(PyTypeError::new_err(
                "Can't convert Python value into this type.",
            )),
        },
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
        AnyValueEnum::PhiValue(_)
        | AnyValueEnum::FunctionValue(_)
        | AnyValueEnum::InstructionValue(_) => None,
    }
}

fn build_if(
    builder: &inkwell::builder::Builder,
    cond: inkwell::values::IntValue,
    build_true: Option<&PyAny>,
    build_false: Option<&PyAny>,
) -> PyResult<()> {
    let insert_block = builder.get_insert_block().unwrap();
    let context = insert_block.get_context();
    let function = insert_block.get_parent().unwrap();

    let then_block = context.append_basic_block(function, "then");
    let else_block = context.append_basic_block(function, "else");
    builder.build_conditional_branch(cond, then_block, else_block);

    let continue_block = context.append_basic_block(function, "continue");

    builder.position_at_end(then_block);
    if let Some(build_true) = build_true {
        build_true.call0()?;
    }
    builder.build_unconditional_branch(continue_block);

    builder.position_at_end(else_block);
    if let Some(build_false) = build_false {
        build_false.call0()?;
    }
    builder.build_unconditional_branch(continue_block);

    builder.position_at_end(continue_block);
    Ok(())
}
