use nebulark_common::{
    config::{AppConfig, Profile},
    error::{Error, Result},
};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

pub struct ProfileManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ProfileManager {
    pub fn load(config_path: impl AsRef<Path>) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Config(format!("mkdir failed: {e}")))?;
        }

        let config = if config_path.exists() {
            let raw = std::fs::read_to_string(&config_path)
                .map_err(|e| Error::Config(format!("read failed: {e}")))?;
            toml::from_str(&raw).map_err(|e| Error::Config(format!("parse failed: {e}")))?
        } else {
            info!("No config found at {config_path:?}, using defaults");
            AppConfig::default()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn save(&self) -> Result<()> {
        let raw = toml::to_string_pretty(&self.config)
            .map_err(|e| Error::Config(format!("serialize failed: {e}")))?;

        std::fs::write(&self.config_path, &raw)
            .map_err(|e| Error::Config(format!("write failed: {e}")))?;

        debug!("Config saved to {:?}", self.config_path);
        Ok(())
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.config.profiles
    }

    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.config.profiles.iter().find(|p| p.name == name)
    }

    pub fn add(&mut self, profile: Profile) -> Result<()> {
        if self.config.profiles.iter().any(|p| p.name == profile.name) {
            return Err(Error::Config(format!(
                "profile '{}' already exists",
                profile.name
            )));
        }
        info!("Adding profile: {}", profile.name);
        self.config.profiles.push(profile);
        self.save()
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        let before = self.config.profiles.len();
        self.config.profiles.retain(|p| p.name != name);
        if self.config.profiles.len() == before {
            return Err(Error::Config(format!("profile '{name}' not found")));
        }
        info!("Removed profile: {name}");
        self.save()
    }

    pub fn default_profile(&self) -> Option<&Profile> {
        self.config
            .default_profile
            .as_deref()
            .and_then(|name| self.get(name))
    }
}
