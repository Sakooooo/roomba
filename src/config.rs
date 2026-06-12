use std::path::Path;

use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Library {
    path: String,
    cache_id: String,
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct Config {
    libraries: Option<Vec<Library>>,
}

pub enum ConfigError {
    FailRead,
    ParseError,
    SerializeError,
    WriteError,
}

pub fn read_config(path: std::path::PathBuf) -> Result<Config, ConfigError> {
    let config_data = match std::fs::read_to_string(&path) {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to read config at {}: {}", path.display(), e);
            return Err(ConfigError::FailRead);
        }
    };

    let config: Config = match toml::from_str(&config_data) {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to parse config {}", e);
            return Err(ConfigError::ParseError);
        }
    };

    Ok(config)
}

pub fn save_config(config: &Config, path: std::path::PathBuf) -> Result<(), ConfigError> {
    let config_text = match toml::to_string_pretty(config) {
        Ok(text) => text,
        Err(e) => {
            println!("Failed to turn Config into string {}", e);
            return Err(ConfigError::SerializeError);
        }
    };

    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        match std::fs::create_dir_all(parent) {
            Ok(_) => println!("Config directory created"),
            Err(e) => println!("Failed to create config directory: {}", e),
        };
    };

    match std::fs::write(&path, config_text) {
        Ok(_) => {
            println!("Config saved at {} successfully!", path.to_string_lossy());
            Ok(())
        }
        Err(e) => {
            println!("Failed to write config! {}", e);
            Err(ConfigError::WriteError)
        }
    }
}

pub fn save_library(state: &mut crate::State, path: String, dirs: AppDirs) {
    let config = &mut state.config;

    let mut libraries: Vec<Library> = if let Some(libraries) = &config.libraries {
        libraries.to_vec()
    } else {
        Vec::new()
    };

    let id = uuid::Uuid::new_v4();

    let library_to_add = Library {
        path,
        cache_id: id.to_string(),
    };

    libraries.push(library_to_add);

    config.libraries = Some(libraries);

    state.config = config.clone();

    let config_path = Path::join(&dirs.config_dir, "config.toml");

    if save_config(&state.config, config_path).is_err() {
        println!("Failed to save configuration, library will only say in memory for this run");
    };
}
