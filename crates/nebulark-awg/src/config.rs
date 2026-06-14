use nebulark_common::types::{PeerConfig, TunnelConfig};

pub fn to_uapi(cfg: &TunnelConfig) -> String {
    let mut out = String::new();
    out.push_str("[Interface]\n");
    out.push_str(&format!("PrivateKey = {}\n", cfg.private_key.0));

    if let Some(port) = cfg.listen_port {
        out.push_str(&format!("ListenPort = {port}\n"));
    }

    let o = &cfg.obfs;
    out.push_str(&format!("Jc = {}\n", o.jc));
    out.push_str(&format!("Jmin = {}\n", o.jmin));
    out.push_str(&format!("Jmax = {}\n", o.jmax));
    out.push_str(&format!("S1 = {}\n", o.s1));
    out.push_str(&format!("S2 = {}\n", o.s2));
    out.push_str(&format!("S3 = {}\n", o.s3));
    out.push_str(&format!("S4 = {}\n", o.s4));
    out.push_str(&format!("H1 = {}\n", o.h1));
    out.push_str(&format!("H2 = {}\n", o.h2));
    out.push_str(&format!("H3 = {}\n", o.h3));
    out.push_str(&format!("H4 = {}\n", o.h4));
    if let Some(i1) = &o.i1 {
        out.push_str(&format!("I1 = {i1}\n"));
    }

    for peer in &cfg.peers {
        out.push_str(&peer_to_conf(peer));
    }
    out
}

fn peer_to_conf(peer: &PeerConfig) -> String {
    let mut out = String::new();
    out.push_str("\n[Peer]\n");
    out.push_str(&format!("PublicKey = {}\n", peer.public_key.0));
    if let Some(psk) = &peer.preshared_key {
        out.push_str(&format!("PresharedKey = {psk}\n"));
    }
    if let Some(ep) = &peer.endpoint {
        out.push_str(&format!("Endpoint = {ep}\n"));
    }
    for ip in &peer.allowed_ips {
        out.push_str(&format!("AllowedIPs = {ip}\n"));
    }
    if let Some(ka) = peer.keepalive {
        out.push_str(&format!("PersistentKeepalive = {ka}\n"));
    }
    out
}