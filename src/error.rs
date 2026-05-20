//! Crate-wide [`Error`] enum and `Result` alias.

use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no configuration argument supplied on argv")]
    MissingArgument,

    #[error("expected exactly one configuration argument on argv, got {0}")]
    MultipleArguments(usize),

    #[error("configuration argument index {0} is unsupported; component binaries accept exactly one argument")]
    UnsupportedArgumentIndex(usize),

    #[error("configuration file path {0:?} ended with unknown extension {1:?}; expected .nota or .rkyv")]
    UnknownExtension(PathBuf, String),

    #[error("configuration file path {0:?} has no extension; expected .nota or .rkyv")]
    ExtensionRequired(PathBuf),

    #[error("inline NOTA argument is not valid UTF-8")]
    NonUtf8InlineArgument,

    #[error("NOTA file read failed: {path:?}: {source}")]
    NotaFileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("rkyv file read failed: {path:?}: {source}")]
    RkyvFileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("NOTA decode failed: {0}")]
    Nota(#[from] nota_codec::Error),

    #[error("rkyv decode failed: {0}")]
    Rkyv(String),

    #[error("configuration type `{0}` does not support rkyv input")]
    RkyvNotSupported(&'static str),
}
