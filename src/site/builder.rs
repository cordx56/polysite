use super::context::BuildContext;
use crate::rule::Rule;
use tokio::task::JoinSet;

pub struct SiteBuilder {
    ctx: BuildContext,
}

impl SiteBuilder {
    pub fn new() -> Self {
        SiteBuilder {
            ctx: BuildContext::new(),
        }
    }
    pub async fn add_rule(mut self, name: impl ToString, rule: Rule) -> Self {
        self.ctx.add_rule(name, rule).await;
        self
    }

    pub async fn build(&mut self) -> Result<(), String> {
        let mut set = JoinSet::new();
        for rule in self.ctx.rules.lock().await.values() {
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
