use crate::*;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tera::Tera;

/// Template engine (tera)
pub struct TemplateEngine {
    tera: Tera,
}
impl TemplateEngine {
    /// Load templates and create
    /// template engine instance
    pub fn new(template_dir: impl AsRef<str>) -> Result<Self> {
        let tera = tera::Tera::new(template_dir.as_ref())
            .map_err(|e| anyhow!("Template error: {:?}", e))?;
        Ok(Self { tera })
    }

    /// Get Arc<TemplateEngine> for sharing template engine for multiple tasks
    pub fn get(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Render HTML using specified template and metadata
    pub fn render(&self, template: impl AsRef<str>, metadata: &Metadata) -> Result<String> {
        let tera_ctx = tera::Context::from_serialize(metadata)
            .map_err(|e| anyhow!("Context serialization error: {:?}", e))?;
        let out = self
            .tera
            .render(template.as_ref(), &tera_ctx)
            .map_err(|e| anyhow!("Tera rendering error: {:?}", e))?;
        Ok(out)
    }
}

/// Template renderer
pub struct TemplateRenderer {
    engine: Arc<TemplateEngine>,
    template: String,
}
impl TemplateRenderer {
    pub fn new(engine: Arc<TemplateEngine>, template: impl ToString) -> Self {
        let template = template.to_string();
        Self { engine, template }
    }
}
impl Compiler for TemplateRenderer {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let engine = self.engine.clone();
        let template = self.template.clone();
        Box::new(compiler!({
            let mut ctx = ctx;
            let metadata = ctx.metadata().await;
            let body = engine.render(&template, &metadata)?;
            ctx.insert_compiling_metadata("body", Metadata::String(body))
                .await?;
            Ok(ctx)
        }))
    }
}
