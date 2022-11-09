#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from antlr4 import CommonTokenStream, FileStream, ParseTreeWalker
from argparse import ArgumentParser
from mock_language.MockLanguageLexer import MockLanguageLexer
from mock_language.MockLanguageParser import MockLanguageParser
from mock_language.MockLanguageListener import MockLanguageListener
from pathlib import Path
from pyqir import BasicQisBuilder, SimpleModule, Value


class QirGenerator(MockLanguageListener):  # type: ignore[misc]
    """
    Class that generates QIR when walking the parse tree
    of a Mock language program.
    """

    def __init__(self, module_id: str, num_qubits: int) -> None:
        """
        :param num_qubits: The total number of qubits used in the compilation.
        :param module_id: An identifier for the created QIR module.
        """
        self.module = SimpleModule(
            module_id, num_qubits=num_qubits, num_results=num_qubits
        )
        self.qis = BasicQisBuilder(self.module.builder)

    def ir(self) -> str:
        return self.module.ir()

    def parse_qubit(self, id: str) -> Value:
        try:
            return self.module.qubits[int(id)]
        except IndexError as e:
            raise ValueError("Parsed progam uses more qubits than allocated") from e

    def parse_result(self, id: str) -> Value:
        try:
            return self.module.results[int(id)]
        except IndexError as e:
            raise ValueError("Parsed progam uses more results than allocated") from e

    def enterXGate(self, ctx: MockLanguageParser.XGateContext) -> None:
        qubit = self.parse_qubit(ctx.target.text)
        self.qis.x(qubit)

    def enterHGate(self, ctx: MockLanguageParser.HGateContext) -> None:
        qubit = self.parse_qubit(ctx.target.text)
        self.qis.h(qubit)

    def enterCNOTGate(self, ctx: MockLanguageParser.CNOTGateContext) -> None:
        control = self.parse_qubit(ctx.control.text)
        target = self.parse_qubit(ctx.target.text)
        self.qis.cx(control, target)

    def enterMzGate(self, ctx: MockLanguageParser.MzGateContext) -> None:
        qubit = self.parse_qubit(ctx.target.text)
        result = self.parse_result(ctx.target.text)
        self.qis.mz(qubit, result)


def mock_program_to_qir(num_qubits: int, input_file: str) -> str:
    """
    Parses a Mock program and generates QIR based on the syntax tree.
    Usually the language-specific compiler would fully validate and
    potentially optimize the program before QIR is generated, but for
    illustration purposes we omit that from this example.

    :param num_qubits: The total number of qubits used in the program.
    :param input_file: Path of the file containing the Mock program.
    """

    lexer = MockLanguageLexer(FileStream(input_file))
    stream = CommonTokenStream(lexer)
    parser = MockLanguageParser(stream)
    tree = parser.document()

    generator = QirGenerator(Path(input_file).stem, num_qubits)
    walker = ParseTreeWalker()
    walker.walk(generator, tree)
    return generator.ir()


if __name__ == "__main__":
    command_line = ArgumentParser()
    command_line.add_argument(
        "input_file", type=str, help="Path of the file containing the Mock program."
    )
    command_line.add_argument(
        "num_qubits", type=int, help="The total number of qubits used in the program."
    )
    command_line.add_argument(
        "-o", "--output_file", type=str, help="Path of the file to write the IR to."
    )
    args = command_line.parse_args()

    generated_qir = mock_program_to_qir(args.num_qubits, args.input_file)

    if args.output_file is not None:
        with open(args.output_file, "w") as file:
            file.write(generated_qir)
    else:
        print(generated_qir)
