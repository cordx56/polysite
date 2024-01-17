use crate::error::{here, Location};
use crate::Context;
use crate::Rule;
use anyhow::{anyhow, Error, Result};
use serde::Serialize;
use thiserror::Error;
use tokio::task::{JoinError, JoinSet};

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Join error: {:?} on {}", .1, .0)]
    JoinError(Location, JoinError),
    #[error("Compile error: {} on {}", .1, .0)]
    CompileError(Location, Error),
}

pub struct Builder {
    ctx: Context,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            ctx: Context::new(),
        }
    }
    pub async fn add_rule(mut self, rule: Rule) -> Self {
        self.ctx.add_rule(rule).await;
        self
    }

    /// Insert metadata
    pub async fn add_context(&self, name: impl ToString, data: impl Serialize) -> Result<()> {
        self.ctx.insert(name, data).await
    }

    /// Run build
    ///
    /// Compile all rules
    pub async fn build(&mut self) -> Result<()> {
        let mut set = JoinSet::new();
        for rule in self.ctx.rules.lock().await.values() {
            let ctx = self.ctx.clone();
            let r = rule.clone();
            set.spawn(async move { r.lock().await.compile(ctx).await });
        }
        while let Some(join_res) = set.join_next().await {
            join_res
                .map_err(|e| anyhow!(BuildError::JoinError(here!(), e)))?
                .map_err(|e| anyhow!(BuildError::CompileError(here!(), e)))?;
        }
        Ok(())
    }
}
