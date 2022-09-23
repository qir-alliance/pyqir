# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, IntPredicate, SimpleModule, const

mod = SimpleModule("if_bool", num_qubits=2, num_results=2)
qis = BasicQisBuilder(mod.builder)

# Use an external function to generate integers that we can compare with icmp.
i32 = mod.types.integer(32)
get_int = mod.add_external_function("get_int", mod.types.function(i32, []))

# Apply X to the qubit if 'a' is 7.
a = mod.builder.call(get_int, [])
assert a is not None
a_eq_7 = mod.builder.icmp(IntPredicate.EQ, a, const(i32, 7))
mod.builder.if_(a_eq_7, lambda: qis.x(mod.qubits[0]))

# Multiple conditions can be combined with 'and' and 'or'.
b = mod.builder.call(get_int, [])
assert b is not None
b_sgt_a = mod.builder.icmp(IntPredicate.SGT, b, a)
or_cond = mod.builder.or_(a_eq_7, b_sgt_a)

# Both the true and false branches can be specified.
b_ne_2 = mod.builder.icmp(IntPredicate.NE, b, const(i32, 2))
and_cond = mod.builder.and_(or_cond, b_ne_2)
mod.builder.if_(
    and_cond,
    true=lambda: qis.h(mod.qubits[1]),
    false=lambda: qis.y(mod.qubits[1]),
)

if __name__ == "__main__":
    print(mod.ir())
