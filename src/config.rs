use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default, Clone, Debug)]
pub struct Config {
    source_dir: Option<PathBuf>,
    target_dir: Option<PathBuf>,
}

impl Config {
    /// Get source directory
    pub fn source_dir(&self) -> PathBuf {
        self.source_dir.clone().unwrap_or(PathBuf::from("site"))
    }
    /// Set source directory
    pub fn set_source_dir(&mut self, path: Option<PathBuf>) {
        self.source_dir = path;
    }
    /// Get target directory
    pub fn target_dir(&self) -> PathBuf {
        self.target_dir.clone().unwrap_or(PathBuf::from("dist"))
    }
    /// Set target directory
    pub fn set_target_dir(&mut self, path: Option<PathBuf>) {
        self.target_dir = path;
    }
}
