use super::snapshot::*;
use crate::*;
use anyhow::{anyhow, Context as _, Result};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// [`Version`] represents compilation file version.
/// If the same version of a source file path is registered for compilation, that file will be
/// skipped.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Version(String);
impl Version {
    pub fn get(&self) -> String {
        self.0.clone()
    }
}
impl Default for Version {
    fn default() -> Self {
        Self("default".to_string())
    }
}
impl<S: ToString> From<S> for Version {
    fn from(value: S) -> Self {
        Self(value.to_string())
    }
}

#[derive(Clone)]
pub struct Compiling {
    metadata: Metadata,
    snapshot_stage: SnapshotStage,
}
impl Compiling {
    pub fn new(snapshot_stage: SnapshotStage) -> Self {
        let metadata = new_object();
        Self {
            metadata,
            snapshot_stage,
        }
    }
}

#[derive(Clone)]
/// Compiling context
///
/// This holds global, compiling (local) and versions' [`Metadata`], snapshot manager.
/// This also provides some helper methods.
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

    /// Get [`Metadata`] that merges global and compiling metadata
    pub async fn metadata(&self) -> Metadata {
        let mut global = self.metadata.lock().await.clone();
        if let Some(c) = &self.compiling {
            global = join_metadata(global, c.metadata.clone());
        }
        global
    }
    /// Get compiling [`Metadata`]
    pub fn compiling_metadata(&self) -> Result<&Metadata> {
        Ok(&self
            .compiling
            .as_ref()
            .ok_or(anyhow!("Not compiling"))?
            .metadata)
    }
    /// Insert global [`Metadata`]
    ///
    /// You can pass anything which can be serialized and deserialized to [`serde_json::Value`].
    pub async fn insert_global_metadata(
        &self,
        name: impl ToString,
        value: impl Serialize,
    ) -> Result<()> {
        let metadata = Metadata::from_serializable(value)?;
        self.metadata
            .lock()
            .await
            .as_object_mut()
            .unwrap()
            .insert(name.to_string(), metadata);
        Ok(())
    }
    /// Insert compiling [`Metadata`]
    ///
    /// You can pass anything which can be serialized and deserialized to [`serde_json::Value`].
    pub fn insert_compiling_metadata(
        &mut self,
        name: impl ToString,
        value: impl Serialize,
    ) -> Result<()> {
        let metadata = Metadata::from_serializable(value)?;
        self.compiling
            .as_mut()
            .ok_or(anyhow!("Not compiling"))?
            .metadata
            .as_object_mut()
            .unwrap()
            .insert(name.to_string(), metadata);
        Ok(())
    }
    /// Insert raw [`Metadata`] value to global metadata
    pub async fn insert_global_raw_metadata(
        &self,
        name: impl ToString,
        metadata: Metadata,
    ) -> Result<()> {
        self.metadata
            .lock()
            .await
            .as_object_mut()
            .unwrap()
            .insert(name.to_string(), metadata);
        Ok(())
    }
    /// Insert raw [`Metadata`] value to compiling metadata
    pub fn insert_compiling_raw_metadata(
        &mut self,
        name: impl ToString,
        metadata: Metadata,
    ) -> Result<()> {
        self.compiling
            .as_mut()
            .ok_or(anyhow!("Not compiling"))?
            .metadata
            .as_object_mut()
            .unwrap()
            .insert(name.to_string(), metadata);
        Ok(())
    }

    /// Get currently compiling [`Version`]
    pub fn version(&self) -> Result<Version> {
        let compiling = self.compiling_metadata()?;
        let version = compiling
            .get(VERSION_META)
            .ok_or(anyhow!("Rule metadata not set!"))?
            .as_str()
            .ok_or(anyhow!("Invalid value"))?;
        Ok(version.into())
    }
    /// Get specified [`Version`] and source's metadata'
    pub async fn get_version_metadata(
        &self,
        version: impl Into<Version>,
        path: &PathBuf,
    ) -> Option<Metadata> {
        let version = version.into();
        let versions = self.versions.lock().await;
        if let Some(v) = versions.get(&version) {
            v.get(path).cloned()
        } else {
            None
        }
    }
    /// Insert specified [`Version`] and source's metadata
    pub async fn insert_version(
        &self,
        version: impl Into<Version>,
        path: PathBuf,
        metadata: Metadata,
    ) -> Result<()> {
        let version = version.into();
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
    /// Wait snapshot until specified stage.
    /// In most cases, you would like to wait until stage 1, that means "first snapshot was taken".
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
    /// Save currently compiling [`Metadata`] as snapshot
    pub async fn save_snapshot(&self) -> Result<()> {
        let rule = self.rule()?;
        let compiling_metadata = self.compiling_metadata()?.clone();
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

    /// Get currently compiling rule name
    pub fn rule(&self) -> Result<String> {
        let compiling = self.compiling_metadata()?;
        let rule = compiling
            .get(RULE_META)
            .ok_or(anyhow!("Rule metadata not set!"))?
            .as_str()
            .ok_or(anyhow!("Invalid value"))?;
        Ok(rule.to_string())
    }

    /// Get currently compiling source file path
    pub fn source(&self) -> Result<PathBuf> {
        let compiling = self.compiling_metadata()?;
        let source = compiling
            .get(SOURCE_FILE_META)
            .ok_or(anyhow!("Source file metadata not set!"))?
            .as_str()
            .ok_or(anyhow!("Invalid value"))?;
        Ok(PathBuf::from(source))
    }
    /// Get currently compiling target file path
    pub fn target(&self) -> Result<PathBuf> {
        let compiling = self.compiling_metadata()?;
        let target = compiling
            .get(TARGET_FILE_META)
            .ok_or(anyhow!("Target file metadata not set!"))?
            .as_str()
            .ok_or(anyhow!("Invalid value"))?;
        Ok(PathBuf::from(target))
    }
    /// Get currently compiling URL path
    pub fn path(&self) -> Result<PathBuf> {
        let compiling = self.compiling_metadata()?;
        let path = compiling
            .get(PATH_META)
            .ok_or(anyhow!("Path metadata not set!"))?
            .as_str()
            .ok_or(anyhow!("Invalid value"))?;
        Ok(PathBuf::from(path))
    }
    /// Get currently compiling body [`Metadata`], which can be [`Vec<u8>`] or [`String`].
    pub fn body(&self) -> Result<Metadata> {
        let compiling = self.compiling_metadata()?;
        let body = compiling
            .get(BODY_META)
            .ok_or(anyhow!("Target file metadata not set!"))?
            .clone();
        Ok(body)
    }
    /// Get [`Config`]
    pub fn config(&self) -> Config {
        self.config.clone()
    }

    /// Get source file body as bytes
    pub fn get_source_body(&self) -> Result<Vec<u8>> {
        let file = self.source()?;
        fs::read(&file).context("File read error")
    }
    /// Get source file string
    pub fn get_source_string(&self) -> Result<String> {
        String::from_utf8(self.get_source_body()?).context("String encode error")
    }
    /// Get source file data as [`Metadata`]
    pub fn get_source_data(&self) -> Result<Metadata> {
        let body = self.get_source_body()?;
        if let Ok(s) = String::from_utf8(body.clone()) {
            Ok(Metadata::String(s))
        } else {
            Ok(Metadata::from_bytes(body))
        }
    }
    /// Create target file's parent directory
    pub fn create_target_parent_dir(&self) -> Result<()> {
        let target = self.target()?;
        let dir = target.parent().unwrap();
        fs::create_dir_all(&dir).context("Directory creation error")
    }
    /// Open target file to write
    pub fn open_target(&self) -> Result<fs::File> {
        self.create_target_parent_dir()?;
        let target = self.target()?;
        let file = fs::File::create(&target).context("Target file open error")?;
        Ok(file)
    }
}
