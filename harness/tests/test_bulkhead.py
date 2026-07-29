"""Completion-metric regressions for :mod:`harness.bulkhead`."""

import threading

import pytest

from harness.bulkhead import BulkheadRejected, ThreadPoolBulkhead


def test_successful_future_updates_success_metrics_and_result() -> None:
    bulkhead = ThreadPoolBulkhead(max_workers=1, max_queue_size=1)
    try:
        future = bulkhead.submit(lambda: "complete")

        assert future.result(timeout=0.5) == "complete"
        metrics = bulkhead.metrics()
        assert metrics.total_calls == 1
        assert metrics.successful_calls == 1
        assert metrics.rejected_calls == 0
        assert metrics.avg_wait_time >= 0.0
    finally:
        bulkhead.shutdown()


def test_failed_future_is_not_recorded_as_success_or_rejection() -> None:
    bulkhead = ThreadPoolBulkhead(max_workers=1, max_queue_size=1)

    def fail() -> None:
        raise ValueError("expected failure")

    try:
        future = bulkhead.submit(fail)

        with pytest.raises(ValueError, match="expected failure"):
            future.result(timeout=0.5)
        metrics = bulkhead.metrics()
        assert metrics.total_calls == 1
        assert metrics.successful_calls == 0
        assert metrics.rejected_calls == 0
        assert metrics.avg_wait_time == 0.0
    finally:
        bulkhead.shutdown()


def test_cancelled_future_is_not_recorded_as_success_or_rejection() -> None:
    bulkhead = ThreadPoolBulkhead(max_workers=1, max_queue_size=1)
    started = threading.Event()
    release = threading.Event()

    def block() -> None:
        started.set()
        assert release.wait(0.5)

    try:
        running = bulkhead.submit(block)
        assert started.wait(0.5)
        cancelled = bulkhead.submit(lambda: "cancelled")
        assert cancelled.cancel()

        release.set()
        running.result(timeout=0.5)
        metrics = bulkhead.metrics()
        assert metrics.total_calls == 2
        assert metrics.successful_calls == 1
        assert metrics.rejected_calls == 0
    finally:
        release.set()
        bulkhead.shutdown()


def test_capacity_rejection_does_not_count_as_success() -> None:
    bulkhead = ThreadPoolBulkhead(max_workers=1, max_queue_size=1)
    started = threading.Event()
    release = threading.Event()

    def block() -> None:
        started.set()
        assert release.wait(0.5)

    try:
        running = bulkhead.submit(block)
        assert started.wait(0.5)
        queued = bulkhead.submit(lambda: "queued")

        with pytest.raises(BulkheadRejected, match="at capacity"):
            bulkhead.submit(lambda: "rejected")

        release.set()
        running.result(timeout=0.5)
        assert queued.result(timeout=0.5) == "queued"
        metrics = bulkhead.metrics()
        assert metrics.total_calls == 3
        assert metrics.successful_calls == 2
        assert metrics.rejected_calls == 1
    finally:
        release.set()
        bulkhead.shutdown()
