# Examples: pyqir_generator

The `pyqir_generator` package provides the ability to generate
[QIR](https://github.com/qir-alliance/qir-spec) using a Python API.

It is intended to be used by code automating translation processes enabling the
conversion in some format to QIR via Python; i.e., this is a low level API
intended to be used as a bridge to existing Python frameworks enabling the
generation of QIR rather than directly consumed by an end-user. It is **not**
intended to be used as a framework for algorithm and application development.

This folder contains the following examples for how to use the `pyqir_generator`
package:

- **Bernstein-Vazirani example**: <br/>
  This example matches most closely how the `pyqir_generator` package is
  intended to be used. It consists of a [Python
  program](https://github.com/qir-alliance/pyqir/tree/main/examples/generator/mock_to_qir.py)
  that uses a "mini-compiler" for a made up [Mock
  language](https://github.com/qir-alliance/pyqir/tree/main/examples/generator/mock_language)
  to parse a program and then walks the created syntax tree to compile it into
  QIR. For simplicity, we used [Antlr](https://www.antlr.org/) to generate the
  parser based on the defined
  [grammar](https://github.com/qir-alliance/pyqir/tree/main/examples/generator/mock_language/MockLanguage.g4)
  and omitted any further compilation or optimization. Before running the example,
  please install the Antlr runtime:

  ```bash
  pip install antlr4-python3-runtime
  ```

  The example can then be run using python, with the generated QIR being outputted to a text file:

  ```bash
  python mock_to_qir.py bernstein_vazirani.txt 8 >> bernstein_vazirani_output.txt
  ```

- **Bell pair example**: <br/>
  This examples consists of a [single
  file](https://github.com/qir-alliance/pyqir/tree/main/examples/generator/bell_pair.py),
  and does not require any additional installation besides the `pyqir_generator`
  package itself. Please be aware that the PyQIR API is not intended to directly
  express quantum applications; its purpose is to be easily usable for *compiler
  and frontend developers* rather than *application developers* - as evidenced,
  e.g., by the fact that there is no `qubit` or `register` type defined within
  the API.

  The example can be run using python, with the generated QIR being outputted to a text file:

  ```bash
  python bell_pair.py >> bell_pair_output.txt
  ```
