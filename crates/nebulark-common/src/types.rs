use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateKey(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwgObfsParams {
    pub jc: u16,
    pub jmin: u16,
    pub jmax: u16,
    pub s1: u16,
    pub s2: u16,
    pub s3: u16,
    pub s4: u16,
    pub h1: u32,
    pub h2: u32,
    pub h3: u32,
    pub h4: u32,
    pub i1: Option<String>,
}

impl Default for AwgObfsParams {
    fn default() -> Self {
        Self {
            jc: 4,
            jmin: 40,
            jmax: 70,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            h1: 0,
            h2: 0,
            h3: 0,
            h4: 0,
            i1: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub private_key: PrivateKey,
    pub listen_port: Option<u16>,
    pub addresses: Vec<String>,
    pub mtu: Option<u16>,
    pub dns: Vec<IpAddr>,
    pub peers: Vec<PeerConfig>,
    pub obfs: AwgObfsParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    pub id: PeerId,
    pub public_key: PublicKey,
    pub preshared_key: Option<String>,
    pub endpoint: Option<SocketAddr>,
    pub allowed_ips: Vec<String>,
    pub keepalive: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TunnelState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TunnelStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub last_handshake_secs: Option<u64>,
}
