# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, types

# PyQIR Generator assumes you want to use static allocation for qubits and
# results, but you can still use dynamic allocation by manually calling the
# appropriate runtime functions.
mod = SimpleModule("dynamic_allocation", num_qubits=0, num_results=0)
qubit_allocate = mod.add_external_function(
    "__quantum__rt__qubit_allocate",
    types.Function([], types.QUBIT)
)
qubit_release = mod.add_external_function(
    "__quantum__rt__qubit_release",
    types.Function([types.QUBIT], types.VOID)
)
result_get_one = mod.add_external_function(
    "__quantum__rt__result_get_one",
    types.Function([], types.RESULT)
)
result_equal = mod.add_external_function(
    "__quantum__rt__result_equal",
    types.Function([types.RESULT, types.RESULT], types.BOOL)
)
m = mod.add_external_function(
    "__quantum__qis__m__body",
    types.Function([types.QUBIT], types.RESULT)
)

# Instead of mod.qubits[i], use __quantum__rt__qubit_allocate.
qubit = mod.builder.call(qubit_allocate, [])

qis = BasicQisBuilder(mod.builder)
qis.h(qubit)

# Instead of qis.mz, use __quantum__qis__m__body.
result = mod.builder.call(m, [qubit])

# Instead of mod.if_result, use __quantum__rt__result_equal and mod.if_.
one = mod.builder.call(result_get_one, [])
result_is_one = mod.builder.call(result_equal, [result, one])
mod.if_(result_is_one, lambda: qis.reset(qubit))

# Be sure to release any allocated qubits when you're done with them.
mod.builder.call(qubit_release, [qubit])

if __name__ == "__main__":
    print(mod.ir())
