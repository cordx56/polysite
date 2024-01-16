use super::Rule;
use crate::Metadata;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Rule {} is not found", .0)]
    RuleNotFound(String),
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
    pub(crate) rules: Arc<Mutex<HashMap<String, Arc<Mutex<Rule>>>>>,
    pub(crate) compiling: Option<Compiling>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(Mutex::new(HashMap::new())),
            compiling: None,
        }
    }

    pub async fn add_rule(&mut self, name: impl ToString, rule: Rule) {
        self.rules
            .lock()
            .await
            .insert(name.to_string(), Arc::new(Mutex::new(rule)));
    }

    pub async fn load(&self, name: impl AsRef<str>) -> Result<Metadata> {
        let named = name.as_ref();
        let notify = {
            self.rules
                .lock()
                .await
                .get(named)
                .ok_or(anyhow!(ContextError::RuleNotFound(named.to_string())))?
                .lock()
                .await
                .get_load_notify()
        };
        if let Some(n) = notify {
            n.notified().await;
        }
        let metadata = self
            .rules
            .lock()
            .await
            .get(named)
            .ok_or(anyhow!(ContextError::RuleNotFound(named.to_string())))?
            .lock()
            .await
            .get_metadata()
            .unwrap();
        Ok(metadata)
    }

    pub fn compiling(&self) -> &Option<Compiling> {
        &self.compiling
    }

    /// Get compiling source file path
    pub fn source(&self) -> PathBuf {
        self.compiling.as_ref().unwrap().source()
    }
    /// Get compiling target file path
    pub fn target(&self) -> PathBuf {
        self.compiling.as_ref().unwrap().target()
    }

    pub(crate) fn set_compiling(&mut self, compiling: Compiling) {
        self.compiling = Some(compiling);
    }
}
