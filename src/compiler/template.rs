use crate::{builder::metadata::BODY_META, *};
use anyhow::{Context as _, Result};
use std::sync::Arc;
use tera::Tera;

/// Template engine (uses [`tera`])
pub struct TemplateEngine {
    tera: Tera,
}
impl TemplateEngine {
    /// Load templates and create
    /// template engine instance
    pub fn new(template_dir: impl AsRef<str>) -> Result<Self> {
        let tera = tera::Tera::new(template_dir.as_ref()).context("Template error")?;
        Ok(Self { tera })
    }

    /// Get `Arc<TemplateEngine>` for sharing template engine for multiple tasks
    pub fn get(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Render HTML using specified template and metadata
    pub fn render(&self, template: impl AsRef<str>, metadata: &Metadata) -> Result<String> {
        let tera_ctx =
            tera::Context::from_serialize(metadata).context("Context serialization error")?;
        let out = self
            .tera
            .render(template.as_ref(), &tera_ctx)
            .context("Tera rendering error")?;
        Ok(out)
    }
}

/// Template renderer renders HTML using specified template and
/// compiling metadata.
pub struct TemplateRenderer {
    engine: Arc<TemplateEngine>,
    template: String,
}
impl TemplateRenderer {
    pub fn new(engine: Arc<TemplateEngine>, template: impl AsRef<str>) -> Self {
        let template = template.as_ref().to_owned();
        Self { engine, template }
    }
}
impl Compiler for TemplateRenderer {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let engine = self.engine.clone();
        let template = self.template.clone();
        Box::new(compile!({
            let mut ctx = ctx;
            let metadata = ctx.metadata();
            let body = engine.render(&template, &metadata)?;
            ctx.insert_compiling_metadata(BODY_META, Metadata::from(body));
            Ok(ctx)
        }))
    }
}
