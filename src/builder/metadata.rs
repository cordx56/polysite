use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, to_value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub const RULE_META: &str = "_rule";
pub const SOURCE_FILE_META: &str = "_source";
pub const TARGET_FILE_META: &str = "_target";
pub const PATH_META: &str = "_path";
pub const VERSION_META: &str = "_version";
pub const BODY_META: &str = "_body";

/// Metadata for storing compile result or your site metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Metadata {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Arc<RwLock<String>>),
    Array(Arc<RwLock<Vec<Metadata>>>),
    Map(Arc<RwLock<HashMap<String, Metadata>>>),
    Bytes(Arc<RwLock<Vec<u8>>>),
}
impl Metadata {
    pub fn new() -> Self {
        Metadata::Map(Arc::new(RwLock::new(HashMap::new())))
    }
    pub fn join(m1: Self, m2: Self) -> Self {
        let mut map1 = match m1 {
            Metadata::Map(map) => map.read().unwrap().clone(),
            _ => HashMap::new(),
        };
        let map2 = match m2 {
            Metadata::Map(map) => map.read().unwrap().clone(),
            _ => HashMap::new(),
        };
        map1.extend(map2);
        Metadata::Map(Arc::new(RwLock::new(map1)))
    }
    pub fn from_ser(s: impl Serialize) -> Result<Self> {
        from_value(to_value(s).context("Serialize error")?).context("Deserialize error")
    }

    pub fn as_str(&self) -> Option<Arc<RwLock<String>>> {
        if let Metadata::String(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }
    pub fn get(&self, k: impl AsRef<str>) -> Option<Metadata> {
        if let Metadata::Map(map) = self {
            map.read().unwrap().get(k.as_ref()).cloned()
        } else {
            None
        }
    }
}
impl From<String> for Metadata {
    fn from(value: String) -> Self {
        Metadata::String(Arc::new(RwLock::new(value)))
    }
}
impl From<&str> for Metadata {
    fn from(value: &str) -> Self {
        Metadata::String(Arc::new(RwLock::new(value.to_owned())))
    }
}
