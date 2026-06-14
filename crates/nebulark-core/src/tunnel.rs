use crate::platform::{PlatformBackend, TunnelHandle};
use nebulark_common::{
    error::{Error, Result},
    types::{TunnelConfig, TunnelState, TunnelStats},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

pub struct TunnelManager {
    backend: Arc<dyn PlatformBackend>,
    state: Mutex<TunnelState>,
    handle: Mutex<Option<TunnelHandle>>,
}

impl TunnelManager {
    pub fn new(backend: Arc<dyn PlatformBackend>) -> Self {
        info!("TunnelManager init on platform: {}", backend.name());
        Self {
            backend,
            state: Mutex::new(TunnelState::Disconnected),
            handle: Mutex::new(None),
        }
    }

    pub async fn connect(&self, cfg: &TunnelConfig) -> Result<()> {
        {
            let state = self.state.lock().await;
            if !matches!(*state, TunnelState::Disconnected | TunnelState::Error(_)) {
                return Err(Error::Tunnel("already connected or connecting".into()));
            }
        }

        *self.state.lock().await = TunnelState::Connecting;
        info!("Connecting tunnel...");

        match self.backend.create_tunnel(cfg).await {
            Ok(handle) => {
                info!("Tunnel up: {}", handle.interface);
                *self.handle.lock().await = Some(handle);
                *self.state.lock().await = TunnelState::Connected;
                Ok(())
            }
            Err(e) => {
                error!("Tunnel connect failed: {e}");
                *self.state.lock().await = TunnelState::Error(e.to_string());
                Err(e)
            }
        }
    }

    pub async fn disconnect(&self) -> Result<()> {
        *self.state.lock().await = TunnelState::Disconnecting;
        info!("Disconnecting tunnel...");

        let handle = self.handle.lock().await.take();
        if let Some(h) = handle {
            self.backend.destroy_tunnel(&h).await?;
        }

        *self.state.lock().await = TunnelState::Disconnected;
        info!("Tunnel disconnected");
        Ok(())
    }

    pub async fn state(&self) -> TunnelState {
        self.state.lock().await.clone()
    }

    pub async fn stats(&self) -> Result<TunnelStats> {
        let handle = self.handle.lock().await;
        match handle.as_ref() {
            Some(h) => self.backend.get_stats(h).await,
            None => Ok(TunnelStats::default()),
        }
    }
}