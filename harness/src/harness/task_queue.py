"""Task Queue - Queue-based execution for agents.

Provides priority queue with backpressure, persistence, and rate limiting.
"""

import asyncio
import time
from dataclasses import dataclass, field
from enum import Enum
from heapq import heapify, heappop, heappush
from typing import Any

from .id_utils import new_id


class TaskPriority(Enum):
    """Task priority levels."""

    CRITICAL = 1
    HIGH = 2
    NORMAL = 3
    LOW = 4
    BACKGROUND = 5


@dataclass
class Task:
    """A task in the queue."""

    id: str
    agent_name: str
    payload: Any
    priority: TaskPriority = TaskPriority.NORMAL
    created_at: float = field(default_factory=time.time)
    scheduled_at: float = 0.0
    started_at: float | None = None
    completed_at: float | None = None
    result: Any = None
    error: str | None = None
    retry_count: int = 0
    max_retries: int = 3
    metadata: dict = field(default_factory=dict)
    queue_order: int = 0

    def __lt__(self, other: Task) -> bool:
        """Compare for heap ordering (lower priority number = higher priority)."""
        if self.priority != other.priority:
            return self.priority.value < other.priority.value
        return self.queue_order < other.queue_order


class TaskQueue:
    """Priority queue for agent tasks with backpressure.

    Usage:
        queue = TaskQueue(max_size=100, rate_limit=10)

        # Add task
        task_id = await queue.submit(
            agent_name=\"coder-1\",
            payload={\"task\": \"write code\"},
            priority=TaskPriority.HIGH
        )

        # Process queue
        async for task in queue:
            result = await process(task)
            queue.complete(task.id, result)
    """

    def __init__(
        self,
        max_size: int = 1000,
        rate_limit: int = 10,  # tasks per second
        rate_window: float = 1.0,
    ):
        self._max_size = max_size
        self._rate_limit = rate_limit
        self._rate_window = rate_window

        self._heap: list[Task] = []
        self._tasks: dict[str, Task] = {}
        self._running: dict[str, Task] = {}  # Currently running
        self._completed: dict[str, Task] = {}

        self._lock = asyncio.Lock()
        self._event = asyncio.Event()

        self._rate_timestamps: list[float] = []
        self._paused = False
        self._next_queue_order = 0

    async def submit(
        self,
        agent_name: str,
        payload: Any,
        priority: TaskPriority = TaskPriority.NORMAL,
        max_retries: int = 3,
        metadata: dict | None = None,
    ) -> str:
        """Submit a task to the queue."""
        async with self._lock:
            # Check backpressure
            if len(self._tasks) >= self._max_size:
                raise QueueFullError(f"Queue full ({self._max_size})")

            # Check rate limit
            if not self._check_rate_limit():
                raise RateLimitError("Rate limit exceeded")

            task = Task(
                id=new_id(),
                agent_name=agent_name,
                payload=payload,
                priority=priority,
                max_retries=max_retries,
                metadata=metadata or {},
                queue_order=self._next_queue_order,
            )
            self._next_queue_order += 1

            heappush(self._heap, task)
            self._tasks[task.id] = task
            self._event.set()

            return task.id

    def _check_rate_limit(self) -> bool:
        """Check if we're within rate limits."""
        now = time.time()

        # Remove old timestamps
        cutoff = now - self._rate_window
        self._rate_timestamps = [t for t in self._rate_timestamps if t > cutoff]

        if len(self._rate_timestamps) >= self._rate_limit:
            return False

        self._rate_timestamps.append(now)
        return True

    async def get(self, timeout: float | None = None) -> Task | None:
        """Get next task from queue."""
        deadline = None if timeout is None else time.monotonic() + timeout

        while True:
            async with self._lock:
                if not self._paused and self._heap:
                    task = heappop(self._heap)
                    self._tasks.pop(task.id, None)
                    task.started_at = time.time()
                    self._running[task.id] = task
                    return task

                if deadline is not None:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        return None
                else:
                    remaining = None

                # Clear only after inspecting state while holding the lock so a
                # concurrent submit/resume cannot be missed between the check
                # and the wait below.
                self._event.clear()

            try:
                if remaining is None:
                    await self._event.wait()
                else:
                    await asyncio.wait_for(self._event.wait(), timeout=remaining)
            except asyncio.TimeoutError:
                return None

    async def __aiter__(self):
        """Async iterator for queue."""
        while True:
            task = await self.get(timeout=1.0)
            if task is None:
                break
            yield task

    def complete(self, task_id: str, result: Any) -> bool:
        """Mark task as completed."""
        if task_id not in self._running:
            return False

        task = self._running.pop(task_id)
        task.result = result
        task.completed_at = time.time()
        self._completed[task.id] = task
        return True

    def fail(self, task_id: str, error: str) -> bool:
        """Mark task as failed."""
        if task_id not in self._running:
            return False

        task = self._running.pop(task_id)

        # Check retry
        if task.retry_count < task.max_retries:
            task.retry_count += 1
            task.error = error
            # Re-queue
            self._tasks[task.id] = task
            heappush(self._heap, task)
            self._event.set()
        else:
            task.error = error
            task.completed_at = time.time()
            self._completed[task.id] = task

        return True

    def cancel(self, task_id: str) -> bool:
        """Cancel a task."""
        # Remove from pending (simple sync version)
        if task_id in self._tasks:
            del self._tasks[task_id]
            # Remove from heap
            self._heap = [t for t in self._heap if t.id != task_id]
            heapify(self._heap)
            return True

        if task_id in self._running:
            # Can't cancel running tasks
            return False

        return False

    def get_status(self) -> dict:
        """Get queue status."""
        return {
            "pending": len(self._tasks),
            "running": len(self._running),
            "completed": len(self._completed),
            "max_size": self._max_size,
            "rate_limit": self._rate_limit,
        }

    def get_task(self, task_id: str) -> Task | None:
        """Get task by ID."""
        return self._tasks.get(task_id) or self._running.get(task_id) or self._completed.get(task_id)

    def pause(self) -> None:
        """Pause queue processing."""
        self._paused = True

    def resume(self) -> None:
        """Resume queue processing."""
        self._paused = False
        self._event.set()


class QueueFullError(Exception):
    """Queue is full."""

    pass


class RateLimitError(Exception):
    """Rate limit exceeded."""

    pass
