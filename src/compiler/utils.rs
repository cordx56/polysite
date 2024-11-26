use crate::*;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Create compiler from closure
impl<F> Compiler for F
where
    F: (Fn(Context) -> CompilerReturn) + Clone + Send + Sync,
{
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        (self)(ctx)
    }
}

/// Wait for other tasks. This may be used to utilize intermediate results.
#[derive(Clone)]
pub struct WaitStage {
    steps: usize,
    current: usize,
}
impl WaitStage {
    pub fn new() -> Self {
        Self {
            steps: 1,
            current: 0,
        }
    }
    /// Specify how many steps to wait. The default is one step.
    pub fn wait_steps(steps: usize) -> Self {
        Self { steps, current: 0 }
    }
}
impl Compiler for WaitStage {
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        if self.current < self.steps {
            self.current += 1;
            compile!(Ok(CompileStep::WaitStage(ctx)))
        } else {
            compile!(Ok(CompileStep::Completed(ctx)))
        }
    }
}

/// Create a large compiler by piping multiple compilers.
/// You may also use [`pipe!`] macro.
#[derive(Clone)]
pub struct PipeCompiler {
    compilers: Vec<Box<dyn Compiler>>,
    ready: Option<Arc<RwLock<(usize, Vec<Box<dyn Compiler>>)>>>,
}
impl PipeCompiler {
    pub fn new() -> Self {
        Self {
            compilers: Vec::new(),
            ready: None,
        }
    }
    /// Add the compiler to the end of the pipeline.
    pub fn add_compiler(mut self, compiler: impl Compiler + 'static) -> Self {
        self.compilers.push(Box::new(compiler));
        self
    }
    fn setup(&mut self) {
        if self.ready.is_none() {
            self.ready = Some(Arc::new(RwLock::new((0, self.compilers.clone()))));
        }
    }
}
impl Compiler for PipeCompiler {
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        self.setup();
        let ready = self.ready.clone().unwrap();
        compile!({
            let (ref mut current, ref mut compilers) = *ready.write().await;
            if let Some(compiler) = compilers.get_mut(*current) {
                let res = compiler.next_step(ctx).await?;
                match res {
                    CompileStep::Completed(ctx) => {
                        *current += 1;
                        if *current == compilers.len() {
                            Ok(CompileStep::Completed(ctx))
                        } else {
                            Ok(CompileStep::InProgress(ctx))
                        }
                    }
                    _ => Ok(res),
                }
            } else {
                Ok(CompileStep::Completed(ctx))
            }
        })
    }
}

/// [`pipe!`] macro may used to make large compiler from piping multiple compilers
///
/// # Example
/// This example will read source as Markdown and write HTML to target.
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
