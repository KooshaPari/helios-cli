"""Completion and bookkeeping regressions for :mod:`harness.worker_pool`."""

import threading
import time

import pytest

from harness.worker_pool import WorkerPool


def test_submit_future_and_callback_observe_completed_result() -> None:
    pool = WorkerPool(num_workers=1)
    pool.start()
    callback_done = threading.Event()
    callbacks: list[tuple[object, object]] = []
    try:
        future = pool.submit(lambda value: value * 2, 21)
        task_id = pool.submit_callback(
            lambda: "callback-result",
            lambda result, error: (callbacks.append((result, error)), callback_done.set()),
        )

        assert future.result(timeout=0.5) == 42
        assert task_id
        assert callback_done.wait(0.5)
        assert callbacks == [("callback-result", None)]
        assert pool.wait_completion(timeout=0.5)
    finally:
        pool.shutdown()


def test_submit_future_and_callback_propagate_execution_exception() -> None:
    pool = WorkerPool(num_workers=1)
    pool.start()
    callback_done = threading.Event()
    callbacks: list[tuple[object, object]] = []

    def fail() -> None:
        raise ValueError("expected failure")

    try:
        future = pool.submit(fail)
        pool.submit_callback(
            fail,
            lambda result, error: (callbacks.append((result, error)), callback_done.set()),
        )

        with pytest.raises(ValueError, match="expected failure"):
            future.result(timeout=0.5)
        assert callback_done.wait(0.5)
        assert callbacks[0][0] is None
        assert isinstance(callbacks[0][1], ValueError)
        assert pool.wait_completion(timeout=0.5)
        assert pool.metrics().failed_tasks == 2
    finally:
        pool.shutdown()


def test_wait_completion_includes_active_work_and_honors_timeout() -> None:
    pool = WorkerPool(num_workers=1)
    pool.start()
    started = threading.Event()
    release = threading.Event()

    def blocked() -> str:
        started.set()
        assert release.wait(0.5)
        return "released"

    try:
        future = pool.submit(blocked)
        assert started.wait(0.5)
        assert not pool.wait_completion(timeout=0.02)
        release.set()
        assert future.result(timeout=0.5) == "released"
        assert pool.wait_completion(timeout=0.5)
    finally:
        release.set()
        pool.shutdown()


def test_shutdown_drains_accepted_work_and_balances_queue_bookkeeping() -> None:
    pool = WorkerPool(num_workers=1)
    pool.start()
    future = pool.submit(lambda: "done")

    pool.shutdown(wait=True)

    assert future.result(timeout=0.1) == "done"
    assert pool._task_queue.unfinished_tasks == 0
