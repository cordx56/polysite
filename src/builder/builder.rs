use crate::{
    error::{here, Location},
    *,
};
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
    pub fn new(config: Config) -> Self {
        Self {
            ctx: Context::new(config),
        }
    }
    pub async fn add_rule(mut self, rule: Rule) -> Self {
        self.ctx.add_rule(rule);
        self
    }

    /// Insert metadata
    pub async fn add_context(&mut self, name: impl ToString, data: impl Serialize) -> Result<()> {
        self.ctx.insert(name, data).await
    }

    /// Run build
    ///
    /// Compile all rules
    pub async fn build(&mut self) -> Result<()> {
        let mut set = JoinSet::new();
        let rules = self.ctx.get_rules().clone();
        for rule in rules.into_values() {
            let ctx = self.ctx.clone();
            set.spawn(async move {
                let cloned = rule.clone();
                let mut locked = cloned.lock().await;
                (rule, locked.compile(ctx).await)
            });
        }
        while let Some(join_res) = set.join_next().await {
            let (rule, res) =
                join_res.map_err(|e| anyhow!("Join error: {:?} on {}", e, here!()))?;
            let mut rule = rule.lock().await;
            let name = rule.get_name();
            let res = res.map_err(|e| anyhow!("Rule {}: compile error: {}", name, e))?;
            self.ctx.insert(name, res).await?;
            rule.set_finished();
        }
        Ok(())
    }
}
