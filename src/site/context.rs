use crate::rule::Rule;
use crate::Metadata;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct BuildContext {
    pub(crate) rules: Arc<Mutex<HashMap<String, Arc<Mutex<Rule>>>>>,
}

impl BuildContext {
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

    pub async fn load(&self, name: impl AsRef<str>) -> Option<Metadata> {
        let named = name.as_ref();
        let notify = {
            self.rules
                .lock()
                .await
                .get(named)
                .unwrap()
                .lock()
                .await
                .get_load_notify()
        };
        if let Some(n) = notify {
            n.notified().await;
        }
        self.rules
            .lock()
            .await
            .get(named)
            .unwrap()
            .lock()
            .await
            .get_metadata()
    }
}
