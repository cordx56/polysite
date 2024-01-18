use crate::{error::here, *};
use anyhow::{anyhow, Error};
use glob::{glob, GlobError, PatternError};
use log::info;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::{
    sync::Notify,
    task::{JoinError, JoinSet},
};

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("No globs are registered")]
    NoGlobs,
    #[error("No compiler is registered")]
    NoCompiler,
    #[error("Glob pattern error: {:?}", .0)]
    GlobPattern(PatternError),
    #[error("Glob error: {:?}", .0)]
    GlobError(GlobError),
    #[error("Join error: {:?}", .0)]
    JoinError(JoinError),
    #[error("{}", .0)]
    UserError(Error),
}

pub trait RoutingMethod: Fn(&PathBuf) -> PathBuf + Send + Sync {}
impl<F> RoutingMethod for F where F: Fn(&PathBuf) -> PathBuf + Send + Sync {}

pub struct Rule {
    name: String,
    match_globs: Option<Vec<String>>,
    routing_method: Option<Arc<Box<dyn RoutingMethod>>>,
    compiler: Option<Arc<Box<dyn Compiler>>>,
    load: bool,
    load_notify: Arc<Notify>,
    version: Version,
    waits: Vec<String>,
}

impl Rule {
    pub fn new(name: impl ToString) -> Self {
        let name = name.to_string();
        Rule {
            name,
            match_globs: None,
            routing_method: None,
            compiler: None,
            load: false,
            load_notify: Arc::new(Notify::new()),
            version: Version::new(None),
            waits: Vec::new(),
        }
    }

    /// Get this rule name
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Set source file globs
    pub fn set_match(mut self, globs: impl IntoIterator<Item = impl ToString>) -> Self {
        let gs = globs.into_iter().map(|s| s.to_string()).collect();
        self.match_globs = Some(gs);
        self
    }

    /// Set routing method
    ///
    /// The function passed to this method will be used to transform source file path to
    /// target file path.
    pub fn set_routing(mut self, routing_method: impl RoutingMethod + 'static) -> Self {
        self.routing_method = Some(Arc::new(Box::new(routing_method)));
        self
    }

    /// Set compiler method
    ///
    /// The function passed to this method will be called in compilation task.
    pub fn set_compiler(mut self, compiler: Box<dyn Compiler>) -> Self {
        self.compiler = Some(Arc::new(compiler));
        self
    }

    /// Set compilation version
    pub fn set_version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Set wait rules
    pub fn set_waits(mut self, waits: impl IntoIterator<Item = impl ToString>) -> Self {
        let ws = waits.into_iter().map(|s| s.to_string()).collect();
        self.waits = ws;
        self
    }

    /// Get load notify
    ///
    /// If compilation task is finished, this method returns None.
    /// Otherwise this method returns Arc<tokio::sync::Notify>.
    pub(crate) fn get_load_notify(&self) -> Option<Arc<Notify>> {
        if self.load {
            None
        } else {
            Some(self.load_notify.clone())
        }
    }
    /// Mark as compilation finished
    pub(crate) fn set_finished(&mut self) {
        self.load = true;
        self.load_notify.notify_waiters();
    }

    async fn wait_all(&self, ctx: &Context) -> Result<(), Error> {
        let mut set = JoinSet::new();
        for w in self.waits.iter() {
            let ctx = ctx.clone();
            let w = w.clone();
            set.spawn(async move { ctx.wait(w).await });
        }
        while let Some(res) = set.join_next().await {
            res.map_err(|e| anyhow!("Join error on {}: {:?}", here!(), e))??;
        }
        Ok(())
    }

    /// Do compilation task
    ///
    /// Send notifications to all waiters when tasks are completed.
    pub(crate) async fn compile(&mut self, ctx: Context) -> CompileResult {
        self.wait_all(&ctx).await?;
        let src_dir = ctx.config().source_dir();
        let match_globs = self
            .match_globs
            .as_ref()
            .ok_or(anyhow!(CompileError::NoGlobs))?
            .iter()
            .map(|g| src_dir.join(PathBuf::from(g)).to_string_lossy().to_string());
        let paths = match_globs
            .map(|g| glob(&g))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!(CompileError::GlobPattern(e)))?
            .into_iter()
            .map(|p| p.collect::<Result<Vec<_>, _>>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!(CompileError::GlobError(e)))?
            .into_iter()
            .flatten()
            .filter(|p| p.is_file())
            .collect::<Vec<_>>();
        let compiler = self
            .compiler
            .clone()
            .ok_or(anyhow!(CompileError::NoCompiler))?;
        let routing = self.routing_method.clone();
        let mut set = JoinSet::new();
        for path in paths {
            // If there is the version already compiled, pass the compilation
            if ctx.get_version(&self.version, &path).await.is_some() {
                continue;
            }
            let target = ctx
                .config()
                .target_dir()
                .join(path.strip_prefix(&src_dir).unwrap());
            let target = match &routing {
                Some(r) => r(&target),
                None => target,
            };
            let compiling = Compiling::new(path, target, self.version.clone());
            let mut new_ctx = ctx.clone();
            new_ctx.set_compiling(Some(compiling));
            set.spawn(compiler.compile(new_ctx));
        }
        let mut results = Vec::new();
        while let Some(res) = set.join_next().await {
            let res = res.map_err(|e| anyhow!(CompileError::JoinError(e)))?;
            let res = res.map_err(|e| anyhow!(CompileError::UserError(e)))?;
            ctx.insert_version(
                self.version.clone(),
                res.source()?,
                res.compiling_metadata()?,
            )
            .await?;
            info!(
                "Compiled: {} -> {}",
                res.source()?.display(),
                res.target()?.display()
            );
            results.push(res.compiling_metadata()?);
        }
        let metadata = Metadata::Array(results);
        ctx.insert_global_metadata(self.get_name(), metadata)
            .await?;
        self.set_finished();
        Ok(ctx)
    }
}
