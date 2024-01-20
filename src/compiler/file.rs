use crate::*;
use anyhow::{anyhow, Ok};
use std::fs::copy;
use std::io::Write;

pub struct FileReader;
impl FileReader {
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for FileReader {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        compiler!({
            let mut ctx = ctx;
            let src = ctx.get_source_string()?;
            ctx.insert_compiling_metadata(BODY_META, src)?;
            Ok(ctx)
        })
    }
}

pub struct FileWriter;
impl FileWriter {
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for FileWriter {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        compiler!({
            let mut target = ctx.open_target().map_err(|e| {
                anyhow!(
                    "Failed to open file {}: {:?}",
                    ctx.target().unwrap().display(),
                    e
                )
            })?;
            let body = ctx.body()?;
            target
                .write(body.as_bytes())
                .map_err(|e| anyhow!("Failed to write file: {:?}", e))?;
            Ok(ctx)
        })
    }
}

pub struct CopyCompiler;
impl CopyCompiler {
    pub fn new() -> Self {
        Self
    }
}
impl Compiler for CopyCompiler {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        compiler!({
            ctx.create_target_dir()?;
            let src = ctx.source()?;
            let tgt = ctx.target()?;
            copy(src, tgt).map_err(|e| anyhow!("Copy error: {:?}", e))?;
            Ok(ctx)
        })
    }
}
