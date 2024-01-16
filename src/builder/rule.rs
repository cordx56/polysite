pub mod routing;

use crate::Compiling;
use crate::Context;
use anyhow::{anyhow, Error};
use glob::{glob, GlobError, PatternError};
use serde_json::Value;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::{
    sync::Notify,
    task::{JoinError, JoinSet},
};

pub type Metadata = Value;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("No globs are registered")]
    NoGlobs,
    #[error("No routing method is registered")]
    NoRouting,
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

pub type CompileResult = Result<Metadata, Error>;
pub trait RoutingMethod: Fn(&PathBuf) -> PathBuf + Send + Sync {}
impl<F> RoutingMethod for F where F: Fn(&PathBuf) -> PathBuf + Send + Sync {}
pub trait CompileMethodFunc:
    Fn(Context) -> Box<dyn Future<Output = CompileResult> + Unpin + Send> + Send + Sync
{
}
impl<F> CompileMethodFunc for F where
    F: Fn(Context) -> Box<dyn Future<Output = CompileResult> + Unpin + Send> + Send + Sync
{
}

#[macro_export]
macro_rules! compiler {
    ($b:expr) => {
        Box::new(Box::pin(async move { $b }))
    };
}

pub struct Rule {
    match_globs: Option<Vec<String>>,
    routing_method: Option<Arc<Box<dyn RoutingMethod>>>,
    compile_method: Option<Arc<Box<dyn CompileMethodFunc>>>,
    metadata: Option<Metadata>,
    load: bool,
    load_notify: Arc<Notify>,
}

impl Rule {
    pub fn new() -> Self {
        Rule {
            match_globs: None,
            routing_method: None,
            compile_method: None,
            metadata: None,
            load: false,
            load_notify: Arc::new(Notify::new()),
        }
    }

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
    pub fn set_compiler(mut self, compile_method_func: impl CompileMethodFunc + 'static) -> Self {
        self.compile_method = Some(Arc::new(Box::new(compile_method_func)));
        self
    }

    /// Get compiled metadata
    pub(crate) fn get_metadata(&self) -> Option<Metadata> {
        self.metadata.clone()
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

    /// Do compilation task
    ///
    /// Send notifications to all waiters when tasks are completed.
    pub(crate) async fn compile(&mut self, ctx: Context) -> CompileResult {
        let match_globs = self
            .match_globs
            .as_ref()
            .ok_or(anyhow!(CompileError::NoGlobs))?;
        let paths = match_globs
            .iter()
            .map(|g| glob(g))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!(CompileError::GlobPattern(e)))?
            .into_iter()
            .map(|p| p.collect::<Result<Vec<_>, _>>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!(CompileError::GlobError(e)))?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let compile_method = self
            .compile_method
            .clone()
            .ok_or(anyhow!(CompileError::NoCompiler))?;
        let routing = self
            .routing_method
            .clone()
            .ok_or(anyhow!(CompileError::NoRouting))?;
        let mut set = JoinSet::new();
        for path in paths {
            let target = routing(&path);
            let compiling = Compiling {
                source: path,
                target,
            };
            let mut new_ctx = ctx.clone();
            new_ctx.set_compiling(compiling);
            set.spawn(compile_method(new_ctx));
        }
        let mut results = Vec::new();
        while let Some(res) = set.join_next().await {
            let res = res.map_err(|e| anyhow!(CompileError::JoinError(e)))?;
            let res = res.map_err(|e| anyhow!(CompileError::UserError(e)))?;
            results.push(res);
        }
        self.load = true;
        self.load_notify.notify_waiters();
        let metadata = Value::Array(results);
        self.metadata = Some(metadata.clone());
        Ok(metadata)
    }
}
