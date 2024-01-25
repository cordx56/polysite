use crate::*;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

/// [`SetMetadata`] sets metadata to context
///
/// # Example
/// ```
/// use polysite::compiler::metadata::SetMetadata;
/// SetMetadata::new().global("site_url", "https://example.ccom")
/// SetMetadata::new()
///     .global("site_url", "https://example.ccom")?
///     .compiling("compiling", vec!["value"])?;
/// ```
pub struct SetMetadata {
    compiling: HashMap<String, Metadata>,
    global: HashMap<String, Metadata>,
}
impl SetMetadata {
    pub fn new() -> Self {
        Self {
            compiling: HashMap::new(),
            global: HashMap::new(),
        }
    }
    pub fn global(mut self, k: impl ToString, v: impl Serialize) -> Result<Self> {
        self.global
            .insert(k.to_string(), Metadata::from_serializable(v)?);
        Ok(self)
    }
    pub fn compiling(mut self, k: impl ToString, v: impl Serialize) -> Result<Self> {
        self.compiling
            .insert(k.to_string(), Metadata::from_serializable(v)?);
        Ok(self)
    }
}
impl Compiler for SetMetadata {
    fn compile(&self, mut ctx: Context) -> CompilerReturn {
        let compiling = self.compiling.clone();
        let global = self.global.clone();
        compile!({
            for (k, v) in global.into_iter() {
                ctx.insert_global_raw_metadata(k, v).await?;
            }
            for (k, v) in compiling.into_iter() {
                ctx.insert_compiling_raw_metadata(k, v)?;
            }
            Ok(ctx)
        })
    }
}
