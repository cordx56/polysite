use polysite::*;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let template_engine = template::TemplateEngine::new("templates/**").unwrap().get();
    let builder = Builder::new(Config::default());
    builder
        .add_rule(
            Rule::new("posts")
                .set_match(["posts/**/*.md"])
                .set_routing(routing::set_ext("html"))
                .set_compiler(
                    markdown::MarkdownCompiler::new(template_engine.clone(), "index.html", None)
                        .unwrap()
                        .get(),
                ),
        )
        .add_rule(
            Rule::new("markdown")
                .set_waits(["posts"])
                .set_match(["**/*.md"])
                .set_routing(routing::set_ext("html"))
                .set_compiler(
                    markdown::MarkdownCompiler::new(template_engine.clone(), "index.html", None)
                        .unwrap()
                        .get(),
                ),
        )
        .add_rule(
            Rule::new("others")
                .set_waits(["markdown"])
                .set_match(["**/*"])
                .set_compiler(file::CopyCompiler::new().get()),
        )
        .build()
        .await
        .unwrap();
}
