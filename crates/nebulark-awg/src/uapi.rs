use nebulark_common::error::{Error, Result};
use nebulark_common::types::TunnelStats;
use std::io::Write;
use std::process::{Command, Stdio};
use tracing::debug;

pub struct UapiClient {
    interface: String,
}

impl UapiClient {
    pub fn new(interface: &str) -> Self {
        Self {
            interface: interface.to_string(),
        }
    }

    pub async fn set_config(&self, uapi_str: &str) -> Result<()> {
        debug!("awg setconf {}: {} bytes", self.interface, uapi_str.len());

        let mut child = Command::new("awg")
            .args(["setconf", &self.interface, "/dev/stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Platform(format!("awg spawn failed: {e}")))?;

        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            stdin.write_all(uapi_str.as_bytes())
                .map_err(|e| Error::Platform(format!("awg stdin write failed: {e}")))?;
        }

        let out = child.wait_with_output()
            .map_err(|e| Error::Platform(format!("awg wait failed: {e}")))?;

        if out.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            Err(Error::Platform(format!("awg setconf failed: {stderr}")))
        }
    }

    pub async fn get_stats(&self) -> Result<TunnelStats> {
        let out = Command::new("awg")
            .args(["show", &self.interface, "dump"])
            .output()
            .map_err(|e| Error::Platform(format!("awg show failed: {e}")))?;

        if !out.status.success() {
            return Ok(TunnelStats::default());
        }

        Ok(parse_dump(&String::from_utf8_lossy(&out.stdout)))
    }
}

fn parse_dump(dump: &str) -> TunnelStats {
    let mut stats = TunnelStats::default();
    for line in dump.lines().skip(1) {
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