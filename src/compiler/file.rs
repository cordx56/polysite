use crate::*;
use anyhow::Context as _;
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
            let target_display = ctx.target()?;
            let target_display = target_display.display();
            let mut target = ctx
                .open_target()
                .with_context(|| format!("Failed to open file {}", target_display))?;
            let body = ctx.body()?;
            target
                .write(body.as_bytes())
                .with_context(|| format!("Failed to write file {}", target_display))?;
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
            copy(src, tgt).context("Copy error")?;
            Ok(ctx)
        })
    }
}
