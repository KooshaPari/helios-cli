"""Request batching for optimizing network calls.

Provides utilities to batch multiple requests together to reduce network overhead.
"""

import asyncio
import logging
import threading
import time
from collections import deque
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Generic, TypeVar

logger = logging.getLogger(__name__)


T = TypeVar("T")
R = TypeVar("R")


@dataclass
class BatchRequest(Generic[T]):
    """A single request in a batch."""

    id: str
    args: tuple = field(default_factory=tuple)
    kwargs: dict = field(default_factory=dict)
    future: asyncio.Future | None = None
    timestamp: float = field(default_factory=time.time)


@dataclass
class BatchResponse(Generic[R]):
    """Response for a batched request."""

    request_id: str
    result: R | None = None
    error: Exception | None = None
    latency_ms: float = 0.0


class RequestBatcher(Generic[T, R]):
    """Batches multiple requests for efficient processing.

    Usage:
        async def process_batch(items):
            # Process all items together
            return [process(item) for item in items]

        batcher = RequestBatcher(process_batch, batch_size=10, flush_timeout=0.1)

        # Queue requests
        result1 = await batcher.submit("req1", item1)
        result2 = await batcher.submit("req2", item2)
    """

    def __init__(
        self,
        processor: Callable[[list[T]], list[R]],
        batch_size: int = 10,
        flush_timeout: float = 0.1,
        max_queue_size: int = 1000,
    ):
        self._processor = processor
        self._batch_size = batch_size
        self._flush_timeout = flush_timeout
        self._max_queue_size = max_queue_size

        self._queue: deque[BatchRequest[T]] = deque()
        self._lock = threading.Lock()
        self._processing = False
        self._flush_task: asyncio.Task[None] | None = None

        # Metrics
        self._total_requests = 0
        self._total_batches = 0
        self._total_errors = 0

    async def submit(self, request_id: str, *args, **kwargs) -> R:
        """Submit a request to be batched."""
        # Check queue size
        with self._lock:
            if len(self._queue) >= self._max_queue_size:
                raise QueueFullError(f"Queue full ({self._max_queue_size})")

            future = asyncio.Future()
            request = BatchRequest(
                id=request_id,
                args=args,
                kwargs=kwargs,
                future=future,
            )
            self._queue.append(request)
            self._total_requests += 1
            should_flush = len(self._queue) >= self._batch_size
            if not should_flush:
                self._schedule_timeout_flush_locked()

        # Trigger flush if batch is full
        if should_flush:
            asyncio.create_task(self._flush())

        try:
            return await future
        except asyncio.CancelledError:
            # A cancelled caller must not leave an unflushable request behind.
            with self._lock:
                try:
                    self._queue.remove(request)
                except ValueError:
                    # The request is already being processed; its cancelled
                    # future is safely ignored when results are mapped back.
                    pass
                else:
                    if not self._queue:
                        self._cancel_timeout_flush_locked()
            raise

    def _schedule_timeout_flush_locked(self) -> None:
        """Schedule one timeout flush while the queue is non-empty.

        The caller must hold ``_lock``.  Keeping a single timer prevents a
        steady stream of below-size submissions from accumulating background
        tasks, while still giving the first queued request its timeout bound.
        """
        if self._flush_task is None or self._flush_task.done():
            self._flush_task = asyncio.create_task(self._flush_after_timeout())

    def _cancel_timeout_flush_locked(self) -> None:
        """Cancel a pending timeout timer, unless it is the current task."""
        flush_task = self._flush_task
        if flush_task is None:
            return

        self._flush_task = None
        if flush_task is not asyncio.current_task():
            flush_task.cancel()

    async def _flush_after_timeout(self) -> None:
        """Flush pending work after the configured batching delay."""
        try:
            await asyncio.sleep(self._flush_timeout)
            await self._flush()
        finally:
            with self._lock:
                if self._flush_task is asyncio.current_task():
                    self._flush_task = None

    async def _flush(self) -> None:
        """Process current batch."""
        with self._lock:
            if self._processing:
                return

            if not self._queue:
                return

            # Take batch
            batch = []
            while self._queue and len(batch) < self._batch_size:
                batch.append(self._queue.popleft())

            self._processing = True
            self._cancel_timeout_flush_locked()

        # Process batch
        try:
            args_list = [req.args for req in batch]
            kwargs_list = [req.kwargs for req in batch]

            # Call processor with all args
            # Simplified: assume processor takes list of first args
            first_args = [req.args[0] if req.args else None for req in batch]
            results = await self._processor(first_args)

            # Map results back
            for i, req in enumerate(batch):
                if req.future and not req.future.done():
                    if i < len(results):
                        req.future.set_result(results[i])
                    else:
                        req.future.set_exception(BatchError("No result"))

            self._total_batches += 1

        except Exception as e:
            logger.error(f"Batch processing error: {e}")
            self._total_errors += 1

            # Set exception for all
            for req in batch:
                if req.future and not req.future.done():
                    req.future.set_exception(e)

        finally:
            with self._lock:
                self._processing = False
                if self._queue:
                    if len(self._queue) >= self._batch_size:
                        # Submitters that arrived while this batch was running
                        # may already have observed ``_processing`` and exited.
                        # Keep a full remainder moving without waiting for a
                        # timeout that was only intended for partial batches.
                        asyncio.create_task(self._flush())
                    else:
                        self._schedule_timeout_flush_locked()

    async def flush(self) -> None:
        """Force flush pending requests."""
        await self._flush()

    def get_stats(self) -> dict:
        """Get batching statistics."""
        return {
            "queue_size": len(self._queue),
            "total_requests": self._total_requests,
            "total_batches": self._total_batches,
            "total_errors": self._total_errors,
            "avg_batch_size": self._total_requests / max(1, self._total_batches),
        }


class QueueFullError(Exception):
    """Raised when request queue is full."""

    pass


class BatchError(Exception):
    """Raised when batch processing fails."""

    pass


# Synchronous version
class SyncRequestBatcher(Generic[T, R]):
    """Synchronous request batcher."""

    def __init__(
        self,
        processor: Callable[[list[T]], list[R]],
        batch_size: int = 10,
        flush_timeout: float = 0.1,
    ):
        self._processor = processor
        self._batch_size = batch_size
        self._flush_timeout = flush_timeout
        self._queue: list[BatchRequest[T]] = []
        self._lock = threading.Lock()
        self._last_flush = time.time()

    def submit(self, request_id: str, item: T) -> R:
        """Submit a synchronous request."""
        with self._lock:
            future = threading.Event()
            request = BatchRequest(
                id=request_id,
                args=(item,),
                future=future,  # type: ignore
            )
            self._queue.append(request)

            # Flush if needed
            if len(self._queue) >= self._batch_size:
                self._flush_locked()

        # Wait for result
        future.wait()
        return future.result

    def _flush_locked(self) -> None:
        """Flush queue (must hold lock)."""
        if not self._queue:
            return

        batch = self._queue[: self._batch_size]
        self._queue = self._queue[self._batch_size :]

        try:
            items = [req.args[0] for req in batch]
            results = self._processor(items)

            for i, req in enumerate(batch):
                if hasattr(req.future, "set_result"):
                    req.future.set_result(results[i] if i < len(results) else None)
        except Exception as e:
            for req in batch:
                if hasattr(req.future, "set_exception"):
                    req.future.set_exception(e)

    def flush(self) -> None:
        """Force flush."""
        with self._lock:
            self._flush_locked()


def batch_requests(batch_size: int = 10, timeout: float = 0.1):
    """Decorator to automatically batch function calls.

    Usage:
        @batch_requests(batch_size=5)
        def make_api_call(items):
            # Batch API call
            return [process(i) for i in items]
    """

    def decorator(func: Callable) -> Callable:
        batcher = RequestBatcher(func, batch_size=batch_size, flush_timeout=timeout)

        async def wrapper(*args, **kwargs):
            return await batcher.submit(str(id(args)), *args, **kwargs)

        return wrapper

    return decorator
