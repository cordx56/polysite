use crate::{
    error::{here, Location},
    to_metadata, Metadata, Rule,
};
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Data {} is not found on {}", .1, .0)]
    DataNotFound(Location, String),
    #[error("Rule {} is not found on {}", .1, .0)]
    RuleNotFound(Location, String),
}

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
    pub(crate) metadata: Arc<Mutex<Metadata>>,
    pub(crate) rules: Arc<Mutex<HashMap<String, Arc<Mutex<Rule>>>>>,
    pub(crate) compiling: Option<Compiling>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            metadata: Arc::new(Mutex::new(json!({}))),
            rules: Arc::new(Mutex::new(HashMap::new())),
            compiling: None,
        }
    }

    pub async fn add_rule(&mut self, rule: Rule) {
        let name = rule.get_name();
        self.rules
            .lock()
            .await
            .insert(name, Arc::new(Mutex::new(rule)));
    }

    /// Wait for specified rule compile completion
    pub async fn wait(&self, name: impl ToString) -> Result<()> {
        let name = name.to_string();
        let notify = {
            self.rules
                .lock()
                .await
                .get(&name)
                .ok_or(anyhow!(ContextError::RuleNotFound(here!(), name)))?
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
    pub async fn get(&self, name: impl ToString) -> Result<Metadata> {
        let name = name.to_string();
        let metadata = self.metadata.lock().await;
        let data = metadata
            .as_object()
            .unwrap()
            .get(&name)
            .ok_or(anyhow!(ContextError::DataNotFound(here!(), name)))?;
        Ok(data.clone())
    }
    /// Insert metadata
    pub async fn insert(&self, name: impl ToString, value: impl Serialize) -> Result<()> {
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
}
