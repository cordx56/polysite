use thiserror::Error;

#[derive(Error, Debug)]
#[error("file {}, line {}", .file, .line)]
pub struct Location {
    pub file: &'static str,
    pub line: u32,
}

macro_rules! here {
    () => {
        Location {
            file: file!(),
            line: line!(),
        }
    };
}
pub(crate) use here;
