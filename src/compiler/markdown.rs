use crate::{
    builder::metadata::*,
    compiler::{
        file::{FileReader, FileWriter},
        path::SetExtension,
        template::{TemplateEngine, TemplateRenderer},
        utils::{PipeCompiler, WaitStage},
    },
    *,
};
use pulldown_cmark::{html::push_html, Options, Parser};
use tracing_error::SpanTrace;

/// Markdown renderer will read [`_body`][crate::builder::metadata::BODY_META] metadata as markdown,
/// render HTML, and store HTML as [`_body`][crate::builder::metadata::BODY_META] metadata.
#[derive(Clone)]
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
    #[tracing::instrument(skip(self, ctx))]
    fn next_step(&mut self, mut ctx: Context) -> CompilerReturn {
        let options = self.options.clone();
        compile!({
            let body = ctx.body().await.ok_or(Error::InvalidMetadata {
                trace: SpanTrace::capture(),
            })?;
            let body = body.as_str().ok_or(Error::InvalidMetadata {
                trace: SpanTrace::capture(),
            })?;
            let fm = fronma::parser::parse::<Value>(&body).map_err(|_| Error::InvalidMetadata {
                trace: SpanTrace::capture(),
            })?;
            let file_metadata = fm.headers;
            let parser = Parser::new_ext(fm.body, options);
            let mut html = String::new();
            push_html(&mut html, parser);
            if let Value::Object(map) = file_metadata {
                for (k, v) in map.into_iter() {
                    ctx.metadata_mut().insert_local(k, v);
                }
            }
            ctx.metadata_mut()
                .insert_local(BODY_META.to_owned(), Value::String(html));
            Ok(CompileStep::Completed(ctx))
        })
    }
}

/// Markdown compiler
///
/// This compiler sets target file extension to .html, read file, render markdown, save snapshot,
/// wait snapshot if you specified, and render HTML using specified [`compiler::template::TemplateEngine`]
/// and output target file.
#[derive(Clone)]
pub struct MarkdownCompiler {
    compiler: PipeCompiler,
}
impl MarkdownCompiler {
    /// Create markdown compiler
    ///
    /// Pass template engine ref, template name and
    /// markdown rendering option
    pub fn new(
        template_engine: TemplateEngine,
        template: impl AsRef<str>,
        options: Option<Options>,
    ) -> Self {
        let template = template.as_ref().to_owned();
        let compiler = pipe!(
            SetExtension::new("html"),
            FileReader::new(),
            MarkdownRenderer::new(options),
            WaitStage::new(),
            TemplateRenderer::new(template_engine, template),
            FileWriter::new(),
        );

        Self { compiler }
    }
}
impl Compiler for MarkdownCompiler {
    #[tracing::instrument(skip(self, ctx))]
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        self.compiler.next_step(ctx)
    }
}
