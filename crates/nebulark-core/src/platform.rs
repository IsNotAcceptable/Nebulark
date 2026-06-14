use async_trait::async_trait;
use nebulark_common::{
    error::Result,
    types::{TunnelConfig, TunnelStats},
};

#[derive(Debug, Clone)]
pub struct TunnelHandle {
    pub interface: String,
}

#[async_trait]
pub trait PlatformBackend: Send + Sync {
    async fn create_tunnel(&self, cfg: &TunnelConfig) -> Result<TunnelHandle>;
    async fn destroy_tunnel(&self, handle: &TunnelHandle) -> Result<()>;
    async fn sync_routes(&self, handle: &TunnelHandle, cfg: &TunnelConfig) -> Result<()>;
    async fn get_stats(&self, handle: &TunnelHandle) -> Result<TunnelStats>;
    fn name(&self) -> &'static str;
}