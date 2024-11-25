use crate::*;
use std::collections::VecDeque;

/// Compiler function
///
/// compiler function takes [`Context`] as parameter,
/// and returns [`CompilerReturn`] which is boxed future.
pub trait CompileFunction: (Fn(Context) -> CompilerReturn) + Clone + Send + Sync {}
impl<F> CompileFunction for F where F: (Fn(Context) -> CompilerReturn) + Clone + Send + Sync {}

/// Create compiler from closure which implements [`CompileFunction`].
impl<F> Compiler for F
where
    F: (Fn(Context) -> CompilerReturn) + Clone + Send + Sync,
{
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        (self)(ctx)
    }
}

/// Pipe compiler
/// Create large compiler from piping small ones
#[derive(Clone)]
pub struct PipeCompiler {
    compilers: VecDeque<Box<dyn Compiler>>,
}
impl PipeCompiler {
    pub fn new() -> Self {
        Self {
            compilers: VecDeque::new(),
        }
    }
    pub fn add_compiler(mut self, compiler: impl Compiler + 'static) -> Self {
        self.compilers.push_back(Box::new(compiler));
        self
    }
}
impl Compiler for PipeCompiler {
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        if let Some(mut compiler) = self.compilers.pop_front() {
            let len = self.compilers.len();
            compile!({
                let res = compiler.next_step(ctx).await?;
                match res {
                    CompileStep::Completed(ctx) => {
                        if len == 0 {
                            Ok(CompileStep::Completed(ctx))
                        } else {
                            Ok(CompileStep::InProgress(ctx))
                        }
                    }
                    _ => Ok(res),
                }
            })
        } else {
            compile!(Ok(CompileStep::Completed(ctx)))
        }
    }
}

/// [`pipe!`] macro may used to make large compiler from
/// piping multiple compilers
///
/// # Example
/// This example will read source as Markdown and write HTML to target.
///
/// ```
/// use polysite::{compiler::*, *};
/// pipe!(
///     path::SetExtension::new("html"),
///     file::FileReader::new(),
///     markdown::MarkdownRenderer::new(None),
///     file::FileWriter::new(),
/// );
/// ```
#[macro_export]
macro_rules! pipe {
    ($f:expr, $($n:expr),+ $(,)?) => {{
        $crate::compiler::utils::PipeCompiler::new()
            .add_compiler($f)
            $(
                .add_compiler($n)
            )+
    }}
}
pub use pipe;
