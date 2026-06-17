mod app;
mod daemon;
mod setup;
mod tray;

use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("nebulark=info".parse()?)
                .add_directive("nebulark_core=info".parse()?)
                .add_directive("nebulark_platform_linux=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("daemon") {
        let config = args.get(2).map(|s| s.as_str()).unwrap_or("");
        let profile = args.get(3).map(|s| s.as_str()).unwrap_or("");
        return run_daemon(config, profile);
    }

    let exe = std::env::current_exe()?;
    if let Err(e) = setup::ensure_polkit_policy(&exe) {
        eprintln!("Warning: could not install polkit policy: {e}");
    }

    #[cfg(target_os = "linux")]
    {
        if !gtk::is_initialized() {
            gtk::init().expect("Failed to initialize GTK");
        }
    }

    let tray = tray::NebularkTray::new().ok();
    let menu_channel = std::sync::Arc::new(
        tray_icon::menu::MenuEvent::receiver().clone()
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Nebulark")
            .with_inner_size([420.0, 680.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Nebulark",
        options,
        Box::new(move |cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "noto".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/NotoSans-Regular.ttf")),
            );
            fonts.font_data.insert(
                "noto_sym".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/NotoSansSymbols-Regular.ttf"
                )),
            );
            let proportional = fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default();
            proportional.clear();
            proportional.push("noto".to_owned());
            proportional.push("noto_sym".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Box::new(app::NebularkApp::new(cc, tray, menu_channel))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}

async fn run_daemon_async(config_path: &str, profile: &str) -> anyhow::Result<()> {
    use nebulark_core::platform::PlatformBackend;
    use nebulark_core::profiles::ProfileManager;
    use std::sync::Arc;

    let mgr = ProfileManager::load(config_path)?;
    let cfg = mgr
        .get(profile)
        .ok_or_else(|| anyhow::anyhow!("Profile '{profile}' not found"))?
        .tunnel
        .clone();

    let backend: Arc<dyn PlatformBackend> =
        Arc::new(nebulark_platform_linux::backend::LinuxBackend::new("nebulark0"));
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
                            stats: None,
                        };
                        let _ = writer
                            .write_all((serde_json::to_string(&r).unwrap() + "\n").as_bytes())
                            .await;
                        std::process::exit(0);
                    }
                    Ok(daemon::IpcRequest::Stats) => {
                        let stats = fetch_stats("nebulark0");
                        daemon::IpcResponse {
                            ok: true,
                            message: "ok".into(),
                            stats: Some(stats),
                        }
                    }
                    Ok(daemon::IpcRequest::Status) => daemon::IpcResponse {
                        ok: true,
                        message: "ok".into(),
                        stats: None,
                    },
                    Err(e) => daemon::IpcResponse {
                        ok: false,
                        message: e.to_string(),
                        stats: None,
                    },
                };
                let _ = writer
                    .write_all((serde_json::to_string(&resp).unwrap() + "\n").as_bytes())
                    .await;
            }
        });
    }
}

fn fetch_stats(iface: &str) -> daemon::TunnelStats {
    let mut stats = daemon::TunnelStats::default();
    let out = match std::process::Command::new("awg")
        .args(["show", iface, "dump"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return stats,
    };
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() >= 7 {
            stats.rx_bytes += fields[5].parse::<u64>().unwrap_or(0);
            stats.tx_bytes += fields[6].parse::<u64>().unwrap_or(0);
            if stats.last_handshake_secs.is_none() {
                stats.last_handshake_secs = fields[4].parse().ok();
            }
        }
    }
    stats
}

fn run_daemon(config_path: &str, profile: &str) -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(run_daemon_async(config_path, profile))
}
