use crate::*;
use anyhow::{anyhow, Ok};
use std::fs::copy;

pub struct CopyCompiler {}

impl Compiler for CopyCompiler {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        compiler!({
            let src = ctx.source();
            let tgt = ctx.target();
            copy(src, tgt).map_err(|e| anyhow!("Copy error: {:?}", e))?;
            Ok(Metadata::Null)
        })
    }
}
