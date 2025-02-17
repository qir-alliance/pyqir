# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import iqm_pyqir

mod = iqm_pyqir.SimpleModule("arithmetic", num_qubits=0, num_results=0)
qis = iqm_pyqir.BasicQisBuilder(mod.builder)

# Declare functions that can produce and consume integers at runtime. See
# external_functions.py.
i32 = iqm_pyqir.IntType(mod.context, 32)
get_int = mod.add_external_function("get_int", iqm_pyqir.FunctionType(i32, []))
take_int = mod.add_external_function(
    "take_int", iqm_pyqir.FunctionType(iqm_pyqir.Type.void(mod.context), [i32])
)

# Add 3 to a number and multiply the result by 2.
a = mod.builder.call(get_int, [])
assert a is not None
# Python numbers need to be converted into QIR constant values. Since it's being
# added to a 32-bit integer returned by get_int, its type needs to be the same.
three = iqm_pyqir.const(i32, 3)
b = mod.builder.add(three, a)
c = mod.builder.mul(iqm_pyqir.const(i32, 2), b)

# Negation can be done by subtracting an integer from zero.
x = mod.builder.call(get_int, [])
assert x is not None
negative_x = mod.builder.sub(iqm_pyqir.const(i32, 0), x)

# Consume the results.
mod.builder.call(take_int, [c])
mod.builder.call(take_int, [negative_x])

if __name__ == "__main__":
    print(mod.ir())
