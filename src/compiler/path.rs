use crate::{
    builder::metadata::{PATH_META, TARGET_FILE_META},
    *,
};
use tracing_error::SpanTrace;

/// [`SetExtension`] will change target file's extension and URL path extension to specified one.
#[derive(Clone)]
pub struct SetExtension(String);
impl SetExtension {
    pub fn new(ext: impl AsRef<str>) -> Self {
        Self(ext.as_ref().to_owned())
    }
}
impl Compiler for SetExtension {
    #[tracing::instrument(skip(self, ctx))]
    fn next_step(&mut self, mut ctx: Context) -> CompilerReturn {
        let ext = self.0.clone();
        compile!({
            let mut target = ctx.target().await.ok_or(Error::InvalidMetadata {
                trace: SpanTrace::capture(),
            })?;
            let mut path = ctx.path().await.ok_or(Error::InvalidMetadata {
                trace: SpanTrace::capture(),
            })?;
            target.set_extension(ext.clone());
            ctx.metadata_mut().insert_local(
                TARGET_FILE_META.to_owned(),
                Metadata::to_value(target.to_string_lossy())?,
            );
            path.set_extension(ext);
            ctx.metadata_mut().insert_local(
                PATH_META.to_owned(),
                Metadata::to_value(path.to_string_lossy())?,
            );
            Ok(CompileStep::Completed(ctx))
        })
    }
}
