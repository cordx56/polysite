use super::metadata::*;
use crate::*;
use std::fs;
use std::path::PathBuf;
use tracing_error::SpanTrace;

/// [`Version`] represents the compilation file version. Once a source file has been built, any
/// subsequent builds of the same source path with the same version will be skipped.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Version(String);
impl Version {
    pub fn get(&self) -> &str {
        &self.0
    }
}
impl Default for Version {
    fn default() -> Self {
        Self("default".to_string())
    }
}
impl<S: AsRef<str>> From<S> for Version {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_owned())
    }
}

#[derive(Clone)]
/// [`Context`] holds [`Metadata`] and [`Config`].
/// The context is passed to [`Compiler::next_step`], and the modified context is returned.
pub struct Context {
    meta: Metadata,
    config: Config,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            meta: Metadata::new(),
            config,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    /// Get currently compiling [`Version`]
    pub async fn version(&self) -> Option<Version> {
        self.meta
            .get(VERSION_META)
            .await
            .map(|v| v.as_str().map(|v| v.into()))
            .flatten()
    }

    /// Get currently compiling rule name
    pub async fn rule(&self) -> Option<String> {
        self.meta
            .get(RULE_META)
            .await
            .map(|v| v.as_str().map(|v| v.to_owned()))
            .flatten()
    }

    /// Get currently compiling source file path
    pub async fn source(&self) -> Option<PathBuf> {
        self.meta
            .get(SOURCE_FILE_META)
            .await
            .map(|v| v.as_str().map(|v| PathBuf::from(v)))
            .flatten()
    }
    /// Get currently compiling target file path
    pub async fn target(&self) -> Option<PathBuf> {
        self.meta
            .get(TARGET_FILE_META)
            .await
            .map(|v| v.as_str().map(|v| PathBuf::from(v)))
            .flatten()
    }
    /// Get currently compiling URL path
    pub async fn path(&self) -> Option<PathBuf> {
        self.meta
            .get(PATH_META)
            .await
            .map(|v| v.as_str().map(|v| PathBuf::from(v)))
            .flatten()
    }
    /// Get currently compiling body [`Value`], which can be [`Vec<u8>`] or [`String`].
    pub async fn body(&self) -> Option<Value> {
        self.meta.get(BODY_META).await
    }
    /// Get [`Config`]
    pub fn config(&self) -> Config {
        self.config.clone()
    }

    /// Get source file body as bytes
    #[tracing::instrument(skip(self))]
    pub async fn source_body(&self) -> Result<Vec<u8>, Error> {
        let file = self.source().await.ok_or(Error::InvalidMetadata {
            trace: SpanTrace::capture(),
        })?;
        fs::read(&file).map_err(|io_error| Error::FileIo {
            trace: SpanTrace::capture(),
            io_error,
        })
    }
    /// Get source file string
    #[tracing::instrument(skip(self))]
    pub async fn source_string(&self) -> Result<String, Error> {
        String::from_utf8(self.source_body().await?).map_err(|_| Error::InvalidMetadata {
            trace: SpanTrace::capture(),
        })
    }
    /// Create target file's parent directory
    #[tracing::instrument(skip(self))]
    pub async fn create_target_parent_dir(&self) -> Result<PathBuf, Error> {
        if let Some(target) = self.target().await {
            let dir = target.parent().unwrap();
            fs::create_dir_all(dir).map_err(|io_error| Error::FileIo {
                trace: SpanTrace::capture(),
                io_error,
            })?;
            Ok(target)
        } else {
            Err(Error::InvalidMetadata {
                trace: SpanTrace::capture(),
            })
        }
    }
    /// Open target file to write
    #[tracing::instrument(skip(self))]
    pub async fn open_target(&self) -> Result<fs::File, Error> {
        let target = self.create_target_parent_dir().await?;
        fs::File::create(&target).map_err(|io_error| Error::FileIo {
            trace: SpanTrace::capture(),
            io_error,
        })
    }
}
