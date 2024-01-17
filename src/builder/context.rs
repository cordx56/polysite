use crate::*;
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Compiling {
    pub(crate) source: PathBuf,
    pub(crate) target: PathBuf,
}
impl Compiling {
    pub fn source(&self) -> PathBuf {
        self.source.clone()
    }
    pub fn target(&self) -> PathBuf {
        self.target.clone()
    }
}

#[derive(Clone)]
pub struct Context {
    metadata: Arc<Mutex<Metadata>>,
    rules: HashMap<String, Arc<Mutex<Rule>>>,
    compiling: Option<Compiling>,
    config: Config,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            metadata: Arc::new(Mutex::new(json!({}))),
            rules: HashMap::new(),
            compiling: None,
            config,
        }
    }

    pub(crate) fn add_rule(&mut self, rule: Rule) {
        let name = rule.get_name();
        self.rules.insert(name, Arc::new(Mutex::new(rule)));
    }
    pub(crate) fn get_rules(&self) -> &HashMap<String, Arc<Mutex<Rule>>> {
        &self.rules
    }

    /// Wait for specified rule compile completion
    pub async fn wait(&self, name: impl AsRef<str>) -> Result<()> {
        let name = name.as_ref();
        let notify = {
            self.rules
                .get(name)
                .ok_or(anyhow!("Rule {} not found", name))?
                .lock()
                .await
                .get_load_notify()
        };
        if let Some(n) = notify {
            n.notified().await;
        }
        Ok(())
    }
    /// Get metadata
    pub async fn metadata(&self) -> Metadata {
        self.metadata.lock().await.clone()
    }
    /// Insert metadata
    pub async fn insert(&mut self, name: impl ToString, value: impl Serialize) -> Result<()> {
        let metadata = to_metadata(value)?;
        self.metadata
            .lock()
            .await
            .as_object_mut()
            .unwrap()
            .insert(name.to_string(), metadata);
        Ok(())
    }

    pub(crate) fn set_compiling(&mut self, compiling: Compiling) {
        self.compiling = Some(compiling);
    }

    /// Get compiling source file path
    pub fn source(&self) -> PathBuf {
        self.compiling.as_ref().unwrap().source()
    }
    /// Get compiling target file path
    pub fn target(&self) -> PathBuf {
        self.compiling.as_ref().unwrap().target()
    }
    /// Get config
    pub fn config(&self) -> Config {
        self.config.clone()
    }

    /// Get source file body
    pub fn get_source_body(&self) -> Vec<u8> {
        let file = self.source();
        fs::read(&file).unwrap()
    }
    /// Get source file string
    pub fn get_source_string(&self) -> String {
        String::from_utf8(self.get_source_body()).unwrap()
    }
    /// Open target file to write
    pub fn open_target(&self) -> Result<fs::File> {
        let target = self.target();
        let dir = target.parent().unwrap();
        fs::create_dir_all(&dir).map_err(|e| anyhow!("Directory creation error: {:?}", e))?;
        let file =
            fs::File::create(&target).map_err(|e| anyhow!("Target file open error: {:?}", e))?;
        Ok(file)
    }
}