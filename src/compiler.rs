pub mod file;
pub mod markdown;
pub mod metadata;
pub mod path;
pub mod snapshot;
pub mod template;
pub mod utils;

pub use utils::pipe;

use crate::*;
use anyhow::Error;
use std::future::Future;
use std::sync::Arc;

/// [`CompileResult`] is type that returned by compiler's compile method.
pub type CompileResult = Result<Context, Error>;
/// [`CompilerReturn`] is boxed `Future`, which executes compile.
pub type CompilerReturn = Box<dyn Future<Output = CompileResult> + Unpin + Send>;

/// Compiler trait
///
/// All compiler must implement this trait.
pub trait Compiler: Send + Sync {
    /// `compile` method, which takes `Context` as parameter
    /// and return `CompilerReturn`, is required to implement.
    fn compile(&self, ctx: Context) -> CompilerReturn;
    /// `get` method is provided to get Arc pointer.
    fn get(self) -> Arc<Self>
    where
        Self: Sized,
    {
        Arc::new(self)
    }
}

/// [`compile!`] macro may used to make compile function which returns boxed Future.
/// This is provided for ease of creating boxed Future.
#[macro_export]
macro_rules! compile {
    ($b:expr) => {
        Box::new(Box::pin(async move { $b }))
    };
}
