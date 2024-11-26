use crate::{builder::metadata::Value, *};
use serde::Serialize;
use std::collections::HashMap;

/// The compiler to set [`Metadata`] for the [`Context`].
#[derive(Clone)]
pub struct SetMetadata {
    compiling: HashMap<String, Value>,
    global: HashMap<String, Value>,
}
impl SetMetadata {
    pub fn new() -> Self {
        Self {
            compiling: HashMap::new(),
            global: HashMap::new(),
        }
    }
    pub fn global(mut self, k: impl AsRef<str>, v: impl Serialize) -> Result<Self, Error> {
        self.global
            .insert(k.as_ref().to_owned(), Metadata::to_value(v)?);
        Ok(self)
    }
    pub fn local(mut self, k: impl AsRef<str>, v: impl Serialize) -> Result<Self, Error> {
        self.compiling
            .insert(k.as_ref().to_owned(), Metadata::to_value(v)?);
        Ok(self)
    }
}
impl Compiler for SetMetadata {
    #[tracing::instrument(skip(self, ctx))]
    fn next_step(&mut self, mut ctx: Context) -> CompilerReturn {
        let s = self.clone();
        compile!({
            for (k, v) in s.global.into_iter() {
                ctx.metadata().insert_global(k, v).await;
            }
            for (k, v) in s.compiling.into_iter() {
                ctx.metadata_mut().insert_local(k, v);
            }
            Ok(CompileStep::Completed(ctx))
        })
    }
}
