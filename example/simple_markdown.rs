use polysite::*;

#[tokio::main]
async fn main() {
    let builder = Builder::new(Config::default());
    builder
        .add_rule(
            Rule::new("markdown")
                .set_match(["**/*.md"])
                .set_routing(routing::set_ext("html"))
                .set_compiler(compiler::markdown::markdown_compiler(
                    "templates/**",
                    "index.html",
                    None,
                )),
        )
        .await
        .build()
        .await
        .unwrap();
}
