project = "PyQIR"
copyright = "2021-2022 QIR Alliance"
author = "QIR Alliance"
html_theme = "alabaster"
exclude_patterns = ["_build"]

extensions = ["myst_parser", "sphinx.ext.autodoc", "sphinx.ext.intersphinx"]
myst_enable_extensions = ["colon_fence"]
myst_heading_anchors = 3
intersphinx_mapping = {"python": ("https://docs.python.org/3", None)}
