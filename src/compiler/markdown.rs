use crate::{error::here, *};
use anyhow::anyhow;
use file::{FileReader, FileWriter};
use pulldown_cmark::{html::push_html, Options, Parser};
use snapshot::{SaveSnapshot, WaitSnapshot};
use std::sync::Arc;
use template::{TemplateEngine, TemplateRenderer};

/// Markdown renderer
pub struct MarkdownRenderer {
    options: Options,
}
impl MarkdownRenderer {
    /// Create markdown compiler
    ///
    /// Pass template engine ref, template name and
    /// markdown rendering option
    pub fn new(options: Option<Options>) -> Self {
        let options = options.unwrap_or(Options::all());
        Self { options }
    }
}
impl Compiler for MarkdownRenderer {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let options = self.options.clone();
        Box::new(compiler!({
            let mut ctx = ctx;
            let body = ctx.get_source_string()?;
            let fm = fronma::parser::parse::<Metadata>(&body)
                .map_err(|e| anyhow!("Front matter parse error on {}: {:?}", here!(), e))?;
            let mut file_metadata = fm.headers;
            let parser = Parser::new_ext(fm.body, options);
            let mut html = String::new();
            push_html(&mut html, parser);
            file_metadata
                .as_object_mut()
                .unwrap()
                .insert("body".to_string(), Metadata::String(html));
            for (k, v) in file_metadata.as_object().unwrap().clone().into_iter() {
                ctx.insert_compiling_metadata(k, v).await?;
            }
            Ok(ctx)
        }))
    }
}

pub struct MarkdownCompiler {
    template: String,
    template_engine: Arc<TemplateEngine>,
    options: Option<Options>,
    wait_snapshots: WaitSnapshot,
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
    ) -> Self {
        let template = template.to_string();
        Self {
            template,
            template_engine,
            options,
            wait_snapshots: WaitSnapshot::new(),
        }
    }
    pub fn wait_snapshot(mut self, rule: impl ToString, until: usize) -> Self {
        self.wait_snapshots = self.wait_snapshots.wait(rule, until);
        self
    }
}
impl Compiler for MarkdownCompiler {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let template = self.template.clone();
        let template_engine = self.template_engine.clone();
        let options = self.options.clone();
        let wait_snapshots = self.wait_snapshots.clone();
        let compiler = pipe!(
            FileReader::new(),
            MarkdownRenderer::new(options),
            SaveSnapshot::new(),
            wait_snapshots,
            TemplateRenderer::new(template_engine, template),
            FileWriter::new(),
        );
        compiler.compile(ctx)
    }
}
