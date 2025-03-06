use crate::config::Config;

pub mod fetcher;

use anyhow::{Context, Ok, Result};
use std::fs;
use url::Url;

pub struct UrlEntry {
    pub url: Url,
    pub tags: Option<Vec<String>>,
}

impl UrlEntry {
    pub fn parse_entry(line: &str) -> Result<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty line"));
        }

        let url = Url::parse(parts[0]).context("Invaild URL format")?;
        let tags = if parts.len() > 1 {
            Some(parts[1..].iter().map(|s| s.to_string()).collect())
        } else {
            None
        };

        Ok(Self { url, tags })
    }

    pub fn from_file(config: &Config) -> Result<Vec<Self>> {
        let urls_path = &config.urls_path;
        let contents =
            fs::read_to_string(&urls_path).context("Failed to read content from file")?;
        let mut entries = Vec::new();
        for line in contents.lines() {
            let entry = Self::parse_entry(line).context("Failed to parse line")?;
            entries.push(entry);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, path::PathBuf};
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_entry() {
        let entry = UrlEntry::parse_entry("http://example.com tag1 tag2").unwrap();
        assert_eq!(entry.url.to_string(), "http://example.com/");
        assert_eq!(
            entry.tags,
            Some(vec!["tag1".to_string(), "tag2".to_string()])
        );

        let entry = UrlEntry::parse_entry("http://example.com").unwrap();
        assert_eq!(entry.url.to_string(), "http://example.com/");
        assert_eq!(entry.tags, None);

        let entry = UrlEntry::parse_entry("");
        assert!(entry.is_err());
    }
    #[test]
    fn test_from_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "https://this-week-in-rust.org/rss.xml").unwrap();
        writeln!(temp_file, "https://blog.cloudflare.com/rss tag ").unwrap();
        writeln!(temp_file, "https://blog.metanoise.in/atom.xml tag1 tag2").unwrap();

        let config = Config {
            db_path: PathBuf::new(),
            urls_path: PathBuf::from(temp_file.path()),
        };

        let entries = UrlEntry::from_file(&config).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries[0].url.to_string(),
            "https://this-week-in-rust.org/rss.xml"
        );
        assert_eq!(entries[0].tags, None);
        assert_eq!(
            entries[1].url.to_string(),
            "https://blog.cloudflare.com/rss"
        );
        assert_eq!(entries[1].tags, Some(vec!["tag".to_string()]));

        assert_eq!(
            entries[2].url.to_string(),
            "https://blog.metanoise.in/atom.xml"
        );
        assert_eq!(
            entries[2].tags,
            Some(vec!["tag1".to_string(), "tag2".to_string()])
        );
    }
}
