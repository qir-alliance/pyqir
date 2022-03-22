# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import SimpleModule, types

mod = SimpleModule("external_functions", num_qubits=1, num_results=0)

# Declare an externally linked function named that has no parameters and returns
# void.
my_function = mod.add_external_function(
    "my_function", types.Function([], types.VOID)
)

# Call the function with no arguments (the empty list).
mod.builder.call(my_function, [])

# Declare another function that takes an integer and a qubit and returns void.
my_gate = mod.add_external_function(
    "my_gate", types.Function([types.INT, types.QUBIT], types.VOID)
)

# Call the functions with a list of arguments.
mod.builder.call(my_gate, [123, mod.qubits[0]])

if __name__ == "__main__":
    print(mod.ir())
