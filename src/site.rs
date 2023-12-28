use crate::rule::Rule;
use std::collections::HashMap;

pub struct SiteBuilder {
    rules: HashMap<String, Rule>,
}

impl SiteBuilder {
    pub fn new() -> Self {
        SiteBuilder {
            rules: HashMap::new(),
        }
    }
    pub fn add_rule<S: ToString>(&mut self, name: S, rule: Rule) -> &mut Self {
        self.rules.insert(name.to_string(), rule);
        self
    }
}
