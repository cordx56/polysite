use crate::{error::here, *};
use anyhow::{anyhow, Ok, Result};
use pulldown_cmark::{html::push_html, Options, Parser};
use std::io::Write;

pub struct MarkdownCompiler {
    template: String,
    tera: tera::Tera,
    options: Options,
}

impl MarkdownCompiler {
    pub fn new(
        template_dir: impl AsRef<str>,
        template: impl ToString,
        options: Option<Options>,
    ) -> Result<Self> {
        let template = template.to_string();
        let tera = tera::Tera::new(template_dir.as_ref())
            .map_err(|e| anyhow!("Template error on {}: {:?}", here!(), e))?;
        let options = options.unwrap_or(Options::all());
        Ok(Self {
            template,
            tera,
            options,
        })
    }
}

impl Compiler for MarkdownCompiler {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let template = self.template.clone();
        let tera = self.tera.clone();
        let options = self.options.clone();
        Box::new(compiler!({
            let body = ctx.get_source_string();
            let fm = fronma::parser::parse::<Metadata>(&body)
                .map_err(|e| anyhow!("Front matter parse error on {}: {:?}", here!(), e))?;
            let header = fm.headers;
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
            let mut metadata = ctx.metadata().await.clone();
            metadata
                .as_object_mut()
                .unwrap()
                .extend(header.as_object().unwrap().clone().into_iter());
            metadata
                .as_object_mut()
                .unwrap()
                .insert("body".to_string(), Metadata::String(html));
            let tera_ctx = tera::Context::from_serialize(metadata).unwrap();
            let out = tera
                .render(&template, &tera_ctx)
                .map_err(|e| anyhow!("Tera rendering error on {}: {:?}", here!(), e))?;
            target
                .write(out.as_bytes())
                .map_err(|e| anyhow!("Failed to write file on {}: {:?}", here!(), e))?;
            Ok(header)
        }))
    }
}
