use crate::error::Error;
use serde::{ser::SerializeMap, Serialize};
use serde_json::{json, to_value, Map, Number};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard};
use tracing_error::SpanTrace;

pub use serde_json::Value;

pub const RULE_META: &str = "_rule";
pub const SOURCE_FILE_META: &str = "_source";
pub const TARGET_FILE_META: &str = "_target";
pub const PATH_META: &str = "_path";
pub const VERSION_META: &str = "_version";
pub const BODY_META: &str = "_body";

#[derive(Clone, Debug)]
pub struct Metadata<G = Arc<RwLock<Value>>, L = Value> {
    global: G,
    local: L,
}

pub type ReadLockedMetadata<'a> = Metadata<Arc<RwLockReadGuard<'a, Value>>, &'a Value>;

impl Metadata {
    pub fn new() -> Self {
        Self {
            global: Arc::new(RwLock::new(json!({}))),
            local: json!({}),
        }
    }
    pub async fn read_lock(&self) -> ReadLockedMetadata {
        Metadata {
            global: Arc::new(self.global.read().await),
            local: &self.local,
        }
    }
    pub fn local(&self) -> &Map<String, Value> {
        self.local.as_object().unwrap()
    }
    pub async fn get(&self, key: &str) -> Option<Value> {
        if let Some(local) = self.local.get(key) {
            Some(local.clone())
        } else {
            self.global.read().await.get(key).cloned()
        }
    }
    pub async fn insert_global(&self, key: String, metadata: Value) {
        self.global
            .write()
            .await
            .as_object_mut()
            .unwrap()
            .insert(key, metadata);
    }
    pub fn insert_local(&mut self, key: String, metadata: Value) {
        self.local.as_object_mut().unwrap().insert(key, metadata);
    }
    pub fn to_value(ser: impl Serialize) -> Result<Value, Error> {
        to_value(ser).map_err(|serde_error| Error::SerdeJson {
            trace: SpanTrace::capture(),
            serde_error,
        })
    }
    pub fn merge(&mut self, other: Metadata) {
        merge_values(&mut self.local, other.local);
    }
}

pub fn merge_values(one: &mut Value, other: Value) {
    match (one, other) {
        (Value::Object(map), Value::Object(other)) => {
            for (key, val) in other.into_iter() {
                if let Some(left) = map.get_mut(&key) {
                    if left.is_object() && val.is_object() {
                        merge_values(left, val);
                        continue;
                    } else if left.is_array() && val.is_array() {
                        merge_values(left, val);
                        continue;
                    }
                }
                map.insert(key, val);
            }
        }
        (Value::Array(arr), Value::Array(other)) => {
            arr.extend(other);
        }
        (a, b) => {
            *a = b;
        }
    }
}

impl Serialize for ReadLockedMetadata<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut iter = HashMap::new();
        for (k, v) in self.global.as_object().unwrap().iter() {
            iter.insert(k, v);
        }
        for (k, v) in self.local.as_object().unwrap().iter() {
            iter.insert(k, v);
        }
        let mut map = serializer.serialize_map(Some(iter.len()))?;
        for (k, v) in iter.into_iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

//pub type Metadata = serde_json::Value;
pub trait BytesValue: Sized {
    fn from_ser(ser: impl Serialize) -> Result<Self, Error>;
    fn as_bytes(&self) -> Option<Vec<u8>>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl BytesValue for Value {
    fn from_ser(ser: impl Serialize) -> Result<Self, Error> {
        serde_json::to_value(ser).map_err(|serde_error| Error::SerdeJson {
            trace: SpanTrace::capture(),
            serde_error,
        })
    }

    fn as_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Value::Array(array) => {
                let data_size = match array.get(0).map(|v| v.as_u64()).flatten() {
                    Some(v) => v,
                    None => return None,
                };
                let byte_size = 8;
                let remove = (data_size - (byte_size - (data_size % byte_size))) as usize;
                let remain = &array[1..remove];
                let mut res = Vec::new();
                for data in remain {
                    let data = match data.as_u64() {
                        Some(v) => v,
                        None => return None,
                    };
                    let bytes = data.to_be_bytes();
                    res.extend_from_slice(bytes.as_slice());
                }
                Some(res)
            }
            Value::String(string) => Some(string.as_bytes().to_vec()),
            _ => None,
        }
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        let data_size = bytes.len();
        let byte_size = 8;
        let data_size = Value::Number(Number::from(data_size));
        let mut array = vec![data_size];
        let len = bytes.len() / byte_size + 1;
        for i in 0..len {
            let slice = &bytes[i * byte_size..];
            let mut data = [0; 8];
            for (i, n) in (0..).zip(&slice[..byte_size]) {
                data[i] = *n;
            }
            let num = u64::from_be_bytes(data);
            let num = Number::from(num);
            let val = Value::Number(num);
            array.push(val);
        }
        let res = Value::Array(array);
        res
    }
}
