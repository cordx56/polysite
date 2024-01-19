pub mod file;
pub mod markdown;
pub mod snapshot;
pub mod template;

use crate::*;
use anyhow::Error;
use std::future::Future;
use std::sync::Arc;

pub type CompileResult = Result<Context, Error>;
pub type CompilerReturn = Box<dyn Future<Output = CompileResult> + Unpin + Send>;

pub trait Compiler: Send + Sync {
    fn compile(&self, ctx: Context) -> CompilerReturn;
    fn get(self) -> Arc<Self>
    where
        Self: Sized,
    {
        Arc::new(self)
    }
}

/// Compiler function
///
/// compiler function takes context as parameter,
/// and returns CompilerReturn which is boxed future.
pub trait CompileFunction: Fn(Context) -> CompilerReturn + Send + Sync {}
impl<F> CompileFunction for F where F: Fn(Context) -> CompilerReturn + Send + Sync {}

/// compiler! macro may used to make compile function
/// which returns boxed Future
#[macro_export]
macro_rules! compiler {
    ($b:expr) => {
        Box::new(Box::pin(async move { $b }))
    };
}

pub struct GenericCompiler {
    compile_method: Box<dyn CompileFunction>,
}
impl GenericCompiler {
    pub fn empty() -> Self {
        Self::from(|ctx| compiler!(Ok(ctx)))
    }
    pub fn from<F: CompileFunction + 'static>(f: F) -> Self {
        Self {
            compile_method: Box::new(f),
        }
    }
}
impl Compiler for GenericCompiler {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        (self.compile_method)(ctx)
    }
}

/// pipe! macro may used to make large compiler from
/// piping multiple compilers
#[macro_export]
macro_rules! pipe {
    ($f:expr, $($n:expr),+ $(,)?) => {{
        $crate::PipeCompiler::new(vec![
            $f.get(),
            $(
                $n.get(),
            )+
        ])
    }}
}

/// Piped compiler
pub struct PipeCompiler {
    compilers: Vec<Arc<dyn Compiler>>,
}
impl PipeCompiler {
    pub fn new(compilers: Vec<Arc<dyn Compiler>>) -> Self {
        Self { compilers }
    }
}
impl Compiler for PipeCompiler {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let compilers = self.compilers.clone();
        compiler!({
            let mut ctx = ctx;
            for c in compilers {
                ctx = c.compile(ctx).await?;
            }
            Ok(ctx)
        })
    }
}
