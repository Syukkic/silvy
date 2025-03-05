use anyhow::{Context, Result};
use dirs::{config_dir, data_local_dir};
use std::fs::{File, create_dir_all};
use std::path::PathBuf;

pub struct Config {
    pub db_path: PathBuf,
    pub urls_path: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self> {
        let config_path = config_dir().context("Unable to determine the config directory")?;
        let local_path = data_local_dir().context("Unable to determine the local directory")?;

        let silvy_config_path = Self::create_dir_if_not_exists(config_path.join("silvy"))?;
        let silvy_local_path = Self::create_dir_if_not_exists(local_path.join("silvy"))?;
        let cache_db = Self::create_file_if_not_exists(silvy_local_path.join("cache.db"))?;
        let urls_file = Self::create_file_if_not_exists(silvy_config_path.join("urls"))?;

        Ok(Self {
            db_path: cache_db,
            urls_path: urls_file,
        })
    }

    fn create_dir_if_not_exists(path: PathBuf) -> Result<PathBuf> {
        if !path.exists() {
            create_dir_all(&path)
                .with_context(|| format!("Failed to create {} directory", path.display()))?
        };

        Ok(path)
    }

    fn create_file_if_not_exists(path: PathBuf) -> Result<PathBuf> {
        if !path.exists() {
            File::create(&path)
                .with_context(|| format!("Failed to create database file: {}", path.display()))?;
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_paths() {
        let config = Config::new().expect("Failed to create config");
        let config_path = config_dir()
            .expect("Unable to determine config directory")
            .join("silvy")
            .canonicalize()
            .expect("Failed to canonicalize config directory");

        let local_path = data_local_dir()
            .expect("Unable to determine local directory")
            .join("silvy")
            .canonicalize()
            .expect("Failed to canonicalize local directory");

        assert_eq!(
            config
                .db_path
                .canonicalize()
                .expect("Failed to canonicalize db_path"),
            local_path.join("cache.db")
        );
        assert_eq!(
            config
                .urls_path
                .canonicalize()
                .expect("Failed to canonicalize urls_path"),
            config_path.join("urls")
        );
    }
}
