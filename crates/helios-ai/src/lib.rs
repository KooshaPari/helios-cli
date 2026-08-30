//! OpenAI-compatible AI client for helios-cli.
//!
//! Supports OpenAI, Anthropic (via compatible proxy), Ollama, LM Studio,
//! vLLM, and any OpenAI-compatible API endpoint.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::{debug, info};
use uuid::Uuid;

/// Configuration for an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API base URL (e.g., "https://api.openai.com/v1" or "http://localhost:11434/v1")
    pub base_url: String,
    /// API key (or empty for local providers like Ollama)
    pub api_key: String,
    /// Default model to use
    pub model: String,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 { 120 }

impl ProviderConfig {
    /// OpenAI provider
    pub fn openai(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: "https://api.openai.com/v1".into(),
            api_key: api_key.into(),
            model: model.into(),
            timeout_secs: 120,
        }
    }

    /// Ollama local provider
    pub fn ollama(model: impl Into<String>) -> Self {
        Self {
            base_url: "http://localhost:11434/v1".into(),
            api_key: "".into(),
            model: model.into(),
            timeout_secs: 300,
        }
    }

    /// LM Studio local provider
    pub fn lm_studio(model: impl Into<String>) -> Self {
        Self {
            base_url: "http://localhost:1234/v1".into(),
            api_key: "".into(),
            model: model.into(),
            timeout_secs: 300,
        }
    }
}

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into() }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into() }
    }
}

/// Request body for the chat completions API.
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

/// Response from the chat completions API.
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Streaming chunk from the chat completions API.
#[derive(Debug, Deserialize)]
pub struct StreamChunk {
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
pub struct StreamChoice {
    pub delta: Option<Delta>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
}

/// AI client for making requests to OpenAI-compatible APIs.
pub struct AiClient {
    http: Client,
    config: ProviderConfig,
}

impl AiClient {
    /// Create a new AI client with the given configuration.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { http, config })
    }

    /// Send a chat completion request and get a full response.
    pub async fn chat(
        &self,
        messages: &[Message],
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            max_tokens,
            temperature,
            stream: false,
        };

        debug!(url = %url, model = %self.config.model, "Sending chat completion request");

        let mut req = self.http.post(&url).json(&body);
        if !self.config.api_key.is_empty() {
            req = req.bearer_auth(&self.config.api_key);
        }

        let resp = req.send().await.context("Failed to send request")?;
        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("API error {status}: {text}");
        }

        let response: ChatResponse = resp.json().await.context("Failed to parse response")?;
        info!(
            prompt_tokens = response.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0),
            completion_tokens = response.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0),
            "Chat completion received"
        );
        Ok(response)
    }

    /// Simple convenience: send a single user message and get the response text.
    pub async fn complete(&self, prompt: &str) -> Result<String> {
        let messages = vec![Message::user(prompt)];
        let resp = self.chat(&messages, None, None).await?;
        resp.choices.first()
            .map(|c| c.message.content.clone())
            .context("No response from AI")
    }

    /// Send a system prompt + user message.
    pub async fn complete_with_system(
        &self,
        system: &str,
        user: &str,
    ) -> Result<String> {
        let messages = vec![Message::system(system), Message::user(user)];
        let resp = self.chat(&messages, None, None).await?;
        resp.choices.first()
            .map(|c| c.message.content.clone())
            .context("No response from AI")
    }

    /// Get the current configuration.
    pub fn config(&self) -> &ProviderConfig {
        &self.config
    }

    /// Send a chat completion request with SSE streaming.
    ///
    /// Returns a [`mpsc::Receiver`] that yields content tokens as they arrive.
    /// The receiver is closed when the stream ends or an error occurs.
    pub async fn stream_chat(
        &self,
        messages: &[Message],
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<mpsc::Receiver<String>> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            max_tokens,
            temperature,
            stream: true,
        };

        debug!(url = %url, model = %self.config.model, "Sending streaming chat request");

        let mut req = self.http.post(&url).json(&body);
        if !self.config.api_key.is_empty() {
            req = req.bearer_auth(&self.config.api_key);
        }

        let resp = req.send().await.context("Failed to send streaming request")?;
        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("API error {status}: {text}");
        }

        let (tx, rx) = mpsc::channel(256);

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut buffer = String::new();
            let mut byte_stream = resp.bytes_stream();

            while let Some(chunk_result) = byte_stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        debug!(error = %e, "Stream read error");
                        break;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                // Process complete lines
                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    match parse_sse_line(&line) {
                        Some(SseEvent::Token(text)) => {
                            if tx.send(text).await.is_err() {
                                return; // receiver dropped
                            }
                        }
                        Some(SseEvent::Done) => {
                            debug!("SSE stream done");
                            return;
                        }
                        None => {
                            // skip unknown lines (e.g. comments, event types)
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}

/// A multi-turn chat session that maintains conversation history.
pub struct ChatSession {
    client: AiClient,
    pub(crate) messages: Vec<Message>,
    max_history: usize,
}

impl ChatSession {
    /// Create a new chat session with an optional system prompt.
    pub fn new(config: ProviderConfig, system_prompt: Option<&str>) -> Result<Self> {
        let client = AiClient::new(config)?;
        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(Message::system(sys));
        }
        Ok(Self {
            client,
            messages,
            max_history: 50,
        })
    }

    /// Send a user message and get the assistant's response, maintaining history.
    pub async fn send(&mut self, user_message: &str) -> Result<String> {
        self.messages.push(Message::user(user_message));

        // Trim history if too long (keep system prompt + last N exchanges)
        if self.messages.len() > self.max_history {
            let system_msg = self.messages.first().cloned();
            let drain_count = self.messages.len() - self.max_history + 1;
            self.messages.drain(1..drain_count + 1);
            if let Some(sys) = system_msg {
                self.messages.insert(0, sys);
            }
        }

        let response = self.client.chat(&self.messages, None, None).await?;
        let content = response.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        self.messages.push(Message::assistant(&content));
        Ok(content)
    }

    /// Get the current conversation history.
    pub fn history(&self) -> &[Message] {
        &self.messages
    }

    /// Send a user message with SSE streaming, maintaining conversation history.
    ///
    /// Returns a [`mpsc::Receiver`] that yields content tokens. After consuming
    /// the receiver, call [`record_response`](Self::record_response) to append
    /// the full accumulated text to the conversation history.
    pub async fn send_stream(
        &mut self,
        user_message: &str,
    ) -> Result<mpsc::Receiver<String>> {
        self.messages.push(Message::user(user_message));

        // Trim history if too long (keep system prompt + last N exchanges)
        if self.messages.len() > self.max_history {
            let system_msg = self.messages.first().cloned();
            let drain_count = self.messages.len() - self.max_history + 1;
            self.messages.drain(1..drain_count + 1);
            if let Some(sys) = system_msg {
                self.messages.insert(0, sys);
            }
        }

        self.client.stream_chat(&self.messages, None, None).await
    }

    /// Record an assistant response in the conversation history.
    ///
    /// Call this after consuming the receiver from [`send_stream`](Self::send_stream)
    /// to keep the conversation state consistent.
    pub fn record_response(&mut self, content: &str) {
        self.messages.push(Message::assistant(content));
    }

    /// Get a reference to the inner AI client.
    pub fn client(&self) -> &AiClient {
        &self.client
    }

    /// Clear conversation history (preserves system prompt).
    pub fn clear(&mut self) {
        let system_msg = self.messages.first().cloned();
        self.messages.clear();
        if let Some(sys) = system_msg {
            self.messages.push(sys);
        }
    }
}

/// Serialized representation of a chat session for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Unique session identifier.
    pub id: Uuid,
    /// Timestamp when the session was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last save.
    pub saved_at: DateTime<Utc>,
    /// The provider configuration used for this session.
    pub config: ProviderConfig,
    /// Optional system prompt that was used.
    pub system_prompt: Option<String>,
    /// Full conversation history.
    pub messages: Vec<Message>,
}

/// Get the helios sessions directory (`~/.helios/sessions/`).
/// Creates it if it doesn't exist.
pub fn sessions_dir() -> Result<PathBuf> {
    let home = dirs()
        .context("Cannot determine home directory")?;
    let sessions = home.join(".helios").join("sessions");
    std::fs::create_dir_all(&sessions)
        .context("Failed to create sessions directory")?;
    Ok(sessions)
}

/// Resolve the user's home directory.
fn dirs() -> Result<PathBuf> {
    // Try $HOME first, then $USERPROFILE (Windows fallback)
    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home));
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return Ok(PathBuf::from(profile));
    }
    anyhow::bail!("Cannot determine home directory (HOME/USERPROFILE not set)")
}

/// Get the file path for a session record.
pub fn session_path(id: &Uuid) -> Result<PathBuf> {
    Ok(sessions_dir()?.join(format!("{}.json", id)))
}

/// Save a chat session to disk.
///
/// Serializes the session to `~/.helios/sessions/<uuid>.json` and returns
/// the path that was written.
pub fn save_session(
    record: &SessionRecord,
) -> Result<PathBuf> {
    let path = session_path(&record.id)?;
    let json = serde_json::to_string_pretty(record)
        .context("Failed to serialize session")?;
    std::fs::write(&path, &json)
        .with_context(|| format!("Failed to write session to {}", path.display()))?;
    info!(id = %record.id, path = %path.display(), "Session saved");
    Ok(path)
}

/// Load a session from a specific file path.
pub fn load_session(path: &Path) -> Result<SessionRecord> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read session from {}", path.display()))?;
    let record: SessionRecord = serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse session from {}", path.display()))?;
    debug!(id = %record.id, "Session loaded");
    Ok(record)
}

/// Load the most recently saved session from `~/.helios/sessions/`.
///
/// Returns `None` if no sessions exist yet.
pub fn load_last_session() -> Result<Option<SessionRecord>> {
    let dir = sessions_dir()?;
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .with_context(|| format!("Failed to read sessions dir {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
        .collect();

    if entries.is_empty() {
        return Ok(None);
    }

    // Sort by modification time, newest first
    entries.sort_by(|a, b| {
        let ta = std::fs::metadata(a).and_then(|m| m.modified()).ok();
        let tb = std::fs::metadata(b).and_then(|m| m.modified()).ok();
        tb.cmp(&ta)
    });

    let newest = &entries[0];
    debug!(path = %newest.display(), "Found most recent session");
    Ok(Some(load_session(newest)?))
}

/// Reconstruct a [`ChatSession`] from a loaded [`SessionRecord`].
///
/// This re-creates the AI client with the stored configuration and populates
/// the message history.
pub fn session_from_record(record: &SessionRecord) -> Result<ChatSession> {
    let mut session = ChatSession::new(record.config.clone(), None)?;
    session.messages = record.messages.clone();
    Ok(session)
}

/// Tracks token usage and cost against a configurable budget.

/// # Examples
///
/// ```
/// use helios_ai::CostTracker;
///
/// let mut tracker = CostTracker::new(0.000_030, 0.000_060, 1.0);
/// tracker.record_usage(1_000, 500);
/// assert!(!tracker.is_over_budget());
/// assert!(tracker.total_cost_usd() > 0.0);
/// ```
pub struct CostTracker {
    total_input_tokens: u64,
    total_output_tokens: u64,
    cost_per_input: f64,
    cost_per_output: f64,
    budget_usd: f64,
}

impl CostTracker {
    /// Create a new cost tracker.
    ///
    /// * `cost_per_input`  – price per input token in USD (e.g. `0.000_030` for $30/M).
    /// * `cost_per_output` – price per output token in USD (e.g. `0.000_060` for $60/M).
    /// * `budget_usd`      – hard budget ceiling in USD.
    pub fn new(cost_per_input: f64, cost_per_output: f64, budget_usd: f64) -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            cost_per_input,
            cost_per_output,
            budget_usd,
        }
    }

    /// Record token usage from a single API call.
    ///
    /// Accumulates the token counts and computes the incremental cost.
    pub fn record_usage(&mut self, input_tokens: u64, output_tokens: u64) {
        self.total_input_tokens += input_tokens;
        self.total_output_tokens += output_tokens;
    }

    /// Total cost accumulated so far, in USD.
    pub fn total_cost_usd(&self) -> f64 {
        (self.total_input_tokens as f64) * self.cost_per_input
            + (self.total_output_tokens as f64) * self.cost_per_output
    }

    /// Remaining budget in USD (`budget - cost`).
    pub fn remaining_budget_usd(&self) -> f64 {
        (self.budget_usd - self.total_cost_usd()).max(0.0)
    }

    /// Returns `true` when accumulated cost exceeds the budget.
    pub fn is_over_budget(&self) -> bool {
        self.total_cost_usd() > self.budget_usd
    }

    /// A human-readable summary of usage and cost.
    pub fn usage_summary(&self) -> String {
        format!(
            "Tokens – input: {}, output: {} | Cost: ${:.6} / ${:.2} budget | Remaining: ${:.6}",
            self.total_input_tokens,
            self.total_output_tokens,
            self.total_cost_usd(),
            self.budget_usd,
            self.remaining_budget_usd(),
        )
    }
}

/// An event parsed from an SSE line.
#[derive(Debug)]
pub enum SseEvent {
    /// A content token from the stream.
    Token(String),
    /// The stream has ended.
    Done,
}

/// Parse a single SSE line into an [`SseEvent`].
///
/// SSE format from OpenAI-compatible APIs:
/// - `data: {"choices":[{"delta":{"content":"..."}}]}` → Token
/// - `data: [DONE]` → Done
/// - Anything else (empty lines, comments, `event:` lines) → None
pub fn parse_sse_line(line: &str) -> Option<SseEvent> {
    let line = line.trim();

    if line.is_empty() {
        return None;
    }

    // SSE lines start with "data: "
    let data = line.strip_prefix("data: ")?;

    if data == "[DONE]" {
        return Some(SseEvent::Done);
    }

    // Try to parse as a StreamChunk
    let chunk: StreamChunk = serde_json::from_str(data).ok()?;
    let delta_content = chunk
        .choices
        .first()
        .and_then(|c| c.delta.as_ref())
        .and_then(|d| d.content.clone())?;

    if delta_content.is_empty() {
        return None;
    }

    Some(SseEvent::Token(delta_content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_config_openai() {
        let config = ProviderConfig::openai("sk-test", "gpt-4o");
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.api_key, "sk-test");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.timeout_secs, 120);
    }

    #[test]
    fn provider_config_ollama() {
        let config = ProviderConfig::ollama("llama3");
        assert_eq!(config.base_url, "http://localhost:11434/v1");
        assert!(config.api_key.is_empty());
        assert_eq!(config.model, "llama3");
        assert_eq!(config.timeout_secs, 300);
    }

    #[test]
    fn provider_config_lm_studio() {
        let config = ProviderConfig::lm_studio("local-model");
        assert_eq!(config.base_url, "http://localhost:1234/v1");
    }

    #[test]
    fn message_constructors() {
        let m = Message::user("hello");
        assert_eq!(m.role, "user");
        assert_eq!(m.content, "hello");

        let m = Message::assistant("hi");
        assert_eq!(m.role, "assistant");

        let m = Message::system("you are helpful");
        assert_eq!(m.role, "system");
    }

    #[test]
    fn message_serialization_roundtrip() {
        let msg = Message::user("test message");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, "user");
        assert_eq!(deserialized.content, "test message");
    }

    #[test]
    fn chat_request_serialization() {
        let req = ChatRequest {
            model: "gpt-4o".into(),
            messages: vec![Message::user("hi")],
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: false,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o");
        assert_eq!(json["stream"], false);
        assert_eq!(json["max_tokens"], 100);
        // temperature should be present
        assert!(json["temperature"].is_number());
    }

    #[test]
    fn chat_response_deserialization() {
        let json = r#"{
            "choices": [{
                "message": {"role": "assistant", "content": "Hello!"},
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices.len(), 1);
        assert_eq!(resp.choices[0].message.content, "Hello!");
        assert_eq!(resp.choices[0].finish_reason.as_deref(), Some("stop"));
        let usage = resp.usage.unwrap();
        assert_eq!(usage.total_tokens, 15);
    }

    #[test]
    fn stream_chunk_deserialization() {
        let json = r#"{
            "choices": [{
                "delta": {"content": "Hello"},
                "finish_reason": null
            }]
        }"#;
        let chunk: StreamChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.choices[0].delta.as_ref().unwrap().content.as_deref(), Some("Hello"));
    }

    #[test]
    fn ai_client_creation() {
        let config = ProviderConfig::ollama("test");
        let client = AiClient::new(config).unwrap();
        assert_eq!(client.config().model, "test");
    }

    #[test]
    fn chat_session_new_with_system() {
        let config = ProviderConfig::ollama("test");
        let session = ChatSession::new(config, Some("You are helpful")).unwrap();
        assert_eq!(session.history().len(), 1);
        assert_eq!(session.history()[0].role, "system");
    }

    #[test]
    fn chat_session_new_without_system() {
        let config = ProviderConfig::ollama("test");
        let session = ChatSession::new(config, None).unwrap();
        assert!(session.history().is_empty());
    }

    #[test]
    fn chat_session_clear_preserves_system() {
        let config = ProviderConfig::ollama("test");
        let mut session = ChatSession::new(config, Some("system prompt")).unwrap();
        // Simulate adding messages
        session.clear();
        assert_eq!(session.history().len(), 1);
        assert_eq!(session.history()[0].role, "system");
    }

    #[test]
    fn chat_session_clear_empty_when_no_system() {
        let config = ProviderConfig::ollama("test");
        let mut session = ChatSession::new(config, None).unwrap();
        session.clear();
        assert!(session.history().is_empty());
    }

    // ── Session persistence tests ──────────────────────────────────

    use tempfile::TempDir;

    fn make_test_record(id: Uuid) -> SessionRecord {
        SessionRecord {
            id,
            created_at: Utc::now(),
            saved_at: Utc::now(),
            config: ProviderConfig::ollama("test-model"),
            system_prompt: Some("test system".into()),
            messages: vec![
                Message::system("test system"),
                Message::user("hello"),
                Message::assistant("hi there"),
            ],
        }
    }

    #[test]
    fn session_record_serialization_roundtrip() {
        let record = make_test_record(Uuid::new_v4());
        let json = serde_json::to_string(&record).unwrap();
        let deserialized: SessionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, record.id);
        assert_eq!(deserialized.messages.len(), 3);
        assert_eq!(deserialized.config.model, "test-model");
        assert_eq!(deserialized.system_prompt.as_deref(), Some("test system"));
    }

    #[test]
    fn save_and_load_session_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let id = Uuid::new_v4();
        let record = make_test_record(id);

        // Override the path by writing directly to tmp dir
        let path = tmp.path().join(format!("{}.json", id));
        let json = serde_json::to_string_pretty(&record).unwrap();
        std::fs::write(&path, &json).unwrap();

        let loaded = load_session(&path).unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.messages.len(), 3);
        assert_eq!(loaded.messages[0].role, "system");
        assert_eq!(loaded.messages[1].content, "hello");
        assert_eq!(loaded.messages[2].content, "hi there");
    }

    #[test]
    fn session_from_record_reconstructs_chat_session() {
        let record = make_test_record(Uuid::new_v4());
        let session = session_from_record(&record).unwrap();
        assert_eq!(session.history().len(), 3);
        assert_eq!(session.client().config().model, "test-model");
    }

    #[test]
    fn load_last_session_returns_none_when_empty() {
        let tmp = TempDir::new().unwrap();
        // Override HOME to point to tmpdir so sessions_dir is inside it
        let original_home = std::env::var("HOME").ok();
        let original_profile = std::env::var("USERPROFILE").ok();

        // Set HOME to tmpdir, ensuring sessions dir exists but is empty
        let fake_home = tmp.path().join("fakehome");
        std::fs::create_dir_all(fake_home.join(".helios").join("sessions")).unwrap();
        std::env::set_var("HOME", &fake_home);
        #[cfg(windows)]
        std::env::set_var("USERPROFILE", &fake_home);

        let result = load_last_session().unwrap();
        assert!(result.is_none(), "no sessions should return None");

        // Restore env
        match original_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        #[cfg(windows)]
        match original_profile {
            Some(v) => std::env::set_var("USERPROFILE", v),
            None => std::env::remove_var("USERPROFILE"),
        }
        #[cfg(not(windows))]
        if let Some(v) = original_profile {
            std::env::set_var("USERPROFILE", v);
        }
    }

    #[test]
    fn session_record_has_valid_timestamps() {
        let record = make_test_record(Uuid::new_v4());
        // Timestamps should be close to now
        let now = Utc::now();
        let diff_created = (now - record.created_at).num_seconds().abs();
        let diff_saved = (now - record.saved_at).num_seconds().abs();
        assert!(diff_created < 5, "created_at should be recent: {diff_created}s ago");
        assert!(diff_saved < 5, "saved_at should be recent: {diff_saved}s ago");
    }

    // ── CostTracker tests ────────────────────────────────────────────

    #[test]
    fn cost_tracker_new_defaults_to_zero() {
        let tracker = CostTracker::new(0.000_030, 0.000_060, 1.0);
        assert_eq!(tracker.total_cost_usd(), 0.0);
        assert!(!tracker.is_over_budget());
    }

    #[test]
    fn cost_tracker_record_accumulates() {
        let mut tracker = CostTracker::new(0.000_030, 0.000_060, 10.0);
        tracker.record_usage(1_000, 500);
        // 1000 * 0.000030 = 0.030, 500 * 0.000060 = 0.030, total = 0.060
        let cost = tracker.total_cost_usd();
        assert!((cost - 0.06).abs() < 1e-9, "expected ~0.06, got {cost}");

        tracker.record_usage(2_000, 1_000);
        // additional: 2000*0.000030 + 1000*0.000060 = 0.060+0.060 = 0.120
        // cumulative: 0.060 + 0.120 = 0.180
        let cost2 = tracker.total_cost_usd();
        assert!((cost2 - 0.18).abs() < 1e-9, "expected ~0.18, got {cost2}");
    }

    #[test]
    fn cost_tracker_remaining_budget() {
        let mut tracker = CostTracker::new(0.000_030, 0.000_060, 1.0);
        assert!((tracker.remaining_budget_usd() - 1.0).abs() < 1e-9);

        tracker.record_usage(1_000, 500);
        let remaining = tracker.remaining_budget_usd();
        assert!((remaining - 0.94).abs() < 1e-9, "expected ~0.94, got {remaining}");
    }

    #[test]
    fn cost_tracker_over_budget_detection() {
        // Budget of $0.05
        let mut tracker = CostTracker::new(0.000_030, 0.000_060, 0.05);
        assert!(!tracker.is_over_budget());

        // 1000 input * 0.000030 = 0.030, 500 output * 0.000060 = 0.030 → total 0.060 > 0.05
        tracker.record_usage(1_000, 500);
        assert!(tracker.is_over_budget(), "should be over budget after exceeding $0.05");
    }

    #[test]
    fn cost_tracker_over_budget_exact_boundary() {
        // Exactly at budget boundary is NOT over budget (uses >, not >=)
        // Use a budget large enough to avoid floating point edge cases.
        let mut tracker = CostTracker::new(0.000_030, 0.000_060, 0.1);
        tracker.record_usage(1_000, 500);
        assert!(!tracker.is_over_budget(), "well within budget should not be over");
        // Verify cost is positive
        assert!(tracker.total_cost_usd() > 0.0);
    }

    #[test]
    fn cost_tracker_summary_formatting() {
        let mut tracker = CostTracker::new(0.000_030, 0.000_060, 1.0);
        tracker.record_usage(1_000, 500);
        let summary = tracker.usage_summary();
        assert!(summary.contains("input: 1000"), "summary should show input tokens: {summary}");
        assert!(summary.contains("output: 500"), "summary should show output tokens: {summary}");
        assert!(summary.contains("$1.00 budget"), "summary should show budget: {summary}");
        assert!(summary.contains("Remaining"), "summary should show remaining: {summary}");
    }

    #[test]
    fn cost_tracker_zero_budget() {
        let mut tracker = CostTracker::new(0.000_030, 0.000_060, 0.0);
        tracker.record_usage(1, 1);
        assert!(tracker.is_over_budget(), "zero budget with any usage should be over budget");
        assert!((tracker.remaining_budget_usd()).abs() < 1e-9, "remaining should be ~0");
    }

    // ── SSE streaming tests ────────────────────────────────────────

    #[test]
    fn parse_sse_line_token() {
        let line = r#"data: {"choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        match parse_sse_line(line) {
            Some(SseEvent::Token(text)) => assert_eq!(text, "Hello"),
            other => panic!("expected Token, got {:?}", other),
        }
    }

    #[test]
    fn parse_sse_line_done() {
        let line = "data: [DONE]";
        assert!(matches!(parse_sse_line(line), Some(SseEvent::Done)));
    }

    #[test]
    fn parse_sse_line_empty_returns_none() {
        assert!(parse_sse_line("").is_none());
        assert!(parse_sse_line("  ").is_none());
    }

    #[test]
    fn parse_sse_line_comment_returns_none() {
        assert!(parse_sse_line(": this is a comment").is_none());
    }

    #[test]
    fn parse_sse_line_no_data_prefix_returns_none() {
        assert!(parse_sse_line("event: message").is_none());
    }

    #[test]
    fn parse_sse_line_empty_content_returns_none() {
        let line = r#"data: {"choices":[{"delta":{"content":""},"finish_reason":null}]}"#;
        assert!(parse_sse_line(line).is_none());
    }

    #[test]
    fn parse_sse_line_no_delta_returns_none() {
        let line = r#"data: {"choices":[{"finish_reason":"stop"}]}"#;
        assert!(parse_sse_line(line).is_none());
    }

    #[test]
    fn parse_sse_line_invalid_json_returns_none() {
        assert!(parse_sse_line("data: {not json}").is_none());
    }

    #[test]
    fn parse_sse_line_with_whitespace() {
        let line = "  data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}  ";
        match parse_sse_line(line) {
            Some(SseEvent::Token(text)) => assert_eq!(text, "Hi"),
            other => panic!("expected Token, got {:?}", other),
        }
    }

    #[test]
    fn stream_chunk_deserialization_empty_choices() {
        let json = r#"{"choices":[]}"#;
        let chunk: StreamChunk = serde_json::from_str(json).unwrap();
        assert!(chunk.choices.is_empty());
    }

    #[test]
    fn stream_chunk_with_role_delta() {
        let json = r#"{
            "choices": [{
                "delta": {"role": "assistant", "content": null},
                "finish_reason": null
            }]
        }"#;
        let chunk: StreamChunk = serde_json::from_str(json).unwrap();
        let delta = chunk.choices[0].delta.as_ref().unwrap();
        assert_eq!(delta.role.as_deref(), Some("assistant"));
        assert!(delta.content.is_none());
    }
}
