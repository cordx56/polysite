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

/// `CompileResult` is type that returned by compiler's compile method.
pub type CompileResult = Result<Context, Error>;
/// `CompilerReturn` is boxed `Future`, which executes compile.
pub type CompilerReturn = Box<dyn Future<Output = CompileResult> + Unpin + Send>;

/// Compiler trait
///
/// All compiler must implement this trait.
/// `compile` method, which takes `Context` as parameter
/// and return `CompilerReturn`, is required to implement.
///
/// `get` method is provided to get Arc pointer.
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
/// compiler function takes `Context` as parameter,
/// and returns `CompilerReturn` which is boxed future.
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
