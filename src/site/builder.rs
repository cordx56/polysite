use super::context::SiteContext;
use crate::rule::Rule;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

pub type Rules = Arc<Mutex<HashMap<String, Arc<Mutex<Rule>>>>>;

pub struct SiteBuilder {
    ctx: Arc<Mutex<SiteContext>>,
}

impl SiteBuilder {
    pub fn new() -> Self {
        SiteBuilder {
            ctx: Arc::new(Mutex::new(SiteContext::new())),
        }
    }
    pub async fn add_rule(self, name: impl ToString, rule: Rule) -> Self {
        self.ctx.lock().await.add_rule(name, rule).await;
        self
    }

    pub async fn build(&mut self) -> Result<(), String> {
        let mut set = JoinSet::new();
        for rule in self.ctx.lock().await.rules.lock().await.values() {
            let ctx = self.ctx.clone();
            let r = rule.clone();
            set.spawn(async move { r.lock().await.compile(ctx).await });
        }
        while let Some(join_res) = set.join_next().await {
            join_res
                .or(Err("Join error".to_string()))?
                .or_else(|e| Err(format!("Build error: ")))?;
        }
        Ok(())
    }
}
