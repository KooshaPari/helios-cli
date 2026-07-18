"""Direct behavioral contracts for :class:`harness.cache_utils.LRUCache`."""

from harness import cache_utils
from harness.cache_utils import LRUCache


def test_set_overwrites_existing_value_and_refreshes_recency():
    cache = LRUCache(max_size=2)
    cache.set("first", "original")
    cache.set("second", "second-value")

    cache.set("first", "replacement")

    assert cache.get("first") == "replacement"
    assert list(cache._cache) == ["second", "first"]


def test_overwritten_key_is_retained_when_lru_eviction_occurs():
    cache = LRUCache(max_size=2)
    cache.set("first", "original")
    cache.set("second", "second-value")
    cache.set("first", "replacement")

    cache.set("third", "third-value")

    assert cache.get("first") == "replacement"
    assert cache.get("second") is None
    assert cache.get("third") == "third-value"


def test_negative_effective_ttl_replacement_clears_prior_expiry(monkeypatch):
    now = 100.0
    monkeypatch.setattr(cache_utils.time, "time", lambda: now)
    cache = LRUCache(default_ttl=-1)
    cache.set("key", "expiring", ttl=10)

    cache.set("key", "non-expiring", ttl=-1)
    now = 200.0

    assert cache.get("key") == "non-expiring"
    assert "key" not in cache._expiry


def test_zero_default_ttl_replacement_clears_prior_expiry(monkeypatch):
    now = 100.0
    monkeypatch.setattr(cache_utils.time, "time", lambda: now)
    cache = LRUCache(default_ttl=0)
    cache.set("key", "expiring", ttl=10)

    cache.set("key", "non-expiring")
    now = 200.0

    assert cache.get("key") == "non-expiring"
    assert "key" not in cache._expiry
