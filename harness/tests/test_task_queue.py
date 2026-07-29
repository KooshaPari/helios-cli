"""Lifecycle and waiting regressions for :mod:`harness.task_queue`."""

import asyncio

import pytest

from harness.task_queue import QueueFullError, RateLimitError, TaskPriority, TaskQueue


def test_terminal_tasks_release_capacity_and_retries_requeue() -> None:
    async def scenario() -> None:
        queue = TaskQueue(max_size=1, rate_limit=10)
        first = await queue.submit("agent", "first", max_retries=1)
        with pytest.raises(QueueFullError):
            await queue.submit("agent", "blocked")

        task = await queue.get()
        assert task is not None and task.id == first
        assert queue.get_status()["pending"] == 0
        assert queue.fail(first, "transient")
        assert queue.get_status()["pending"] == 1
        retry = await queue.get()
        assert retry is not None and retry.id == first and retry.retry_count == 1
        assert queue.complete(first, "done")
        assert queue.get_status() == {
            "pending": 0,
            "running": 0,
            "completed": 1,
            "max_size": 1,
            "rate_limit": 10,
        }
        second = await queue.submit("agent", "second")
        assert second != first

        terminal = TaskQueue(max_size=1, rate_limit=10)
        terminal_id = await terminal.submit("agent", "terminal", max_retries=0)
        assert (await terminal.get()).id == terminal_id
        assert terminal.fail(terminal_id, "permanent")
        assert terminal.get_status()["completed"] == 1
        assert await terminal.submit("agent", "admitted-after-terminal-failure")

    asyncio.run(scenario())


def test_get_none_waits_until_submit_and_finite_timeout_returns_none() -> None:
    async def scenario() -> None:
        queue = TaskQueue()
        waiting = asyncio.create_task(queue.get(timeout=None))
        await asyncio.sleep(0.02)
        assert not waiting.done()
        task_id = await queue.submit("agent", "payload")
        task = await asyncio.wait_for(waiting, timeout=0.2)
        assert task is not None and task.id == task_id
        assert await queue.get(timeout=0.01) is None

    asyncio.run(scenario())


def test_priority_fifo_cancel_rate_limit_and_pause() -> None:
    async def scenario() -> None:
        queue = TaskQueue(rate_limit=2, rate_window=60)
        normal_one = await queue.submit("agent", 1)
        high = await queue.submit("agent", 2, priority=TaskPriority.HIGH)
        with pytest.raises(RateLimitError):
            await queue.submit("agent", 3)
        assert queue.cancel(normal_one)
        assert not queue.cancel("missing")
        queue.pause()
        blocked = asyncio.create_task(queue.get(timeout=0.05))
        assert await blocked is None
        queue.resume()
        task = await queue.get(timeout=0.05)
        assert task is not None and task.id == high

        fifo = TaskQueue(rate_limit=10)
        first = await fifo.submit("agent", 1)
        second = await fifo.submit("agent", 2)
        assert (await fifo.get()).id == first
        assert (await fifo.get()).id == second

    asyncio.run(scenario())
