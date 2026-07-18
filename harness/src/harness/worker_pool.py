"""Worker pool for background task processing.

Provides thread pool and async worker management.
"""

import threading
import time
from collections.abc import Callable
from concurrent.futures import Future, ThreadPoolExecutor
from dataclasses import dataclass
from enum import Enum
from queue import Empty, Queue


class WorkerState(Enum):
    """Worker states."""

    IDLE = "idle"
    BUSY = "busy"
    STOPPING = "stopping"
    STOPPED = "stopped"


@dataclass
class WorkerMetrics:
    """Worker pool metrics."""

    total_tasks: int = 0
    completed_tasks: int = 0
    failed_tasks: int = 0
    avg_latency_ms: float = 0.0
    active_workers: int = 0


class WorkerPool:
    """Thread pool for background tasks.

    Usage:
        pool = WorkerPool(num_workers=4)
        pool.start()
        pool.submit(my_task, arg1, arg2)
        pool.shutdown()
    """

    def __init__(self, num_workers: int = 4, queue_size: int = 100):
        self.num_workers = num_workers
        self.queue_size = queue_size
        self._executor = ThreadPoolExecutor(max_workers=num_workers)
        self._task_queue: Queue = Queue(maxsize=queue_size)
        self._workers: list[threading.Thread] = []
        self._running = False
        self._metrics = WorkerMetrics()
        self._lock = threading.Lock()
        self._completion_condition = threading.Condition()
        self._pending_enqueues = 0

    def start(self):
        """Start the worker pool."""
        self._running = True
        for i in range(self.num_workers):
            t = threading.Thread(target=self._worker_loop, daemon=True)
            t.start()
            self._workers.append(t)

    def _worker_loop(self):
        """Worker loop."""
        while True:
            try:
                task = self._task_queue.get(timeout=1)
                try:
                    if task is None:
                        return

                    func, args, kwargs, result_future = task
                    start = time.time()

                    try:
                        result = func(*args, **kwargs)
                    except BaseException as error:
                        with self._lock:
                            self._metrics.failed_tasks += 1
                        result_future.set_exception(error)
                    else:
                        with self._lock:
                            self._metrics.completed_tasks += 1
                        result_future.set_result(result)
                    finally:
                        elapsed = (time.time() - start) * 1000
                        with self._lock:
                            n = self._metrics.completed_tasks + self._metrics.failed_tasks
                            self._metrics.avg_latency_ms = (
                                self._metrics.avg_latency_ms * (n - 1) + elapsed
                            ) / n
                finally:
                    # Keep Queue.join()/wait_completion correct while a task is active
                    # and when a task raises.
                    self._task_queue.task_done()
                    with self._completion_condition:
                        self._completion_condition.notify_all()

            except Empty:
                continue
            except Exception:
                pass

    def submit(self, func: Callable, *args, task_id: str | None = None, **kwargs) -> Future:
        """Submit a task to the pool."""
        with self._lock:
            if not self._running:
                raise RuntimeError("Pool not started")
            self._metrics.total_tasks += 1
        result_future = Future()
        with self._completion_condition:
            self._pending_enqueues += 1

        try:
            self._executor.submit(
                self._enqueue_task, func, args, kwargs, result_future
            )
        except BaseException as error:
            with self._completion_condition:
                self._pending_enqueues -= 1
                self._completion_condition.notify_all()
            result_future.set_exception(error)
        return result_future

    def _enqueue_task(self, func, args, kwargs, result_future):
        """Internal task wrapper."""
        try:
            self._task_queue.put((func, args, kwargs, result_future))
        except BaseException as error:
            result_future.set_exception(error)
        finally:
            with self._completion_condition:
                self._pending_enqueues -= 1
                self._completion_condition.notify_all()

    def submit_callback(self, func: Callable, callback: Callable, *args, **kwargs) -> str:
        """Submit task with callback on completion."""
        def wrapped_callback(fut):
            try:
                result = fut.result()
                callback(result, None)
            except Exception as e:
                callback(None, e)

        task_id = str(time.time())
        future = self.submit(func, *args, task_id=task_id, **kwargs)
        future.add_done_callback(wrapped_callback)

        return task_id

    def shutdown(self, wait: bool = True):
        """Shutdown the pool."""
        self._running = False

        # With the default wait=True, finish submitting accepted work before
        # placing the sentinels.  wait=False retains its non-blocking behavior.
        self._executor.shutdown(wait=wait)

        # Signal workers to stop after the queued work.
        for _ in range(self.num_workers):
            self._task_queue.put(None)

        if wait:
            for worker in self._workers:
                worker.join()
        self._workers.clear()

    def metrics(self) -> WorkerMetrics:
        """Get pool metrics."""
        with self._lock:
            return WorkerMetrics(
                total_tasks=self._metrics.total_tasks,
                completed_tasks=self._metrics.completed_tasks,
                failed_tasks=self._metrics.failed_tasks,
                avg_latency_ms=self._metrics.avg_latency_ms,
                active_workers=len([t for t in self._workers if t.is_alive()]),
            )

    def wait_completion(self, timeout: float | None = None) -> bool:
        """Wait for all tasks to complete."""
        deadline = None if timeout is None else time.monotonic() + timeout
        with self._completion_condition:
            while self._pending_enqueues or self._task_queue.unfinished_tasks:
                if deadline is None:
                    self._completion_condition.wait()
                    continue
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    return False
                self._completion_condition.wait(remaining)
            return True


# Global worker pool
_worker_pool: WorkerPool | None = None


def get_worker_pool(num_workers: int = 4) -> WorkerPool:
    """Get or create global worker pool."""
    global _worker_pool
    if _worker_pool is None:
        _worker_pool = WorkerPool(num_workers=num_workers)
        _worker_pool.start()
    return _worker_pool


# Example
if __name__ == "__main__":
    pool = WorkerPool(num_workers=2)
    pool.start()

    def task(x):
        time.sleep(0.5)
        return x * 2

    # Submit tasks
    futures = []
    for i in range(5):
        f = pool.submit(task, i)
        futures.append(f)

    print(f"Metrics: {pool.metrics()}")

    pool.shutdown()
    print("Pool stopped")
