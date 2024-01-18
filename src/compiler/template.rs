use crate::{error::here, Metadata};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tera::{Context, Tera};

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Load templates and create
    /// template engine instance
    pub fn new(template_dir: impl AsRef<str>) -> Result<Self> {
        let tera = tera::Tera::new(template_dir.as_ref())
            .map_err(|e| anyhow!("Template error on {}: {:?}", here!(), e))?;
        Ok(Self { tera })
    }

    /// Get Arc<TemplateEngine> for sharing template engine for multiple tasks
    pub fn get(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Render HTML using specified template and metadata
    pub fn render(&self, template: impl AsRef<str>, metadata: &Metadata) -> Result<String> {
        let tera_ctx = Context::from_serialize(metadata)
            .map_err(|e| anyhow!("Context serialization error on {}: {:?}", here!(), e))?;
        let out = self
            .tera
            .render(template.as_ref(), &tera_ctx)
            .map_err(|e| anyhow!("Tera rendering error on {}: {:?}", here!(), e))?;
        Ok(out)
    }
}
