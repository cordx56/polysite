use polysite::{
    compiler::{markdown::MarkdownCompiler, metadata::SetMetadata, template::TemplateEngine},
    *,
};

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let template_engine = TemplateEngine::new("templates/**").unwrap();
    Builder::new(Config::default())
        .add_step([Rule::new(
            "metadata",
            SetMetadata::new()
                .global("site_title", "Hello, polysite!")
                .unwrap(),
        )])
        .add_step([
            Rule::new(
                "posts",
                MarkdownCompiler::new(template_engine.clone(), "index.html", None),
            )
            .set_globs(["posts/**/*.md"]),
            Rule::new(
                "markdown",
                MarkdownCompiler::new(template_engine.clone(), "index.html", None),
            )
            .set_globs(["**/*.md"]),
        ])
        .build()
        .await
        .unwrap();
}
