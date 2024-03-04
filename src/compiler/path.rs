use crate::{
    builder::metadata::{PATH_META, TARGET_FILE_META},
    *,
};

/// [`SetExtension`] will change target file's extension and URL path extension to specified one.
pub struct SetExtension(String);
impl SetExtension {
    pub fn new(ext: impl AsRef<str>) -> Self {
        Self(ext.as_ref().to_owned())
    }
}
impl Compiler for SetExtension {
    fn compile(&self, mut ctx: Context) -> CompilerReturn {
        let ext = self.0.clone();
        compile!({
            let mut target = ctx.target()?;
            let mut path = ctx.path()?;
            target.set_extension(ext.clone());
            ctx.insert_compiling_metadata(
                TARGET_FILE_META,
                Metadata::from(target.to_string_lossy().to_string()),
            );
            path.set_extension(ext);
            ctx.insert_compiling_metadata(
                PATH_META,
                Metadata::from(path.to_string_lossy().to_string()),
            );
            Ok(ctx)
        })
    }
}
