use crate::{builder::metadata::BODY_META, *};
use serde_json::Value;
use std::sync::Arc;
use tera::Tera;

/// Template engine (uses [`tera`])
#[derive(Clone)]
pub struct TemplateEngine {
    tera: Tera,
}
impl TemplateEngine {
    /// Load templates and create
    /// template engine instance
    pub fn new(template_dir: impl AsRef<str>) -> Result<Self, Error> {
        let tera = tera::Tera::new(template_dir.as_ref()).map_err(|err| Error::user_error(err))?;
        Ok(Self { tera })
    }

    /// Get `Arc<TemplateEngine>` for sharing template engine for multiple tasks
    pub fn get(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Render HTML using specified template and metadata
    pub async fn render(
        &self,
        template: impl AsRef<str>,
        metadata: &Metadata,
    ) -> Result<String, Error> {
        let tera_ctx = tera::Context::from_serialize(metadata.read_lock().await)
            .map_err(|err| Error::user_error(err))?;
        self.tera
            .render(template.as_ref(), &tera_ctx)
            .map_err(|err| Error::user_error(err))
    }
}

/// Template renderer renders HTML using specified template and
/// compiling metadata.
#[derive(Clone)]
pub struct TemplateRenderer {
    engine: TemplateEngine,
    template: String,
}
impl TemplateRenderer {
    pub fn new(engine: TemplateEngine, template: impl AsRef<str>) -> Self {
        let template = template.as_ref().to_owned();
        Self { engine, template }
    }
}
impl Compiler for TemplateRenderer {
    #[tracing::instrument(skip(self, ctx))]
    fn next_step(&mut self, mut ctx: Context) -> CompilerReturn {
        let engine = self.engine.clone();
        let template = self.template.clone();
        compile!({
            let metadata = ctx.metadata();
            let body = engine.render(&template, &metadata).await?;
            ctx.metadata_mut()
                .insert_local(BODY_META.to_owned(), Value::String(body));
            Ok(CompileStep::Completed(ctx))
        })
    }
}
