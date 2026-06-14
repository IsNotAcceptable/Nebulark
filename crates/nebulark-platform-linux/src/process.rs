use nebulark_common::error::{Error, Result};
use std::process::Stdio;
use tokio::process::Command;
use tracing::debug;

pub async fn run(prog: &str, args: &[&str]) -> Result<String> {
    debug!("run: {prog} {}", args.join(" "));
    let out = Command::new(prog)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| Error::Platform(format!("{prog} spawn failed: {e}")))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(Error::Platform(format!("{prog} failed: {stderr}")))
    }
}

pub async fn awg_quick(action: &str, interface: &str) -> Result<()> {
    let bin = which_awg_quick();
    run(&bin, &[action, interface]).await?;
    Ok(())
}

fn which_awg_quick() -> String {
    for candidate in &["amneziawg-quick", "awg-quick", "wg-quick"] {
        if std::process::Command::new("which")
            .arg(candidate)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return candidate.to_string();
        }
    }
    "awg-quick".to_string()
}