# Generated from .\MockLanguage.g4 by ANTLR 4.9.1
# encoding: utf-8
from antlr4 import *
from io import StringIO
import sys
if sys.version_info[1] > 5:
	from typing import TextIO
else:
	from typing.io import TextIO


def serializedATN():
    with StringIO() as buf:
        buf.write("\3\u608b\ua72a\u8133\ub9ed\u417c\u3be7\u7786\u5964\3\n")
        buf.write("\32\4\2\t\2\4\3\t\3\3\2\7\2\b\n\2\f\2\16\2\13\13\2\3\2")
        buf.write("\3\2\3\3\3\3\3\3\3\3\3\3\3\3\3\3\3\3\3\3\5\3\30\n\3\3")
        buf.write("\3\2\2\4\2\4\2\2\2\33\2\t\3\2\2\2\4\27\3\2\2\2\6\b\5\4")
        buf.write("\3\2\7\6\3\2\2\2\b\13\3\2\2\2\t\7\3\2\2\2\t\n\3\2\2\2")
        buf.write("\n\f\3\2\2\2\13\t\3\2\2\2\f\r\7\2\2\3\r\3\3\2\2\2\16\17")
        buf.write("\7\3\2\2\17\30\7\7\2\2\20\21\7\4\2\2\21\30\7\7\2\2\22")
        buf.write("\23\7\5\2\2\23\24\7\7\2\2\24\30\7\7\2\2\25\26\7\6\2\2")
        buf.write("\26\30\7\7\2\2\27\16\3\2\2\2\27\20\3\2\2\2\27\22\3\2\2")
        buf.write("\2\27\25\3\2\2\2\30\5\3\2\2\2\4\t\27")
        return buf.getvalue()


class MockLanguageParser ( Parser ):

    grammarFileName = "MockLanguage.g4"

    atn = ATNDeserializer().deserialize(serializedATN())

    decisionsToDFA = [ DFA(ds, i) for i, ds in enumerate(atn.decisionToState) ]

    sharedContextCache = PredictionContextCache()

    literalNames = [ "<INVALID>", "'x'", "'h'", "'cnot'", "'mz'" ]

    symbolicNames = [ "<INVALID>", "<INVALID>", "<INVALID>", "<INVALID>", 
                      "<INVALID>", "QubitId", "Whitespace", "Comment", "Invalid" ]

    RULE_document = 0
    RULE_instruction = 1

    ruleNames =  [ "document", "instruction" ]

    EOF = Token.EOF
    T__0=1
    T__1=2
    T__2=3
    T__3=4
    QubitId=5
    Whitespace=6
    Comment=7
    Invalid=8

    def __init__(self, input:TokenStream, output:TextIO = sys.stdout):
        super().__init__(input, output)
        self.checkVersion("4.9.1")
        self._interp = ParserATNSimulator(self, self.atn, self.decisionsToDFA, self.sharedContextCache)
        self._predicates = None




    class DocumentContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser
            self.instructions = None # InstructionContext
            self.eof = None # Token

        def EOF(self):
            return self.getToken(MockLanguageParser.EOF, 0)

        def instruction(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(MockLanguageParser.InstructionContext)
            else:
                return self.getTypedRuleContext(MockLanguageParser.InstructionContext,i)


        def getRuleIndex(self):
            return MockLanguageParser.RULE_document

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterDocument" ):
                listener.enterDocument(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitDocument" ):
                listener.exitDocument(self)




    def document(self):

        localctx = MockLanguageParser.DocumentContext(self, self._ctx, self.state)
        self.enterRule(localctx, 0, self.RULE_document)
        self._la = 0 # Token type
        try:
            self.enterOuterAlt(localctx, 1)
            self.state = 7
            self._errHandler.sync(self)
            _la = self._input.LA(1)
            while (((_la) & ~0x3f) == 0 and ((1 << _la) & ((1 << MockLanguageParser.T__0) | (1 << MockLanguageParser.T__1) | (1 << MockLanguageParser.T__2) | (1 << MockLanguageParser.T__3))) != 0):
                self.state = 4
                localctx.instructions = self.instruction()
                self.state = 9
                self._errHandler.sync(self)
                _la = self._input.LA(1)

            self.state = 10
            localctx.eof = self.match(MockLanguageParser.EOF)
        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class InstructionContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser


        def getRuleIndex(self):
            return MockLanguageParser.RULE_instruction

     
        def copyFrom(self, ctx:ParserRuleContext):
            super().copyFrom(ctx)



    class MzGateContext(InstructionContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a MockLanguageParser.InstructionContext
            super().__init__(parser)
            self.name = None # Token
            self.target = None # Token
            self.copyFrom(ctx)

        def QubitId(self):
            return self.getToken(MockLanguageParser.QubitId, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterMzGate" ):
                listener.enterMzGate(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitMzGate" ):
                listener.exitMzGate(self)


    class XGateContext(InstructionContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a MockLanguageParser.InstructionContext
            super().__init__(parser)
            self.name = None # Token
            self.target = None # Token
            self.copyFrom(ctx)

        def QubitId(self):
            return self.getToken(MockLanguageParser.QubitId, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterXGate" ):
                listener.enterXGate(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitXGate" ):
                listener.exitXGate(self)


    class HGateContext(InstructionContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a MockLanguageParser.InstructionContext
            super().__init__(parser)
            self.name = None # Token
            self.target = None # Token
            self.copyFrom(ctx)

        def QubitId(self):
            return self.getToken(MockLanguageParser.QubitId, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterHGate" ):
                listener.enterHGate(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitHGate" ):
                listener.exitHGate(self)


    class CNOTGateContext(InstructionContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a MockLanguageParser.InstructionContext
            super().__init__(parser)
            self.name = None # Token
            self.control = None # Token
            self.target = None # Token
            self.copyFrom(ctx)

        def QubitId(self, i:int=None):
            if i is None:
                return self.getTokens(MockLanguageParser.QubitId)
            else:
                return self.getToken(MockLanguageParser.QubitId, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterCNOTGate" ):
                listener.enterCNOTGate(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitCNOTGate" ):
                listener.exitCNOTGate(self)



    def instruction(self):

        localctx = MockLanguageParser.InstructionContext(self, self._ctx, self.state)
        self.enterRule(localctx, 2, self.RULE_instruction)
        try:
            self.state = 21
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [MockLanguageParser.T__0]:
                localctx = MockLanguageParser.XGateContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 12
                localctx.name = self.match(MockLanguageParser.T__0)
                self.state = 13
                localctx.target = self.match(MockLanguageParser.QubitId)
                pass
            elif token in [MockLanguageParser.T__1]:
                localctx = MockLanguageParser.HGateContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 14
                localctx.name = self.match(MockLanguageParser.T__1)
                self.state = 15
                localctx.target = self.match(MockLanguageParser.QubitId)
                pass
            elif token in [MockLanguageParser.T__2]:
                localctx = MockLanguageParser.CNOTGateContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 16
                localctx.name = self.match(MockLanguageParser.T__2)
                self.state = 17
                localctx.control = self.match(MockLanguageParser.QubitId)
                self.state = 18
                localctx.target = self.match(MockLanguageParser.QubitId)
                pass
            elif token in [MockLanguageParser.T__3]:
                localctx = MockLanguageParser.MzGateContext(self, localctx)
                self.enterOuterAlt(localctx, 4)
                self.state = 19
                localctx.name = self.match(MockLanguageParser.T__3)
                self.state = 20
                localctx.target = self.match(MockLanguageParser.QubitId)
                pass
            else:
                raise NoViableAltException(self)

        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx





