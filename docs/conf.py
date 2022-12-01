project = "PyQIR"
copyright = "2021-2022 QIR Alliance"
author = "QIR Alliance"
html_theme = "furo"
exclude_patterns = ["_build"]
extensions = ["myst_parser", "sphinx.ext.autodoc", "sphinx.ext.intersphinx"]

autodoc_default_options = {
    "members": None,
    "undoc-members": True,
    "show-inheritance": True,
}
intersphinx_mapping = {"python": ("https://docs.python.org/3", None)}
myst_heading_anchors = 3
