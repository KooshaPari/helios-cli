"""Deterministic unit tests for rate-limit wait-time calculations."""

from harness import rate_limit


def test_sliding_window_wait_time_is_zero_without_requests(monkeypatch):
    monkeypatch.setattr(rate_limit.time, "time", lambda: 100.0)

    limiter = rate_limit.SlidingWindowLimiter(max_requests=2, window_seconds=10.0)

    assert limiter.wait_time() == 0.0


def test_sliding_window_wait_time_starts_at_one_window_when_saturated(monkeypatch):
    monkeypatch.setattr(rate_limit.time, "time", lambda: 100.0)
    limiter = rate_limit.SlidingWindowLimiter(max_requests=2, window_seconds=10.0)

    assert limiter.is_allowed()
    assert limiter.is_allowed()

    assert limiter.wait_time() == 10.0


def test_sliding_window_wait_time_decreases_as_clock_advances(monkeypatch):
    clock = [100.0]
    monkeypatch.setattr(rate_limit.time, "time", lambda: clock[0])
    limiter = rate_limit.SlidingWindowLimiter(max_requests=1, window_seconds=10.0)

    assert limiter.is_allowed()
    clock[0] = 106.5

    assert limiter.wait_time() == 3.5


def test_sliding_window_wait_time_is_zero_after_oldest_request_expires(monkeypatch):
    clock = [100.0]
    monkeypatch.setattr(rate_limit.time, "time", lambda: clock[0])
    limiter = rate_limit.SlidingWindowLimiter(max_requests=1, window_seconds=10.0)

    assert limiter.is_allowed()
    clock[0] = 110.1

    assert limiter.wait_time() == 0.0
