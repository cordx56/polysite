use crate::{
    builder::metadata::BODY_META,
    compiler::{
        file::{FileReader, FileWriter},
        path::SetExtension,
        snapshot::{SaveSnapshot, WaitSnapshot},
        template::{TemplateEngine, TemplateRenderer},
    },
    *,
};
use anyhow::anyhow;
use pulldown_cmark::{html::push_html, Options, Parser};
use std::sync::Arc;

/// Markdown renderer will read [`_body`][crate::builder::metadata::BODY_META] metadata as markdown,
/// render HTML, and store HTML as [`_body`][crate::builder::metadata::BODY_META] metadata.
pub struct MarkdownRenderer {
    options: Options,
}
impl MarkdownRenderer {
    /// Create markdown renderer
    ///
    /// Pass [`pulldown_cmark::Options`] to render markdown to HTML
    pub fn new(options: Option<Options>) -> Self {
        let options = options.unwrap_or(Options::all());
        Self { options }
    }
}
impl Compiler for MarkdownRenderer {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let options = self.options.clone();
        Box::new(compile!({
            let mut ctx = ctx;
            let body = ctx.body()?;
            let body = body.as_str().ok_or(anyhow!("Body is not string"))?;
            let fm = fronma::parser::parse::<Metadata>(&body)
                .map_err(|e| anyhow!("Front matter parse error: {:?}", e))?;
            let file_metadata = fm.headers;
            let parser = Parser::new_ext(fm.body, options);
            let mut html = String::new();
            push_html(&mut html, parser);
            for (k, v) in file_metadata.as_object().unwrap().clone().into_iter() {
                ctx.insert_compiling_metadata(k, v)?;
            }
            ctx.insert_compiling_metadata(BODY_META, html)?;
            Ok(ctx)
        }))
    }
}

/// Markdown compiler
///
/// This compiler sets target file extension to .html, read file, render markdown, save snapshot,
/// wait snapshot if you specified, and render HTML using specified [`compiler::template::TemplateEngine`]
/// and output target file.
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
        template: impl AsRef<str>,
        options: Option<Options>,
    ) -> Self {
        let template = template.as_ref().to_owned();
        Self {
            template,
            template_engine,
            options,
            wait_snapshots: WaitSnapshot::new(),
        }
    }
    pub fn wait_snapshot(mut self, rule: impl AsRef<str>, until: usize) -> Self {
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
            SetExtension::new("html"),
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
