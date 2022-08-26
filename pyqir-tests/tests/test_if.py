# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from abc import ABCMeta, abstractmethod
from pyqir.generator import BasicQisBuilder, IntPredicate, SimpleModule, const, types
from pyqir.evaluator import GateLogger, GateSet, NonadaptiveEvaluator
import pytest
import tempfile
from typing import Any, Callable, List, Optional


class _Brancher(metaclass=ABCMeta):
    """
    An abstraction that allows a single test case to test both boolean and
    result conditional branching.
    """

    @property
    @abstractmethod
    def module(self) -> SimpleModule:
        """The underlying module."""
        pass

    @abstractmethod
    def oracle(self) -> Any:
        """
        Queries an oracle to produce a nondeterminstic condition in the type
        that this brancher supports.
        """
        pass

    @abstractmethod
    def if_(
        self,
        cond: Any,
        true: Callable[[], None] = lambda: None,
        false: Callable[[], None] = lambda: None,
    ) -> None:
        """
        Branches on a condition, which must be the type that this brancher
        supports.
        """
        pass


class _ResultBrancher(_Brancher):
    def __init__(self, num_queries: int) -> None:
        self._module = SimpleModule("test_if", num_queries, num_queries)
        self._index = 0

    @property
    def module(self) -> SimpleModule:
        return self._module

    def oracle(self) -> Any:
        i = self._index
        self._index += 1
        qis = BasicQisBuilder(self._module.builder)
        qis.mz(self._module.qubits[i], self._module.results[i])
        return self._module.results[i]

    def if_(
        self,
        cond: Any,
        true: Callable[[], None] = lambda: None,
        false: Callable[[], None] = lambda: None,
    ) -> None:
        self._module.if_result(cond, true, false)


class _QisResultBrancher(_Brancher):
    def __init__(self, num_queries: int) -> None:
        self._brancher = _ResultBrancher(num_queries)

    @property
    def module(self) -> SimpleModule:
        return self._brancher.module

    def oracle(self) -> Any:
        return self._brancher.oracle()

    def if_(
        self,
        cond: Any,
        true: Callable[[], None] = lambda: None,
        false: Callable[[], None] = lambda: None,
    ) -> None:
        qis = BasicQisBuilder(self._brancher.module.builder)
        with pytest.deprecated_call():
            qis.if_result(cond, true, false)


class _BoolBrancher(_Brancher):
    def __init__(self, num_queries: int) -> None:
        self._brancher = _ResultBrancher(num_queries)
        self._read_result = self._brancher.module.add_external_function(
            "__quantum__qis__read_result__body", types.Function([types.RESULT], types.BOOL))

    @property
    def module(self) -> SimpleModule:
        return self._brancher.module

    def oracle(self) -> Any:
        result = self._brancher.oracle()
        return self._brancher.module.builder.call(self._read_result, [result])

    def if_(
        self,
        cond: Any,
        true: Callable[[], None] = lambda: None,
        false: Callable[[], None] = lambda: None,
    ) -> None:
        self._brancher.module.if_(cond, true, false)


def _result_branchers(num_queries: int) -> List[Callable[[], _Brancher]]:
    return [
        lambda: _ResultBrancher(num_queries),
        lambda: _QisResultBrancher(num_queries),
    ]


def _branchers(num_queries: int) -> List[Callable[[], _Brancher]]:
    return _result_branchers(num_queries) + [lambda: _BoolBrancher(num_queries)]


@pytest.fixture
def brancher(request: pytest.FixtureRequest) -> _Brancher:
    brancher = request.param()
    if isinstance(brancher, _Brancher):
        return brancher
    else:
        raise TypeError("Brancher factory didn't produce a brancher.")


def _eval(
    module: SimpleModule,
    gates: GateSet,
    result_stream: Optional[List[bool]] = None,
) -> None:
    with tempfile.NamedTemporaryFile(suffix=".ll") as f:
        f.write(module.ir().encode("utf-8"))
        f.flush()
        NonadaptiveEvaluator().eval(f.name, gates, None, result_stream)


@pytest.mark.parametrize("brancher", _branchers(1), indirect=True)
@pytest.mark.parametrize("result", [True, False])
def test_empty_blocks(brancher: _Brancher, result: bool) -> None:
    cond = brancher.oracle()
    brancher.if_(cond)

    logger = GateLogger()
    _eval(brancher.module, logger, [result])
    assert logger.instructions == ["m qubit[0] => out[0]"]


@pytest.mark.parametrize("brancher", _branchers(1), indirect=True)
def test_one_block_executes_on_one(brancher: _Brancher) -> None:
    cond = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)
    brancher.if_(cond, lambda: qis.x(brancher.module.qubits[0]))

    logger = GateLogger()
    _eval(brancher.module, logger, [True])
    assert logger.instructions == ["m qubit[0] => out[0]", "x qubit[0]"]


@pytest.mark.parametrize("brancher", _branchers(1), indirect=True)
def test_zero_block_executes_on_zero(brancher: _Brancher) -> None:
    cond = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)
    brancher.if_(cond, false=lambda: qis.x(brancher.module.qubits[0]))

    logger = GateLogger()
    _eval(brancher.module, logger)
    assert logger.instructions == ["m qubit[0] => out[0]", "x qubit[0]"]


@pytest.mark.parametrize("brancher", _branchers(1), indirect=True)
def test_execution_continues_after_hit_conditional_one(brancher: _Brancher) -> None:
    cond = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)
    brancher.if_(cond, lambda: qis.x(brancher.module.qubits[0]))
    qis.h(brancher.module.qubits[0])

    logger = GateLogger()
    _eval(brancher.module, logger, [True])
    assert logger.instructions == [
        "m qubit[0] => out[0]",
        "x qubit[0]",
        "h qubit[0]",
    ]


@pytest.mark.parametrize("brancher", _branchers(1), indirect=True)
def test_execution_continues_after_missed_conditional_one(brancher: _Brancher) -> None:
    cond = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)
    brancher.if_(cond, lambda: qis.x(brancher.module.qubits[0]))
    qis.h(brancher.module.qubits[0])

    logger = GateLogger()
    _eval(brancher.module, logger, [False])
    assert logger.instructions == ["m qubit[0] => out[0]", "h qubit[0]"]


@pytest.mark.parametrize("brancher", _branchers(1), indirect=True)
def test_execution_continues_after_hit_conditional_zero(brancher: _Brancher) -> None:
    cond = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)
    brancher.if_(cond, false=lambda: qis.x(brancher.module.qubits[0]))
    qis.h(brancher.module.qubits[0])

    logger = GateLogger()
    _eval(brancher.module, logger, [False])
    assert logger.instructions == [
        "m qubit[0] => out[0]",
        "x qubit[0]",
        "h qubit[0]",
    ]


@pytest.mark.parametrize("brancher", _branchers(1), indirect=True)
def test_execution_continues_after_missed_conditional_zero(brancher: _Brancher) -> None:
    cond = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)
    brancher.if_(cond, false=lambda: qis.x(brancher.module.qubits[0]))
    qis.h(brancher.module.qubits[0])

    logger = GateLogger()
    _eval(brancher.module, logger, [True])
    assert logger.instructions == ["m qubit[0] => out[0]", "h qubit[0]"]


@pytest.mark.parametrize("brancher", _branchers(1), indirect=True)
def test_execution_continues_after_conditional_if_else(brancher: _Brancher) -> None:
    cond = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)

    brancher.if_(
        cond,
        lambda: qis.x(brancher.module.qubits[0]),
        lambda: qis.y(brancher.module.qubits[0]),
    )

    qis.h(brancher.module.qubits[0])

    logger = GateLogger()
    _eval(brancher.module, logger)
    assert logger.instructions == [
        "m qubit[0] => out[0]",
        "y qubit[0]",
        "h qubit[0]",
    ]


@pytest.mark.parametrize("brancher", _branchers(2), indirect=True)
def test_nested_if(brancher: _Brancher) -> None:
    cond0 = brancher.oracle()
    cond1 = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)

    brancher.if_(
        cond0,
        lambda: brancher.if_(
            cond1,
            lambda: qis.x(brancher.module.qubits[0]),
        ),
    )

    logger = GateLogger()
    _eval(brancher.module, logger, [True, True])
    assert logger.instructions == [
        "m qubit[0] => out[0]",
        "m qubit[1] => out[1]",
        "x qubit[0]",
    ]


@pytest.mark.parametrize("brancher", _branchers(2), indirect=True)
def test_nested_if_not(brancher: _Brancher) -> None:
    cond0 = brancher.oracle()
    cond1 = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)

    brancher.if_(
        cond0,
        false=lambda: brancher.if_(
            cond1,
            false=lambda: qis.x(brancher.module.qubits[0]),
        ),
    )

    logger = GateLogger()
    _eval(brancher.module, logger, [False, False])
    assert logger.instructions == [
        "m qubit[0] => out[0]",
        "m qubit[1] => out[1]",
        "x qubit[0]",
    ]


@pytest.mark.parametrize("brancher", _branchers(2), indirect=True)
def test_nested_if_then_else(brancher: _Brancher) -> None:
    cond0 = brancher.oracle()
    cond1 = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)

    brancher.if_(
        cond0,
        true=lambda: brancher.if_(
            cond1,
            false=lambda: qis.x(brancher.module.qubits[0]),
        ),
    )

    logger = GateLogger()
    _eval(brancher.module, logger, [True, False])
    assert logger.instructions == [
        "m qubit[0] => out[0]",
        "m qubit[1] => out[1]",
        "x qubit[0]",
    ]


@pytest.mark.parametrize("brancher", _branchers(2), indirect=True)
def test_nested_else_then_if(brancher: _Brancher) -> None:
    cond0 = brancher.oracle()
    cond1 = brancher.oracle()
    qis = BasicQisBuilder(brancher.module.builder)

    brancher.if_(
        cond0,
        false=lambda: brancher.if_(
            cond1,
            true=lambda: qis.x(brancher.module.qubits[0]),
        ),
    )

    print(brancher.module.ir())
    logger = GateLogger()
    _eval(brancher.module, logger, [False, True])
    assert logger.instructions == [
        "m qubit[0] => out[0]",
        "m qubit[1] => out[1]",
        "x qubit[0]",
    ]


@pytest.mark.parametrize("brancher", _result_branchers(1), indirect=True)
def test_results_default_to_zero_if_not_measured(brancher: _Brancher) -> None:
    qis = BasicQisBuilder(brancher.module.builder)

    brancher.if_(
        brancher.module.results[0],
        true=lambda: qis.x(brancher.module.qubits[0]),
        false=lambda: qis.h(brancher.module.qubits[0]),
    )

    logger = GateLogger()
    _eval(brancher.module, logger)
    assert logger.instructions == ["h qubit[0]"]


def test_icmp_if_true() -> None:
    brancher = _BoolBrancher(1)
    x = brancher.oracle()
    module = brancher.module
    cond = module.builder.icmp(IntPredicate.EQ, x, const(types.Int(1), 0))

    qis = BasicQisBuilder(module.builder)
    brancher.if_(
        cond,
        lambda: qis.x(module.qubits[0]),
        lambda: qis.h(module.qubits[0]),
    )

    logger = GateLogger()
    _eval(brancher.module, logger, [False])
    assert logger.instructions == ["m qubit[0] => out[0]", "x qubit[0]"]


def test_icmp_if_false() -> None:
    brancher = _BoolBrancher(1)
    x = brancher.oracle()
    module = brancher.module
    cond = module.builder.icmp(IntPredicate.EQ, x, const(types.Int(1), 0))

    qis = BasicQisBuilder(brancher.module.builder)
    brancher.if_(
        cond,
        lambda: qis.x(module.qubits[0]),
        lambda: qis.h(module.qubits[0]),
    )

    logger = GateLogger()
    _eval(brancher.module, logger, [True])
    assert logger.instructions == ["m qubit[0] => out[0]", "h qubit[0]"]
