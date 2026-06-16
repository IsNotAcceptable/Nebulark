use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn socket_path() -> PathBuf {
    std::env::temp_dir().join("nebulark.sock")
}

pub fn pid_path() -> PathBuf {
    std::env::temp_dir().join("nebulark.pid")
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum IpcRequest {
    Disconnect,
    Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub ok: bool,
    pub message: String,
}

pub fn is_connected() -> bool {
    if !socket_path().exists() {
        return false;
    }
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    if let Ok(mut stream) = UnixStream::connect(socket_path()) {
        let req = serde_json::to_string(&IpcRequest::Status).unwrap_or_default();
        let _ = stream.write_all((req + "\n").as_bytes());
        let mut buf = String::new();
        if stream.read_to_string(&mut buf).is_ok() {
            return serde_json::from_str::<IpcResponse>(&buf)
                .map(|r| r.ok)
                .unwrap_or(false);
        }
    }
    let _ = std::fs::remove_file(socket_path());
    false
}

pub fn disconnect() -> anyhow::Result<()> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    let mut stream = UnixStream::connect(socket_path())?;
    let req = serde_json::to_string(&IpcRequest::Disconnect)?;
    stream.write_all((req + "\n").as_bytes())?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    Ok(())
}

pub fn spawn_daemon(exe: &std::path::Path, config_path: &str, profile: &str) -> anyhow::Result<()> {
    let log = std::env::temp_dir().join("nebulark-daemon.log");
    let log_file = std::fs::File::create(&log)?;

    std::process::Command::new("sudo")
        .args([exe.to_str().unwrap(), "daemon", config_path, profile])
        .stdin(std::process::Stdio::null())
        .stdout(log_file.try_clone()?)
        .stderr(log_file)
        .spawn()?;
    Ok(())
}
