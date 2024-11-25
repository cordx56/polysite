use std::{error, fmt, io};
use tracing_error::SpanTrace;

#[derive(Debug)]
pub enum Error {
    InvalidMetadata {
        trace: SpanTrace,
    },
    InvalidRule {
        trace: SpanTrace,
    },
    SerdeJson {
        trace: SpanTrace,
        serde_error: serde_json::Error,
    },
    FileIo {
        trace: SpanTrace,
        io_error: io::Error,
    },
    User {
        trace: SpanTrace,
        user_error: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl Error {
    pub fn user_error(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::User {
            trace: SpanTrace::capture(),
            user_error: Box::new(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidMetadata { trace } => {
                writeln!(f, "invalid metadata:")?;
                trace.fmt(f)?;
                Ok(())
            }
            Error::InvalidRule { trace } => {
                writeln!(f, "invalid rule:")?;
                trace.fmt(f)?;
                Ok(())
            }
            Error::SerdeJson { trace, serde_error } => {
                writeln!(f, "serde JSON failed:")?;
                trace.fmt(f)?;
                writeln!(f, "error cause:")?;
                writeln!(f, "{:?}", serde_error)?;
                Ok(())
            }
            Error::FileIo { trace, io_error } => {
                writeln!(f, "file IO failed:")?;
                trace.fmt(f)?;
                writeln!(f, "error cause:")?;
                writeln!(f, "{:?}", io_error)?;
                Ok(())
            }
            Error::User { trace, user_error } => {
                writeln!(f, "user error:")?;
                trace.fmt(f)?;
                writeln!(f, "error cause:")?;
                writeln!(f, "{:?}", user_error)?;
                Ok(())
            }
        }
    }
}

impl error::Error for Error {}
