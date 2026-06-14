use crate::process::run;
use nebulark_common::error::Result;
use tracing::info;

pub async fn create_awg_iface(iface: &str) -> Result<()> {
    info!("Creating amneziawg interface: {iface}");
    if run("ip", &["link", "add", "dev", iface, "type", "amneziawg"])
        .await
        .is_err()
    {
        run("ip", &["link", "add", "dev", iface, "type", "wireguard"]).await?;
    }
    Ok(())
}

pub async fn setup_iface(iface: &str, addresses: &[String], mtu: Option<u16>) -> Result<()> {
    if let Some(mtu) = mtu {
        run("ip", &["link", "set", "dev", iface, "mtu", &mtu.to_string()]).await?;
    }
    for addr in addresses {
        info!("Adding address {addr} to {iface}");
        run("ip", &["address", "add", addr, "dev", iface]).await?;
    }
    run("ip", &["link", "set", "dev", iface, "up"]).await?;
    Ok(())
}

pub async fn delete_iface(iface: &str) -> Result<()> {
    info!("Deleting interface: {iface}");
    run("ip", &["link", "del", "dev", iface]).await.map(|_| ())
}

pub async fn add_routes(iface: &str, allowed_ips: &[String]) -> Result<()> {
    for ip in allowed_ips {
        info!("Adding route: {ip} via {iface}");
        let _ = run("ip", &["route", "add", ip, "dev", iface]).await;
    }
    Ok(())
}

pub async fn del_routes(iface: &str, allowed_ips: &[String]) -> Result<()> {
    for ip in allowed_ips {
        let _ = run("ip", &["route", "del", ip, "dev", iface]).await;
    }
    Ok(())
}

pub async fn set_dns(iface: &str, dns: &[std::net::IpAddr]) -> Result<()> {
    if dns.is_empty() {
        return Ok(());
    }
    let dns_strs: Vec<String> = dns.iter().map(|ip| ip.to_string()).collect();
    let mut args = vec!["dns", iface];
    let dns_refs: Vec<&str> = dns_strs.iter().map(|s| s.as_str()).collect();
    args.extend(dns_refs.iter());
    info!("Setting DNS on {iface}: {dns_strs:?}");
    let _ = run("resolvectl", &args).await;
    Ok(())
}