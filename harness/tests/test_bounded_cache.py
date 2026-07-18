import pytest

from harness.bounded_cache import BoundedCache


@pytest.mark.parametrize("max_size", [0, -1])
def test_constructor_rejects_non_positive_capacity(max_size: int) -> None:
    with pytest.raises(ValueError, match="greater than zero"):
        BoundedCache(max_size=max_size)


def test_constructor_rejects_boolean_capacity() -> None:
    with pytest.raises(TypeError, match="must be an integer"):
        BoundedCache(max_size=True)


@pytest.mark.parametrize("max_size", [0, -1])
def test_setter_rejects_non_positive_capacity_without_mutation(max_size: int) -> None:
    cache = BoundedCache[str](max_size=2)
    cache.set("first", "one")
    cache.set("second", "two")

    with pytest.raises(ValueError, match="greater than zero"):
        cache.max_size = max_size

    assert cache.max_size == 2
    assert len(cache) == 2
    assert cache.get("first") == "one"
    assert cache.get("second") == "two"


def test_setter_rejects_boolean_capacity_without_mutation() -> None:
    cache = BoundedCache[str](max_size=1)
    cache.set("key", "value")

    with pytest.raises(TypeError, match="must be an integer"):
        cache.max_size = False

    assert cache.max_size == 1
    assert cache.get("key") == "value"


def test_valid_capacity_change_preserves_eviction_behavior() -> None:
    cache = BoundedCache[str](max_size=3)
    cache.set("first", "one")
    cache.set("second", "two")
    cache.set("third", "three")

    cache.max_size = 2

    assert cache.max_size == 2
    assert len(cache) == 1
    assert cache.get("first") is None
    assert cache.get("second") is None
    assert cache.get("third") == "three"
