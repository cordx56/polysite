use crate::*;
use std::sync::Arc;

pub struct GenericCompiler {
    compile_method: Box<dyn CompileFunction>,
}
impl GenericCompiler {
    pub fn empty() -> Self {
        Self::from(|ctx| compile!(Ok(ctx)))
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

/// Pipe compiler
/// Create large compiler from piping small ones
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
        compile!({
            let mut ctx = ctx;
            for c in compilers {
                ctx = c.compile(ctx).await?;
            }
            Ok(ctx)
        })
    }
}
