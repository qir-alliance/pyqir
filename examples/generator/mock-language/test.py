import sys
from antlr4 import *
from out.MockLanguageLexer import MockLanguageLexer
from out.MockLanguageParser import MockLanguageParser
from out.MockLanguageListener import MockLanguageListener

class Listener(MockLanguageListener) :
    def __init__(self, output):
        self.output = output
        self.output.write('starting...\n')

    def enterXGate(self, ctx:MockLanguageParser.XGateContext):
        self.output.write('X ' + ctx.target.text + '\n')

    def enterHGate(self, ctx:MockLanguageParser.HGateContext):
        self.output.write('H ' + ctx.target.text + '\n')

    def enterCNOTGate(self, ctx:MockLanguageParser.CNOTGateContext):
        self.output.write('CX ' + ctx.control.text + ' ' + ctx.target.text + '\n')

    def enterMzGate(self, ctx:MockLanguageParser.MzGateContext):
        self.output.write('M ' + ctx.target.text + '\n')

def main(argv):
    input = FileStream(argv[1])
    lexer = MockLanguageLexer(input)
    stream = CommonTokenStream(lexer)
    parser = MockLanguageParser(stream)
    tree = parser.document()

    output = open("output.txt","w")
    
    listener = Listener(output)
    walker = ParseTreeWalker()
    walker.walk(listener, tree)
        
    output.close()      

if __name__ == '__main__':
    main(sys.argv)
