"""Timeout construction coverage for :mod:`harness.http_pool`."""

import asyncio

import pytest

from harness.http_pool import (
    HTTPConnectionPool,
    PoolConfig,
    get_async_http_client,
    get_http_client,
)


def _reset_pool_safely() -> None:
    """Close factory clients without exercising ``HTTPConnectionPool.close``.

    The production async-close path has separate event-loop behavior.  These
    synchronous factory tests own the clients they create and close them using
    a fresh loop, so singleton cleanup cannot leak clients between tests.
    """
    with HTTPConnectionPool._lock:
        pool = HTTPConnectionPool._instance
        HTTPConnectionPool._instance = None

    if pool is None:
        return

    if pool._client is not None:
        pool._client.close()
        pool._client = None
    if pool._async_client is not None:
        asyncio.run(pool._async_client.aclose())
        pool._async_client = None


@pytest.fixture(autouse=True)
def reset_http_connection_pool() -> None:
    _reset_pool_safely()
    try:
        yield
    finally:
        _reset_pool_safely()


def _configured_timeout() -> PoolConfig:
    return PoolConfig(
        timeout=47.0,
        connect_timeout=11.0,
        read_timeout=23.0,
        pool_timeout=5.0,
    )


def _assert_configured_timeout(timeout: object) -> None:
    assert getattr(timeout, "connect") == 11.0
    assert getattr(timeout, "read") == 23.0
    assert getattr(timeout, "write") == 47.0
    assert getattr(timeout, "pool") == 5.0


def test_get_http_client_configures_all_timeout_phases() -> None:
    client = get_http_client(_configured_timeout())

    _assert_configured_timeout(client.timeout)


def test_get_async_http_client_configures_all_timeout_phases() -> None:
    client = get_async_http_client(_configured_timeout())

    _assert_configured_timeout(client.timeout)
