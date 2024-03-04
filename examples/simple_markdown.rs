use polysite::{
    compiler::{markdown::MarkdownCompiler, template::TemplateEngine},
    *,
};

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let template_engine = TemplateEngine::new("templates/**").unwrap().get();
    Builder::new(Config::default())
        .insert_metadata("site_title", Metadata::from("Hello, polysite!"))
        .add_step([
            Rule::new("posts")
                .set_globs(["posts/**/*.md"])
                .set_compiler(
                    MarkdownCompiler::new(template_engine.clone(), "index.html", None).get(),
                ),
            Rule::new("markdown").set_globs(["**/*.md"]).set_compiler(
                MarkdownCompiler::new(template_engine.clone(), "index.html", None).get(),
            ),
        ])
        .build()
        .await
        .unwrap();
}
