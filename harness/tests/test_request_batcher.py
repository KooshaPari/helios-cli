"""Regression tests for asynchronous and synchronous request batching."""

import asyncio
import concurrent.futures
import threading
import time

import pytest

from harness.request_batcher import BatchError, RequestBatcher, SyncRequestBatcher


def assert_no_background_tasks() -> None:
    """Fail if the batcher left an unfinished task in this event loop."""
    current_task = asyncio.current_task()
    assert not [
        task
        for task in asyncio.all_tasks()
        if task is not current_task and not task.done()
    ]


def test_below_size_batch_flushes_after_configured_timeout() -> None:
    """A single submit must not wait indefinitely for a full batch."""

    async def scenario() -> None:
        calls: list[list[int]] = []

        async def processor(items: list[int]) -> list[int]:
            calls.append(items)
            return [item * 2 for item in items]

        batcher = RequestBatcher(processor, batch_size=2, flush_timeout=0.02)
        assert await asyncio.wait_for(batcher.submit("one", 3), timeout=0.5) == 6
        assert calls == [[3]]
        await asyncio.sleep(0)
        assert batcher._flush_task is None
        assert_no_background_tasks()

    asyncio.run(scenario())


def test_cancelled_pending_submit_is_removed_without_timer_leak() -> None:
    """Cancelling the only queued caller removes it and stops its timer."""

    async def scenario() -> None:
        calls: list[list[int]] = []

        async def processor(items: list[int]) -> list[int]:
            calls.append(items)
            return items

        batcher = RequestBatcher(processor, batch_size=2, flush_timeout=0.02)
        pending = asyncio.create_task(batcher.submit("cancel", 1))
        await asyncio.sleep(0)
        pending.cancel()
        with pytest.raises(asyncio.CancelledError):
            await pending

        await asyncio.sleep(0.05)
        assert calls == []
        assert batcher.get_stats()["queue_size"] == 0
        assert batcher._flush_task is None
        assert_no_background_tasks()

    asyncio.run(scenario())


def test_processor_error_resets_lifecycle_for_later_timeout_batch() -> None:
    """A failed timeout flush must not strand later below-size submissions."""

    async def scenario() -> None:
        attempts = 0

        async def processor(items: list[int]) -> list[int]:
            nonlocal attempts
            attempts += 1
            if attempts == 1:
                raise RuntimeError("expected processor failure")
            return items

        batcher = RequestBatcher(processor, batch_size=2, flush_timeout=0.02)
        with pytest.raises(RuntimeError, match="expected processor failure"):
            await asyncio.wait_for(batcher.submit("bad", 1), timeout=0.5)
        assert await asyncio.wait_for(batcher.submit("good", 2), timeout=0.5) == 2
        assert batcher.get_stats()["total_errors"] == 1
        assert batcher.get_stats()["total_batches"] == 1
        await asyncio.sleep(0)
        assert batcher._flush_task is None
        assert_no_background_tasks()

    asyncio.run(scenario())


def test_full_batch_cancels_timeout_and_processes_concurrent_submits_once() -> None:
    """Concurrent submitters form one full batch without retaining a timer."""

    async def scenario() -> None:
        calls: list[list[int]] = []

        async def processor(items: list[int]) -> list[int]:
            calls.append(items)
            await asyncio.sleep(0)
            return [item + 1 for item in items]

        batcher = RequestBatcher(processor, batch_size=3, flush_timeout=1)
        results = await asyncio.gather(
            batcher.submit("one", 1),
            batcher.submit("two", 2),
            batcher.submit("three", 3),
        )
        assert results == [2, 3, 4]
        assert calls == [[1, 2, 3]]
        await asyncio.sleep(0)
        assert batcher._flush_task is None
        assert_no_background_tasks()

    asyncio.run(scenario())


def test_full_remainder_submitted_while_processing_is_flushed_immediately() -> None:
    """A second full batch must not wait for the partial-batch timeout."""

    async def scenario() -> None:
        started = asyncio.Event()
        release = asyncio.Event()
        calls: list[list[int]] = []

        async def processor(items: list[int]) -> list[int]:
            calls.append(items)
            if len(calls) == 1:
                started.set()
                await release.wait()
            return items

        batcher = RequestBatcher(processor, batch_size=2, flush_timeout=1)
        first = [
            asyncio.create_task(batcher.submit("one", 1)),
            asyncio.create_task(batcher.submit("two", 2)),
        ]
        await asyncio.wait_for(started.wait(), timeout=0.5)
        second = [
            asyncio.create_task(batcher.submit("three", 3)),
            asyncio.create_task(batcher.submit("four", 4)),
        ]
        release.set()

        assert await asyncio.wait_for(asyncio.gather(*first, *second), timeout=0.5) == [
            1,
            2,
            3,
            4,
        ]
        assert calls == [[1, 2], [3, 4]]
        await asyncio.sleep(0)
        assert batcher._flush_task is None
        assert_no_background_tasks()

    asyncio.run(scenario())


def test_sync_below_size_batch_flushes_after_configured_timeout() -> None:
    """A synchronous undersized request is released by its single timer."""
    calls: list[list[int]] = []
    batcher = SyncRequestBatcher(
        lambda items: calls.append(items) or [item * 2 for item in items],
        batch_size=2,
        flush_timeout=0.02,
    )

    assert batcher.submit("one", 3) == 6
    assert calls == [[3]]
    assert batcher._flush_timer is None


def test_sync_concurrent_full_batch_maps_each_result_to_its_submitter() -> None:
    """Concurrent full batches are detached before processor execution."""
    calls: list[list[int]] = []
    barrier = threading.Barrier(3)
    batcher = SyncRequestBatcher(
        lambda items: calls.append(items) or [item * 10 for item in items],
        batch_size=3,
        flush_timeout=1,
    )

    def submit(item: int) -> tuple[int, int]:
        barrier.wait()
        return item, batcher.submit(str(item), item)

    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
        results = dict(executor.map(submit, [1, 2, 3]))

    assert results == {1: 10, 2: 20, 3: 30}
    assert len(calls) == 1
    assert sorted(calls[0]) == [1, 2, 3]
    assert batcher._flush_timer is None


def test_sync_processor_exception_does_not_strand_later_requests() -> None:
    """A processor error is delivered and the next request can recover."""
    attempts = 0

    def processor(items: list[int]) -> list[int]:
        nonlocal attempts
        attempts += 1
        if attempts == 1:
            raise RuntimeError("expected processor failure")
        return items

    batcher = SyncRequestBatcher(processor, batch_size=1, flush_timeout=1)
    with pytest.raises(RuntimeError, match="expected processor failure"):
        batcher.submit("bad", 1)
    assert batcher.submit("good", 2) == 2
    assert batcher._flush_timer is None


def test_sync_short_processor_results_raise_batch_error() -> None:
    """Every request receives either its result or a deterministic BatchError."""
    batcher = SyncRequestBatcher(
        lambda items: items[:1], batch_size=2, flush_timeout=1
    )
    barrier = threading.Barrier(2)

    def submit(item: int) -> int:
        barrier.wait()
        return batcher.submit(str(item), item)

    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        futures = [executor.submit(submit, item) for item in [1, 2]]
        outcomes = []
        for future in futures:
            try:
                outcomes.append(future.result())
            except BatchError:
                outcomes.append("batch-error")

    assert outcomes.count("batch-error") == 1
    assert len([outcome for outcome in outcomes if outcome != "batch-error"]) == 1


def test_sync_timer_full_and_explicit_flush_do_not_duplicate_processing() -> None:
    """A full batch cancels its timer; later explicit flush is a no-op."""
    calls: list[list[int]] = []
    batcher = SyncRequestBatcher(
        lambda items: calls.append(items) or items,
        batch_size=2,
        flush_timeout=0.05,
    )

    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor:
        pending = executor.submit(batcher.submit, "one", 1)
        deadline = time.monotonic() + 0.5
        while not batcher._queue and time.monotonic() < deadline:
            time.sleep(0.001)
        assert batcher._queue
        assert batcher.submit("two", 2) == 2
        assert pending.result(timeout=0.5) == 1

    batcher.flush()
    time.sleep(0.08)
    assert calls == [[1, 2]]
    assert batcher._flush_timer is None
