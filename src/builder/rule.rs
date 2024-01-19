use crate::*;
use anyhow::{anyhow, Error};
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
    router: Option<Arc<dyn Router>>,
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
            router: None,
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

    /// Set router
    ///
    /// The router passed to this method will be used to transform source file path to
    /// target file path.
    pub fn set_router(mut self, router: Arc<dyn Router>) -> Self {
        self.router = Some(router);
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
                    .map_err(|e| anyhow!("Glob pattern error: {:?}", e))?
                    .into_iter()
                    .map(|p| p.collect::<Result<Vec<_>, _>>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| anyhow!("Glob error: {:?}", e))?
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
    ///
    /// Send notifications to all waiters when tasks are completed.
    pub(crate) async fn compile(&mut self, ctx: Context) -> CompileResult {
        let matched = self
            .matched
            .clone()
            .ok_or(anyhow!("Condition is not evaluated"))?;
        let compiler = self
            .compiler
            .clone()
            .ok_or(anyhow!("No compiler is registered"))?;
        let router = self.router.clone();
        let mut set = JoinSet::new();
        for path in matched {
            let src_dir = ctx.config().source_dir();
            let target = ctx
                .config()
                .target_dir()
                .join(path.strip_prefix(&src_dir).unwrap_or(&path));
            let target = match &router {
                Some(r) => r.route(target),
                None => target,
            };
            let compiling = Compiling::new(path, target, self.version.clone());
            let mut new_ctx = ctx.clone();
            new_ctx.set_compiling(Some(compiling));
            set.spawn(compiler.compile(new_ctx));
        }
        let mut results = Vec::new();
        while let Some(res) = set.join_next().await {
            let res = res.map_err(|e| anyhow!("Join error: {:?}", e))?;
            let res = res.map_err(|e| anyhow!("Compile error: {}", e))?;
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
        Ok(ctx)
    }
}
