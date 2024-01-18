use super::template::TemplateEngine;
use crate::{error::here, *};
use anyhow::{anyhow, Ok, Result};
use pulldown_cmark::{html::push_html, Options, Parser};
use std::io::Write;
use std::sync::Arc;

pub struct MarkdownCompiler {
    template: String,
    template_engine: Arc<TemplateEngine>,
    options: Options,
}

impl MarkdownCompiler {
    /// Create markdown compiler
    ///
    /// Pass template engine ref, template name and
    /// markdown rendering option
    pub fn new(
        template_engine: Arc<TemplateEngine>,
        template: impl ToString,
        options: Option<Options>,
    ) -> Result<Self> {
        let template = template.to_string();
        let options = options.unwrap_or(Options::all());
        Ok(Self {
            template,
            template_engine,
            options,
        })
    }
}

impl Compiler for MarkdownCompiler {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let template = self.template.clone();
        let template_engine = self.template_engine.clone();
        let options = self.options.clone();
        Box::new(compiler!({
            let body = ctx.get_source_string();
            let fm = fronma::parser::parse::<Metadata>(&body)
                .map_err(|e| anyhow!("Front matter parse error on {}: {:?}", here!(), e))?;
            let mut file_metadaata = fm.headers;
            let parser = Parser::new_ext(fm.body, options);
            let mut html = String::new();
            push_html(&mut html, parser);
            let mut target = ctx.open_target().map_err(|e| {
                anyhow!(
                    "Failed to open file {} on {}: {:?}",
                    ctx.target().display(),
                    here!(),
                    e
                )
            })?;
            file_metadaata
                .as_object_mut()
                .unwrap()
                .insert("body".to_string(), Metadata::String(html));
            let mut metadata = ctx.metadata().await.clone();
            metadata
                .as_object_mut()
                .unwrap()
                .extend(file_metadaata.as_object().unwrap().clone().into_iter());
            let out = template_engine.render(&template, &metadata)?;
            target
                .write(out.as_bytes())
                .map_err(|e| anyhow!("Failed to write file on {}: {:?}", here!(), e))?;
            Ok(file_metadaata)
        }))
    }
}
