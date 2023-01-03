# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pyqir
from pyqir import (
    BasicBlock,
    Builder,
    Context,
    Function,
    Linkage,
    Module,
    ModuleFlagBehavior,
)

context = Context()
mod = pyqir.qir_module(
    context,
    "dynamic_allocation",
    qir_major_version=1,
    qir_minor_version=0,
    dynamic_qubit_management=True,
    dynamic_result_management=True,
)
builder = Builder(context)

# define external calls and type definitions
qubit_type = pyqir.qubit_type(context)
result_type = pyqir.result_type(context)

# PyQIR assumes you want to use static allocation for qubits and results, but
# you can still use dynamic allocation by manually calling the appropriate
# runtime functions.
qubit_allocate = Function(
    pyqir.FunctionType(qubit_type, []),
    Linkage.EXTERNAL,
    "__quantum__rt__qubit_allocate",
    mod,
)

qubit_release = Function(
    pyqir.FunctionType(pyqir.Type.void(context), [qubit_type]),
    Linkage.EXTERNAL,
    "__quantum__rt__qubit_release",
    mod,
)

result_get_one = Function(
    pyqir.FunctionType(result_type, []),
    Linkage.EXTERNAL,
    "__quantum__rt__result_get_one",
    mod,
)

result_equal = Function(
    pyqir.FunctionType(pyqir.IntType(context, 1), [result_type, result_type]),
    Linkage.EXTERNAL,
    "__quantum__rt__result_equal",
    mod,
)

m = Function(
    pyqir.FunctionType(result_type, [qubit_type]),
    Linkage.EXTERNAL,
    "__quantum__qis__m__body",
    mod,
)

# Create entry point
num_qubits = 1
num_results = 1
entry_point = pyqir.entry_point(mod, "main", num_qubits, num_results)
builder.insert_at_end(BasicBlock(context, "entry", entry_point))

# Define entry point body
qubit_return = builder.call(qubit_allocate, [])

assert qubit_return is not None
qubit = qubit_return

qis = pyqir.BasicQisBuilder(builder)
qis.h(qubit)

# Instead of qis.mz, use __quantum__qis__m__body.
result = builder.call(m, [qubit])
assert result is not None

# Instead of if_result, use __quantum__rt__result_equal and mod.if_.
one = builder.call(result_get_one, [])
assert one is not None
result_is_one = builder.call(result_equal, [result, one])
assert result_is_one is not None
builder.if_(result_is_one, lambda: qis.reset(qubit))

# Be sure to release any allocated qubits when you're done with them.
builder.call(qubit_release, [qubit])

if __name__ == "__main__":
    print(str(mod))
