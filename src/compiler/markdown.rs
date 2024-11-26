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

/// [`MarkdownRenderer`] reads the body from [`BODY_META`], renders it to HTML, and saves the HTML to [`BODY_META`].
#[derive(Clone)]
pub struct MarkdownRenderer {
    options: Options,
}
impl MarkdownRenderer {
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

/// [`MarkdownCompiler`] sets the target file extension to .html, reads the file, renders Markdown, waits for other tasks, renders HTML using the specified [`TemplateEngine`], and outputs it to the target file.
#[derive(Clone)]
pub struct MarkdownCompiler {
    compiler: PipeCompiler,
}
impl MarkdownCompiler {
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
