//! `forge3_client` — JSON-RPC 2.0 client for the forge3d multi-agent daemon.
//!
//! # Wire format
//!
//! Each message is framed as a 4-byte big-endian length prefix followed by that
//! many bytes of UTF-8 JSON. The underlying transport is a Unix-domain stream
//! socket at `${XDG_RUNTIME_DIR:-/tmp}/forge3/daemon.sock`.
//!
//! # Example
//!
//! ```no_run
//! use forge3_client::Forge3Client;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = Forge3Client::connect().await.unwrap();
//!     if client.ping().await {
//!         let info = client.register("my-agent", std::process::id(), "building").await.unwrap();
//!         client.heartbeat("my-agent").await.unwrap();
//!         client.deregister("my-agent").await.unwrap();
//!     }
//! }
//! ```

use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during forge3d communication.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// An underlying I/O error (connection refused, broken pipe, etc.).
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// A protocol-level error — invalid framing, malformed JSON, or a
    /// JSON-RPC error response from the daemon.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// The daemon closed the connection before a full frame could be read.
    #[error("disconnected from forge3d daemon")]
    Disconnected,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Maximum accepted frame payload size (4 MiB).
const MAX_FRAME_BYTES: u32 = 4 * 1024 * 1024;

/// Return the default UDS socket path for forge3d.
///
/// The path is `${XDG_RUNTIME_DIR:-/tmp}/forge3/daemon.sock` to match the
/// daemon's default resolution in `forge3d::server::Sockets`.
pub fn default_socket_path() -> String {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
    format!("{runtime}/forge3/daemon.sock")
}

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 response types (private)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RpcResponse {
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<RpcErrorBody>,
    id: Value,
}

#[derive(Debug, Deserialize)]
struct RpcErrorBody {
    code: i64,
    message: String,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// A JSON-RPC 2.0 client connected to the forge3d daemon over a Unix-domain
/// socket.
///
/// All public methods require `&mut self` because they multiplex reads and
/// writes over a single stream.  The client is **not** safe for concurrent
/// access — callers should wrap it in a `Mutex` if sharing across tasks.
#[derive(Debug)]
pub struct Forge3Client {
    stream: UnixStream,
    next_id: AtomicU64,
}

impl Forge3Client {
    /// Connect to the forge3d daemon at the default socket path.
    ///
    /// The path is resolved from `$XDG_RUNTIME_DIR` (falling back to `/tmp`).
    pub async fn connect() -> Result<Self, ClientError> {
        Self::connect_with_path(default_socket_path()).await
    }

    /// Connect to the forge3d daemon at a custom socket path.
    pub async fn connect_with_path(path: impl AsRef<Path>) -> Result<Self, ClientError> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self {
            stream,
            next_id: AtomicU64::new(1),
        })
    }

    // -- private helpers -------------------------------------------------

    /// Allocate a monotonically increasing request id.
    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Send a JSON-RPC request and decode the response.
    ///
    /// Handles the full framing layer (4-byte BE length + UTF-8 JSON),
    /// JSON serialization, and error extraction.
    async fn call(&mut self, method: &str, params: Value) -> Result<Value, ClientError> {
        let id = self.next_id();
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "id": id,
            "params": params,
        });

        // Serialise and write the frame.
        let payload =
            serde_json::to_vec(&request).map_err(|e| ClientError::Protocol(e.to_string()))?;
        let frame_len = payload.len() as u32;
        self.stream.write_all(&frame_len.to_be_bytes()).await?;
        self.stream.write_all(&payload).await?;
        self.stream.flush().await?;

        // Read the response header (4-byte BE length).
        let mut header = [0u8; 4];
        match self.stream.read_exact(&mut header).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(ClientError::Disconnected);
            }
            Err(e) => return Err(ClientError::Io(e)),
        }

        let payload_len = u32::from_be_bytes(header);
        if payload_len == 0 || payload_len > MAX_FRAME_BYTES {
            return Err(ClientError::Protocol(format!(
                "invalid frame payload length: {payload_len} (max {MAX_FRAME_BYTES})"
            )));
        }

        // Read the JSON payload.
        let mut buf = vec![0u8; payload_len as usize];
        self.stream.read_exact(&mut buf).await?;

        // Parse the JSON-RPC response envelope.
        let resp: RpcResponse =
            serde_json::from_slice(&buf).map_err(|e| ClientError::Protocol(e.to_string()))?;

        // Surface JSON-RPC errors.
        if let Some(err) = resp.error {
            return Err(ClientError::Protocol(format!(
                "JSON-RPC error {}: {}",
                err.code, err.message
            )));
        }

        resp.result.ok_or_else(|| ClientError::Protocol("response missing 'result'".into()))
    }

    // -- public API ------------------------------------------------------

    /// Ping the daemon.  Returns `true` if the daemon responded with `{"ok": true}`.
    pub async fn ping(&mut self) -> bool {
        match self.call("ping", json!({})).await {
            Ok(v) => v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Register an agent with the daemon.
    ///
    /// `lane` is a free-form string describing the agent's work state
    /// (canonical values: `"building"`, `"shipped"`, `"maintain"`,
    /// `"exploring"`).  Returns the server's [`AgentInfo`] JSON.
    pub async fn register(
        &mut self,
        agent_id: &str,
        pid: u32,
        lane: &str,
    ) -> Result<Value, ClientError> {
        self.call(
            "agent.register",
            json!({
                "agent_id": agent_id,
                "pid": pid,
                "lane": lane,
            }),
        )
        .await
    }

    /// Send a heartbeat for `agent_id`, renewing its 60-second lease.
    pub async fn heartbeat(&mut self, agent_id: &str) -> Result<Value, ClientError> {
        self.call("agent.heartbeat", json!({ "agent_id": agent_id }))
            .await
    }

    /// Deregister an agent.  The daemon responds with `{"ok": true}` on success.
    pub async fn deregister(&mut self, agent_id: &str) -> Result<Value, ClientError> {
        self.call("agent.deregister", json!({ "agent_id": agent_id }))
            .await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_socket_path_uses_xdg_or_tmp() {
        let path = default_socket_path();
        // Must end with the expected basename.
        assert!(
            path.ends_with("/forge3/daemon.sock"),
            "unexpected path: {path}"
        );
    }

    #[tokio::test]
    async fn connect_refused_yields_io_error() {
        // Attempting to connect to a non-existent socket should give us an Io
        // error, not something else.
        let tmp = tempfile::tempdir().unwrap();
        let bogus = tmp.path().join("nonexistent.sock");
        let err = Forge3Client::connect_with_path(&bogus).await.unwrap_err();
        assert!(
            matches!(err, ClientError::Io(_)),
            "expected Io error, got {err:?}"
        );
    }
}
