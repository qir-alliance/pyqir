# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import List, Optional

import pyqir
from pyqir import Module, Function, Context, add_string_attribute, Linkage, FunctionType


def entry_point(
    module: Module,
    name: str,
    required_num_qubits: int,
    required_num_results: int,
    qir_profiles: str = "custom",
    output_labeling_schema: Optional[str] = None,
) -> Function:
    """
    Creates an entry point.

    :param Module module: The parent module.
    :param str name: The entry point name.
    :param int required_num_qubits: The number of qubits required by the entry point.
    :param int required_num_results: The number of results required by the entry point.
    :param Optional[str] qir_profiles: Value identifying the profile the entry point has been compiled for. Use base_profile when QIR is compliant.
    :param Optional[str] output_labeling_schema: An arbitrary string value that identifies the schema used by a compiler frontend that produced the IR to label the recorded output
    :returns: An entry point.
    """
    void = pyqir.Type.void(module.context)
    function = Function(FunctionType(void, []), Linkage.EXTERNAL, name, module)
    add_string_attribute(function, "entry_point")
    add_string_attribute(function, "required_num_qubits", str(required_num_qubits))
    add_string_attribute(function, "required_num_results", str(required_num_results))
    add_string_attribute(function, "qir_profiles", qir_profiles)

    add_string_attribute(
        function,
        "output_labeling_schema",
        output_labeling_schema,
    )

    return function
