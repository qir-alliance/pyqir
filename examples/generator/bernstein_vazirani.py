#!/usr/bin/env python3

# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import List, Callable, Tuple
from pyqir_generator import QirBuilder


ParityFunction: type = Callable[[Tuple[List[str], str]], None]


class BernsteinVazirani:
    """
    Implementation of the Bernstein-Vazirani quantum algorithm
    """

    def __init__(self, qubit_count: int = 8, pattern: int = 12):
        """
        :param qubit_count: number of qubits to use
        :type qubit_count: int
        :param pattern: integer representation of the bitstring pattern
        :type pattern: int
        """
        self.builder = QirBuilder("Bernstein-Vazirani")
        bitstring = self.int_as_bool_array(pattern, qubit_count)
        Uf = self.parity_operation(bitstring)
        self.parity_via_fourier_sampling(Uf, qubit_count)

    def write_ir_file(self, file_path: str):
        self.builder.write(file_path)

    def get_ir_string(self) -> str:
        return self.builder.get_ir_string()

    def generate_ir_file(file_path: str, qubit_count=8, pattern=12):
        instance = BernsteinVazirani(qubit_count, pattern)
        instance.write(file_path)

    def parity_operation_impl(self, pattern: List[bool],
                              queryRegister: List[str], target: str) -> None:
        """
        To demonstrate the Bernstein–Vazirani algorithm, we define
        a function which returns black-box operations (List[str] => None) of
        the form

            U_f |𝑥〉|𝑦〉 = |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉,

        In particular, we define 𝑓 by providing the pattern 𝑟⃗. Thus, we can
        easily assert that the pattern measured by the Bernstein–Vazirani
        algorithm matches the pattern we used to define 𝑓.

        We will typically only call this function by partially applying it from
        within a matching function.

        :param pattern: The bitstring 𝑟⃗ used to define the function 𝑓.
        :type pattern: List[bool]
        :param queryRegister: qubit ids to perform operations against
        :type queryRegister: List[str]
        :param target: name of the target qubit
        :type target: str
        """

        if len(queryRegister) != len(pattern):
            raise ValueError(
                'Length of input register must be equal to the pattern length.'
            )

        for patternBit, controlQubit in zip(pattern, queryRegister):
            if patternBit:
                self.builder.cx(controlQubit, target)

    def parity_operation(self, pattern: List[bool]) -> ParityFunction:
        """
        Given a bitstring 𝑟⃗ = (r₀, …, rₙ₋₁), returns an operation implementing
        a unitary 𝑈 that acts on 𝑛 + 1 qubit ids as
            𝑈 |𝑥〉|𝑦〉 = |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉,
        where 𝑓(𝑥) = Σᵢ 𝑥ᵢ 𝑟ᵢ mod 2.

        :param pattern: The bitstring 𝑟⃗ used to define the function 𝑓.
        :type pattern: List[bool]
        Returns:
            An operation implementing 𝑈.
        """
        return lambda register, target: self.parity_operation_impl(
            pattern,
            register,
            target
        )

    def parity_via_fourier_sampling(self, Uf: ParityFunction, n: int) -> None:
        """
        parity_via_fourier_sampling implements the Bernstein-Vazirani quantum
        algorithm. This algorithm computes for a given Boolean function that
        is promised to be a parity 𝑓(𝑥₀, …, 𝑥ₙ₋₁) = Σᵢ 𝑟ᵢ 𝑥ᵢ a result in
        form of a bit vector (𝑟₀, …, 𝑟ₙ₋₁) corresponding to the parity
        function. Note that it is promised that the function is actually a
        parity function.

        :param Uf: A quantum operation that implements
             |𝑥〉|𝑦〉 ↦ |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉, where 𝑓 is a Boolean function that
             implements a parity Σᵢ 𝑟ᵢ 𝑥ᵢ.
        :type Uf: ParityFunction
        :param n: The number of bits of the input register |𝑥〉.
        :type n: int


        Returns:
        This function returns None but the generated QIR will ruturn an array
        of type `Result[]` that contains the parity 𝑟⃗ = (𝑟₀, …, 𝑟ₙ₋₁).
        The result output is inferred by the declaration of classical registers

        See Also
        - For details see Section 1.4.3 of Nielsen & Chuang.

        References
        - [ *Ethan Bernstein and Umesh Vazirani*,
            SIAM J. Comput., 26(5), 1411–1473, 1997 ]
            (https:#doi.org/10.1137/S0097539796300921)
        """

        # Now, we allocate n + 1 clean qubits.
        # Note that the function Uf is defined on inputs of the form (x, y),
        # where x has n bits and y has 1 bit.
        self.builder.add_quantum_register("qubit", n)
        self.builder.add_quantum_register("target", 1)
        self.builder.add_classical_register("output", n)

        target = "target0"
        queryRegister: List[str] = []
        for i in range(n):
            queryRegister.append(f"qubit{i}")

        # The last qubit needs to be flipped so that the function will
        # actually be computed into the phase when Uf is applied.
        self.builder.x(target)

        # Now, a Hadamard transform is applied to each of the qubits.
        for qubit in queryRegister:
            self.builder.h(qubit)

        self.builder.h(target)
        # We now apply Uf to the n+1 qubits, computing |x, y〉 ↦ |x, y ⊕ f(x)〉.
        Uf(queryRegister, target)

        # As the last step before the measurement, a Hadamard transform is
        # applied to all qubits except last one. We could apply the transform
        # to the last qubit also, but this would not affect the final outcome.
        for qubit in queryRegister:
            self.builder.h(qubit)

        self.builder.reset(target)

        output_ids: List[str] = []
        for i in range(n):
            output_ids.append(f"output{i}")

        for qubit, output in zip(queryRegister, output_ids):
            self.builder.m(qubit, output)
            self.builder.reset(qubit)

    def int_as_bool_array(self, number: int, bits: int) -> List[bool]:
        """
        Produces a binary representation of a non-negative integer, using the
        little-endian representation for the returned array.

        :param number: A non-negative integer to be converted to an array of
            boolean values.
        :type number: int
        :param bits: The number of bits in the binary representation
             of `number`.
        :type bits: int

        Returns: An list of boolean values representing `number`.
        """
        if(bits < 0 or bits > 63):
            raise ValueError("`bits` must be between 0 and 63 {2^bits}")
        if(number < 0 or number >= 2 ** bits):
            raise ValueError(
                f"`number` must be between 0 and 2^{bits} - 1 but was {number}"
            )

        return [bool(number & (1 << n)) for n in range(bits)]


if __name__ == "__main__":
    print(BernsteinVazirani().get_ir_string())
