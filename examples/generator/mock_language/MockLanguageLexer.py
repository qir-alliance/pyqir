# Generated from .\MockLanguage.g4 by ANTLR 4.9.1
from antlr4 import *
from io import StringIO
from typing.io import TextIO
import sys



def serializedATN():
    with StringIO() as buf:
        buf.write("\3\u608b\ua72a\u8133\ub9ed\u417c\u3be7\u7786\u5964\2\n")
        buf.write(":\b\1\4\2\t\2\4\3\t\3\4\4\t\4\4\5\t\5\4\6\t\6\4\7\t\7")
        buf.write("\4\b\t\b\4\t\t\t\3\2\3\2\3\3\3\3\3\4\3\4\3\4\3\4\3\4\3")
        buf.write("\5\3\5\3\5\3\6\6\6!\n\6\r\6\16\6\"\3\7\6\7&\n\7\r\7\16")
        buf.write("\7\'\3\7\3\7\3\b\3\b\3\b\3\b\7\b\60\n\b\f\b\16\b\63\13")
        buf.write("\b\3\b\3\b\3\t\3\t\3\t\3\t\2\2\n\3\3\5\4\7\5\t\6\13\7")
        buf.write("\r\b\17\t\21\n\3\2\5\3\2\62;\5\2\13\f\17\17\"\"\4\2\f")
        buf.write("\f\17\17\2<\2\3\3\2\2\2\2\5\3\2\2\2\2\7\3\2\2\2\2\t\3")
        buf.write("\2\2\2\2\13\3\2\2\2\2\r\3\2\2\2\2\17\3\2\2\2\2\21\3\2")
        buf.write("\2\2\3\23\3\2\2\2\5\25\3\2\2\2\7\27\3\2\2\2\t\34\3\2\2")
        buf.write("\2\13 \3\2\2\2\r%\3\2\2\2\17+\3\2\2\2\21\66\3\2\2\2\23")
        buf.write("\24\7z\2\2\24\4\3\2\2\2\25\26\7j\2\2\26\6\3\2\2\2\27\30")
        buf.write("\7e\2\2\30\31\7p\2\2\31\32\7q\2\2\32\33\7v\2\2\33\b\3")
        buf.write("\2\2\2\34\35\7o\2\2\35\36\7|\2\2\36\n\3\2\2\2\37!\t\2")
        buf.write("\2\2 \37\3\2\2\2!\"\3\2\2\2\" \3\2\2\2\"#\3\2\2\2#\f\3")
        buf.write("\2\2\2$&\t\3\2\2%$\3\2\2\2&\'\3\2\2\2\'%\3\2\2\2\'(\3")
        buf.write("\2\2\2()\3\2\2\2)*\b\7\2\2*\16\3\2\2\2+,\7\61\2\2,-\7")
        buf.write("\61\2\2-\61\3\2\2\2.\60\n\4\2\2/.\3\2\2\2\60\63\3\2\2")
        buf.write("\2\61/\3\2\2\2\61\62\3\2\2\2\62\64\3\2\2\2\63\61\3\2\2")
        buf.write("\2\64\65\b\b\2\2\65\20\3\2\2\2\66\67\13\2\2\2\678\3\2")
        buf.write("\2\289\b\t\2\29\22\3\2\2\2\6\2\"\'\61\3\2\3\2")
        return buf.getvalue()


class MockLanguageLexer(Lexer):

    atn = ATNDeserializer().deserialize(serializedATN())

    decisionsToDFA = [ DFA(ds, i) for i, ds in enumerate(atn.decisionToState) ]

    T__0 = 1
    T__1 = 2
    T__2 = 3
    T__3 = 4
    QubitId = 5
    Whitespace = 6
    Comment = 7
    Invalid = 8

    channelNames = [ u"DEFAULT_TOKEN_CHANNEL", u"HIDDEN" ]

    modeNames = [ "DEFAULT_MODE" ]

    literalNames = [ "<INVALID>",
            "'x'", "'h'", "'cnot'", "'mz'" ]

    symbolicNames = [ "<INVALID>",
            "QubitId", "Whitespace", "Comment", "Invalid" ]

    ruleNames = [ "T__0", "T__1", "T__2", "T__3", "QubitId", "Whitespace", 
                  "Comment", "Invalid" ]

    grammarFileName = "MockLanguage.g4"

    def __init__(self, input=None, output:TextIO = sys.stdout):
        super().__init__(input, output)
        self.checkVersion("4.9.1")
        self._interp = LexerATNSimulator(self, self.atn, self.decisionsToDFA, PredictionContextCache())
        self._actions = None
        self._predicates = None


