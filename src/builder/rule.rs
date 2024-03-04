use super::{context::Compiling, metadata::*, snapshot::*};
use crate::*;
use anyhow::{anyhow, Context as _, Error};
use glob::glob;
use log::info;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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
    pub fn new(name: impl AsRef<str>) -> Self {
        let name = name.as_ref().to_owned();
        Rule {
            name,
            conditions: None,
            matched: None,
            compiler: None,
            version: Version::default(),
        }
    }

    /// Get this rule name
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Set a list of source file match globs
    pub fn set_globs(mut self, globs: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let gs = globs.into_iter().map(|s| s.as_ref().to_owned()).collect();
        self.conditions = Some(Conditions::Globs(gs));
        self
    }
    /// Set source files name to create
    pub fn set_create(mut self, create: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let create = create.into_iter().map(|s| s.as_ref().to_owned()).collect();
        self.conditions = Some(Conditions::Create(create));
        self
    }

    /// Set compiler
    ///
    /// A [`Arc`] pointer of [`Compiler`] passed to this method will be called in compilation task.
    pub fn set_compiler(mut self, compiler: Arc<dyn Compiler>) -> Self {
        self.compiler = Some(compiler);
        self
    }

    /// Set compilation version
    ///
    /// If the same version of a source file path is registered for compilation, that file will be
    /// skipped.
    /// Also read [`Version`].
    pub fn set_version(mut self, version: impl Into<Version>) -> Self {
        self.version = version.into();
        self
    }

    /// Evaluate conditions and save it's result
    pub(super) fn eval_conditions(&mut self, ctx: &Context) -> Result<(), Error> {
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
            if ctx.get_version_metadata(self.version.clone(), &p).is_none() {
                ctx.insert_version(self.version.clone(), p.clone(), Metadata::Null)?;
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
                self.eval_conditions(&ctx)?;
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
        for source in matched {
            let src_dir = ctx.config().source_dir();
            let path = source.strip_prefix(&src_dir).unwrap_or(&source);
            let target = ctx.config().target_dir().join(path);
            let path = PathBuf::from("/").join(path);
            // Make Snapshot stage
            let snapshot_stage = SnapshotStage::new();
            snapshot_manager.push(snapshot_stage.clone());
            // Make compiling data
            let compiling = Compiling::new(snapshot_stage);
            let mut new_ctx = ctx.clone();
            new_ctx.set_compiling(Some(compiling));
            new_ctx.insert_compiling_metadata(RULE_META, Metadata::from(self.get_name()));
            new_ctx.insert_compiling_metadata(
                SOURCE_FILE_META,
                Metadata::from(source.to_string_lossy().to_string()),
            );
            new_ctx.insert_compiling_metadata(
                TARGET_FILE_META,
                Metadata::from(target.to_string_lossy().to_string()),
            );
            new_ctx.insert_compiling_metadata(
                PATH_META,
                Metadata::from(path.to_string_lossy().to_string()),
            );
            new_ctx.insert_compiling_metadata(VERSION_META, Metadata::from(self.version.get()));
            tasks.push(compiler.compile(new_ctx));
        }
        ctx.register_snapshot_manager(self.get_name(), snapshot_manager);
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
            )?;
            info!(
                "Compiled: {} -> {}",
                res.source()?.display(),
                res.target()?.display()
            );
            results.push(compiling_metadata);
        }
        let metadata = Metadata::Array(Arc::new(Mutex::new(results)));
        ctx.insert_global_metadata(self.get_name(), metadata);
        Ok(ctx)
    }
}
