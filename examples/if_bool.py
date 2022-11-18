# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pyqir

mod = pyqir.SimpleModule("if_bool", num_qubits=2, num_results=2)
qis = pyqir.BasicQisBuilder(mod.builder)

# Use an external function to generate integers that we can compare with icmp.
i32 = pyqir.IntType(mod.context, 32)
get_int = mod.add_external_function("get_int", pyqir.FunctionType(i32, []))

# Apply X to the qubit if 'a' is 7.
a = mod.builder.call(get_int, [])
assert a is not None
a_eq_7 = mod.builder.icmp(pyqir.IntPredicate.EQ, a, pyqir.const(i32, 7))
mod.builder.if_(a_eq_7, lambda: qis.x(mod.qubits[0]))

# Multiple conditions can be combined with 'and' and 'or'.
b = mod.builder.call(get_int, [])
assert b is not None
b_sgt_a = mod.builder.icmp(pyqir.IntPredicate.SGT, b, a)
or_cond = mod.builder.or_(a_eq_7, b_sgt_a)

# Both the true and false branches can be specified.
b_ne_2 = mod.builder.icmp(pyqir.IntPredicate.NE, b, pyqir.const(i32, 2))
and_cond = mod.builder.and_(or_cond, b_ne_2)
mod.builder.if_(
    and_cond,
    true=lambda: qis.h(mod.qubits[1]),
    false=lambda: qis.y(mod.qubits[1]),
)

if __name__ == "__main__":
    print(mod.ir())
