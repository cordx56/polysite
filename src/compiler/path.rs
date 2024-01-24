use crate::*;

/// [`SetExtension`] will change target file's extension and URL path extension to specified one.
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
            let mut path = ctx.path()?;
            target.set_extension(ext.clone());
            ctx.insert_compiling_metadata(TARGET_FILE_META, target)?;
            path.set_extension(ext);
            ctx.insert_compiling_metadata(PATH_META, path)?;
            Ok(ctx)
        })
    }
}
