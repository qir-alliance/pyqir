from pyqir_generator.instruction import *
from pyqir_generator.module import Module, Register
from pyqir_jit import NonadaptiveJit, GateLogger
from pyqir_jit.gateset import GateSet
import tempfile
import unittest


class EvalTestCase(unittest.TestCase):
    def test_if(self) -> None:
        module = Module(
            name="test_if",
            bits=[Register("r", 1)],
            qubits=[Register("q", 1)],
            instructions=[If("r0", [X("q0")])]
        )

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, [])

    def test_if_not(self) -> None:
        module = Module(
            name="test_if_not",
            bits=[Register("r", 1)],
            qubits=[Register("q", 1)],
            instructions=[If("r0", false=[X("q0")])]
        )

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, ["x qubit[0]"])

    def test_if_continue(self) -> None:
        module = Module(
            name="test_if",
            bits=[Register("r", 1)],
            qubits=[Register("q", 1)],
            instructions=[
                If("r0", [X("q0")]),
                H("q0"),
            ]
        )

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, ["h qubit[0]"])

    def test_if_not_continue(self) -> None:
        module = Module(
            name="test_if_not",
            bits=[Register("r", 1)],
            qubits=[Register("q", 1)],
            instructions=[
                If("r0", false=[X("q0")]),
                H("q0"),
            ]
        )

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, ["x qubit[0]", "h qubit[0]"])

    def test_nested_if(self) -> None:
        module = Module(
            name="test_if",
            bits=[Register("r", 2)],
            qubits=[Register("q", 1)],
            instructions=[
                If("r0", [
                    If("r1", false=[X("q0")])
                ])
            ]
        )

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, [])

    def test_nested_if_not(self) -> None:
        module = Module(
            name="test_if",
            bits=[Register("r", 2)],
            qubits=[Register("q", 1)],
            instructions=[
                If("r0", false=[
                    If("r1", false=[X("q0")])
                ])
            ]
        )

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, ["x qubit[0]"])


def _eval(module: Module, gates: GateSet) -> None:
    with tempfile.TemporaryFile(suffix=".ll") as f:
        f.write(module.ir().encode("utf-8"))
        NonadaptiveJit().eval(f.name, gates)


if __name__ == "__main__":
    unittest.main()
