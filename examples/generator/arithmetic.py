# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, types

mod = SimpleModule("external_functions", num_qubits=1, num_results=0)
qis = BasicQisBuilder(mod.builder)

# Declare functions that can produce and consume integers at runtime. See
# external_functions.py.
get_int = mod.add_external_function("get_int", types.Function([], types.INT))
take_int = mod.add_external_function(
    "take_int", types.Function([types.INT], types.VOID))

# Do some integer arithmetic.
n = mod.builder.call(get_int, [])
result = mod.builder.mul(2, mod.builder.neg(n))

# Consume the result.
mod.builder.call(take_int, [result])

if __name__ == "__main__":
    print(mod.ir())
