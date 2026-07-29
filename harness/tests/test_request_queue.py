"""Behavioral coverage for the async request queue."""

import asyncio

import pytest

from harness.request_queue import Priority, QueueFull, RequestQueue


def test_dequeue_uses_max_wait_when_timeout_is_omitted_or_none() -> None:
    async def scenario() -> None:
        queue = RequestQueue(max_wait=0.2)

        omitted = asyncio.create_task(queue.dequeue())
        explicit_none = asyncio.create_task(queue.dequeue(timeout=None))
        await asyncio.sleep(0)
        assert not omitted.done()
        assert not explicit_none.done()

        await queue.enqueue("first")
        await queue.enqueue("second")
        assert (await omitted).payload == "first"
        assert (await explicit_none).payload == "second"

    asyncio.run(scenario())


def test_dequeue_zero_timeout_is_immediate() -> None:
    async def scenario() -> None:
        queue = RequestQueue(max_wait=1)
        assert await asyncio.wait_for(queue.dequeue(timeout=0), timeout=0.1) is None
        assert queue.metrics() == {"enqueued": 0, "dequeued": 0, "rejected": 0, "expired": 0}

    asyncio.run(scenario())


def test_dequeue_positive_timeout_overrides_max_wait() -> None:
    async def scenario() -> None:
        queue = RequestQueue(max_wait=1)
        assert await asyncio.wait_for(queue.dequeue(timeout=0.01), timeout=0.1) is None

    asyncio.run(scenario())


def test_dequeue_preserves_priority_fifo_and_metrics() -> None:
    async def scenario() -> None:
        queue = RequestQueue(max_size=3)
        await queue.enqueue("normal-one", Priority.NORMAL)
        await queue.enqueue("high", Priority.HIGH)
        await queue.enqueue("normal-two", Priority.NORMAL)

        assert (await queue.dequeue()).payload == "high"
        assert (await queue.dequeue()).payload == "normal-one"
        assert (await queue.dequeue()).payload == "normal-two"
        assert queue.metrics() == {"enqueued": 3, "dequeued": 3, "rejected": 0, "expired": 0}

    asyncio.run(scenario())


def test_enqueue_full_queue_preserves_rejection_metric() -> None:
    async def scenario() -> None:
        queue = RequestQueue(max_size=1)
        await queue.enqueue("accepted")
        with pytest.raises(QueueFull, match="Queue at capacity"):
            await queue.enqueue("rejected")
        assert queue.metrics() == {"enqueued": 1, "dequeued": 0, "rejected": 1, "expired": 0}

    asyncio.run(scenario())
