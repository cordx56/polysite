use crate::{
    error::{here, Location},
    Config, Context, Rule,
};
use anyhow::{anyhow, Error, Result};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::{JoinError, JoinSet};

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Join error: {:?} on {}", .1, .0)]
    JoinError(Location, JoinError),
    #[error("Compile error: {} on {}", .1, .0)]
    CompileError(Location, Error),
}

pub struct Builder {
    ctx: Arc<Mutex<Context>>,
}

impl Builder {
    pub fn new(config: Config) -> Self {
        Self {
            ctx: Arc::new(Mutex::new(Context::new(config))),
        }
    }
    pub async fn add_rule(self, rule: Rule) -> Self {
        self.ctx.lock().await.add_rule(rule);
        self
    }

    /// Insert metadata
    pub async fn add_context(&self, name: impl ToString, data: impl Serialize) -> Result<()> {
        self.ctx.lock().await.insert(name, data)
    }

    /// Run build
    ///
    /// Compile all rules
    pub async fn build(&mut self) -> Result<()> {
        let mut set = JoinSet::new();
        let rules = self.ctx.lock().await.get_rules().clone();
        for rule in rules.into_values() {
            let ctx = self.ctx.lock().await.clone();
            set.spawn(async move { rule.lock().await.compile(ctx).await });
        }
        while let Some(join_res) = set.join_next().await {
            join_res
                .map_err(|e| anyhow!(BuildError::JoinError(here!(), e)))?
                .map_err(|e| anyhow!(BuildError::CompileError(here!(), e)))?;
        }
        Ok(())
    }
}
