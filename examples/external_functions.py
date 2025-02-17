# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import iqm_pyqir

mod = iqm_pyqir.SimpleModule("external_functions", num_qubits=1, num_results=0)
qis = iqm_pyqir.BasicQisBuilder(mod.builder)

void = iqm_pyqir.Type.void(mod.context)
i64 = iqm_pyqir.IntType(mod.context, 64)

# Declare an externally linked function named that has no parameters and returns
# void.
my_function = mod.add_external_function("my_function", iqm_pyqir.FunctionType(void, []))

# Call the function with no arguments (the empty list).
mod.builder.call(my_function, [])

# Declare a function that takes an integer and a qubit and returns void.
my_gate = mod.add_external_function(
    "my_gate", iqm_pyqir.FunctionType(void, [i64, iqm_pyqir.qubit_type(mod.context)])
)

# Call the functions with a list of arguments.
mod.builder.call(my_gate, [iqm_pyqir.const(i64, 123), mod.qubits[0]])

# Declare a function that returns a double.
get_angle = mod.add_external_function(
    "get_angle", iqm_pyqir.FunctionType(iqm_pyqir.Type.double(mod.context), [])
)

# Use the return value of the function as the input to a rotation gate.
angle = mod.builder.call(get_angle, [])
assert angle is not None
qis.rz(angle, mod.qubits[0])

if __name__ == "__main__":
    print(mod.ir())
