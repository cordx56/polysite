use crate::*;

/// `SetExtension` will change target file's extension to
/// specified one.
pub struct SetExtension(String);
impl SetExtension {
    pub fn new(ext: impl ToString) -> Self {
        Self(ext.to_string())
    }
}
impl Compiler for SetExtension {
    fn compile(&self, mut ctx: Context) -> CompilerReturn {
        let ext = self.0.clone();
        compile!({
            let mut target = ctx.target()?;
            target.set_extension(ext);
            ctx.insert_compiling_metadata(TARGET_FILE_META, target)?;
            Ok(ctx)
        })
    }
}
