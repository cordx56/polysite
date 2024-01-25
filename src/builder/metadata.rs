use anyhow::{anyhow, Context as _, Result};
use serde::Serialize;
use serde_json::{from_str, json, to_string, Number, Value};

pub const RULE_META: &str = "_rule";
pub const SOURCE_FILE_META: &str = "_source";
pub const TARGET_FILE_META: &str = "_target";
pub const PATH_META: &str = "_path";
pub const VERSION_META: &str = "_version";
pub const BODY_META: &str = "_body";

/// Use [`serde_json::Value`] as metadata because the [`serde::Serialize`] trait is not object
/// safe.
pub type Metadata = Value;

/// Trait to handle bytes data as metadata
pub trait MetadataAsBytes {
    fn as_bytes(&self) -> Result<Vec<u8>>;
    fn from_bytes(bytes: Vec<u8>) -> Self;
}
impl MetadataAsBytes for Metadata {
    fn as_bytes(&self) -> Result<Vec<u8>> {
        let array = self.as_array().ok_or(anyhow!("Not bytes data"))?;
        let data_size = array
            .get(0)
            .ok_or(anyhow!("Invalid bytes data"))?
            .as_u64()
            .ok_or(anyhow!("Invalid bytes data"))?;
        let byte_size = 8;
        let remove = byte_size - (data_size % byte_size);
        let (_, remain) = array.split_at(1);
        let mut res = Vec::new();
        for data in remain {
            let data = data.as_u64().ok_or(anyhow!("Invalid bytes data"))?;
            let bytes = data.to_be_bytes();
            res.extend_from_slice(bytes.as_slice());
        }
        for _ in 0..remove {
            res.pop();
        }
        Ok(res)
    }
    fn from_bytes(mut bytes: Vec<u8>) -> Self {
        let data_size = bytes.len();
        let byte_size = 8;
        let rem = data_size % byte_size;
        for _ in 0..(byte_size - rem) {
            bytes.push(0);
        }
        let data_size = Metadata::Number(Number::from(data_size));
        let mut array = vec![data_size];
        let mut data;
        let mut remain = bytes.as_slice();
        let len = remain.len() / 8;
        for _ in 0..len {
            (data, remain) = remain.split_at(byte_size);
            let num = u64::from_be_bytes(data.try_into().unwrap());
            let num = Number::from(num);
            let val = Metadata::Number(num);
            array.push(val);
        }
        let res = Metadata::Array(array);
        res
    }
}

/// Convert any [`Serialize`] object into
pub trait FromSerializable {
    fn from_serializable(data: impl Serialize) -> Result<Self>
    where
        Self: Sized;
}
impl FromSerializable for Metadata {
    /// Convert any serializable value into [`Metadata`]
    fn from_serializable(data: impl Serialize) -> Result<Self> {
        from_str(&to_string(&data).context("Serialize error")?).context("Deserialize error")
    }
}

pub fn new_object() -> Metadata {
    json!({})
}

pub fn join_metadata(m1: Metadata, m2: Metadata) -> Metadata {
    let mut m1 = if m1.is_object() { m1 } else { new_object() };
    let m2 = if m2.is_object() { m2 } else { new_object() };
    m1.as_object_mut()
        .unwrap()
        .extend(m2.as_object().unwrap().clone());
    m1
}
