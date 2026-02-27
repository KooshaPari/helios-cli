//! WebSocket Transport for streaming

use super::TransportConfig;

pub struct WebSocketTransport {
    _config: TransportConfig,
}

impl WebSocketTransport {
    pub fn new(config: TransportConfig) -> Self {
        Self { _config: config }
    }

    pub async fn connect(&self) -> Result<(), String> {
        // Placeholder - would use tokio-tungstenite
        Ok(())
    }

    pub async fn stream(&self, _body: &[u8]) -> Result<(), String> {
        Ok(())
    }
}
