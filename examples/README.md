# PyQIR Examples

PyQIR generates, evaluates, and parses
[Quantum Intermediate Representation (QIR)](https://github.com/qir-alliance/qir-spec).

Code generation easily integrates the QIR toolchain into existing Python-based
frontends. It's intended to be used by code automating translation processes
enabling the conversion in some format to QIR via Python; i.e., this is a
low-level API intended to be used as a bridge to existing Python frameworks
enabling the generation of QIR rather than directly consumed by an end-user. It
is **not** intended to be used as a framework for algorithm and application
development.

Evaluation supports
[just-in-time compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation)
as well a mechanism to define what actions to perform when a gate is applied.
It's intended for easily testing and experimenting with QIR code and connecting
it to low-level Python-based lab software such as
[QCoDeS](https://qcodes.github.io/Qcodes/examples/15_minutes_to_QCoDeS.html#Introduction).

## Installation

For more information about how to install the PyQIR packages to run the examples, see the [docs](https://www.qir-alliance.org/pyqir/).

## Generating Bernstein-Vazirani

This example consists of a [Python program](mock_to_qir.py) that uses a
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

## Parsing QIR gate instructions

This example shows how to build a [Python program](bernstein_vazirani.py),
get its `bitcode`, load into another `Module`, analyze the `QIR`, and
finally print out a simple log of the quantum gates applied during execution.

The example can be run using python:

```bash
python bernstein_vazirani.py
```

## Bell pair

This examples consists of a [single file](bell_pair.py), and does not require
any additional installation besides PyQIR itself. Please be aware that the PyQIR
API is not intended to directly express quantum applications; its purpose is to
be easily usable for *compiler and frontend developers* rather than *application
developers*.

The example can be run using python, with the generated QIR being written to a
text file:

```bash
python bell_pair.py >> bell_pair_output.txt
```

## Branching

PyQIR supports branching on boolean conditions as shown in [if_bool.py](if_bool.py).
It's also possible to use a measurement result as the condition as shown in [if_result.py](if_result.py).

## External functions

[external_functions.py](external_functions.py) shows how to call external
functions using PyQIR. The functions are declared in the module with the type of
its parameters and return value, but without an implementation, so it can be
linked with a separate library at compile time.

Note that it's not currently possible to use the return value of an external
function in subsequent instructions.

## Python subset to QIR

[python2qir.py](python2qir.py) transforms a subset of the Python language into QIR, by using:

- the built-in `ast` (Abstract Syntax Tree) library to parse the source code
- PyQIR to generate and display QIR

Here, we transform a Qiskit circuit without using the Qiskit package.

## Teleport

Genarates a quantum teleportation example and prints out the generated QIR.

The example can be run using python:

```bash
python teleport.py
```
