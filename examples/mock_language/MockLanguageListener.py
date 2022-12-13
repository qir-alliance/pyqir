# Generated from MockLanguage.g4 by ANTLR 4.11.1
from antlr4 import *
if __name__ is not None and "." in __name__:
    from .MockLanguageParser import MockLanguageParser
else:
    from MockLanguageParser import MockLanguageParser

# This class defines a complete listener for a parse tree produced by MockLanguageParser.
class MockLanguageListener(ParseTreeListener):

    # Enter a parse tree produced by MockLanguageParser#document.
    def enterDocument(self, ctx:MockLanguageParser.DocumentContext):
        pass

    # Exit a parse tree produced by MockLanguageParser#document.
    def exitDocument(self, ctx:MockLanguageParser.DocumentContext):
        pass


    # Enter a parse tree produced by MockLanguageParser#XGate.
    def enterXGate(self, ctx:MockLanguageParser.XGateContext):
        pass

    # Exit a parse tree produced by MockLanguageParser#XGate.
    def exitXGate(self, ctx:MockLanguageParser.XGateContext):
        pass


    # Enter a parse tree produced by MockLanguageParser#HGate.
    def enterHGate(self, ctx:MockLanguageParser.HGateContext):
        pass

    # Exit a parse tree produced by MockLanguageParser#HGate.
    def exitHGate(self, ctx:MockLanguageParser.HGateContext):
        pass


    # Enter a parse tree produced by MockLanguageParser#CNOTGate.
    def enterCNOTGate(self, ctx:MockLanguageParser.CNOTGateContext):
        pass

    # Exit a parse tree produced by MockLanguageParser#CNOTGate.
    def exitCNOTGate(self, ctx:MockLanguageParser.CNOTGateContext):
        pass


    # Enter a parse tree produced by MockLanguageParser#MzGate.
    def enterMzGate(self, ctx:MockLanguageParser.MzGateContext):
        pass

    # Exit a parse tree produced by MockLanguageParser#MzGate.
    def exitMzGate(self, ctx:MockLanguageParser.MzGateContext):
        pass



del MockLanguageParser