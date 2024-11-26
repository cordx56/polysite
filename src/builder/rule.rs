use super::compile::CompileRunner;
use crate::*;
use glob::glob;
use std::path::PathBuf;
use tracing_error::SpanTrace;

/// The [`Rule`] is used to define the rule name, source files, [`Version`], and the [`Compiler`] used for building.
/// The results of the compilation are saved in the [`Metadata`], using the rule's name as the key.
pub struct Rule {
    name: String,
    globs: Option<Vec<String>>,
    creates: Option<Vec<String>>,
    compiler: Box<dyn Compiler>,
    version: Version,
}

impl Rule {
    pub fn new(name: impl AsRef<str>, compiler: impl Compiler + 'static) -> Self {
        let name = name.as_ref().to_owned();
        Rule {
            name,
            globs: None,
            creates: None,
            compiler: Box::new(compiler),
            version: Version::default(),
        }
    }

    /// Get this rule's name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Set a list of glob patterns to compile.
    pub fn set_globs(mut self, globs: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.globs = Some(globs.into_iter().map(|s| s.as_ref().to_owned()).collect());
        self
    }
    /// Set a list of source file names to create.
    pub fn set_create(mut self, create: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.creates = Some(create.into_iter().map(|s| s.as_ref().to_owned()).collect());
        self
    }

    /// Set compilation [`Version`]
    pub fn set_version(mut self, version: impl Into<Version>) -> Self {
        self.version = version.into();
        self
    }

    /// Do compilation task
    #[tracing::instrument(skip(self, ctx))]
    pub(crate) async fn compile(self, ctx: Context) -> Result<Context, Error> {
        let src_dir = ctx.config().source_dir();
        let paths: Vec<_> = match (&self.globs, &self.creates) {
            (Some(globs), None) => {
                let globs = globs
                    .iter()
                    .map(|g| src_dir.join(PathBuf::from(g)).to_string_lossy().to_string());
                let paths = globs
                    .map(|g| glob(&g))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| Error::InvalidRule {
                        trace: SpanTrace::capture(),
                    })?
                    .into_iter()
                    .map(|p| p.collect::<Result<Vec<_>, _>>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| Error::InvalidRule {
                        trace: SpanTrace::capture(),
                    })?
                    .into_iter()
                    .flatten()
                    .filter(|p| p.is_file())
                    .collect();
                paths
            }
            (None, Some(paths)) => {
                let paths = paths
                    .iter()
                    .map(|p| src_dir.join(PathBuf::from(p)))
                    .collect();
                paths
            }
            _ => {
                return Err(Error::InvalidRule {
                    trace: SpanTrace::capture(),
                })
            }
        };
        let version_files: Option<Vec<_>> = ctx
            .metadata()
            .read_lock()
            .await
            .get_version(&self.version)
            .map(|v| v.into_iter().map(|(source, _)| source).collect());
        let paths: Vec<_> = paths
            .into_iter()
            .filter(|p| {
                version_files
                    .as_ref()
                    .map(|v| {
                        v.iter()
                            .filter(|w| &**w == &*p.to_string_lossy())
                            .next()
                            .is_none()
                    })
                    .unwrap_or(true)
            })
            .collect();

        let name = self.get_name().to_owned();
        let src_dir = ctx.config().source_dir();
        let target_dir = ctx.config().target_dir();
        let runner = CompileRunner::new(name, self.version, ctx, self.compiler);
        for source in paths {
            let path = source.strip_prefix(&src_dir).unwrap_or(&source);
            let target = target_dir.join(path);
            let path = PathBuf::from("/").join(path);
            runner.spawn_compile(source, target, path).await;
        }
        let ctx = runner.join().await?;
        Ok(ctx)
    }
}
