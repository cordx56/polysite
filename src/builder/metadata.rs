use crate::error::{here, Location};
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::{from_str, json, to_string, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Serialize error: {} on {}", .1, .0)]
    SerializeError(Location, serde_json::Error),
    #[error("Deserialize error: {} on {}", .1, .0)]
    DeserializeError(Location, serde_json::Error),
}

/// Use serde_json::Value as metadata
/// because the serde::Serialize trait is not object safe
pub type Metadata = Value;

pub fn to_metadata(data: impl Serialize) -> Result<Metadata> {
    from_str(&to_string(&data).map_err(|e| anyhow!(MetadataError::SerializeError(here!(), e)))?)
        .map_err(|e| anyhow!(MetadataError::DeserializeError(here!(), e)))
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
