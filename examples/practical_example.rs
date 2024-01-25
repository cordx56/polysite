use polysite::{
    compiler::{
        file::CopyCompiler, markdown::MarkdownCompiler, metadata::SetMetadata,
        template::TemplateEngine,
    },
    *,
};

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let template_engine = TemplateEngine::new("templates/**").unwrap().get();
    Builder::new(Config::default())
        .add_step(
            [Rule::new("metadata").set_create(["metadata"]).set_compiler(
                SetMetadata::new()
                    .global("site_title", "Hello, polysite!")
                    .unwrap()
                    .global("site_url", "https://example.com")
                    .unwrap()
                    .get(),
            )],
        )
        .add_step([Rule::new("posts")
            .set_globs(["posts/**/*.md"])
            .set_compiler(
                MarkdownCompiler::new(template_engine.clone(), "practical.html", None)
                    .wait_snapshot("posts", 1)
                    .get(),
            )])
        .add_step([
            Rule::new("markdown").set_globs(["**/*.md"]).set_compiler(
                MarkdownCompiler::new(template_engine.clone(), "practical.html", None).get(),
            ),
            Rule::new("others")
                .set_globs(["**/*"])
                .set_compiler(CopyCompiler::new().get()),
        ])
        .build()
        .await
        .unwrap();
}
