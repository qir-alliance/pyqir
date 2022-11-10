project = "PyQIR"
copyright = "2021-2022 QIR Alliance"
author = "QIR Alliance"
html_theme = "alabaster"
exclude_patterns = ["_build"]

extensions = [
    "enum_tools.autoenum",
    "myst_parser",
    "sphinx.ext.autodoc",
    "sphinx.ext.intersphinx",
]

myst_enable_extensions = ["colon_fence"]
myst_heading_anchors = 3

intersphinx_mapping = {
    "sphinx": ("https://www.sphinx-doc.org/en/master", None),
    "myst-parser": ("https://myst-parser.readthedocs.io/en/latest/", None),
}

autodoc_type_aliases = {"Type": "Type"}
