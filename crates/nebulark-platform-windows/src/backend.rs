use async_trait::async_trait;
use nebulark_common::{
    error::{Error, Result},
    types::{TunnelConfig, TunnelStats},
};
use nebulark_core::platform::{PlatformBackend, TunnelHandle};
use tracing::info;

pub struct WindowsBackend {
    iface_name: String,
}

impl WindowsBackend {
    pub fn new(iface_name: impl Into<String>) -> Self {
        Self {
            iface_name: iface_name.into(),
        }
    }
}

#[async_trait]
impl PlatformBackend for WindowsBackend {
    fn name(&self) -> &'static str {
        "windows"
    }

    async fn create_tunnel(&self, _cfg: &TunnelConfig) -> Result<TunnelHandle> {
        info!("Windows tunnel stub: {}", self.iface_name);
        Err(Error::Platform(
            "Windows backend not yet implemented".into(),
        ))
    }

    async fn destroy_tunnel(&self, _handle: &TunnelHandle) -> Result<()> {
        Err(Error::Platform(
            "Windows backend not yet implemented".into(),
        ))
    }

    async fn sync_routes(&self, _handle: &TunnelHandle, _cfg: &TunnelConfig) -> Result<()> {
        Err(Error::Platform(
            "Windows backend not yet implemented".into(),
        ))
    }

    async fn get_stats(&self, _handle: &TunnelHandle) -> Result<TunnelStats> {
        Ok(TunnelStats::default())
    }
}
