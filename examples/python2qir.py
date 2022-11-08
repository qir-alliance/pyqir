#!/usr/bin/env python3

# Transforms a subset of the Python language into QIR, by using:
# - the built-in ast (Asbtract Syntax Tree) library to parse the source code
# - the pyqir-generator package to generate and display QIR
# Here, we transform a Qiskit circuit without using the Qiskit package

import ast
from pyqir import BasicQisBuilder, SimpleModule


def main() -> None:
    # Open and parse the input file
    with open("python2qir_qiskit_input.py", "r") as source:
        tree = ast.parse(source.read())

    # Walk the Abstract Syntax Tree (AST) and translate into QIR with pyqir-generator
    analyzer = Analyzer()
    analyzer.visit(tree)

    print("\n\n== Output QIR ==")
    print(analyzer.module.ir())


class Analyzer(ast.NodeVisitor):
    module: SimpleModule
    builder: BasicQisBuilder

    def visit_Call(self, node: ast.Call) -> None:
        if isinstance(node.func, ast.Name):
            name: ast.Name = node.func
            if name.id == "QuantumCircuit":
                num_qubits = int_value(node.args[0])
                num_results = int_value(node.args[1])
                self.module = SimpleModule("python2qir", num_qubits, num_results)
                self.builder = BasicQisBuilder(self.module.builder)

        if isinstance(node.func, ast.Attribute):
            attribute: ast.Attribute = node.func
            if attribute.attr == "cx":
                control = int_value(node.args[0])
                target = int_value(node.args[1])
                self.builder.cx(self.module.qubits[control], self.module.qubits[target])
            if attribute.attr == "h":
                qubit = int_value(node.args[0])
                self.builder.h(self.module.qubits[qubit])
            if attribute.attr == "measure":
                qubit = int_value(node.args[0])
                bit = int_value(node.args[1])
                self.builder.mz(self.module.qubits[qubit], self.module.results[bit])
            if attribute.attr == "z":
                qubit = int_value(node.args[0])
                self.builder.z(self.module.qubits[qubit])

        self.generic_visit(node)


def int_value(e: ast.expr) -> int:
    assert isinstance(e, ast.Constant)
    value = e.value
    assert isinstance(value, int)
    return value


if __name__ == "__main__":
    main()
