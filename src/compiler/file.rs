use crate::{builder::metadata::*, *};
use std::fs::copy;
use std::io::Write;
use tracing_error::SpanTrace;

/// [`FileReader`] reads the source file as a [`String`] and stores the data using [`SOURCE_FILE_META`] as the key.
#[derive(Clone)]
pub struct FileReader;
impl FileReader {
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for FileReader {
    #[tracing::instrument(skip(self, ctx))]
    fn next_step(&mut self, mut ctx: Context) -> CompilerReturn {
        compile!({
            let src = ctx.source_body().await?;
            let meta = match String::from_utf8(src.clone()) {
                Ok(s) => Value::from(s),
                Err(_) => Value::from_bytes(&src),
            };
            ctx.metadata_mut().insert_local(BODY_META.to_owned(), meta);
            Ok(CompileStep::Completed(ctx))
        })
    }
}

/// [`FileWriter`] writes the data stored in [`BODY_META`] to the target file, which path is saved in [`TARGET_FILE_META`].
#[derive(Clone)]
pub struct FileWriter;
impl FileWriter {
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for FileWriter {
    #[tracing::instrument(skip(self, ctx))]
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        compile!({
            let mut target = ctx.open_target().await?;
            let body = ctx.body().await.ok_or(Error::InvalidMetadata {
                trace: SpanTrace::capture(),
            })?;
            let write = if let Some(s) = body.as_str() {
                target.write(s.as_bytes())
            } else if let Some(bytes) = body.as_bytes() {
                target.write(&bytes)
            } else {
                return Err(Error::InvalidMetadata {
                    trace: SpanTrace::capture(),
                });
            };
            write.map_err(|io_error| Error::FileIo {
                trace: SpanTrace::capture(),
                io_error,
            })?;
            Ok(CompileStep::Completed(ctx))
        })
    }
}

/// [`CopyCompiler`] simply copies source file to target file
#[derive(Clone)]
pub struct CopyCompiler;
impl CopyCompiler {
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for CopyCompiler {
    #[tracing::instrument(skip(self, ctx))]
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        compile!({
            ctx.create_target_parent_dir().await?;
            let src = ctx.source().await;
            let tgt = ctx.target().await;
            match (src, tgt) {
                (Some(src), Some(tgt)) => {
                    copy(src, tgt).map_err(|io_error| Error::FileIo {
                        trace: SpanTrace::capture(),
                        io_error,
                    })?;
                    Ok(CompileStep::Completed(ctx))
                }
                _ => Err(Error::InvalidMetadata {
                    trace: SpanTrace::capture(),
                }),
            }
        })
    }
}
