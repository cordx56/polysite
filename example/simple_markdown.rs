use polysite::*;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let template_engine = template::TemplateEngine::new("templates/**").unwrap().get();
    let builder = Builder::new(Config::default());
    builder
        .add_step([
            Rule::new("posts")
                .set_globs(["posts/**/*.md"])
                .set_router(SetExtRouter::new("html").get())
                .set_compiler(
                    markdown::MarkdownCompiler::new(template_engine.clone(), "index.html", None)
                        .unwrap()
                        .get(),
                ),
            Rule::new("markdown")
                .set_globs(["**/*.md"])
                .set_router(SetExtRouter::new("html").get())
                .set_compiler(
                    markdown::MarkdownCompiler::new(template_engine.clone(), "index.html", None)
                        .unwrap()
                        .get(),
                ),
            Rule::new("others")
                .set_globs(["**/*"])
                .set_compiler(file::CopyCompiler::new().get()),
        ])
        .build()
        .await
        .unwrap();
}
