//! Transport Selector - Auto-selects optimal transport

use super::TransportConfig;
use super::TransportType;
use std::path::Path;

/// Auto-selects the best transport based on environment
pub struct TransportSelector {
    config: TransportConfig,
}

impl TransportSelector {
    pub fn new(config: TransportConfig) -> Self {
        Self { config }
    }

    /// Select transport using an optional override.
    pub fn select_with_preference(&self, transport: Option<TransportType>) -> TransportType {
        transport.unwrap_or_else(|| self.select())
    }

    /// Select the best transport
    pub fn select(&self) -> TransportType {
        // Priority: Unix Socket > WebSocket > HTTP/2
        if self.is_unix_socket_available() {
            TransportType::UnixSocket
        } else if self.is_websocket_available() {
            TransportType::WebSocket
        } else {
            TransportType::Http2
        }
    }

    fn is_unix_socket_available(&self) -> bool {
        Path::new(&self.config.unix_socket_path).exists()
    }

    fn is_websocket_available(&self) -> bool {
        // WebSocket is always "available" if configured
        !self.config.ws_url.is_empty()
    }
}

impl Default for TransportSelector {
    fn default() -> Self {
        Self::new(TransportConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::TransportConfig;
    use super::TransportSelector;
    use super::TransportType;
    use pretty_assertions::assert_eq;

    #[test]
    fn select_with_preference_auto_falls_back_to_auto() {
        let selector = TransportSelector::new(TransportConfig::default());
        assert_eq!(selector.select_with_preference(None), selector.select());
    }

    #[test]
    fn select_with_preference_respects_explicit_transport() {
        let selector = TransportSelector::new(TransportConfig::default());

        assert_eq!(
            selector.select_with_preference(Some(TransportType::UnixSocket)),
            TransportType::UnixSocket
        );
        assert_eq!(
            selector.select_with_preference(Some(TransportType::WebSocket)),
            TransportType::WebSocket
        );
        assert_eq!(
            selector.select_with_preference(Some(TransportType::Http2)),
            TransportType::Http2
        );
        assert_eq!(
            selector.select_with_preference(Some(TransportType::Grpc)),
            TransportType::Grpc
        );
    }
}
