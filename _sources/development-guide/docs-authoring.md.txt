# Authoring PyQIR Documentation

On this page, we briefly describe some of the process for writing, maintaining, and improving PyQIR documentation.

## About Sphinx and MyST

PyQIR documentation is written using the [Sphinx documentation engine](https://www.sphinx-doc.org/), together with the [MyST variant of Markdown](https://myst-parser.readthedocs.io/). As an extension to Markdown, MyST includes all common Markdown syntax (e.g.: headings, emphasis, code blocks). In addition, MyST extends plain Markdown with additional syntax called _roles_ and _directives_ that allow Sphinx and its plugins to add new functionality to MyST.

In particular, _roles_ allow Sphinx to define new inline markup (within a paragraph), while _directives_ allow Sphinx to define new block markup (at the level of paragraphs). For instance, the `math` role can be used to embed LaTeX-style equations:

```md
Fresh qubits start in the {math}`\left|0\right\rangle` state.
```

Here, ``{math}`...` `` applies the `math` role to the contents of the immediately following backticks.

Similarly, directives can be used to typeset entire paragraphs. For example, to add a table of contents to a page, the `{toctree}` directive can be used:

````md
```{toctree}
---
maxdepth: 2
---

index
```
````

MyST directives can be specified with additional arguments via YAML headers, denoted by `---` at the start of directive contents.

For more details about Sphinx, please see {ref}`sphinx:contents`.
For more details on MyST additions to standard Markdown, please see {ref}`myst-parser:example_syntax`.

## Links in Sphinx documentation

While MyST supports traditional Markdown links, Sphinx provides additional roles for links that allow for verifying link targets, automatically generating link text from document and section headings, and other enhancements. Thus, when writing PyQIR documentation, we recommend the following:

| Link target | Syntax | Example |
|---|---|---|
| External page | `[link text](url)` | `[https://github.com/qir-alliance/]` |
| Page in PyQIR docs | ``{doc}`/path/from/docs-root` `` | ``{doc}`/development-guide/building` `` |
| Page in PyQIR docs (custom link text) | ``{doc}`Link text </path/from/docs-root>` `` | ``{doc}`Building PyQIR </development-guide/building>` `` |
| Section in PyQIR docs | ``{ref}`target_name` `` | ``{ref}`genindex` ``

```{note}
Some other Sphinx documentation sets (e.g.: NumPy, or the docs for MyST itself) may expose intersphinx metadata that allow for using `{ref}` roles to refer to those docsets; when appropriate, those should be added to the `conf.py` file for PyQIR documentation.
```

For more details, please see {ref}`myst-parser:syntax/referencing` and {ref}`myst-parser:syntax/targets`.
