use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    libraries: Option<Vec<String>>,
}

impl Config {
    /// read the coonfig file or create a new one if it doesn't exist
    pub fn read_from_file_or_new(dirs: Option<AppDirs>) -> Config {
        if let Some(appdirs) = dirs {
            let config_path = appdirs.config_dir;

            let config_file = std::path::Path::join(&config_path, "config.toml");

            if config_path.exists() {
                if config_file.exists() {
                    let config_file_contents = match std::fs::read_to_string(&config_file) {
                        Ok(c) => c,
                        Err(e) => {
                            println!(
                                "Failed to read config file {}: {}",
                                config_file.to_string_lossy(),
                                e
                            );
                            println!("Using defaults");
                            return Config::default();
                        }
                    };

                    let config: Config =
                        toml::from_str(&config_file_contents).unwrap_or_else(|e| {
                            println!("Failed to parse config, {}", e);
                            println!("Using defaults");
                            return Config::default();
                        });

                    return config;
                } else {
                    return Config::default();
                }
            } else {
                match std::fs::create_dir(config_path) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Failed to create directory: {}", e);
                        return Config::default();
                    }
                };

                match std::fs::write(
                    config_file,
                    toml::to_string(&Config::default()).unwrap_or(String::from("")),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Failed to create config file: {}", e);
                        return Config::default();
                    }
                };

                println!("Created config file.");
                return Config::default();
            }
        } else {
            return Config::default();
        }
    }
}
