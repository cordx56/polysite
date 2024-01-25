use crate::*;
use anyhow::{Context as _, Result};
use log::info;
use serde::Serialize;
use std::fs::remove_dir_all;
use tokio::task::JoinSet;

/// A site builder to use build one site
///
/// # Examples
/// ```
/// use polysite::{
///     compiler::{markdown::MarkdownCompiler, template::TemplateEngine},
///     *,
/// };
///
/// #[tokio::main]
/// async fn main() {
///     let template_engine = TemplateEngine::new("templates/**").unwrap().get();
///     Builder::new(Config::default())
///         .insert_metadata("site_title", "Hello, polysite!")
///         .await
///         .unwrap()
///         .add_step([
///             Rule::new("posts")
///                 .set_globs(["posts/**/*.md"])
///                 .set_compiler(
///                     MarkdownCompiler::new(template_engine.clone(), "index.html", None).get(),
///                 ),
///             Rule::new("markdown").set_globs(["**/*.md"]).set_compiler(
///                 MarkdownCompiler::new(template_engine.clone(), "index.html", None).get(),
///             ),
///         ])
///         .build()
///         .await
///         .unwrap();
/// }
/// ````
pub struct Builder {
    ctx: Context,
    steps: Vec<Vec<Rule>>,
}

impl Builder {
    /// Create new builder with config
    pub fn new(config: Config) -> Self {
        Self {
            ctx: Context::new(config),
            steps: Vec::new(),
        }
    }

    /// Insert global metadata
    ///
    /// You can pass anything which can be serialized and deserialized to
    /// [`serde_json::Value`](https://docs.rs/serde_json/1/serde_json/enum.Value.html).
    pub async fn insert_metadata(self, name: impl ToString, data: impl Serialize) -> Result<Self> {
        self.ctx.insert_global_metadata(name, data).await?;
        Ok(self)
    }

    /// Add build step
    ///
    /// This method receives rules as a parameter and push as build step
    /// Rules registered in a same step will be built concurrently
    pub fn add_step(mut self, step: impl IntoIterator<Item = Rule>) -> Self {
        self.steps.push(step.into_iter().collect());
        self
    }

    /// Run build
    ///
    /// Run all registered build steps
    pub async fn build(self) -> Result<()> {
        let conf = self.ctx.config();
        let target_dir = conf.target_dir();
        if conf.target_clean() && target_dir.is_dir() {
            remove_dir_all(&target_dir).context("Target directory compiling error")?;
            info!("Target directory ({}) cleaned", target_dir.display());
        }
        for step in self.steps.into_iter() {
            let mut set = JoinSet::new();
            for mut rule in step.into_iter() {
                let ctx = self.ctx.clone();
                rule.eval_conditions(&ctx).await?;
                set.spawn(async move { (rule.get_name(), rule.compile(ctx).await) });
            }
            while let Some(join_res) = set.join_next().await {
                let (name, res) = join_res.context("Join error")?;
                let _ctx = res.with_context(|| format!("Rule {}: compile error", name))?;
            }
        }
        Ok(())
    }
}
