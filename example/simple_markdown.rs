use polysite::*;

#[tokio::main]
async fn main() {
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
        .await
        .build()
        .await
        .unwrap();
}
