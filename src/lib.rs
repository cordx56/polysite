pub mod builder;
pub mod compiler;
pub mod config;

pub use builder::*;
pub use compiler::{CompileFunction, CompileResult, Compiler, CompilerReturn};
pub use config::Config;

#[cfg(test)]
mod tests {
    use super::*;

    struct PrintCompiler;
    impl PrintCompiler {
        fn new() -> Self {
            Self
        }
    }
    impl Compiler for PrintCompiler {
        fn compile(&self, ctx: Context) -> CompilerReturn {
            Box::new(compile!({
                let src = ctx.source()?;
                let tgt = ctx.target()?;
                println!("{} -> {}", src.display(), tgt.display());
                Ok(ctx)
            }))
        }
    }

    #[tokio::test]
    async fn build_site() {
        let config = Config::default().set_source_dir("src");
        let builder = Builder::new(config);
        let result = builder
            // Add one rule as build step
            .add_step([Rule::new("hello")
                .set_globs(["builder/**/*.rs"])
                .set_compiler(PrintCompiler::new().get())])
            // Rules in same step will build concurrently, but
            // the file matching is evaluated in order
            .add_step([
                Rule::new("compile").set_globs(["compiler/*"]).set_compiler(
                    pipe!(
                        compiler::path::SetExtension::new("txt"),
                        PrintCompiler::new()
                    )
                    .get(),
                ),
                Rule::new("compile").set_globs(["**/*"]).set_compiler(
                    pipe!(
                        compiler::path::SetExtension::new("txt"),
                        compiler::utils::GenericCompiler::from(|ctx| {
                            compile!({
                                println!("{}", ctx.source()?.display());
                                Ok(ctx)
                            })
                        })
                    )
                    .get(),
                ),
            ])
            .build()
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
