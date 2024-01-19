use crate::*;

pub struct SaveSnapshot;
impl SaveSnapshot {
    /// Save snapshot
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for SaveSnapshot {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        compiler!({
            ctx.save_snapshot().await?;
            Ok(ctx)
        })
    }
}

#[derive(Clone)]
pub struct WaitSnapshot {
    rule_stage_set: Vec<(String, usize)>,
}
impl WaitSnapshot {
    /// Wait until first snapshot created
    pub fn new() -> Self {
        Self {
            rule_stage_set: Vec::new(),
        }
    }
    /// Add wait rule and until
    /// In most cases, until is 1
    pub fn wait(mut self, rule: impl ToString, until: usize) -> Self {
        self.rule_stage_set.push((rule.to_string(), until));
        self
    }
}
impl Compiler for WaitSnapshot {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let set = self.rule_stage_set.clone();
        compiler!({
            for (rule, until) in set {
                ctx.wait_snapshot_until(rule, until).await?
            }
            Ok(ctx)
        })
    }
}
