use super::snapshot::*;
use crate::{error::here, *};
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Version(String);
impl Version {
    pub fn new(s: Option<String>) -> Self {
        Self(s.unwrap_or("default".to_string()))
    }
    pub fn get(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone)]
pub(crate) struct Compiling {
    rule: String,
    source: PathBuf,
    target: PathBuf,
    version: Version,
    metadata: Metadata,
    snapshot_stage: SnapshotStage,
}
impl Compiling {
    pub fn new(
        rule: String,
        source: PathBuf,
        target: PathBuf,
        version: Version,
        snapshot_stage: SnapshotStage,
    ) -> Self {
        let mut metadata = new_object();
        let obj = metadata.as_object_mut().unwrap();
        obj.insert(
            "source".to_string(),
            Metadata::String(source.to_string_lossy().to_string()),
        );
        obj.insert(
            "target".to_string(),
            Metadata::String(target.to_string_lossy().to_string()),
        );
        obj.insert("version".to_string(), Metadata::String(version.get()));
        Self {
            rule,
            source,
            target,
            version,
            metadata,
            snapshot_stage,
        }
    }
}

#[derive(Clone)]
pub struct Context {
    metadata: Arc<Mutex<Metadata>>,
    versions: Arc<Mutex<HashMap<Version, HashMap<PathBuf, Metadata>>>>,
    compiling: Option<Compiling>,
    snapshot_managers: Arc<Mutex<HashMap<String, SnapshotManager>>>,
    config: Config,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            metadata: Arc::new(Mutex::new(json!({}))),
            versions: Arc::new(Mutex::new(HashMap::new())),
            compiling: None,
            snapshot_managers: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Get metadata
    pub async fn metadata(&self) -> Metadata {
        let mut global = self.metadata.lock().await.clone();
        if let Some(c) = &self.compiling {
            global = join_metadata(global, c.metadata.clone());
        }
        global
    }
    /// Get compiling metadata
    pub fn compiling_metadata(&self) -> Result<Metadata> {
        Ok(self
            .compiling
            .as_ref()
            .ok_or(anyhow!("Not compiling on {}", here!()))?
            .metadata
            .clone())
    }
    /// Insert global metadata
    pub async fn insert_global_metadata(
        &self,
        name: impl ToString,
        value: impl Serialize,
    ) -> Result<()> {
        let metadata = to_metadata(value)?;
        self.metadata
            .lock()
            .await
            .as_object_mut()
            .unwrap()
            .insert(name.to_string(), metadata);
        Ok(())
    }
    /// Insert compiling metadata
    pub async fn insert_compiling_metadata(
        &mut self,
        name: impl ToString,
        value: impl Serialize,
    ) -> Result<()> {
        let metadata = to_metadata(value)?;
        self.compiling
            .as_mut()
            .ok_or(anyhow!("Not compiling on {}", here!()))?
            .metadata
            .as_object_mut()
            .unwrap()
            .insert(name.to_string(), metadata);
        Ok(())
    }

    /// Get compiling version
    pub fn version(&self) -> Result<Version> {
        Ok(self
            .compiling
            .as_ref()
            .ok_or(anyhow!("Not compiling on {}", here!()))?
            .version
            .clone())
    }
    /// Get version
    pub async fn get_version(&self, version: &Version, path: &PathBuf) -> Option<Metadata> {
        let versions = self.versions.lock().await;
        if let Some(v) = versions.get(version) {
            v.get(path).cloned()
        } else {
            None
        }
    }
    /// Insert version
    pub async fn insert_version(
        &self,
        version: Version,
        path: PathBuf,
        metadata: Metadata,
    ) -> Result<()> {
        let mut versions = self.versions.lock().await;
        let version = match versions.get_mut(&version) {
            Some(v) => v,
            None => {
                versions.insert(version.clone(), HashMap::new());
                versions.get_mut(&version).unwrap()
            }
        };
        version.insert(path, metadata);
        Ok(())
    }

    pub(crate) fn set_compiling(&mut self, compiling: Option<Compiling>) {
        self.compiling = compiling;
    }

    /// Register SnapshotManager
    pub(crate) async fn register_snapshot_manager(&self, s: impl ToString, m: SnapshotManager) {
        self.snapshot_managers.lock().await.insert(s.to_string(), m);
    }
    /// Wait snapshot
    pub async fn wait_snapshot_until(&self, name: impl ToString, stage: usize) -> Result<()> {
        let name = name.to_string();
        self.snapshot_managers
            .lock()
            .await
            .get(&name)
            .ok_or(anyhow!("Rule {} not found", name))?
            .wait_until(stage)
            .await;
        Ok(())
    }
    /// Save current compiling metadata as snapshot
    pub async fn save_snapshot(&self) -> Result<()> {
        let rule = self.rule()?;
        let compiling_metadata = self.compiling_metadata()?;
        let mut locked = self.metadata.lock().await;
        let obj = locked.as_object_mut().unwrap();
        if let Some(Metadata::Array(a)) = obj.get_mut(&rule) {
            a.push(compiling_metadata);
        } else {
            obj.insert(rule, Metadata::Array(vec![compiling_metadata]));
        }
        self.compiling
            .as_ref()
            .unwrap()
            .snapshot_stage
            .notify_waiters()
            .await;
        Ok(())
    }

    /// Get compiling rule name
    pub fn rule(&self) -> Result<String> {
        Ok(self
            .compiling
            .as_ref()
            .ok_or(anyhow!("Not compiling on {}", here!()))?
            .rule
            .clone())
    }

    /// Get compiling source file path
    pub fn source(&self) -> Result<PathBuf> {
        Ok(self
            .compiling
            .as_ref()
            .ok_or(anyhow!("Not compiling on {}", here!()))?
            .source
            .clone())
    }
    /// Get compiling target file path
    pub fn target(&self) -> Result<PathBuf> {
        Ok(self
            .compiling
            .as_ref()
            .ok_or(anyhow!("Not compiling on {}", here!()))?
            .target
            .clone())
    }
    /// Get config
    pub fn config(&self) -> Config {
        self.config.clone()
    }

    /// Get source file body
    pub fn get_source_body(&self) -> Result<Vec<u8>> {
        let file = self.source()?;
        fs::read(&file).map_err(|e| anyhow!("File read error: {:?}", e))
    }
    /// Get source file string
    pub fn get_source_string(&self) -> Result<String> {
        String::from_utf8(self.get_source_body()?)
            .map_err(|e| anyhow!("String encode error: {:?}", e))
    }
    pub fn create_target_dir(&self) -> Result<()> {
        let target = self.target()?;
        let dir = target.parent().unwrap();
        fs::create_dir_all(&dir).map_err(|e| anyhow!("Directory creation error: {:?}", e))
    }
    /// Open target file to write
    pub fn open_target(&self) -> Result<fs::File> {
        self.create_target_dir()?;
        let target = self.target()?;
        let file =
            fs::File::create(&target).map_err(|e| anyhow!("Target file open error: {:?}", e))?;
        Ok(file)
    }
}
