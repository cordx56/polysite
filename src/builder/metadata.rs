use crate::error::{here, Location};
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::{from_str, to_string, Value};
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
