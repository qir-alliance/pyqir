from pyqir.generator.module import SimpleModule
from pyqir.generator.qis import BasicQisBuilder
from pyqir_jit import NonadaptiveJit, GateLogger
from pyqir_jit.gateset import GateSet
import tempfile
import unittest


class IfTestCase(unittest.TestCase):
    def test_if(self) -> None:
        module = SimpleModule("test_if", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], lambda: qis.x(module.qubits[0]))

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, [])

    def test_if_not(self) -> None:
        module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], zero=lambda: qis.x(module.qubits[0]))

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, ["x qubit[0]"])

    def test_if_continue(self) -> None:
        module = SimpleModule("test_if", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], lambda: qis.x(module.qubits[0]))
        qis.h(module.qubits[0])

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, ["h qubit[0]"])

    def test_if_not_continue(self) -> None:
        module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], zero=lambda: qis.x(module.qubits[0]))
        qis.h(module.qubits[0])

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, ["x qubit[0]", "h qubit[0]"])

    def test_nested_if(self) -> None:
        module = SimpleModule("test_if", num_qubits=1, num_results=2)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.m(module.qubits[0], module.results[1])

        qis.if_result(
            module.results[0],
            lambda: qis.if_result(
                module.results[1],
                zero=lambda: qis.x(module.qubits[0])
            )
        )

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, [])

    def test_nested_if_not(self) -> None:
        module = SimpleModule("test_if", num_qubits=1, num_results=2)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.m(module.qubits[0], module.results[1])

        qis.if_result(
            module.results[0],
            zero=lambda: qis.if_result(
                module.results[1],
                zero=lambda: qis.x(module.qubits[0])
            )
        )

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, ["x qubit[0]"])

    def test_if_not_measured(self) -> None:
        module = SimpleModule(
            "test_if_not_measured", num_qubits=1, num_results=1
        )

        qis = BasicQisBuilder(module.builder)
        qis.if_result(module.results[0])

        # TODO: The generator should treat unmeasured results as zero.
        with self.assertRaises(BaseException):
            module.ir()


def _eval(module: SimpleModule, gates: GateSet) -> None:
    with tempfile.NamedTemporaryFile(suffix=".ll") as f:
        f.write(module.ir().encode("utf-8"))
        f.flush()
        NonadaptiveJit().eval(f.name, gates)


if __name__ == "__main__":
    unittest.main()
