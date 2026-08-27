//! OpenAI-compatible AI client for helios-cli.
//!
//! Supports OpenAI, Anthropic (via compatible proxy), Ollama, LM Studio,
//! vLLM, and any OpenAI-compatible API endpoint.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

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
}
