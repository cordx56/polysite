pub mod builder;
pub mod compiler;
pub mod config;
pub mod error;
pub mod routing;

pub use builder::*;
pub use compiler::*;
pub use config::Config;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;
    use std::path::PathBuf;

    struct PrintCompiler {}
    impl PrintCompiler {
        fn new() -> Box<Self> {
            Box::new(Self {})
        }
    }
    impl Compiler for PrintCompiler {
        fn compile(&self, ctx: Context) -> CompilerReturn {
            Box::new(compiler!({
                let src = ctx.source().to_string_lossy().to_string();
                let tgt = ctx.target().to_string_lossy().to_string();
                println!("{} -> {}", src, tgt);
                Ok(Metadata::String(tgt))
            }))
        }
    }

    #[tokio::test]
    async fn build_site() {
        let mut config = Config::default();
        config.set_source_dir(Some(PathBuf::from("src")));
        let builder = Builder::new(config);
        let result = builder
            .add_rule(
                Rule::new("compile")
                    .set_match(["**/*"])
                    .set_routing(routing::set_ext("txt"))
                    .set_compiler(
                        GenericCompiler::from(|ctx| {
                            compiler!({
                                ctx.wait("hello").await?;
                                println!("{:?}", ctx.metadata().await);
                                Ok(Metadata::Null)
                            })
                        })
                        .get(),
                    ),
            )
            .await
            .add_rule(
                Rule::new("hello")
                    .set_match(["**/*"])
                    .set_routing(routing::set_ext("txt"))
                    .set_compiler(PrintCompiler::new()),
            )
            .await
            .build()
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
