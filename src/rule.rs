use std::sync::Arc;
use tokio::sync::Notify;

pub type CompileResult = Result<(), ()>;
pub trait RouteMethod: Fn(String) -> String {}
pub trait CompileMethod: Fn(&mut Rule) -> CompileResult {}

pub struct Rule {
    match_glob: Option<String>,
    route_method: Option<Box<dyn RouteMethod>>,
    compile_method: Option<Arc<Box<dyn CompileMethod>>>,
    load: bool,
    load_notify: Arc<Notify>,
}

impl Rule {
    pub fn new() -> Self {
        Rule {
            match_glob: None,
            route_method: None,
            compile_method: None,
            load: false,
            load_notify: Arc::new(Notify::new()),
        }
    }

    pub fn set_match(&mut self, glob: impl ToString) -> &mut Self {
        self.match_glob = Some(glob.to_string());
        self
    }
    pub fn set_route(&mut self, route_method: impl RouteMethod + 'static) -> &mut Self {
        self.route_method = Some(Box::new(route_method));
        self
    }

    /// Set compiler method
    ///
    /// This method will be called in compilation task.
    pub fn set_compiler(&mut self, compile_method: impl CompileMethod + 'static) -> &mut Self {
        self.compile_method = Some(Arc::new(Box::new(compile_method)));
        self
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
    pub async fn compile(&mut self) -> CompileResult {
        let match_glob = self.match_glob.as_ref().ok_or(())?;
        let compile_method = self.compile_method.clone().ok_or(())?;
        compile_method(self)?;
        // Done
        self.load = true;
        self.load_notify.notify_waiters();
        Ok(())
    }
}
