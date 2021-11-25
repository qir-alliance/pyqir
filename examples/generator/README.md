# Examples: pyqir_generator

The `pyqir_generator` package provides the ability to generate [QIR](https://github.com/qir-alliance/qir-spec) using a Python API.

It is intended to be used by code automating translation processes enabling the conversion in some format to QIR via Python; i.e., this is a low level API intended to be used as a bridge to existing Python frameworks enabling the generation of QIR rather than directly consumed by an end-user. It is **not** intended to be used as a framework for algorithm and application development.

This folder contains two examples for how to use the `pyqir_generator` package:  
    - [Bernstein-Vazirani](mock_to_qir.py), give the Mock language [program description](bernstein_vazirani.txt)  
    - [Bell pair](bell_pair.py)

The **Bernstein-Vazirani example** matches most closely how the `pyqir_generator` package is intended to be used. It consists of a [Python program](mock_to_qir.py) that uses a "mini-compiler" for a made up [Mock language](mock_language) to parse a program and then walks the created syntax tree to compile it into QIR.
For simplicity, we used [Antlr](https://www.antlr.org/) to generate the parser based on the defined [grammar](mock_language/MockLanguage.g4) and omitted any further compilation or optimization. To run the example, please install the Antlr runtime:
```
pip install antlr4-python3-runtime
```

The **Bell pair example** consists of a [single file](bell_pair.py), and does not require any additional installation besides the `pyqir_generator` package itself. Please be aware that the PyQIR API is not intended to directly express quantum applications; its purpose is to be easily usable for *compiler developers* rather than *application developers* - as evidenced, e.g., by the fact that there is no `qubit` or `register` type defined within the API.