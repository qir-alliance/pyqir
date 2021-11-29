#!/usr/bin/env python3

# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

import sys
from antlr4 import *
from argparse import *
from mock_language.MockLanguageLexer import MockLanguageLexer
from mock_language.MockLanguageParser import MockLanguageParser
from mock_language.MockLanguageListener import MockLanguageListener
from pathlib import Path
from pyqir_generator import QirBuilder


class QirGenerator(MockLanguageListener):
    """
    Class that generates QIR when walking the parse tree 
    of a Mock language program.
    """    
    
    def __init__(self, nr_qubits: int, module_id: str):
        """
        :param nr_qubits: The total number of qubits used in the compilation.
        :param module_id: An identifier for the created QIR module.
        """
        self.builder = QirBuilder(module_id)
        self.builder.add_quantum_register("q", nr_qubits)
        self.builder.add_classical_register("m", nr_qubits)
        self.nr_qubits = nr_qubits

    @property
    def ir_string(self) -> str:
        return self.builder.get_ir_string()

    def write_to_file(self, file_path: str) -> str:
        """
        :param file_path: Path of the file to write the IR to.
        """
        with open(file_path, 'w') as file:
            file.write(self.ir_string)

    def enterXGate(self, ctx: MockLanguageParser.XGateContext):
        if int(ctx.target.text) >= self.nr_qubits:
            raise ValueError("Parsed progam uses more qubits than allocated")

        self.builder.x("q" + ctx.target.text)

    def enterHGate(self, ctx: MockLanguageParser.HGateContext):
        if int(ctx.target.text) >= self.nr_qubits:
            raise ValueError("Parsed progam uses more qubits than allocated")

        self.builder.h("q" + ctx.target.text)

    def enterCNOTGate(self, ctx: MockLanguageParser.CNOTGateContext):
        if int(ctx.target.text) >= self.nr_qubits:
            raise ValueError("Parsed progam uses more qubits than allocated")

        self.builder.cx("q" + ctx.control.text, "q" + ctx.target.text)

    def enterMzGate(self, ctx: MockLanguageParser.MzGateContext):
        if int(ctx.target.text) >= self.nr_qubits:
            raise ValueError("Parsed progam uses more qubits than allocated")

        self.builder.m("q" + ctx.target.text, "m" + ctx.target.text)


def mock_program_to_qir(nr_qubits: int, input_file: str) -> str:
    """
    Parses a Mock program and generates QIR based on the syntax tree.
    Usually the language-specific compiler would fully validate and 
    potentially optimize the program before QIR is generated, but for 
    illustration purposes we omit that from this example.

    :param nr_qubits: The total number of qubits used in the program.
    :param input_file: Path of the file containing the Mock program.
    """

    lexer = MockLanguageLexer(FileStream(input_file))
    stream = CommonTokenStream(lexer)
    parser = MockLanguageParser(stream)
    tree = parser.document()
    
    generator = QirGenerator(nr_qubits, Path(input_file).stem)
    walker = ParseTreeWalker()
    walker.walk(generator, tree)
    return generator.ir_string


if __name__ == '__main__':

    command_line = ArgumentParser()
    command_line.add_argument(
        'input_file', type=str,
        help='Path of the file containing the Mock program.')
    command_line.add_argument(
        'nr_qubits', type=int,
        help='The total number of qubits used in the program.')
    command_line.add_argument(
        '-o', '--output_file', type=str,
        help='Path of the file to write the IR to.')
    args = command_line.parse_args()

    generated_qir = mock_program_to_qir(args.nr_qubits, args.input_file)

    if args.output_file is not None:
        with open(args.output_file, 'w') as file:
            file.write(generated_qir)
    else: print(generated_qir)

