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
    Completed(Context),
    InProgress(Context),
    WaitStage(Context),
}

/// [`CompileResult`] is the result type that returned by compiler's compile method.
pub type CompileResult = Result<CompileStep, Error>;
/// [`CompilerReturn`] is boxed `Future`, which executes compile.
pub type CompilerReturn = Pin<Box<dyn Future<Output = CompileResult> + Send>>;

/// Compiler trait
///
/// All compiler must implement this trait.
pub trait Compiler: DynClone + Send + Sync {
    /*
    /// Takes a value of type [`Context`] as an argument and calls
    /// [`Compiler::next_step`] method repeatedly until it no longer returns a value.
    /// This executes all steps of the compilation process.
    fn compile(&self, mut ctx: Context) -> CompilerReturn {
        compile! {
            loop {
                ctx = match self.next_step(ctx) {
                    CompileStepResult::Completed(v) => return v.await,
                    CompileStepResult::InProgress(v) => v.await?,
                }
            }
        }
    }
    */
    /// Executes the compilation of the next step.
    fn next_step(&mut self, ctx: Context) -> CompilerReturn;
}
/*
impl<C: Compiler> Compiler for Arc<C> {
    async fn compile(&self, ctx: Context) -> CompilerReturn {
        C::compile(&self, ctx)
    }
}
*/
clone_trait_object!(Compiler);

/// [`compile!`] macro may used to make compile function which returns boxed Future.
/// This is provided for ease of creating boxed Future.
#[macro_export]
macro_rules! compile {
    ($b:expr) => {
        (Box::pin(async move { $b }) as _)
    };
}
