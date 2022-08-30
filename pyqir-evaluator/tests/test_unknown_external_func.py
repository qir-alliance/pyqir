# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.evaluator import GateLogger, NonadaptiveEvaluator
import pytest
import tempfile


def test_unknown_external_func() -> None:
    content = """
        ; ModuleID = 'test_unknown_external_func'
        source_filename = "test_unknown_external_func"

        %String = type opaque

        declare %String* @__quantum__rt__bool_to_string(i1)

        define void @main() #1 {
        entry:
            call %String* @__quantum__rt__bool_to_string(i1 1)
            ret void
        }

        attributes #1 = { "EntryPoint" }
    """

    evaluator = NonadaptiveEvaluator()
    logger = GateLogger()
    with pytest.raises(OSError) as excinfo:
        with tempfile.NamedTemporaryFile("wt", suffix=".ll") as fd:
            fd.write(content)
            fd.flush()
            evaluator.eval(fd.name, logger)
    assert (str(excinfo.value)
            == "Unsupported functions `__quantum__rt__bool_to_string`.")


def test_multiple_unknown_external_funcs() -> None:
    content = """
        ; ModuleID = 'test_unknown_external_func'
        source_filename = "test_unknown_external_func"

        %String = type opaque

        declare %String* @__quantum__rt__bool_to_string(i1)
        declare %String* @__quantum__rt__int_to_string(i1)

        define void @main() #1 {
        entry:
            call %String* @__quantum__rt__bool_to_string(i1 1)
            call %String* @__quantum__rt__int_to_string(i1 1)
            ret void
        }

        attributes #1 = { "EntryPoint" }
    """

    evaluator = NonadaptiveEvaluator()
    logger = GateLogger()
    with pytest.raises(OSError) as excinfo:
        with tempfile.NamedTemporaryFile("wt", suffix=".ll") as fd:
            fd.write(content)
            fd.flush()
            evaluator.eval(fd.name, logger)
    assert (str(excinfo.value)
            == "Unsupported functions `__quantum__rt__bool_to_string`, `__quantum__rt__int_to_string`.")
