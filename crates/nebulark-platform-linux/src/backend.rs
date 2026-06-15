use crate::netdev;
use async_trait::async_trait;
use nebulark_awg::{config::to_uapi, uapi::UapiClient};
use nebulark_common::{
    error::{Error, Result},
    types::{TunnelConfig, TunnelStats},
};
use nebulark_core::platform::{PlatformBackend, TunnelHandle};
use tracing::info;

pub struct LinuxBackend {
    iface_name: String,
}

impl LinuxBackend {
    pub fn new(iface_name: impl Into<String>) -> Self {
        Self {
            iface_name: iface_name.into(),
        }
    }
}

#[async_trait]
impl PlatformBackend for LinuxBackend {
    fn name(&self) -> &'static str {
        "linux"
    }

    async fn create_tunnel(&self, cfg: &TunnelConfig) -> Result<TunnelHandle> {
        let iface = &self.iface_name;

        netdev::create_awg_iface(iface).await?;

        let conf = to_uapi(cfg);
        let client = UapiClient::new(iface);
        client.set_config(&conf).await.map_err(|e| {
            Error::Platform(format!("awg setconf failed: {e}"))
        })?;

        for peer in &cfg.peers {
            apply_peer_settings(iface, peer).await?;
        }

        netdev::setup_iface(iface, &cfg.addresses, cfg.mtu).await?;

        for peer in &cfg.peers {
            netdev::add_routes(iface, &peer.allowed_ips).await?;
        }

        netdev::set_dns(iface, &cfg.dns).await?;

        info!("Linux tunnel up: {iface}");
        Ok(TunnelHandle { interface: iface.clone() })
    }

    async fn destroy_tunnel(&self, handle: &TunnelHandle) -> Result<()> {
        netdev::delete_iface(&handle.interface).await?;
        info!("Linux tunnel down: {}", handle.interface);
        Ok(())
    }

    async fn sync_routes(&self, handle: &TunnelHandle, cfg: &TunnelConfig) -> Result<()> {
        for peer in &cfg.peers {
            netdev::add_routes(&handle.interface, &peer.allowed_ips).await?;
        }
        Ok(())
    }

    async fn get_stats(&self, handle: &TunnelHandle) -> Result<TunnelStats> {
        let client = UapiClient::new(&handle.interface);
        client.get_stats().await
    }
}

async fn apply_peer_settings(
    iface: &str,
    peer: &nebulark_common::types::PeerConfig,
) -> nebulark_common::error::Result<()> {
    use crate::process::run;

    let mut args = vec![
        "set".to_string(),
        iface.to_string(),
        "peer".to_string(),
        peer.public_key.0.clone(),
    ];

    if let Some(ep) = &peer.endpoint {
        args.push("endpoint".to_string());
        args.push(ep.to_string());
    }

    if let Some(ka) = peer.keepalive {
        args.push("persistent-keepalive".to_string());
        args.push(ka.to_string());
    }

    if !peer.allowed_ips.is_empty() {
        args.push("allowed-ips".to_string());
        args.push(peer.allowed_ips.join(","));
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run("awg", &args_refs).await.map(|_| ())
}