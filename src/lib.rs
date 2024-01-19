pub mod builder;
pub mod compiler;
pub mod config;
pub mod error;
pub mod router;

pub use builder::*;
pub use compiler::*;
pub use config::Config;
pub use router::*;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;
    use std::path::PathBuf;

    struct PrintCompiler;
    impl PrintCompiler {
        fn new() -> Self {
            Self
        }
    }
    impl Compiler for PrintCompiler {
        fn compile(&self, ctx: Context) -> CompilerReturn {
            Box::new(compiler!({
                let src = ctx.source()?;
                let tgt = ctx.target()?;
                println!("{} -> {}", src.display(), tgt.display());
                Ok(ctx)
            }))
        }
    }

    #[tokio::test]
    async fn build_site() {
        let mut config = Config::default();
        config.set_source_dir(Some(PathBuf::from("src")));
        let builder = Builder::new(config);
        let result = builder
            // Add one rule as build step
            .add_step([Rule::new("hello")
                .set_globs(["builder/**/*.rs"])
                .set_compiler(PrintCompiler::new().get())])
            // Rules in same step will build concurrently, but
            // the file matching is evaluated in order
            .add_step([
                Rule::new("compile")
                    .set_globs(["compiler/*"])
                    .set_router(SetExtRouter::new("txt").get())
                    .set_compiler(
                        GenericCompiler::from(|ctx| {
                            compiler!({
                                println!("{}", ctx.source()?.display());
                                Ok(ctx)
                            })
                        })
                        .get(),
                    ),
                Rule::new("compile")
                    .set_globs(["**/*"])
                    .set_router(SetExtRouter::new("txt").get())
                    .set_compiler(
                        GenericCompiler::from(|ctx| {
                            compiler!({
                                println!("{}", ctx.source()?.display());
                                Ok(ctx)
                            })
                        })
                        .get(),
                    ),
            ])
            .build()
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
