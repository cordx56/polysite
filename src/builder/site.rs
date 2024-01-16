use crate::Context;
use crate::Rule;
use anyhow::{anyhow, Error, Result};
use thiserror::Error;
use tokio::task::{JoinError, JoinSet};

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Join error: {:?}", .0)]
    JoinError(JoinError),
    #[error("Compile error: {}", .0)]
    CompileError(Error),
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
    pub async fn add_rule(mut self, name: impl ToString, rule: Rule) -> Self {
        self.ctx.add_rule(name, rule).await;
        self
    }

    pub async fn build(&mut self) -> Result<()> {
        let mut set = JoinSet::new();
        for rule in self.ctx.rules.lock().await.values() {
            let ctx = self.ctx.clone();
            let r = rule.clone();
            set.spawn(async move { r.lock().await.compile(ctx).await });
        }
        while let Some(join_res) = set.join_next().await {
            join_res
                .map_err(|e| anyhow!(BuildError::JoinError(e)))?
                .map_err(|e| anyhow!(BuildError::CompileError(e)))?;
        }
        Ok(())
    }
}
