use crate::{builder::metadata::BODY_META, *};
use anyhow::{anyhow, Context as _};
use std::fs::copy;
use std::io::Write;

/// [`FileReader`] compiler will read source file as String and
/// store data as `_body` metadata.
pub struct FileReader;
impl FileReader {
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for FileReader {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        compile!({
            let mut ctx = ctx;
            let src = ctx.get_source_data()?;
            ctx.insert_compiling_metadata(BODY_META, src);
            Ok(ctx)
        })
    }
}

/// [`FileWriter`] compiler will write data stored in `_body` metadata to target file.
pub struct FileWriter;
impl FileWriter {
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for FileWriter {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        compile!({
            let target_display = ctx.target()?;
            let target_display = target_display.display();
            let mut target = ctx
                .open_target()
                .with_context(|| format!("Failed to open file {}", target_display))?;
            let body = ctx.body()?;
            if let Some(s) = body.as_str() {
                target
                    .write(s.read().unwrap().as_bytes())
                    .with_context(|| format!("Failed to write file {}", target_display))?;
            } else if let Metadata::Bytes(bytes) = body {
                target
                    .write(bytes.read().unwrap().as_slice())
                    .with_context(|| format!("Failed to write file {}", target_display))?;
            } else {
                return Err(anyhow!("Invalid body format"));
            }
            Ok(ctx)
        })
    }
}

/// [`CopyCompiler`] will simply copies source file to target file
pub struct CopyCompiler;
impl CopyCompiler {
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for CopyCompiler {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        compile!({
            ctx.create_target_parent_dir()?;
            let src = ctx.source()?;
            let tgt = ctx.target()?;
            copy(src, tgt).context("Copy error")?;
            Ok(ctx)
        })
    }
}
