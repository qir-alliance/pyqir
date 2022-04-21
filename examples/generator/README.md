# Examples: pyqir-generator

The `pyqir-generator` package provides the ability to generate
[QIR](https://github.com/qir-alliance/qir-spec) using a Python API.

It is intended to be used by code automating translation processes enabling the
conversion in some format to QIR via Python; i.e., this is a low level API
intended to be used as a bridge to existing Python frameworks enabling the
generation of QIR rather than directly consumed by an end-user. It is **not**
intended to be used as a framework for algorithm and application development.

This folder contains the following examples for how to use the `pyqir-generator`
package:

## Bernstein-Vazirani

This example matches most closely how the `pyqir-generator` package is intended
to be used. It consists of a [Python program](mock_to_qir.py) that uses a
"mini-compiler" for a made up [Mock language](mock_language) to parse a program
and then walks the created syntax tree to compile it into QIR. For simplicity,
we used [ANTLR](https://www.antlr.org/) to generate the parser based on the
defined [grammar](mock_language/MockLanguage.g4) and omitted any further
compilation or optimization. Before running the example, please install the
ANTLR runtime:

```bash
pip install antlr4-python3-runtime
```

The example can then be run using python, with the generated QIR being written
to a text file:

```bash
python mock_to_qir.py bernstein_vazirani.txt 7 >> bernstein_vazirani_output.txt
```

## Bell pair

This examples consists of a [single file](bell_pair.py), and does not require
any additional installation besides the `pyqir-generator` package itself. Please
be aware that the PyQIR API is not intended to directly express quantum
applications; its purpose is to be easily usable for *compiler and frontend
developers* rather than *application developers*.

The example can be run using python, with the generated QIR being written to a
text file:

```bash
python bell_pair.py >> bell_pair_output.txt
```

## Branching on measurement results

PyQIR supports simple branching based on the result of measuring a qubit.
[if.py](if.py) shows how to use this feature to execute certain gates
conditionally.

Note that the branch condition must be a single result. Conditions based on
classically computed booleans are not currently supported. However, conjunctions
and disjunctions of results can be emulated by chaining multiple branches
together.

## External functions

[external_functions.py](external_functions.py) shows how to call external
functions using PyQIR. The functions are declared in the module with the type of
its parameters and return value, but without an implementation, so it can be
linked with a separate library at compile time.

Note that it's not currently possible to use the return value of an external
function in subsequent instructions.

## Python subset to QIR

[python2qir.py](python2qir.py) transforms a subset of the Python language into QIR, by using:
- the built-in `ast` (Asbtract Syntax Tree) library to parse the source code
- the `pyqir-generator` package to generate and display QIR

Here, we transform a Qiskit circuit without using the Qiskit package.


