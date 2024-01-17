use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default, Clone, Debug)]
pub struct Config {
    src_dir: Option<PathBuf>,
    dist_dir: Option<PathBuf>,
}

impl Config {
    pub fn src_dir(&self) -> PathBuf {
        self.src_dir.clone().unwrap_or(PathBuf::from("site"))
    }
    pub fn dist_dir(&self) -> PathBuf {
        self.dist_dir.clone().unwrap_or(PathBuf::from("dist"))
    }
}
