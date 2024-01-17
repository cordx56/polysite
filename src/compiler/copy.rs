use crate::*;
use anyhow::{anyhow, Ok};
use std::fs::copy;

pub struct CopyCompiler;
impl CopyCompiler {
    pub fn new() -> Self {
        CopyCompiler
    }
}

impl Compiler for CopyCompiler {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        compiler!({
            ctx.create_target_dir()?;
            let src = ctx.source();
            let tgt = ctx.target();
            copy(src, tgt).map_err(|e| anyhow!("Copy error: {:?}", e))?;
            Ok(Metadata::Null)
        })
    }
}
