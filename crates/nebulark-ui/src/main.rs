mod app;
mod daemon;

use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("nebulark=info".parse()?)
            .add_directive("nebulark_core=info".parse()?)
            .add_directive("nebulark_platform_linux=info".parse()?))
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("daemon") {
        let config = args.get(2).map(|s| s.as_str()).unwrap_or("");
        let profile = args.get(3).map(|s| s.as_str()).unwrap_or("");
        return run_daemon(config, profile);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Nebulark")
            .with_inner_size([420.0, 520.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Nebulark",
        options,
        Box::new(|cc| Box::new(app::NebularkApp::new(cc))),
    ).map_err(|e| anyhow::anyhow!("{e}"))
}

async fn run_daemon_async(config_path: &str, profile: &str) -> anyhow::Result<()> {
    use nebulark_core::platform::PlatformBackend;
    use nebulark_core::profiles::ProfileManager;
    use std::sync::Arc;

    let mgr = ProfileManager::load(config_path)?;
    let cfg = mgr.get(profile)
        .ok_or_else(|| anyhow::anyhow!("Profile '{profile}' not found"))?
        .tunnel.clone();

    let backend: Arc<dyn PlatformBackend> = Arc::new(
        nebulark_platform_linux::backend::LinuxBackend::new("nebulark0")
    );
    let tunnel = Arc::new(nebulark_core::tunnel::TunnelManager::new(backend));

    if let Err(e) = tunnel.connect(&cfg).await {
        eprintln!("Connect failed: {e}");
        std::process::exit(1);
    }
    println!("✓ Connected");

    let _ = std::fs::write(daemon::pid_path(), std::process::id().to_string());

    use std::os::unix::fs::PermissionsExt;
    let sock = daemon::socket_path();
    let _ = std::fs::remove_file(&sock);
    let listener = tokio::net::UnixListener::bind(&sock)?;
    let _ = std::fs::set_permissions(&sock, std::fs::Permissions::from_mode(0o666));

    loop {
        let (stream, _) = listener.accept().await?;
        let tunnel = tunnel.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let (reader, mut writer) = stream.into_split();
            let mut lines = BufReader::new(reader).lines();
            if let Ok(Some(line)) = lines.next_line().await {
                let resp = match serde_json::from_str::<daemon::IpcRequest>(&line) {
                    Ok(daemon::IpcRequest::Disconnect) => {
                        let _ = tunnel.disconnect().await;
                        let _ = std::fs::remove_file(daemon::socket_path());
                        let _ = std::fs::remove_file(daemon::pid_path());
                        let r = daemon::IpcResponse {
                            ok: true,
                            message: "Disconnected".into(),
                        };
                        let _ = writer
                            .write_all((serde_json::to_string(&r).unwrap() + "\n").as_bytes())
                            .await;
                        std::process::exit(0);
                    }
                    Ok(daemon::IpcRequest::Status) => {
                        let state = tunnel.state().await;
                        daemon::IpcResponse { ok: true, message: format!("{state:?}") }
                    }
                    Err(e) => daemon::IpcResponse { ok: false, message: e.to_string() },
                };
                let _ = writer
                    .write_all((serde_json::to_string(&resp).unwrap() + "\n").as_bytes())
                    .await;
            }
        });
    }
}

fn run_daemon(config_path: &str, profile: &str) -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(run_daemon_async(config_path, profile))
}