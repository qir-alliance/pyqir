# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, const, types

mod = SimpleModule("arithmetic", num_qubits=0, num_results=0)
qis = BasicQisBuilder(mod.builder)

# Declare functions that can produce and consume integers at runtime. See
# external_functions.py.
i32 = types.Int(32)
get_int = mod.add_external_function("get_int", types.Function([], i32))
take_int = mod.add_external_function(
    "take_int", types.Function([i32], types.VOID))

# Do some integer arithmetic.
n = mod.builder.call(get_int, [])
result = mod.builder.mul(const(i32, 2), mod.builder.neg(n))

# Consume the result.
mod.builder.call(take_int, [result])

if __name__ == "__main__":
    print(mod.ir())
