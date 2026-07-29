from harness.context_compactor import ContextWindowManager


def test_over_budget_context_is_compacted_and_reports_estimated_tokens():
    manager = ContextWindowManager(max_tokens=20)
    manager.add_context(
        "user",
        [{"role": "user", "content": f"message-{index:02d}"} for index in range(11)],
    )

    optimized = manager.get_optimized_context()

    assert optimized[0]["content"] == "[Previous 1 messages summarized - 2.0 tokens]"
    assert len(optimized) == 11
    assert manager._compactor.get_stats() == {
        "message_count": 11,
        "total_tokens": 22.0,
        "max_tokens": 20,
        "utilization": 1.1,
        "strategy": "summarize",
    }


def test_under_budget_context_preserves_source_order_and_payloads():
    manager = ContextWindowManager(max_tokens=100)
    manager.add_context("user", [{"role": "user", "content": "user payload", "metadata": {"id": 4}}])
    manager.add_context("assistant", [{"role": "assistant", "content": "assistant payload", "metadata": {"id": 3}}])
    manager.add_context("tools", [{"role": "tool", "content": "tool payload", "metadata": {"id": 2}}])
    manager.add_context("system", [{"role": "system", "content": "system payload", "metadata": {"id": 1}}])

    assert manager.get_optimized_context() == [
        {"role": "system", "content": "system payload", "metadata": {"id": 1}},
        {"role": "tool", "content": "tool payload", "metadata": {"id": 2}},
        {"role": "assistant", "content": "assistant payload", "metadata": {"id": 3}},
        {"role": "user", "content": "user payload", "metadata": {"id": 4}},
    ]


def test_repeated_optimization_does_not_double_count_or_mutate_context():
    manager = ContextWindowManager(max_tokens=100)
    manager.add_context(
        "system",
        [{"role": "system", "content": "system context", "metadata": {"source": "test"}}],
    )
    manager.add_context("user", [{"role": "user", "content": "user context"}])

    before = [(message, message.token_count) for messages in manager._contexts.values() for message in messages]
    first = manager.get_optimized_context()
    first_stats = manager._compactor.get_stats()
    second = manager.get_optimized_context()

    assert second == first
    assert manager._compactor.get_stats() == first_stats
    assert [(message, message.token_count) for messages in manager._contexts.values() for message in messages] == before
