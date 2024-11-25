use crate::*;
use log::info;
use std::fs::remove_dir_all;
use tokio::task::JoinSet;
use tracing_error::SpanTrace;

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
///         .insert_metadata("site_title", Metadata::from("Hello, polysite!"))
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
    #[tracing::instrument(skip(self))]
    pub async fn build(mut self) -> Result<(), Error> {
        let conf = self.ctx.config();
        let target_dir = conf.target_dir();
        if conf.target_clean() && target_dir.is_dir() {
            remove_dir_all(&target_dir).map_err(|io_error| Error::FileIo {
                trace: SpanTrace::capture(),
                io_error,
            })?;
            info!("Target directory ({}) cleaned", target_dir.display());
        }
        for step in self.steps.into_iter() {
            let mut set = JoinSet::new();
            for rule in step.into_iter() {
                let ctx = self.ctx.clone();
                set.spawn(rule.compile(ctx));
            }
            while let Some(res) = set.join_next().await {
                let ctx = res.unwrap()?;
                self.ctx.metadata_mut().merge(ctx.metadata().clone());
            }
        }
        Ok(())
    }
}
