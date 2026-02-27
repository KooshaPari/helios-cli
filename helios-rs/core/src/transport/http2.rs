//! HTTP/2 Transport

use super::TransportConfig;
use super::pool::ConnectionPool;

pub struct Http2Transport {
    _config: TransportConfig,
    _pool: ConnectionPool,
}

impl Http2Transport {
    pub fn new(config: TransportConfig) -> Self {
        let pool = ConnectionPool::new(config.pool_size, config.pool_size / 5);
        Self {
            _config: config,
            _pool: pool,
        }
    }

    pub async fn request(&self, _body: &[u8]) -> Result<Vec<u8>, String> {
        // Placeholder - would use reqwest/hyper with HTTP/2
        Ok(vec![])
    }
}
