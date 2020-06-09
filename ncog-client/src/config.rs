use atomicwrites::{AllowOverwrite, AtomicFile};
use kludgine::prelude::*;
use std::{fs, io::Write, path::PathBuf};

lazy_static! {
    static ref CONFIG: KludgineHandle<UserConfig> = KludgineHandle::new(UserConfig::load());
}

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserConfig {
    installation_id: Option<uuid::Uuid>,
}

impl UserConfig {
    fn config_path() -> PathBuf {
        let mut config_path = dirs::config_dir().expect("No config directory found");
        config_path.push("ncog");
        if !config_path.exists() {
            fs::create_dir(&config_path).expect("Error creating config folder");
        }
        config_path.push("config.toml");
        config_path
    }

    pub fn load() -> UserConfig {
        let config_path = Self::config_path();
        if config_path.exists() {
            if let Ok(config_data) = fs::read_to_string(config_path) {
                if let Ok(config) = toml::from_str(&config_data) {
                    return config;
                }
            }
        }

        UserConfig::default()
    }

    fn save(&self) {
        let config_path = Self::config_path();
        if let Ok(data) = toml::to_string(self) {
            let af = AtomicFile::new(config_path, AllowOverwrite);
            if let Err(err) = af.write(|f| f.write_all(data.as_bytes())) {
                println!("Error saving config {}", err);
            }
        } else {
            println!("Error serializing config");
        }
    }

    pub async fn set_installation_id(installation_id: uuid::Uuid) {
        let mut config = CONFIG.write().await;
        config.installation_id = Some(installation_id);
        config.save();
    }

    pub async fn installation_id() -> Option<uuid::Uuid> {
        let config = CONFIG.read().await;
        config.installation_id
    }
}
