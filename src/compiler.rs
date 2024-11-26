pub mod file;
pub mod markdown;
pub mod metadata;
pub mod path;
pub mod template;
pub mod utils;

pub use utils::pipe;

use crate::*;
use dyn_clone::{clone_trait_object, DynClone};
use std::future::Future;
use std::pin::Pin;

pub enum CompileStep {
    /// The compilation task is completed.
    Completed(Context),
    /// Compilation task is in progress.
    InProgress(Context),
    /// Wait for other tasks to finish the same step.
    WaitStage(Context),
}

/// [`CompileResult`] is the result type that returned by compiler's compile method.
pub type CompileResult = Result<CompileStep, Error>;
/// [`CompilerReturn`] is boxed [`Future`], which executes compile.
pub type CompilerReturn = Pin<Box<dyn Future<Output = CompileResult> + Send>>;

/// All compiler must implement [`Compiler`] trait.
pub trait Compiler: DynClone + Send + Sync {
    /// Executes the next step of the compilation
    fn next_step(&mut self, ctx: Context) -> CompilerReturn;
}
clone_trait_object!(Compiler);

/// [`compile!`] macro may used to make pinned, boxed Future, which is async block.
#[macro_export]
macro_rules! compile {
    ($b:expr) => {
        (Box::pin(async move { $b }) as _)
    };
}
