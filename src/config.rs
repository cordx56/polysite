use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    source_dir: PathBuf,
    target_dir: PathBuf,
    target_clean: bool,
}

impl Config {
    /// Get source directory
    pub fn source_dir(&self) -> PathBuf {
        self.source_dir.clone()
    }
    /// Set source directory
    pub fn set_source_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.source_dir = path.into();
        self
    }
    /// Get target directory
    pub fn target_dir(&self) -> PathBuf {
        self.target_dir.clone()
    }
    /// Set target directory
    pub fn set_target_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.target_dir = path.into();
        self
    }
    /// Get target directory clean config
    pub fn target_clean(&self) -> bool {
        self.target_clean
    }
    /// Set target directory clean config
    pub fn set_target_clean(mut self, clean: bool) -> Self {
        self.target_clean = clean;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source_dir: PathBuf::from("site"),
            target_dir: PathBuf::from("dist"),
            target_clean: true,
        }
    }
}
