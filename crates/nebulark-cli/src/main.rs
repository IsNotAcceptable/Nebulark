mod commands;
mod daemon;
mod server;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "nebulark", about = "AmneziaWG 2.0 tunnel client", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Connect { target: String },
    Disconnect,
    Status,
    Import {
        path: String,
        #[arg(short, long)]
        name: Option<String>,
    },
    List,
    #[command(hide = true)]
    Daemon { target: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("nebulark=info".parse()?)
                .add_directive("nebulark_core=info".parse()?)
                .add_directive("nebulark_platform_linux=info".parse()?),
        )
        .init();

    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or_else(|| {
        let home = std::env::var("SUDO_USER")
            .ok()
            .and_then(|user| {
                std::process::Command::new("getent")
                    .args(["passwd", &user])
                    .output()
                    .ok()
                    .and_then(|o| {
                        String::from_utf8(o.stdout).ok()
                            .and_then(|s| s.split(':').nth(5)
                                .map(|h| std::path::PathBuf::from(h.trim())))
                    })
            })
            .unwrap_or_else(|| dirs_next::home_dir().unwrap_or_default());

        home.join(".config")
            .join("nebulark")
            .join("config.toml")
            .to_string_lossy()
            .to_string()
    });

    match cli.command {
        Commands::Connect { target } => commands::connect(&config_path, &target).await?,
        Commands::Disconnect => commands::disconnect().await?,
        Commands::Status => commands::status().await?,
        Commands::Import { path, name } => {
            commands::import(&config_path, &path, name.as_deref()).await?
        }
        Commands::List => commands::list(&config_path).await?,
        Commands::Daemon { target } => {
            let mgr = nebulark_core::profiles::ProfileManager::load(&config_path)?;
            let cfg = mgr
                .get(&target)
                .ok_or_else(|| anyhow::anyhow!("Profile '{target}' not found"))?
                .tunnel
                .clone();
            server::run_daemon(cfg, commands::make_backend()).await;
        }
    }

    Ok(())
}