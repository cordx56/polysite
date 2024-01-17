use polysite::*;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let builder = Builder::new(Config::default());
    builder
        .add_rule(
            Rule::new("markdown")
                .set_match(["**/*.md"])
                .set_routing(routing::set_ext("html"))
                .set_compiler(
                    compiler::markdown::MarkdownCompiler::new("templates/**", "index.html", None)
                        .unwrap()
                        .get(),
                ),
        )
        .add_rule(
            Rule::new("others")
                .set_waits(["markdown"])
                .set_match(["**/*"])
                .set_compiler(compiler::copy::CopyCompiler::new().get()),
        )
        .build()
        .await
        .unwrap();
}
