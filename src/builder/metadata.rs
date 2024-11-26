use crate::{error::Error, *};
use serde::{ser::SerializeMap, Serialize};
use serde_json::{json, to_value, Map, Number};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard};
use tracing_error::SpanTrace;

pub use serde_json::Value;

pub const RULE_META: &str = "_rule";
pub const SOURCE_FILE_META: &str = "_source";
pub const TARGET_FILE_META: &str = "_target";
pub const PATH_META: &str = "_path";
pub const VERSION_META: &str = "_version";
pub const BODY_META: &str = "_body";
pub const VERSIONS_META: &str = "_versions";

/// [`Metadata`] holds global and local metadata, which is represented as a [`Value`].
#[derive(Clone, Debug)]
pub struct Metadata {
    global: Arc<RwLock<Value>>,
    local: Value,
}

/// Read locked metadata, which contains locked global metadata.
#[derive(Clone, Debug)]
pub struct ReadLockedMetadata<'a> {
    metadata: &'a Metadata,
    locked: Arc<RwLockReadGuard<'a, Value>>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            global: Arc::new(RwLock::new(json!({
                VERSIONS_META: json!({}),
            }))),
            local: json!({}),
        }
    }
    pub async fn read_lock(&self) -> ReadLockedMetadata {
        ReadLockedMetadata {
            metadata: self,
            locked: Arc::new(self.global.read().await),
        }
    }
    pub fn local(&self) -> &Map<String, Value> {
        self.local.as_object().unwrap()
    }
    pub async fn global(&self) -> RwLockReadGuard<Map<String, Value>> {
        RwLockReadGuard::map(self.global.read().await, |v| v.as_object().unwrap())
    }
    pub async fn global_mut(&self) -> RwLockMappedWriteGuard<Map<String, Value>> {
        RwLockWriteGuard::map(self.global.write().await, |v| v.as_object_mut().unwrap())
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
    #[tracing::instrument(skip(ser))]
    pub fn to_value(ser: impl Serialize) -> Result<Value, Error> {
        to_value(ser).map_err(|serde_error| Error::SerdeJson {
            trace: SpanTrace::capture(),
            serde_error,
        })
    }
    pub fn merge(&mut self, other: Metadata) {
        merge_values(&mut self.local, other.local);
    }

    /// Get currently compiling [`Version`]
    pub fn version(&self) -> Option<Version> {
        self.local
            .get(VERSION_META)
            .map(|v| v.as_str().map(|v| v.into()))
            .flatten()
    }

    /// Get currently compiling rule name
    pub fn rule(&self) -> Option<String> {
        self.local
            .get(RULE_META)
            .map(|v| v.as_str().map(|v| v.to_owned()))
            .flatten()
    }

    /// Get currently compiling source file path
    pub fn source(&self) -> Option<PathBuf> {
        self.local
            .get(SOURCE_FILE_META)
            .map(|v| v.as_str().map(|v| PathBuf::from(v)))
            .flatten()
    }
    /// Get currently compiling target file path
    pub fn target(&self) -> Option<PathBuf> {
        self.local
            .get(TARGET_FILE_META)
            .map(|v| v.as_str().map(|v| PathBuf::from(v)))
            .flatten()
    }
    /// Get currently compiling URL path
    pub fn path(&self) -> Option<PathBuf> {
        self.local
            .get(PATH_META)
            .map(|v| v.as_str().map(|v| PathBuf::from(v)))
            .flatten()
    }
    /// Get currently compiling body [`Value`], which can be [`Vec<u8>`] or [`String`].
    pub fn body(&self) -> Option<&Value> {
        self.local.get(BODY_META)
    }
}
impl ReadLockedMetadata<'_> {
    pub fn get_version(&self, version: &Version) -> Option<HashMap<String, Metadata>> {
        self.locked
            .get(VERSIONS_META)
            .unwrap()
            .get(version.get())
            .map(|w| w.as_object())
            .flatten()
            .map(|v| {
                HashMap::from_iter(v.iter().map(|(path, w)| {
                    (
                        path.to_owned(),
                        Metadata {
                            global: self.metadata.global.clone(),
                            local: w.clone(),
                        },
                    )
                }))
            })
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
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
        for (k, v) in self.locked.as_object().unwrap().iter() {
            iter.insert(k, v);
        }
        for (k, v) in self.metadata.local.as_object().unwrap().iter() {
            iter.insert(k, v);
        }
        let mut map = serializer.serialize_map(Some(iter.len()))?;
        for (k, v) in iter.into_iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

/// Trait for reading and writing binary data for [`Metadata`].
pub trait BytesValue: Sized {
    fn from_ser(ser: impl Serialize) -> Result<Self, Error>;
    fn as_bytes(&self) -> Option<Vec<u8>>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl BytesValue for Value {
    #[tracing::instrument(skip(ser))]
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
