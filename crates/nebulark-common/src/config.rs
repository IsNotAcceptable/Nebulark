use serde::{Deserialize, Serialize};
use crate::types::TunnelConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub tunnel: TunnelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub profiles: Vec<Profile>,
    pub default_profile: Option<String>,
    pub autoconnect: bool,
}