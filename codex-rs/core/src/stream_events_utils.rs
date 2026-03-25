use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use codex_protocol::config_types::ModeKind;
use codex_protocol::items::TurnItem;
use codex_utils_stream_parser::strip_citations;
use tokio_util::sync::CancellationToken;

use crate::codex::Session;
use crate::codex::TurnContext;
use crate::error::CodexErr;
use crate::error::Result;
use crate::function_tool::FunctionCallError;
use crate::memories::citations::get_thread_id_from_citations;
<<<<<<< HEAD
=======
use crate::memories::citations::parse_memory_citation;
>>>>>>> upstream_main
use crate::parse_turn_item;
use crate::state_db;
use crate::tools::parallel::ToolCallRuntime;
use crate::tools::router::ToolRouter;
use codex_protocol::models::DeveloperInstructions;
use codex_protocol::models::FunctionCallOutputBody;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::models::ResponseItem;
use codex_utils_stream_parser::strip_proposed_plan_blocks;
use futures::Future;
use tracing::debug;
use tracing::instrument;

<<<<<<< HEAD
=======
const GENERATED_IMAGE_ARTIFACTS_DIR: &str = "generated_images";

pub(crate) fn image_generation_artifact_path(
    codex_home: &Path,
    session_id: &str,
    call_id: &str,
) -> PathBuf {
    let sanitize = |value: &str| {
        let mut sanitized: String = value
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else {
                    '_'
                }
            })
            .collect();
        if sanitized.is_empty() {
            sanitized = "generated_image".to_string();
        }
        sanitized
    };

    codex_home
        .join(GENERATED_IMAGE_ARTIFACTS_DIR)
        .join(sanitize(session_id))
        .join(format!("{}.png", sanitize(call_id)))
}

>>>>>>> upstream_main
fn strip_hidden_assistant_markup(text: &str, plan_mode: bool) -> String {
    let (without_citations, _) = strip_citations(text);
    if plan_mode {
        strip_proposed_plan_blocks(&without_citations)
    } else {
        without_citations
    }
}

<<<<<<< HEAD
=======
fn strip_hidden_assistant_markup_and_parse_memory_citation(
    text: &str,
    plan_mode: bool,
) -> (
    String,
    Option<codex_protocol::memory_citation::MemoryCitation>,
) {
    let (without_citations, citations) = strip_citations(text);
    let visible_text = if plan_mode {
        strip_proposed_plan_blocks(&without_citations)
    } else {
        without_citations
    };
    (visible_text, parse_memory_citation(citations))
}

>>>>>>> upstream_main
pub(crate) fn raw_assistant_output_text_from_item(item: &ResponseItem) -> Option<String> {
    if let ResponseItem::Message { role, content, .. } = item
        && role == "assistant"
    {
        let combined = content
            .iter()
            .filter_map(|ci| match ci {
                codex_protocol::models::ContentItem::OutputText { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<String>();
        return Some(combined);
    }
    None
}

<<<<<<< HEAD
=======
async fn save_image_generation_result(
    codex_home: &std::path::Path,
    session_id: &str,
    call_id: &str,
    result: &str,
) -> Result<PathBuf> {
    let bytes = BASE64_STANDARD
        .decode(result.trim().as_bytes())
        .map_err(|err| {
            CodexErr::InvalidRequest(format!("invalid image generation payload: {err}"))
        })?;
    let path = image_generation_artifact_path(codex_home, session_id, call_id);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, bytes).await?;
    Ok(path)
}

>>>>>>> upstream_main
/// Persist a completed model response item and record any cited memory usage.
pub(crate) async fn record_completed_response_item(
    sess: &Session,
    turn_context: &TurnContext,
    item: &ResponseItem,
) {
    sess.record_conversation_items(turn_context, std::slice::from_ref(item))
        .await;
<<<<<<< HEAD
    record_stage1_output_usage_for_completed_item(turn_context, item).await;
}

=======
    maybe_mark_thread_memory_mode_polluted_from_web_search(sess, turn_context, item).await;
    record_stage1_output_usage_for_completed_item(turn_context, item).await;
}

async fn maybe_mark_thread_memory_mode_polluted_from_web_search(
    sess: &Session,
    turn_context: &TurnContext,
    item: &ResponseItem,
) {
    if !turn_context
        .config
        .memories
        .no_memories_if_mcp_or_web_search
        || !matches!(item, ResponseItem::WebSearchCall { .. })
    {
        return;
    }
    state_db::mark_thread_memory_mode_polluted(
        sess.services.state_db.as_deref(),
        sess.conversation_id,
        "record_completed_response_item",
    )
    .await;
}

>>>>>>> upstream_main
async fn record_stage1_output_usage_for_completed_item(
    turn_context: &TurnContext,
    item: &ResponseItem,
) {
    let Some(raw_text) = raw_assistant_output_text_from_item(item) else {
        return;
    };

    let (_, citations) = strip_citations(&raw_text);
    let thread_ids = get_thread_id_from_citations(citations);
    if thread_ids.is_empty() {
        return;
    }

<<<<<<< HEAD
    if let Some(db) = state_db::get_state_db(turn_context.config.as_ref(), None).await {
=======
    if let Some(db) = state_db::get_state_db(turn_context.config.as_ref()).await {
>>>>>>> upstream_main
        let _ = db.record_stage1_output_usage(&thread_ids).await;
    }
}

/// Handle a completed output item from the model stream, recording it and
/// queuing any tool execution futures. This records items immediately so
/// history and rollout stay in sync even if the turn is later cancelled.
pub(crate) type InFlightFuture<'f> =
    Pin<Box<dyn Future<Output = Result<ResponseInputItem>> + Send + 'f>>;

#[derive(Default)]
pub(crate) struct OutputItemResult {
    pub last_agent_message: Option<String>,
    pub needs_follow_up: bool,
    pub tool_future: Option<InFlightFuture<'static>>,
}

pub(crate) struct HandleOutputCtx {
    pub sess: Arc<Session>,
    pub turn_context: Arc<TurnContext>,
    pub tool_runtime: ToolCallRuntime,
    pub cancellation_token: CancellationToken,
}

#[instrument(level = "trace", skip_all)]
pub(crate) async fn handle_output_item_done(
    ctx: &mut HandleOutputCtx,
    item: ResponseItem,
    previously_active_item: Option<TurnItem>,
) -> Result<OutputItemResult> {
    let mut output = OutputItemResult::default();
    let plan_mode = ctx.turn_context.collaboration_mode.mode == ModeKind::Plan;

    match ToolRouter::build_tool_call(ctx.sess.as_ref(), item.clone()).await {
        // The model emitted a tool call; log it, persist the item immediately, and queue the tool execution.
        Ok(Some(call)) => {
            let payload_preview = call.payload.log_payload().into_owned();
            tracing::info!(
                thread_id = %ctx.sess.conversation_id,
                "ToolCall: {} {}",
                call.tool_name,
                payload_preview
            );

            record_completed_response_item(ctx.sess.as_ref(), ctx.turn_context.as_ref(), &item)
                .await;

            let cancellation_token = ctx.cancellation_token.child_token();
            let tool_future: InFlightFuture<'static> = Box::pin(
                ctx.tool_runtime
                    .clone()
                    .handle_tool_call(call, cancellation_token),
            );

            output.needs_follow_up = true;
            output.tool_future = Some(tool_future);
        }
        // No tool call: convert messages/reasoning into turn items and mark them as complete.
        Ok(None) => {
<<<<<<< HEAD
            if let Some(turn_item) = handle_non_tool_response_item(&item, plan_mode) {
=======
            if let Some(turn_item) = handle_non_tool_response_item(
                ctx.sess.as_ref(),
                ctx.turn_context.as_ref(),
                &item,
                plan_mode,
            )
            .await
            {
>>>>>>> upstream_main
                if previously_active_item.is_none() {
                    let mut started_item = turn_item.clone();
                    if let TurnItem::ImageGeneration(item) = &mut started_item {
                        item.status = "in_progress".to_string();
                        item.revised_prompt = None;
                        item.result.clear();
                        item.saved_path = None;
                    }
                    ctx.sess
                        .emit_turn_item_started(&ctx.turn_context, &started_item)
                        .await;
                }

                ctx.sess
                    .emit_turn_item_completed(&ctx.turn_context, turn_item)
                    .await;
            }
<<<<<<< HEAD

=======
>>>>>>> upstream_main
            record_completed_response_item(ctx.sess.as_ref(), ctx.turn_context.as_ref(), &item)
                .await;
            let last_agent_message = last_assistant_message_from_item(&item, plan_mode);

            output.last_agent_message = last_agent_message;
        }
        // Guardrail: the model issued a LocalShellCall without an id; surface the error back into history.
        Err(FunctionCallError::MissingLocalShellCallId) => {
            let msg = "LocalShellCall without call_id or id";
            ctx.turn_context
                .session_telemetry
                .log_tool_failed("local_shell", msg);
            tracing::error!(msg);

            let response = ResponseInputItem::FunctionCallOutput {
                call_id: String::new(),
                output: FunctionCallOutputPayload {
                    body: FunctionCallOutputBody::Text(msg.to_string()),
                    ..Default::default()
                },
            };
            record_completed_response_item(ctx.sess.as_ref(), ctx.turn_context.as_ref(), &item)
                .await;
            if let Some(response_item) = response_input_to_response_item(&response) {
                ctx.sess
                    .record_conversation_items(
                        &ctx.turn_context,
                        std::slice::from_ref(&response_item),
                    )
                    .await;
            }

            output.needs_follow_up = true;
        }
        // The tool request should be answered directly (or was denied); push that response into the transcript.
        Err(FunctionCallError::RespondToModel(message)) => {
            let response = ResponseInputItem::FunctionCallOutput {
                call_id: String::new(),
                output: FunctionCallOutputPayload {
                    body: FunctionCallOutputBody::Text(message),
                    ..Default::default()
                },
            };
            record_completed_response_item(ctx.sess.as_ref(), ctx.turn_context.as_ref(), &item)
                .await;
            if let Some(response_item) = response_input_to_response_item(&response) {
                ctx.sess
                    .record_conversation_items(
                        &ctx.turn_context,
                        std::slice::from_ref(&response_item),
                    )
                    .await;
            }

            output.needs_follow_up = true;
        }
        // A fatal error occurred; surface it back into history.
        Err(FunctionCallError::Fatal(message)) => {
            return Err(CodexErr::Fatal(message));
        }
    }

    Ok(output)
}

<<<<<<< HEAD
pub(crate) fn handle_non_tool_response_item(
=======
pub(crate) async fn handle_non_tool_response_item(
    sess: &Session,
    turn_context: &TurnContext,
>>>>>>> upstream_main
    item: &ResponseItem,
    plan_mode: bool,
) -> Option<TurnItem> {
    debug!(?item, "Output item");

    match item {
        ResponseItem::Message { .. }
        | ResponseItem::Reasoning { .. }
        | ResponseItem::WebSearchCall { .. }
        | ResponseItem::ImageGenerationCall { .. } => {
            let mut turn_item = parse_turn_item(item)?;
            if let TurnItem::AgentMessage(agent_message) = &mut turn_item {
                let combined = agent_message
                    .content
                    .iter()
                    .map(|entry| match entry {
                        codex_protocol::items::AgentMessageContent::Text { text } => text.as_str(),
                    })
                    .collect::<String>();
<<<<<<< HEAD
                let stripped = strip_hidden_assistant_markup(&combined, plan_mode);
=======
                let (stripped, memory_citation) =
                    strip_hidden_assistant_markup_and_parse_memory_citation(&combined, plan_mode);
>>>>>>> upstream_main
                agent_message.content =
                    vec![codex_protocol::items::AgentMessageContent::Text { text: stripped }];
                agent_message.memory_citation = memory_citation;
            }
            if let TurnItem::ImageGeneration(image_item) = &mut turn_item {
                let session_id = sess.conversation_id.to_string();
                match save_image_generation_result(
                    turn_context.config.codex_home.as_path(),
                    &session_id,
                    &image_item.id,
                    &image_item.result,
                )
                .await
                {
                    Ok(path) => {
                        image_item.saved_path = Some(path.to_string_lossy().into_owned());
                        let image_output_path = image_generation_artifact_path(
                            turn_context.config.codex_home.as_path(),
                            &session_id,
                            "<image_id>",
                        );
                        let image_output_dir = image_output_path
                            .parent()
                            .unwrap_or(turn_context.config.codex_home.as_path());
                        let message: ResponseItem = DeveloperInstructions::new(format!(
                            "Generated images are saved to {} as {} by default.",
                            image_output_dir.display(),
                            image_output_path.display(),
                        ))
                        .into();
                        let copy_message: ResponseItem = DeveloperInstructions::new(
                            "If you need to use a generated image at another path, copy it and leave the original in place unless the user explicitly asks you to delete it."
                                .to_string(),
                        )
                        .into();
                        sess.record_conversation_items(turn_context, &[message, copy_message])
                            .await;
                    }
                    Err(err) => {
                        let output_path = image_generation_artifact_path(
                            turn_context.config.codex_home.as_path(),
                            &session_id,
                            &image_item.id,
                        );
                        let output_dir = output_path
                            .parent()
                            .unwrap_or(turn_context.config.codex_home.as_path());
                        tracing::warn!(
                            call_id = %image_item.id,
                            output_dir = %output_dir.display(),
                            "failed to save generated image: {err}"
                        );
                    }
                }
            }
            Some(turn_item)
        }
        ResponseItem::FunctionCallOutput { .. }
        | ResponseItem::CustomToolCallOutput { .. }
        | ResponseItem::ToolSearchOutput { .. } => {
            debug!("unexpected tool output from stream");
            None
        }
        _ => None,
    }
}

pub(crate) fn last_assistant_message_from_item(
    item: &ResponseItem,
    plan_mode: bool,
) -> Option<String> {
    if let Some(combined) = raw_assistant_output_text_from_item(item) {
        if combined.is_empty() {
            return None;
        }
        let stripped = strip_hidden_assistant_markup(&combined, plan_mode);
        if stripped.trim().is_empty() {
            return None;
        }
        return Some(stripped);
    }
    None
}

pub(crate) fn response_input_to_response_item(input: &ResponseInputItem) -> Option<ResponseItem> {
    match input {
        ResponseInputItem::FunctionCallOutput { call_id, output } => {
            Some(ResponseItem::FunctionCallOutput {
                call_id: call_id.clone(),
                output: output.clone(),
            })
        }
        ResponseInputItem::CustomToolCallOutput {
            call_id,
            name,
            output,
        } => Some(ResponseItem::CustomToolCallOutput {
            call_id: call_id.clone(),
            name: name.clone(),
            output: output.clone(),
        }),
        ResponseInputItem::McpToolCallOutput { call_id, output } => {
            let output = output.as_function_call_output_payload();
            Some(ResponseItem::FunctionCallOutput {
                call_id: call_id.clone(),
                output,
            })
        }
        ResponseInputItem::ToolSearchOutput {
            call_id,
            status,
            execution,
            tools,
        } => Some(ResponseItem::ToolSearchOutput {
            call_id: Some(call_id.clone()),
            status: status.clone(),
            execution: execution.clone(),
            tools: tools.clone(),
        }),
        _ => None,
    }
}

#[cfg(test)]
<<<<<<< HEAD
mod tests {
    use super::handle_non_tool_response_item;
    use super::last_assistant_message_from_item;
    use codex_protocol::items::TurnItem;
    use codex_protocol::models::ContentItem;
    use codex_protocol::models::ResponseItem;
    use pretty_assertions::assert_eq;

    fn assistant_output_text(text: &str) -> ResponseItem {
        ResponseItem::Message {
            id: Some("msg-1".to_string()),
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: text.to_string(),
            }],
            end_turn: Some(true),
            phase: None,
        }
    }

    #[test]
    fn handle_non_tool_response_item_strips_citations_from_assistant_message() {
        let item = assistant_output_text("hello<oai-mem-citation>doc1</oai-mem-citation> world");

        let turn_item =
            handle_non_tool_response_item(&item, false).expect("assistant message should parse");

        let TurnItem::AgentMessage(agent_message) = turn_item else {
            panic!("expected agent message");
        };
        let text = agent_message
            .content
            .iter()
            .map(|entry| match entry {
                codex_protocol::items::AgentMessageContent::Text { text } => text.as_str(),
            })
            .collect::<String>();
        assert_eq!(text, "hello world");
    }

    #[test]
    fn last_assistant_message_from_item_strips_citations_and_plan_blocks() {
        let item = assistant_output_text(
            "before<oai-mem-citation>doc1</oai-mem-citation>\n<proposed_plan>\n- x\n</proposed_plan>\nafter",
        );

        let message = last_assistant_message_from_item(&item, true)
            .expect("assistant text should remain after stripping");

        assert_eq!(message, "before\nafter");
    }

    #[test]
    fn last_assistant_message_from_item_returns_none_for_citation_only_message() {
        let item = assistant_output_text("<oai-mem-citation>doc1</oai-mem-citation>");

        assert_eq!(last_assistant_message_from_item(&item, false), None);
    }

    #[test]
    fn last_assistant_message_from_item_returns_none_for_plan_only_hidden_message() {
        let item = assistant_output_text("<proposed_plan>\n- x\n</proposed_plan>");

        assert_eq!(last_assistant_message_from_item(&item, true), None);
    }
}
=======
#[path = "stream_events_utils_tests.rs"]
mod tests;
>>>>>>> upstream_main
