use crate::*;
use anyhow::{anyhow, Result};
use log::info;
use serde::Serialize;
use std::fs::remove_dir_all;
use tokio::task::JoinSet;

pub struct Builder {
    ctx: Context,
    steps: Vec<Vec<Rule>>,
}

impl Builder {
    /// Create new builder with config
    pub fn new(config: Config) -> Self {
        Self {
            ctx: Context::new(config),
            steps: Vec::new(),
        }
    }

    /// Insert global metadata
    ///
    /// You can pass anything which can be serialized and deserialized to
    /// serde_json::Value
    pub async fn insert_metadata(
        &mut self,
        name: impl ToString,
        data: impl Serialize,
    ) -> Result<()> {
        self.ctx.insert_global_metadata(name, data).await
    }

    /// Add build step
    ///
    /// This method receives rules as a parameter and push as build step
    /// Rules registered in a same step will be built concurrently
    pub fn add_step(mut self, step: impl IntoIterator<Item = Rule>) -> Self {
        self.steps.push(step.into_iter().collect());
        self
    }

    /// Run build
    ///
    /// Run all registered build steps
    pub async fn build(self) -> Result<()> {
        let conf = self.ctx.config();
        let target_dir = conf.target_dir();
        if conf.target_clean() && target_dir.is_dir() {
            remove_dir_all(&target_dir)
                .map_err(|e| anyhow!("Target directory cleaning error: {:?}", e))?;
            info!("Target directory ({}) cleaned", target_dir.display());
        }
        for step in self.steps.into_iter() {
            let mut set = JoinSet::new();
            for mut rule in step.into_iter() {
                let ctx = self.ctx.clone();
                rule.eval_conditions(&ctx).await?;
                set.spawn(async move { (rule.get_name(), rule.compile(ctx).await) });
            }
            while let Some(join_res) = set.join_next().await {
                let (name, res) = join_res.map_err(|e| anyhow!("Join error: {:?}", e))?;
                let _ctx = res.map_err(|e| anyhow!("Rule {}: compile error: {}", name, e))?;
            }
        }
        Ok(())
    }
}
