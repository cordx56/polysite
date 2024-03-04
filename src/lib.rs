//! Highly customizable, polymorphic static site generator library, polysite.
//!
//! This crate is inspired by [Hakyll][hakyll] written in Haskell.
//!
//! # Difference from other static site generator
//! I know [Zola][zola], static site generator written in Rust.
//! But zola is not enough customizable for me.
//! So I create this crate.
//!
//! # How to use
//! If you would like to simply build site written in Markdown, use [`compiler::markdown::MarkdownCompiler`].
//! The example is in [`examples/simple_markdown.rs`][simple_example].
//!
//! # How to create compiler
//! If you would like to create new compiler, implement [`Compiler`] trait for your compiler type.
//! [`Compiler::compile`] method is used for compile source file.
//!
//! [`compile!`] macro is provided for ease of creating boxed Future.
//!
//! If you would like to pipe some compilers, use [`pipe!`] macro.
//!
//! If you would like to create small compiler using closure, use
//! [`compiler::utils::GenericCompiler`].
//!
//! # Metadata
//! polysite uses metadata to save compile result and metadata can be used in other compilation.
//!
//! There are some default metadata:
//! - [`_rule`][builder::metadata::RULE_META]: Compiling rule name
//! - [`_version`][builder::metadata::VERSION_META]: Compiling file version
//! - [`_source`][builder::metadata::SOURCE_FILE_META]: source file path
//! - [`_target`][builder::metadata::TARGET_FILE_META]: target file path
//! - [`_path`][builder::metadata::PATH_META]: absolute URL path
//! - [`_body`][builder::metadata::BODY_META]: Content body. Some compilers save the result to this metadata.
//!
//! You can use these default metadata to create new compiler.
//!
//! # Example
//! Practical example is here.
//! Other examples are in [repository][examples].
//!
//! ```
//! use polysite::{
//!     compiler::{
//!         file::CopyCompiler, markdown::MarkdownCompiler, metadata::SetMetadata,
//!         template::TemplateEngine,
//!     },
//!     *,
//! };
//!
//! #[tokio::main]
//! async fn main() {
//!     simple_logger::SimpleLogger::new().env().init().unwrap();
//!     let template_engine = TemplateEngine::new("templates/**").unwrap().get();
//!     Builder::new(Config::default())
//!         .add_step(
//!             [Rule::new("metadata").set_create(["metadata"]).set_compiler(
//!                 SetMetadata::new()
//!                     .global("site_title", "Hello, polysite!")
//!                     .unwrap()
//!                     .global("site_url", "https://example.com")
//!                     .unwrap()
//!                     .get(),
//!             )],
//!         )
//!         .add_step([Rule::new("posts")
//!             .set_globs(["posts/**/*.md"])
//!             .set_compiler(
//!                 MarkdownCompiler::new(template_engine.clone(), "practical.html", None)
//!                     .wait_snapshot("posts", 1)
//!                     .get(),
//!             )])
//!         .add_step([
//!             Rule::new("markdown").set_globs(["**/*.md"]).set_compiler(
//!                 MarkdownCompiler::new(template_engine.clone(), "practical.html", None).get(),
//!             ),
//!             Rule::new("others")
//!                 .set_globs(["**/*"])
//!                 .set_compiler(CopyCompiler::new().get()),
//!         ])
//!         .build()
//!         .await
//!         .unwrap();
//! }
//! ```
//!
//! [hakyll]: https://jaspervdj.be/hakyll/
//! [zola]: https://www.getzola.org/
//! [simple_example]: https://github.com/cordx56/polysite/blob/main/examples/simple_markdown.rs
//! [examples]: https://github.com/cordx56/polysite/blob/main/examples

pub mod builder;
pub mod compiler;
pub mod config;

pub use builder::{
    builder::Builder,
    context::{Context, Version},
    metadata::Metadata,
    rule::{Conditions, Rule},
};
pub use compiler::{CompileResult, Compiler, CompilerReturn};
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
