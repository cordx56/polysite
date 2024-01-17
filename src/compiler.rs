pub mod markdown;

use crate::*;
use anyhow::Error;
use std::future::Future;

pub type CompileResult = Result<Metadata, Error>;
pub type CompilerReturn = Box<dyn Future<Output = CompileResult> + Unpin + Send>;

pub trait Compiler: Send + Sync {
    fn compile(&self, ctx: Context) -> CompilerReturn;
    fn get(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

pub trait CompileFunction: Fn(Context) -> CompilerReturn + Send + Sync {}
impl<F> CompileFunction for F where F: Fn(Context) -> CompilerReturn + Send + Sync {}

#[macro_export]
macro_rules! compiler {
    ($b:expr) => {
        Box::new(Box::pin(async move {
            use $crate::to_metadata;
            let res = $b;
            res.map(|v| to_metadata(v))?
        }))
    };
}

pub struct GenericCompiler {
    compile_method: Box<dyn CompileFunction>,
}
impl GenericCompiler {
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
