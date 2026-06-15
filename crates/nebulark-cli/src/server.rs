use crate::daemon::{socket_path, IpcRequest, IpcResponse};
use nebulark_common::types::TunnelConfig;
use nebulark_core::{platform::PlatformBackend, tunnel::TunnelManager};
use std::{os::unix::fs::PermissionsExt, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixListener,
};
use tracing::{error, info};

pub async fn run_daemon(cfg: TunnelConfig, backend: Arc<dyn PlatformBackend>) {
    let tunnel = Arc::new(TunnelManager::new(backend));

    if let Err(e) = tunnel.connect(&cfg).await {
        error!("Connect failed: {e}");
        std::process::exit(1);
    }
    println!("✓ Connected");

    let pid = std::process::id();
    let _ = std::fs::write(crate::daemon::pid_path(), pid.to_string());

    let sock = socket_path();
    let _ = std::fs::remove_file(&sock);
    let listener = match UnixListener::bind(&sock) {
        Ok(l) => l,
        Err(e) => {
            error!("IPC bind failed: {e}");
            return;
        }
    };
    let _ = std::fs::set_permissions(&sock, std::fs::Permissions::from_mode(0o666));
    info!("IPC listening on {sock:?}");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let tunnel = tunnel.clone();
                tokio::spawn(async move {
                    let (reader, mut writer) = stream.into_split();
                    let mut lines = BufReader::new(reader).lines();

                    if let Ok(Some(line)) = lines.next_line().await {
                        let resp = match serde_json::from_str::<IpcRequest>(&line) {
                            Ok(IpcRequest::Disconnect) => {
                                info!("IPC: disconnect requested");
                                match tunnel.disconnect().await {
                                    Ok(_) => {
                                        let _ = std::fs::remove_file(socket_path());
                                        let _ = std::fs::remove_file(crate::daemon::pid_path());
                                        let resp = IpcResponse {
                                            ok: true,
                                            message: "Disconnected".into(),
                                        };
                                        let _ = writer
                                            .write_all(
                                                (serde_json::to_string(&resp).unwrap() + "\n")
                                                    .as_bytes(),
                                            )
                                            .await;
                                        std::process::exit(0);
                                    }
                                    Err(e) => IpcResponse {
                                        ok: false,
                                        message: e.to_string(),
                                    },
                                }
                            }
                            Ok(IpcRequest::Status) => {
                                let state = tunnel.state().await;
                                IpcResponse {
                                    ok: true,
                                    message: format!("{state:?}"),
                                }
                            }
                            Err(e) => IpcResponse {
                                ok: false,
                                message: format!("bad request: {e}"),
                            },
                        };
                        let _ = writer
                            .write_all(
                                (serde_json::to_string(&resp).unwrap() + "\n").as_bytes(),
                            )
                            .await;
                    }
                });
            }
            Err(e) => error!("IPC accept error: {e}"),
        }
    }
}