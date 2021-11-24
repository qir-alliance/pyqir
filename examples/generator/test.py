#!/usr/bin/env python3

# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

import sys
from antlr4 import *
from mock_language.MockLanguageLexer import MockLanguageLexer
from mock_language.MockLanguageParser import MockLanguageParser
from mock_language.MockLanguageListener import MockLanguageListener
from pyqir_generator import QirBuilder


class QirGenerator(MockLanguageListener) :
    def __init__(self, nr_qubits, module_id):
        self.builder = QirBuilder(module_id)
        self.builder.add_quantum_register("q", nr_qubits)
        self.builder.add_classical_register("m", nr_qubits)

    def get_ir_string(self) -> str:
        return self.builder.get_ir_string()

    def write_to_file(self, file_name) -> str:
        file = open(file_name,"w")
        file.write(self.get_ir_string())
        file.close()

    def enterXGate(self, ctx: MockLanguageParser.XGateContext):
        self.builder.x("q" + ctx.target.text)

    def enterHGate(self, ctx:MockLanguageParser.HGateContext):
        self.builder.h("q" + ctx.target.text)

    def enterCNOTGate(self, ctx:MockLanguageParser.CNOTGateContext):
        self.builder.cx("q" + ctx.control.text, "q" + ctx.target.text)

    def enterMzGate(self, ctx:MockLanguageParser.MzGateContext):
        self.builder.m("q" + ctx.target.text, "m" + ctx.target.text)

def main(argv):
    nr_qubits = int(argv[1])
    input_file = FileStream(argv[2])

    lexer = MockLanguageLexer(input_file)
    stream = CommonTokenStream(lexer)
    parser = MockLanguageParser(stream)
    tree = parser.document()

    module_id = "test"
    
    generator = QirGenerator(nr_qubits, module_id)
    walker = ParseTreeWalker()
    walker.walk(generator, tree)
        
    print(generator.get_ir_string())
    #generator.write_to_file("output.txt")

if __name__ == '__main__':
    main(sys.argv)

