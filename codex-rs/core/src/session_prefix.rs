use codex_protocol::protocol::AgentStatus;

/// Helpers for model-visible session state markers that are stored in user-role
/// messages but are not user intent.
use crate::contextual_user_message::SUBAGENT_NOTIFICATION_FRAGMENT;

<<<<<<< HEAD
pub(crate) fn format_subagent_notification_message(agent_id: &str, status: &AgentStatus) -> String {
=======
// TODO(jif) unify with structured schema
pub(crate) fn format_subagent_notification_message(
    agent_reference: &str,
    status: &AgentStatus,
) -> String {
>>>>>>> upstream_main
    let payload_json = serde_json::json!({
        "agent_path": agent_reference,
        "status": status,
    })
    .to_string();
    SUBAGENT_NOTIFICATION_FRAGMENT.wrap(payload_json)
}

<<<<<<< HEAD
pub(crate) fn format_subagent_context_line(agent_id: &str, agent_nickname: Option<&str>) -> String {
    match agent_nickname.filter(|nickname| !nickname.is_empty()) {
        Some(agent_nickname) => format!("- {agent_id}: {agent_nickname}"),
        None => format!("- {agent_id}"),
=======
pub(crate) fn format_subagent_context_line(
    agent_reference: &str,
    agent_nickname: Option<&str>,
) -> String {
    match agent_nickname.filter(|nickname| !nickname.is_empty()) {
        Some(agent_nickname) => format!("- {agent_reference}: {agent_nickname}"),
        None => format!("- {agent_reference}"),
>>>>>>> upstream_main
    }
}
