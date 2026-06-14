use nebulark_common::config::Profile;
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