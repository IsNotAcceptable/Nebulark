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
    let has_default_v4 = allowed_ips.iter().any(|ip| ip == "0.0.0.0/0");
    let has_default_v6 = allowed_ips.iter().any(|ip| ip == "::/0");

    for ip in allowed_ips {
        if ip == "0.0.0.0/0" || ip == "::/0" {
            continue;
        }
        info!("Adding route: {ip} via {iface}");
        let proto = if ip.contains(':') { "-6" } else { "-4" };
        let _ = run("ip", &[proto, "route", "add", ip, "dev", iface]).await;
    }

    if has_default_v4 || has_default_v6 {
        add_default_routes(iface, has_default_v4, has_default_v6).await?;
    }
    Ok(())
}

async fn add_default_routes(iface: &str, v4: bool, v6: bool) -> Result<()> {
    let fwmark_out = run("awg", &["show", iface, "fwmark"]).await.unwrap_or_default();
    let fwmark = fwmark_out.trim();

    let table: u32 = 51820;
    if fwmark.is_empty() || fwmark == "off" {
        run("awg", &["set", iface, "fwmark", &table.to_string()]).await?;
    }
    let fwmark_val = if fwmark.is_empty() || fwmark == "off" {
        table.to_string()
    } else {
        fwmark.to_string()
    };

    let table_str = table.to_string();

    if v4 {
        let _ = run("ip", &["-4", "route", "add", "0.0.0.0/0", "dev", iface, "table", &table_str]).await;
        let _ = run("ip", &["-4", "rule", "add", "not", "fwmark", &fwmark_val, "table", &table_str]).await;
        let _ = run("ip", &["-4", "rule", "add", "table", "main", "suppress_prefixlength", "0"]).await;
        info!("Added default IPv4 route via {iface} (table {table_str})");
    }

    if v6 {
        let _ = run("ip", &["-6", "route", "add", "::/0", "dev", iface, "table", &table_str]).await;
        let _ = run("ip", &["-6", "rule", "add", "not", "fwmark", &fwmark_val, "table", &table_str]).await;
        let _ = run("ip", &["-6", "rule", "add", "table", "main", "suppress_prefixlength", "0"]).await;
        info!("Added default IPv6 route via {iface} (table {table_str})");
    }
    Ok(())
}

pub async fn del_routes(iface: &str, allowed_ips: &[String]) -> Result<()> {
    let has_default_v4 = allowed_ips.iter().any(|ip| ip == "0.0.0.0/0");
    let has_default_v6 = allowed_ips.iter().any(|ip| ip == "::/0");

    for ip in allowed_ips {
        if ip == "0.0.0.0/0" || ip == "::/0" { continue; }
        let proto = if ip.contains(':') { "-6" } else { "-4" };
        let _ = run("ip", &[proto, "route", "del", ip, "dev", iface]).await;
    }

    let table_str = "51820";

    if has_default_v4 {
        let _ = run("ip", &["-4", "rule", "delete", "table", table_str]).await;
        let _ = run("ip", &["-4", "rule", "delete", "table", "main", "suppress_prefixlength", "0"]).await;
        let _ = run("ip", &["-4", "route", "delete", "0.0.0.0/0", "table", table_str]).await;
    }

    if has_default_v6 {
        let _ = run("ip", &["-6", "rule", "delete", "table", table_str]).await;
        let _ = run("ip", &["-6", "rule", "delete", "table", "main", "suppress_prefixlength", "0"]).await;
        let _ = run("ip", &["-6", "route", "delete", "::/0", "table", table_str]).await;
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