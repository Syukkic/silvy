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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_paths() {
        let temp_config_dir = TempDir::new().expect("Failed to create temp config dir");
        let temp_local_dir = TempDir::new().expect("Failed to create temp local dir");

        let mock_config_dir = temp_config_dir.path().to_path_buf();
        let mock_local_dir = temp_local_dir.path().to_path_buf();

        let silvy_config_path = Config::create_dir_if_not_exists(mock_config_dir.join("silvy"))
            .expect("Failed to create config directory");
        let silvy_local_path = Config::create_dir_if_not_exists(mock_local_dir.join("silvy"))
            .expect("Failed to create local directory");

        let cache_db = Config::create_file_if_not_exists(silvy_local_path.join("cache.db"))
            .expect("Failed to create cache.db");
        let urls_file = Config::create_file_if_not_exists(silvy_config_path.join("urls"))
            .expect("Failed to create urls file");

        assert_eq!(
            cache_db.canonicalize().unwrap(),
            silvy_local_path.join("cache.db").canonicalize().unwrap()
        );
        assert_eq!(
            urls_file.canonicalize().unwrap(),
            silvy_config_path.join("urls").canonicalize().unwrap()
        );

        assert!(fs::metadata(&cache_db).is_ok());
        assert!(fs::metadata(&urls_file).is_ok());
    }
}
