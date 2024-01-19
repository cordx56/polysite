use std::path::PathBuf;
use std::sync::Arc;

pub trait Router: Send + Sync {
    fn route(&self, p: PathBuf) -> PathBuf;
    fn get(self) -> Arc<Self>
    where
        Self: Sized,
    {
        Arc::new(self)
    }
}

pub struct SetExtRouter(String);
impl SetExtRouter {
    pub fn new(ext: impl ToString) -> Self {
        Self(ext.to_string())
    }
}
impl Router for SetExtRouter {
    fn route(&self, mut p: PathBuf) -> PathBuf {
        p.set_extension(&self.0);
        p
    }
}
