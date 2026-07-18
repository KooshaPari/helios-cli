"""Direct behavioral contracts for :class:`harness.cache_utils.LRUCache`."""

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
