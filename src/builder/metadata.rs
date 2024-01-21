use anyhow::{Context as _, Result};
use serde::Serialize;
use serde_json::{from_str, json, to_string, Value};

/// Use serde_json::Value as metadata
/// because the serde::Serialize trait is not object safe
pub type Metadata = Value;

pub fn to_metadata(data: impl Serialize) -> Result<Metadata> {
    from_str(&to_string(&data).context("Serialize error")?).context("Deserialize error")
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
