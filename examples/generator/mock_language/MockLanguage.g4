// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

// This grammar implements a Mock-language to illustrate 
// the use of PyQIR as part of a compiler toolchain.

grammar MockLanguage;

document : instructions=instruction* eof=EOF;

instruction
    : name='x' target=QubitId # XGate
    | name='h' target=QubitId # HGate
    | name='cnot' control=QubitId target=QubitId # CNOTGate
    | name='mz' target=QubitId # MzGate
    ;

QubitId : [0-9]+;

Whitespace : (' ' | '\n' | '\r' | '\t')+ -> channel(HIDDEN);

Comment : '//' ~('\n' | '\r')* -> channel(HIDDEN);

Invalid : . -> channel(HIDDEN);

