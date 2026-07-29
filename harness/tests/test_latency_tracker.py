"""Regression tests for latency statistics aggregation."""

from concurrent.futures import ThreadPoolExecutor

from harness.latency_tracker import LatencyTracker, NetworkMetrics


def _complete(callable_):
    with ThreadPoolExecutor(max_workers=1) as executor:
        return executor.submit(callable_).result(timeout=1)


def test_get_all_stats_completes_with_empty_tracker():
    assert _complete(LatencyTracker().get_all_stats) == {}


def test_get_all_stats_completes_with_populated_endpoints_and_counts():
    tracker = LatencyTracker()
    tracker.record("users", 10.0)
    tracker.record("users", 20.0, success=False)
    tracker.record("orders", 30.0)

    stats = _complete(tracker.get_all_stats)

    assert set(stats) == {"users", "orders"}
    assert stats["users"].count == 2
    assert stats["users"].success_count == 1
    assert stats["users"].failure_count == 1
    assert stats["orders"].count == 1


def test_network_metrics_summary_completes_with_recorded_requests():
    metrics = NetworkMetrics()
    metrics.record_request("users", 10.0, success=True)
    metrics.record_request("orders", 20.0, success=False)

    summary = _complete(metrics.get_summary)

    assert summary["total_requests"] == 2
    assert summary["failed_requests"] == 1
    assert set(summary["endpoints"]) == {"users", "orders"}
