use crate::*;
use std::sync::Arc;

/// Compiler function
///
/// compiler function takes `Context` as parameter,
/// and returns `CompilerReturn` which is boxed future.
pub trait CompileFunction: Fn(Context) -> CompilerReturn + Send + Sync {}
impl<F> CompileFunction for F where F: Fn(Context) -> CompilerReturn + Send + Sync {}

/// Generic compiler
/// You can create new compiler using this.
pub struct GenericCompiler {
    compile_method: Box<dyn CompileFunction>,
}
impl GenericCompiler {
    pub fn empty() -> Self {
        Self::from(|ctx| compile!(Ok(ctx)))
    }
    /// Create compiler from closure which implements [`CompileFunction`].
    ///
    /// # Example
    /// ```
    /// use polysite::{compiler::utils::GenericCompiler, *};
    /// GenericCompiler::from(|ctx| compile!({ Ok(ctx) }));
    /// ```
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

/// [`pipe!`] macro may used to make large compiler from
/// piping multiple compilers
///
/// # Example
/// This example will write source file as HTML file as-is.
///
/// ```
/// use polysite::{compiler::*, *};
/// pipe!(
///     path::SetExtension::new("html"),
///     file::FileReader::new(),
///     file::FileWriter::new(),
/// );
/// ```
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
pub use pipe;
