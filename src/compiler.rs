pub mod file;
pub mod markdown;
pub mod path;
pub mod snapshot;
pub mod template;
pub mod utils;

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

/// pipe! macro may used to make large compiler from
/// piping multiple compilers
#[macro_export]
macro_rules! pipe {
    ($f:expr, $($n:expr),+ $(,)?) => {{
        $crate::compiler::utils::PipeCompiler::new(vec![
            $f.get(),
            $(
                $n.get(),
            )+
        ])
    }}
}
