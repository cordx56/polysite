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
            ctx.insert_compiling_metadata("body".to_string(), src)
                .await?;
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
            let metadata = ctx.metadata().await;
            let body = metadata
                .as_object()
                .unwrap()
                .get("body")
                .ok_or(anyhow!("body field not found in metadata"))?;
            let s = body
                .as_str()
                .ok_or(anyhow!("body field in metadata is not string"))?;
            target
                .write(s.as_bytes())
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
