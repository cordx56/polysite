use crate::site::context::SiteContext;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

pub type CompileResult = Result<(), ()>;
pub trait RouteMethod: Fn(String) -> String + Send + Sync {}
pub trait CompileMethod:
    Fn(Arc<Mutex<SiteContext>>, &mut Rule) -> CompileResult + Send + Sync
{
}
impl<F> CompileMethod for F where
    F: Fn(Arc<Mutex<SiteContext>>, &mut Rule) -> CompileResult + Send + Sync
{
}

pub type Metadata = HashMap<String, String>;

pub struct Rule {
    match_globs: Option<Vec<String>>,
    route_method: Option<Box<dyn RouteMethod>>,
    compile_method: Option<Arc<Box<dyn CompileMethod>>>,
    metadata: Option<Metadata>,
    load: bool,
    load_notify: Arc<Notify>,
}

impl Rule {
    pub fn new() -> Self {
        Rule {
            match_globs: None,
            route_method: None,
            compile_method: None,
            metadata: None,
            load: false,
            load_notify: Arc::new(Notify::new()),
        }
    }

    pub fn set_match(mut self, globs: impl IntoIterator<Item = impl ToString>) -> Self {
        let gs = globs.into_iter().map(|s| s.to_string()).collect();
        self.match_globs = Some(gs);
        self
    }
    pub fn set_route(mut self, route_method: impl RouteMethod + 'static) -> Self {
        self.route_method = Some(Box::new(route_method));
        self
    }

    /// Set compiler method
    ///
    /// This method will be called in compilation task.
    pub fn set_compiler(mut self, compile_method: impl CompileMethod + 'static) -> Self {
        self.compile_method = Some(Arc::new(Box::new(compile_method)));
        self
    }

    /// Get compiled metadata
    pub fn get_metadata(&self) -> Option<Metadata> {
        self.metadata.clone()
    }

    /// Get load notify
    ///
    /// If compilation task is finished, this method returns None.
    /// Otherwise this method returns Arc<tokio::sync::Notify>.
    pub fn get_load_notify(&self) -> Option<Arc<Notify>> {
        if self.load {
            None
        } else {
            Some(self.load_notify.clone())
        }
    }

    /// Do compilation task
    ///
    /// Send notifications to all waiters when tasks are completed.
    pub(crate) async fn compile(&mut self, ctx: Arc<Mutex<SiteContext>>) -> CompileResult {
        //let match_globs = self.match_globs.as_ref().ok_or(())?;
        let compile_method = self.compile_method.clone().ok_or(())?;
        compile_method(ctx, self)?;
        // Done
        self.load = true;
        self.load_notify.notify_waiters();
        Ok(())
    }
}
