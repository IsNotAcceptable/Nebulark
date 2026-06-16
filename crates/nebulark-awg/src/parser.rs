use nebulark_common::{
    error::{Error, Result},
    types::{AwgObfsParams, PeerConfig, PeerId, PrivateKey, PublicKey, TunnelConfig},
};
use std::net::SocketAddr;

pub fn parse_conf(input: &str) -> Result<TunnelConfig> {
    let mut private_key = None;
    let mut listen_port = None;
    let mut addresses = vec![];
    let mut mtu = None;
    let mut dns = vec![];
    let mut obfs = AwgObfsParams::default();
    let mut peers = vec![];
    let mut current_peer: Option<PeerConfig> = None;
    let mut section = Section::None;

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[Interface]" {
            section = Section::Interface;
            continue;
        }
        if line == "[Peer]" {
            if let Some(peer) = current_peer.take() {
                peers.push(peer);
            }
            current_peer = Some(PeerConfig {
                id: PeerId(uuid_stub()),
                public_key: PublicKey(String::new()),
                preshared_key: None,
                endpoint: None,
                allowed_ips: vec![],
                keepalive: None,
            });
            section = Section::Peer;
            continue;
        }

        let (key, val) = match line.split_once('=') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        match section {
            Section::Interface => match key {
                "PrivateKey" => private_key = Some(PrivateKey(val.to_string())),
                "ListenPort" => listen_port = val.parse().ok(),
                "Address" => {
                    for part in val.split(',') {
                        addresses.push(part.trim().to_string());
                    }
                }
                "MTU" => mtu = val.parse().ok(),
                "DNS" => {
                    for part in val.split(',') {
                        if let Ok(ip) = part.trim().parse() {
                            dns.push(ip);
                        }
                    }
                }
                "Jc" => obfs.jc = val.parse().unwrap_or(0),
                "Jmin" => obfs.jmin = val.parse().unwrap_or(0),
                "Jmax" => obfs.jmax = val.parse().unwrap_or(0),
                "S1" => obfs.s1 = val.parse().unwrap_or(0),
                "S2" => obfs.s2 = val.parse().unwrap_or(0),
                "S3" => obfs.s3 = val.parse().unwrap_or(0),
                "S4" => obfs.s4 = val.parse().unwrap_or(0),
                "H1" => obfs.h1 = val.parse().unwrap_or(0),
                "H2" => obfs.h2 = val.parse().unwrap_or(0),
                "H3" => obfs.h3 = val.parse().unwrap_or(0),
                "H4" => obfs.h4 = val.parse().unwrap_or(0),
                "I1" => obfs.i1 = Some(val.to_string()), // храним as-is
                _ => {}
            },
            Section::Peer => {
                if let Some(peer) = current_peer.as_mut() {
                    match key {
                        "PublicKey" => peer.public_key = PublicKey(val.to_string()),
                        "PresharedKey" => peer.preshared_key = Some(val.to_string()),
                        "Endpoint" => peer.endpoint = val.parse::<SocketAddr>().ok(),
                        "AllowedIPs" => {
                            peer.allowed_ips
                                .extend(val.split(',').map(|s| s.trim().to_string()));
                        }
                        "PersistentKeepalive" => peer.keepalive = val.parse().ok(),
                        _ => {}
                    }
                }
            }
            Section::None => {}
        }
    }

    if let Some(peer) = current_peer {
        peers.push(peer);
    }

    Ok(TunnelConfig {
        private_key: private_key.ok_or_else(|| Error::Config("missing PrivateKey".into()))?,
        listen_port,
        addresses,
        mtu,
        dns,
        peers,
        obfs,
    })
}

enum Section {
    None,
    Interface,
    Peer,
}

fn uuid_stub() -> String {
    format!(
        "peer-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
    )
}
