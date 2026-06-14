use crate::daemon::{socket_path, IpcRequest, IpcResponse};
use nebulark_awg::parser::parse_conf;
use nebulark_common::config::Profile;
use nebulark_core::{platform::PlatformBackend, profiles::ProfileManager, tunnel::TunnelManager};
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};
use tracing::info;

pub fn make_backend() -> Arc<dyn PlatformBackend> {
    #[cfg(target_os = "linux")]
    {
        Arc::new(nebulark_platform_linux::backend::LinuxBackend::new("nebulark0"))
    }
    #[cfg(target_os = "windows")]
    {
        Arc::new(nebulark_platform_windows::backend::WindowsBackend::new("nebulark0"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        panic!("Unsupported platform")
    }
}

pub async fn connect(config_path: &str, target: &str) -> anyhow::Result<()> {
    if socket_path().exists() {
        anyhow::bail!("Already connected. Run 'nebulark disconnect' first.");
    }

    let mgr = ProfileManager::load(config_path)?;
    let cfg = if std::path::Path::new(target).exists() {
        info!("Loading .conf file: {target}");
        let raw = std::fs::read_to_string(target)?;
        parse_conf(&raw)?
    } else {
        info!("Loading profile: {target}");
        mgr.get(target)
            .ok_or_else(|| anyhow::anyhow!("Profile '{target}' not found"))?
            .tunnel
            .clone()
    };

    let exe = std::env::current_exe()?;
    std::process::Command::new(exe)
        .args(["__daemon", target])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    for _ in 0..50 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if socket_path().exists() {
            println!("✓ Connected");
            return Ok(());
        }
    }
    anyhow::bail!("Daemon did not start in time. Check logs with RUST_LOG=info.")
}

pub async fn disconnect() -> anyhow::Result<()> {
    let resp = ipc_call(IpcRequest::Disconnect).await?;
    if resp.ok {
        println!("✓ {}", resp.message);
    } else {
        anyhow::bail!("{}", resp.message);
    }
    Ok(())
}

pub async fn status() -> anyhow::Result<()> {
    if !socket_path().exists() {
        println!("Status: Disconnected");
        return Ok(());
    }
    let resp = ipc_call(IpcRequest::Status).await?;
    println!("Status: {}", resp.message);
    Ok(())
}

pub async fn import(
    config_path: &str,
    path: &str,
    name: Option<&str>,
) -> anyhow::Result<()> {
    let raw = std::fs::read_to_string(path)?;
    let tunnel = parse_conf(&raw)?;
    let profile_name = name.map(|s| s.to_string()).unwrap_or_else(|| {
        std::path::Path::new(path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let profile = Profile { name: profile_name.clone(), tunnel };
    let mut mgr = ProfileManager::load(config_path)?;
    mgr.add(profile)?;
    println!("✓ Imported profile '{profile_name}'");
    Ok(())
}

pub async fn list(config_path: &str) -> anyhow::Result<()> {
    let mgr = ProfileManager::load(config_path)?;
    let profiles = mgr.profiles();
    if profiles.is_empty() {
        println!("No profiles. Use 'nebulark import <file.conf>' to add one.");
    } else {
        println!("Profiles:");
        for p in profiles {
            println!("  - {}", p.name);
        }
    }
    Ok(())
}

async fn ipc_call(req: IpcRequest) -> anyhow::Result<IpcResponse> {
    let stream = UnixStream::connect(socket_path())
        .await
        .map_err(|_| anyhow::anyhow!("Not connected (no daemon running)"))?;

    let (reader, mut writer) = stream.into_split();
    writer
        .write_all((serde_json::to_string(&req)? + "\n").as_bytes())
        .await?;

    let mut lines = BufReader::new(reader).lines();
    let line = lines
        .next_line()
        .await?
        .ok_or_else(|| anyhow::anyhow!("Empty IPC response"))?;

    Ok(serde_json::from_str(&line)?)
}