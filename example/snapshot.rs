use polysite::{
    compiler::{markdown::MarkdownCompiler, template::TemplateEngine},
    *,
};

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let template_engine = TemplateEngine::new("templates/**").unwrap().get();
    let builder = Builder::new(Config::default());
    builder
        .add_step([
            Rule::new("posts")
                .set_globs(["posts/**/*.md"])
                .set_compiler(
                    MarkdownCompiler::new(template_engine.clone(), "snapshot.html", None)
                        .wait_snapshot("posts", 1)
                        .get(),
                ),
            Rule::new("markdown").set_globs(["**/*.md"]).set_compiler(
                MarkdownCompiler::new(template_engine.clone(), "index.html", None).get(),
            ),
        ])
        .build()
        .await
        .unwrap();
}
