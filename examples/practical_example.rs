use polysite::{
    compiler::{
        file::CopyCompiler, markdown::MarkdownCompiler, metadata::SetMetadata,
        template::TemplateEngine,
    },
    *,
};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    let subscriber =
        tracing_subscriber::Registry::default().with(tracing_error::ErrorLayer::default());
    tracing::subscriber::set_global_default(subscriber).unwrap();

    simple_logger::SimpleLogger::new().env().init().unwrap();

    let template_engine = TemplateEngine::new("templates/**").unwrap();
    Builder::new(Config::default())
        .add_step([Rule::new(
            "metadata",
            SetMetadata::new()
                .global("site_title", "Hello, polysite!")
                .unwrap()
                .global("site_url", "https://example.com")
                .unwrap(),
        )
        .set_create(["metadata"])])
        .add_step([Rule::new(
            "posts",
            MarkdownCompiler::new(template_engine.clone(), "practical.html", None),
        )
        .set_globs(["posts/**/*.md"])])
        .add_step([
            Rule::new(
                "markdown",
                MarkdownCompiler::new(template_engine.clone(), "practical.html", None),
            )
            .set_globs(["**/*.md"]),
            Rule::new("others", CopyCompiler::new()).set_globs(["**/*"]),
        ])
        .build()
        .await
        .unwrap();
}
