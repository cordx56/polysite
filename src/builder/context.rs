use super::{metadata::*, snapshot::*};
use crate::*;
use anyhow::{anyhow, Context as _, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

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
impl<S: AsRef<str>> From<S> for Version {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_owned())
    }
}

#[derive(Clone)]
pub struct Compiling {
    metadata: Metadata,
    snapshot_stage: SnapshotStage,
}
impl Compiling {
    pub fn new(snapshot_stage: SnapshotStage) -> Self {
        let metadata = Metadata::new();
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
    metadata: Metadata,
    versions: Arc<RwLock<HashMap<Version, HashMap<PathBuf, Metadata>>>>,
    compiling: Option<Compiling>,
    snapshot_managers: Arc<RwLock<HashMap<String, SnapshotManager>>>,
    config: Config,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            metadata: Metadata::new(),
            versions: Arc::new(RwLock::new(HashMap::new())),
            compiling: None,
            snapshot_managers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get [`Metadata`] that merges global and compiling metadata
    pub fn metadata(&self) -> Metadata {
        let mut global = self.metadata.clone();
        if let Some(c) = &self.compiling {
            global = Metadata::join(global, c.metadata.clone());
        }
        let mut map = HashMap::new();
        for (rule, snap) in self.snapshot_managers.read().unwrap().iter() {
            if let Some(meta) = snap.metadata() {
                map.insert(
                    rule.to_owned(),
                    Metadata::Array(Arc::new(RwLock::new(meta))),
                );
            }
        }
        global = Metadata::join(global, Metadata::Map(Arc::new(RwLock::new(map))));
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
    /// Insert [`Metadata`] value to global metadata
    pub fn insert_global_metadata(&self, name: impl AsRef<str>, metadata: Metadata) {
        if let Metadata::Map(map) = &self.metadata {
            map.write()
                .unwrap()
                .insert(name.as_ref().to_owned(), metadata);
        }
    }
    /// Insert [`Metadata`] value to compiling metadata
    pub fn insert_compiling_metadata(&mut self, name: impl AsRef<str>, metadata: Metadata) {
        if let Some(compiling) = &self.compiling {
            if let Metadata::Map(map) = &compiling.metadata {
                map.write()
                    .unwrap()
                    .insert(name.as_ref().to_owned(), metadata);
            }
        }
    }

    /// Get currently compiling [`Version`]
    pub fn version(&self) -> Result<Version> {
        let compiling = self.compiling_metadata()?;
        let version = compiling
            .get(VERSION_META)
            .ok_or(anyhow!("Rule metadata not set!"))?
            .as_str()
            .ok_or(anyhow!("Invalid value"))?;
        let version = version.read().unwrap();
        let version = version.as_str();
        Ok(version.into())
    }
    /// Get specified [`Version`] and source's metadata'
    pub fn get_version_metadata(
        &self,
        version: impl Into<Version>,
        path: &PathBuf,
    ) -> Option<Metadata> {
        let version = version.into();
        let versions = self.versions.read().unwrap();
        if let Some(v) = versions.get(&version) {
            v.get(path).cloned()
        } else {
            None
        }
    }
    /// Insert specified [`Version`] and source's metadata
    pub fn insert_version(
        &self,
        version: impl Into<Version>,
        path: PathBuf,
        metadata: Metadata,
    ) -> Result<()> {
        let version = version.into();
        let mut versions = self.versions.write().unwrap();
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

    /// Register [`SnapshotManager`]
    pub(crate) fn register_snapshot_manager(&self, s: impl AsRef<str>, m: SnapshotManager) {
        self.snapshot_managers
            .write()
            .unwrap()
            .insert(s.as_ref().to_owned(), m);
    }
    /// Wait snapshot until specified stage.
    /// In most cases, you would like to wait until stage 1, that means "first snapshot was taken".
    pub async fn wait_snapshot_until(&self, name: impl AsRef<str>, stage: usize) -> Result<()> {
        let name = name.as_ref();
        let manager = {
            self.snapshot_managers
                .read()
                .unwrap()
                .get(name)
                .ok_or(anyhow!("Rule {} not found", name))?
                .clone()
        };
        manager.wait_until(stage).await;
        Ok(())
    }
    /// Save currently compiling [`Metadata`] as snapshot
    pub fn save_snapshot(&self) -> Result<()> {
        let compiling_metadata = self.compiling_metadata()?.clone();
        self.compiling
            .as_ref()
            .context("Not compiling")?
            .snapshot_stage
            .push(compiling_metadata);
        Ok(())
    }

    /// Get currently compiling rule name
    pub fn rule(&self) -> Result<String> {
        let compiling = self.compiling_metadata()?;
        let binding = compiling
            .get(RULE_META)
            .ok_or(anyhow!("Rule metadata not set!"))?;
        let rule = binding
            .as_str()
            .ok_or(anyhow!("Invalid value"))?
            .read()
            .unwrap()
            .clone();
        Ok(rule)
    }

    /// Get currently compiling source file path
    pub fn source(&self) -> Result<PathBuf> {
        let compiling = self.compiling_metadata()?;
        let source = compiling
            .get(SOURCE_FILE_META)
            .ok_or(anyhow!("Source file metadata not set!"))?
            .as_str()
            .ok_or(anyhow!("Invalid value"))?
            .read()
            .unwrap()
            .clone();
        Ok(PathBuf::from(source))
    }
    /// Get currently compiling target file path
    pub fn target(&self) -> Result<PathBuf> {
        let compiling = self.compiling_metadata()?;
        let target = compiling
            .get(TARGET_FILE_META)
            .ok_or(anyhow!("Target file metadata not set!"))?
            .as_str()
            .ok_or(anyhow!("Invalid value"))?
            .read()
            .unwrap()
            .clone();
        Ok(PathBuf::from(target))
    }
    /// Get currently compiling URL path
    pub fn path(&self) -> Result<PathBuf> {
        let compiling = self.compiling_metadata()?;
        let path = compiling
            .get(PATH_META)
            .ok_or(anyhow!("Path metadata not set!"))?
            .as_str()
            .ok_or(anyhow!("Invalid value"))?
            .read()
            .unwrap()
            .clone();
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
            Ok(Metadata::String(Arc::new(RwLock::new(s))))
        } else {
            Ok(Metadata::Bytes(Arc::new(RwLock::new(body))))
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
