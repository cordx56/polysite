//! Highly customizable, polymorphic static site generator library, polysite.
//!
//! This crate is inspired by [Hakyll][hakyll] written in Haskell.
//!
//! # Concept
//!
//! The concept of polysite is _a set of components_ that can be used for static site generation. You can make your own static site generator _by combining_ the various parts in polysite.
//! polysite uses [`Metadata`] to store the result of each step of the compilation. Each step is passed a [`Context`], modifies metadata, and returns a [`CompileResult`], which represents the result of that step of the compilation task. Each step of compilation process is implemented by implementing [`Compiler::next_step`]. You can create a large compiler by _piping_ together multiple compilers.
//!
//! # How to use
//! If you would like to simply build site written in Markdown, use [`compiler::markdown::MarkdownCompiler`].
//! The example is in [`examples/simple_markdown.rs`][simple_example].
//!
//! # How to create compiler
//! If you would like to create a new compiler, implement [`Compiler`] trait for your type.
//! [`Compiler::next_step`] method is used to define each step of the compilation task.
//!
//! [`compile!`] macro is provided for ease of creating pinned and boxed Future.
//!
//! If you would like to pipe some compilers, use [`pipe!`] macro.
//!
//! [`Compiler`] trait is implemented for closures that take a [`Context`] as an argument and return a [`CompilerReturn`].
//!
//! # Metadata
//! polysite uses [`Metadata`] to save compilation result and it can be used in other compilation task.
//!
//! There are some default metadata:
//! - [`_rule`][builder::metadata::RULE_META]: Compiling rule name
//! - [`_version`][builder::metadata::VERSION_META]: Compiling file version
//! - [`_source`][builder::metadata::SOURCE_FILE_META]: source file path
//! - [`_target`][builder::metadata::TARGET_FILE_META]: target file path
//! - [`_path`][builder::metadata::PATH_META]: absolute URL path
//! - [`_body`][builder::metadata::BODY_META]: Content body. For the result of each compilation task.
//!
//! You can use these default key of [`Metadata`] to create new compiler.
//!
//! # Example
//! Practical example is here.
//! Other examples are in [repository][examples].
//! ```
//! use polysite::{
//!     compiler::{
//!         file::CopyCompiler, markdown::MarkdownCompiler, metadata::SetMetadata,
//!         template::TemplateEngine,
//!     },
//!     *,
//! };
//! use tracing_subscriber::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let subscriber =
//!         tracing_subscriber::Registry::default().with(tracing_error::ErrorLayer::default());
//!     tracing::subscriber::set_global_default(subscriber).unwrap();
//!
//!     simple_logger::SimpleLogger::new().env().init().unwrap();
//!
//!     let template_engine = TemplateEngine::new("templates/**").unwrap();
//!     Builder::new(Config::default())
//!         .add_step([Rule::new(
//!             "metadata",
//!             SetMetadata::new()
//!                 .global("site_title", "Hello, polysite!")
//!                 .unwrap()
//!                 .global("site_url", "https://example.com")
//!                 .unwrap(),
//!         )
//!         .set_create(["metadata"])])
//!         .add_step([Rule::new(
//!             "posts",
//!             MarkdownCompiler::new(template_engine.clone(), "practical.html", None),
//!         )
//!         .set_globs(["posts/**/*.md"])])
//!         .add_step([
//!             Rule::new(
//!                 "markdown",
//!                 MarkdownCompiler::new(template_engine.clone(), "practical.html", None),
//!             )
//!             .set_globs(["**/*.md"]),
//!             Rule::new("others", CopyCompiler::new()).set_globs(["**/*"]),
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
pub mod error;

#[doc(inline)]
pub use builder::{
    builder::Builder,
    context::{Context, Version},
    metadata::Metadata,
    rule::Rule,
};
#[doc(inline)]
pub use compiler::{CompileResult, CompileStep, Compiler, CompilerReturn};
#[doc(inline)]
pub use config::Config;
#[doc(inline)]
pub use error::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct PrintCompiler;
    impl PrintCompiler {
        fn new() -> Self {
            Self
        }
    }
    impl Compiler for PrintCompiler {
        fn next_step(&mut self, ctx: Context) -> CompilerReturn {
            compile!({
                let src = ctx.source().await.unwrap();
                let tgt = ctx.target().await.unwrap();
                println!("{} -> {}", src.display(), tgt.display());
                Ok(CompileStep::Completed(ctx))
            })
        }
    }

    #[tokio::test]
    async fn build_site() {
        let config = Config::default().set_source_dir("src");
        let builder = Builder::new(config);
        let result = builder
            // Add one rule as build step
            .add_step([Rule::new("hello", PrintCompiler::new()).set_globs(["builder/**/*.rs"])])
            // Rules in same step will build concurrently, but
            // the file matching is evaluated in order
            .add_step([
                Rule::new(
                    "compile",
                    pipe!(
                        compiler::path::SetExtension::new("txt"),
                        PrintCompiler::new()
                    ),
                )
                .set_globs(["compiler/*"]),
                Rule::new(
                    "compile",
                    pipe!(
                        compiler::path::SetExtension::new("txt"),
                        |ctx: Context| compile!({
                            println!("{}", ctx.source().await.unwrap().display());
                            Ok(CompileStep::Completed(ctx))
                        })
                    ),
                )
                .set_globs(["**/*"]),
            ])
            .build()
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
