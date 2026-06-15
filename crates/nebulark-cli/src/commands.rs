use crate::daemon::{socket_path, IpcRequest, IpcResponse};
use nebulark_awg::parser::parse_conf;
use nebulark_common::config::Profile;
use nebulark_core::{platform::PlatformBackend, profiles::ProfileManager};
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

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
    if !std::path::Path::new(target).exists() {
        mgr.get(target)
            .ok_or_else(|| anyhow::anyhow!("Profile '{target}' not found"))?;
    }

    let exe = std::env::current_exe()?;

    let log_path = std::env::temp_dir().join("nebulark-daemon.log");
    let log_file = std::fs::File::create(&log_path)?;

    let mut cmd = if nix::unistd::getuid().is_root() {
        let mut c = std::process::Command::new(&exe);
        c.args(["--config", config_path, "daemon", target]);
        c
    } else {
        let mut c = std::process::Command::new("sudo");
        c.args([
            "-E",
            exe.to_str().unwrap(),
            "--config", config_path,
            "daemon", target,
        ]);
        c
    };

    cmd.stdin(std::process::Stdio::null())
        .stdout(log_file.try_clone()?)
        .stderr(log_file)
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn daemon: {e}"))?;

    for i in 0..80 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if socket_path().exists() {
            println!("✓ Connected");
            return Ok(());
        }
        if i % 20 == 19 {
            eprintln!("Waiting for daemon... ({} sec)", (i + 1) / 10);
        }
    }

    if let Ok(log) = std::fs::read_to_string(&log_path) {
        if !log.is_empty() {
            eprintln!("Daemon log:\n{log}");
        }
    }

    anyhow::bail!("Daemon did not start in time.")
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

    match ipc_call(IpcRequest::Status).await {
        Ok(resp) => println!("Status: {}", resp.message),
        Err(_) => {
            let _ = std::fs::remove_file(socket_path());
            let _ = std::fs::remove_file(crate::daemon::pid_path());
            println!("Status: Disconnected (stale socket removed)");
        }
    }
    Ok(())
}

pub async fn status_check() -> anyhow::Result<bool> {
    if !socket_path().exists() {
        return Ok(false);
    }
    match ipc_call(IpcRequest::Status).await {
        Ok(resp) => Ok(resp.ok),
        Err(_) => {
            let _ = std::fs::remove_file(socket_path());
            let _ = std::fs::remove_file(crate::daemon::pid_path());
            Ok(false)
        }
    }
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