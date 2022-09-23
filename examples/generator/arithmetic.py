# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, const

mod = SimpleModule("arithmetic", num_qubits=0, num_results=0)
qis = BasicQisBuilder(mod.builder)
types = mod.types

# Declare functions that can produce and consume integers at runtime. See
# external_functions.py.
i32 = types.integer(32)
get_int = mod.add_external_function("get_int", types.function(i32, []))
take_int = mod.add_external_function("take_int", types.function(types.void, [i32]))

# Add 3 to a number and multiply the result by 2.
a = mod.builder.call(get_int, [])
assert a is not None
# Python numbers need to be converted into QIR constant values. Since it's being
# added to a 32-bit integer returned by get_int, its type needs to be the same.
three = const(i32, 3)
b = mod.builder.add(three, a)
c = mod.builder.mul(const(i32, 2), b)

# Negation can be done by subtracting an integer from zero.
x = mod.builder.call(get_int, [])
assert x is not None
negative_x = mod.builder.sub(const(i32, 0), x)

# Consume the results.
mod.builder.call(take_int, [c])
mod.builder.call(take_int, [negative_x])

if __name__ == "__main__":
    print(mod.ir())
