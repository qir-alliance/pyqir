# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# pkgutil-style namespace package is required by Maturin.
# See: https://github.com/PyO3/maturin/issues/811
__path__ = __import__("pkgutil").extend_path(__path__, __name__)
