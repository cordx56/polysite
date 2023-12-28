use crate::rule::{Metadata, Rule};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SiteContext {
    pub(crate) rules: Arc<Mutex<HashMap<String, Arc<Mutex<Rule>>>>>,
}

impl SiteContext {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_rule(&mut self, name: impl ToString, rule: Rule) {
        self.rules
            .lock()
            .await
            .insert(name.to_string(), Arc::new(Mutex::new(rule)));
    }

    pub async fn load(&self, name: impl ToString) -> Metadata {
        let named = name.to_string();
        if let Some(notify) = self
            .rules
            .lock()
            .await
            .get(&named)
            .unwrap()
            .lock()
            .await
            .get_load_notify()
        {
            notify.notified().await;
        }
        self.rules
            .lock()
            .await
            .get(&named)
            .unwrap()
            .lock()
            .await
            .get_metadata()
            .unwrap()
    }
}
