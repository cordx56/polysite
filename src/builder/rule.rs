use super::snapshot::*;
use crate::*;
use anyhow::{anyhow, Context as _, Error};
use glob::glob;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;

#[derive(Debug)]
pub enum Conditions {
    Globs(Vec<String>),
    Create(Vec<String>),
}

pub struct Rule {
    name: String,
    conditions: Option<Conditions>,
    matched: Option<Vec<PathBuf>>,
    compiler: Option<Arc<dyn Compiler>>,
    version: Version,
}

impl Rule {
    /// Create new rule
    pub fn new(name: impl ToString) -> Self {
        let name = name.to_string();
        Rule {
            name,
            conditions: None,
            matched: None,
            compiler: None,
            version: Version::new(None),
        }
    }

    /// Get this rule name
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Set source file globs
    pub fn set_globs(mut self, globs: impl IntoIterator<Item = impl ToString>) -> Self {
        let gs = globs.into_iter().map(|s| s.to_string()).collect();
        self.conditions = Some(Conditions::Globs(gs));
        self
    }
    /// Set source file creation
    pub fn set_create(mut self, create: impl IntoIterator<Item = impl ToString>) -> Self {
        let create = create.into_iter().map(|s| s.to_string()).collect();
        self.conditions = Some(Conditions::Create(create));
        self
    }

    /// Set compiler
    ///
    /// The compiler passed to this method will be called in compilation task.
    pub fn set_compiler(mut self, compiler: Arc<dyn Compiler>) -> Self {
        self.compiler = Some(compiler);
        self
    }

    /// Set compilation version
    pub fn set_version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Evaluate conditions and save it's result
    pub(super) async fn eval_conditions(&mut self, ctx: &Context) -> Result<(), Error> {
        let src_dir = ctx.config().source_dir();
        let paths: Vec<_> = match self
            .conditions
            .as_ref()
            .ok_or(anyhow!("No conditions are specified!"))?
        {
            Conditions::Globs(globs) => {
                let globs = globs
                    .iter()
                    .map(|g| src_dir.join(PathBuf::from(g)).to_string_lossy().to_string());
                let paths = globs
                    .map(|g| glob(&g))
                    .collect::<Result<Vec<_>, _>>()
                    .context("Glob pattern error")?
                    .into_iter()
                    .map(|p| p.collect::<Result<Vec<_>, _>>())
                    .collect::<Result<Vec<_>, _>>()
                    .context("Glob error")?
                    .into_iter()
                    .flatten()
                    .filter(|p| p.is_file())
                    .collect();
                paths
            }
            Conditions::Create(paths) => {
                let paths = paths
                    .iter()
                    .map(|p| src_dir.join(PathBuf::from(p)))
                    .collect();
                paths
            }
        };
        let mut res = Vec::new();
        for p in paths.into_iter() {
            if ctx.get_version(&self.version, &p).await.is_none() {
                ctx.insert_version(self.version.clone(), p.clone(), Metadata::Null)
                    .await?;
                res.push(p)
            }
        }
        self.matched = Some(res);
        Ok(())
    }

    /// Do compilation task
    pub(crate) async fn compile(&mut self, ctx: Context) -> CompileResult {
        let matched = match self.matched.clone() {
            Some(m) => m,
            None => {
                self.eval_conditions(&ctx).await?;
                self.matched
                    .clone()
                    .ok_or(anyhow!("Condition evaluation error"))?
            }
        };
        let compiler = self
            .compiler
            .clone()
            .ok_or(anyhow!("No compiler is registered"))?;
        let snapshot_manager = SnapshotManager::new();
        let mut tasks = Vec::new();
        for path in matched {
            let src_dir = ctx.config().source_dir();
            let target = ctx
                .config()
                .target_dir()
                .join(path.strip_prefix(&src_dir).unwrap_or(&path));
            // Make Snapshot stage
            let snapshot_stage = SnapshotStage::new();
            snapshot_manager.push(snapshot_stage.clone()).await;
            // Make compiling data
            let compiling = Compiling::new(snapshot_stage);
            let mut new_ctx = ctx.clone();
            new_ctx.set_compiling(Some(compiling));
            new_ctx.insert_compiling_metadata(RULE_META, self.get_name())?;
            new_ctx
                .insert_compiling_metadata(SOURCE_FILE_META, path.to_string_lossy().to_string())?;
            new_ctx.insert_compiling_metadata(
                TARGET_FILE_META,
                target.to_string_lossy().to_string(),
            )?;
            new_ctx.insert_compiling_metadata(VERSION_META, self.version.get())?;
            tasks.push(compiler.compile(new_ctx));
        }
        ctx.register_snapshot_manager(self.get_name(), snapshot_manager)
            .await;
        // Start compilation tasks
        let mut set = JoinSet::new();
        for task in tasks {
            set.spawn(task);
        }
        let mut results = Vec::new();
        while let Some(res) = set.join_next().await {
            let res = res.context("Join error")?;
            let res = res.context("Compile error")?;
            let compiling_metadata = res.compiling_metadata()?.clone();
            ctx.insert_version(
                self.version.clone(),
                res.source()?,
                compiling_metadata.clone(),
            )
            .await?;
            info!(
                "Compiled: {} -> {}",
                res.source()?.display(),
                res.target()?.display()
            );
            results.push(compiling_metadata);
        }
        let metadata = Metadata::Array(results);
        ctx.insert_global_metadata(self.get_name(), metadata)
            .await?;
        Ok(ctx)
    }
}
