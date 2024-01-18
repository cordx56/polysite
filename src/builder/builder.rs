use crate::{error::here, *};
use anyhow::{anyhow, Result};
use serde::Serialize;
use tokio::task::JoinSet;

pub struct Builder {
    ctx: Context,
}

impl Builder {
    pub fn new(config: Config) -> Self {
        Self {
            ctx: Context::new(config),
        }
    }
    pub fn add_rule(mut self, rule: Rule) -> Self {
        self.ctx.add_rule(rule);
        self
    }

    /// Insert metadata
    pub async fn insert_metadata(
        &mut self,
        name: impl ToString,
        data: impl Serialize,
    ) -> Result<()> {
        self.ctx.insert_global_metadata(name, data).await
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
                let mut rule = rule.lock().await;
                (rule.get_name(), rule.compile(ctx).await)
            });
        }
        while let Some(join_res) = set.join_next().await {
            let (name, res) =
                join_res.map_err(|e| anyhow!("Join error: {:?} on {}", e, here!()))?;
            let _ctx = res.map_err(|e| anyhow!("Rule {}: compile error: {}", name, e))?;
        }
        Ok(())
    }
}
