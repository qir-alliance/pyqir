# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pyqir

mod = pyqir.SimpleModule("dynamic_allocation", num_qubits=0, num_results=0)
qubit_type = pyqir.qubit_type(mod.context)
result_type = pyqir.result_type(mod.context)

# PyQIR assumes you want to use static allocation for qubits and results, but
# you can still use dynamic allocation by manually calling the appropriate
# runtime functions.
qubit_allocate = mod.add_external_function(
    "__quantum__rt__qubit_allocate", pyqir.FunctionType(qubit_type, [])
)
qubit_release = mod.add_external_function(
    "__quantum__rt__qubit_release",
    pyqir.FunctionType(pyqir.Type.void(mod.context), [qubit_type]),
)
result_get_one = mod.add_external_function(
    "__quantum__rt__result_get_one", pyqir.FunctionType(result_type, [])
)
result_equal = mod.add_external_function(
    "__quantum__rt__result_equal",
    pyqir.FunctionType(pyqir.IntType(mod.context, 1), [result_type, result_type]),
)
m = mod.add_external_function(
    "__quantum__qis__m__body", pyqir.FunctionType(result_type, [qubit_type])
)

# Instead of mod.qubits[i], use __quantum__rt__qubit_allocate.
qubit_return = mod.builder.call(qubit_allocate, [])
assert qubit_return is not None
qubit = qubit_return

qis = pyqir.BasicQisBuilder(mod.builder)
qis.h(qubit)

# Instead of qis.mz, use __quantum__qis__m__body.
result = mod.builder.call(m, [qubit])
assert result is not None

# Instead of mod.if_result, use __quantum__rt__result_equal and mod.if_.
one = mod.builder.call(result_get_one, [])
assert one is not None
result_is_one = mod.builder.call(result_equal, [result, one])
assert result_is_one is not None
mod.builder.if_(result_is_one, lambda: qis.reset(qubit))

# Be sure to release any allocated qubits when you're done with them.
mod.builder.call(qubit_release, [qubit])

if __name__ == "__main__":
    print(mod.ir())
