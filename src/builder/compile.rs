use super::metadata::*;
use crate::*;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::{
    sync::{Notify, RwLock},
    task::JoinSet,
};

#[derive(Clone)]
pub struct CompileRunner {
    rule: String,
    context: Context,
    compiler: Box<dyn Compiler>,
    results: Arc<RwLock<Vec<(usize, Metadata)>>>,
    tasks: Arc<RwLock<JoinSet<Result<Context, Error>>>>,
    notify: Arc<Notify>,
}

impl CompileRunner {
    pub fn new(rule: String, context: Context, compiler: Box<dyn Compiler>) -> Self {
        Self {
            rule,
            context,
            compiler,
            results: Arc::new(RwLock::new(Vec::new())),
            tasks: Arc::new(RwLock::new(JoinSet::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn update_context(&self) {
        let res: Vec<_> = self
            .results
            .read()
            .await
            .iter()
            .map(|(_, meta)| Value::Object(meta.local().clone()))
            .collect();
        let res = Value::Array(res);
        self.context
            .metadata()
            .insert_global(self.rule.clone(), res)
            .await;
    }

    pub async fn spawn_compile(
        &self,
        version: impl AsRef<str>,
        source: PathBuf,
        target: PathBuf,
        path: PathBuf,
    ) {
        let mut s = self.clone();
        s.context
            .metadata_mut()
            .insert_local(RULE_META.to_owned(), Value::from(self.rule.clone()));
        s.context.metadata_mut().insert_local(
            VERSION_META.to_owned(),
            Value::from(version.as_ref().to_owned()),
        );
        s.context.metadata_mut().insert_local(
            SOURCE_FILE_META.to_owned(),
            Value::from(source.to_string_lossy()),
        );
        s.context.metadata_mut().insert_local(
            TARGET_FILE_META.to_owned(),
            Value::from(target.to_string_lossy()),
        );
        s.context
            .metadata_mut()
            .insert_local(PATH_META.to_owned(), Value::from(path.to_string_lossy()));

        let task_id = {
            let mut write = s.results.write().await;
            write.push((0, Metadata::new()));
            write.len() - 1
        };

        self.tasks.write().await.spawn(async move {
            let mut ctx = s.context.clone();
            loop {
                match s.compiler.next_step(ctx).await? {
                    CompileStep::Completed(v) => {
                        ctx = v;
                        {
                            let (stage, meta) = &mut s.results.write().await[task_id];
                            *stage += 1;
                            *meta = ctx.metadata().clone();
                        }
                        s.update_context().await;
                        s.notify.notify_waiters();
                        return Ok(ctx);
                    }
                    CompileStep::InProgress(v) => {
                        ctx = v;
                        {
                            let (stage, meta) = &mut s.results.write().await[task_id];
                            *stage += 1;
                            *meta = ctx.metadata().clone();
                        }
                        s.update_context().await;
                        s.notify.notify_waiters();
                    }
                    CompileStep::WaitStage(v) => {
                        ctx = v;
                        let stage;
                        {
                            let (s, meta) = &mut s.results.write().await[task_id];
                            *s += 1;
                            *meta = ctx.metadata().clone();
                            stage = *s;
                        }
                        s.update_context().await;
                        loop {
                            if let Some(min) =
                                s.results.read().await.iter().map(|(stage, _)| *stage).min()
                            {
                                if stage <= min {
                                    break;
                                }
                            }
                            s.notify.notified().await;
                        }
                    }
                }
            }
        });
    }

    pub async fn join(self) -> Result<Context, Error> {
        let mut ctx = self.context;
        let mut tasks = self.tasks.write().await;
        while let Some(res) = tasks.join_next().await {
            ctx = res.unwrap()?;
            log::info!(
                "Compiled: {} -> {}",
                ctx.source().await.unwrap().display(),
                ctx.target().await.unwrap().display(),
            );
        }
        Ok(ctx)
    }
}
