use dirs::{config_dir, data_local_dir};
use std::{fs::create_dir_all, path::PathBuf};

pub struct Config {
    pub db_path: PathBuf,
    pub urls_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let config_path = config_dir().expect("Unable to determine the config directory");
        let local_path = data_local_dir().expect("Unable to determine the local directory");

        let silvy_config_path = Self::create_dir_if_not_exists(config_path.join("silvy"));
        let silvy_local_path = Self::create_dir_if_not_exists(local_path.join("silvy"));

        Self {
            db_path: silvy_local_path.join("cache.db"),
            urls_path: silvy_config_path.join("urls"),
        }
    }

    fn create_dir_if_not_exists(path: PathBuf) -> PathBuf {
        if !path.exists() {
            if let Err(e) = create_dir_all(&path) {
                eprintln!(
                    "Warning: Failed to create {} directory: {}",
                    path.display(),
                    e
                );
            }
        }
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_paths() {
        let config = Config::new();
        let config_path = config_dir().expect("Unable to determine config directory");
        let local_path = data_local_dir().expect("Unable to determine config directory");

        assert_eq!(config.db_path, local_path.join("./silvy/cache.db"));
        assert_eq!(config.urls_path, config_path.join("./silvy/urls"));
    }
}
