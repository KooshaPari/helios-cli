//! Transport Layer for Helios
//!
//! Provides multi-transport support for high-performance LLM communication:
//! - HTTP/2 (default)
//! - WebSocket (streaming)
//! - Unix Domain Socket (local)
//! - gRPC (typed)

pub mod grpc;
pub mod http2;
pub mod pool;
pub mod selector;
pub mod unix_socket;
pub mod websocket;

pub use pool::ConnectionPool;
pub use selector::TransportSelector;

/// Transport types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Http2,
    WebSocket,
    UnixSocket,
    Grpc,
}

impl Default for TransportType {
    fn default() -> Self {
        Self::Http2
    }
}

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub http_url: String,
    pub ws_url: String,
    pub unix_socket_path: String,
    pub grpc_url: String,
    pub pool_size: usize,
    pub timeout_ms: u64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: TransportType::Http2,
            http_url: "http://127.0.0.1:8317".to_string(),
            ws_url: "ws://127.0.0.1:8317/ws".to_string(),
            unix_socket_path: "/tmp/cliproxy.sock".to_string(),
            grpc_url: "127.0.0.1:50051".to_string(),
            pool_size: 100,
            timeout_ms: 60000,
        }
    }
}
