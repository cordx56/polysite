use polysite::{
    compiler::{file::CopyCompiler, markdown::MarkdownCompiler, template::TemplateEngine},
    *,
};

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let template_engine = TemplateEngine::new("templates/**").unwrap().get();
    Builder::new(Config::default())
        .insert_metadata("site_title", Metadata::from("Hello, polysite!"))
        .insert_metadata("site_url", Metadata::from("https://example.com"))
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
