//! [`ConfigurationSource`] — the three argv-derived transports
//! (inline NOTA, NOTA file, rkyv file) and the dispatch from
//! each into a typed configuration record.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use nota_next::{NotaDecode, NotaSource};

use crate::configuration::ConfigurationRecord;
use crate::error::{Error, Result};

/// One argv-derived configuration source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationSource {
    /// One argv token containing a complete NOTA record. The token
    /// begins with `(`.
    InlineNota(String),

    /// A path ending in `.nota` — the file body is a single NOTA record.
    NotaFile(PathBuf),

    /// A path ending in `.rkyv` — the file body is an rkyv archive.
    RkyvFile(PathBuf),
}

impl ConfigurationSource {
    /// Parse process argv (skipping `argv[0]`) into one source variant.
    pub fn from_argv() -> Result<Self> {
        Self::from_args(std::env::args_os().skip(1))
    }

    /// Parse a positional slice of `OsStr`-like values into one source variant.
    /// The testable counterpart of [`from_argv`].
    ///
    /// [`from_argv`]: ConfigurationSource::from_argv
    pub fn from_args<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut collected = args.into_iter();
        let first = collected.next().ok_or(Error::MissingArgument)?.as_ref().to_owned();
        let extra_count = collected.count();
        if extra_count > 0 {
            return Err(Error::MultipleArguments(extra_count + 1));
        }
        Self::dispatch_single(&first)
    }

    /// Compatibility alias for old callers that passed `0` to select
    /// the only supported configuration argument.
    ///
    /// Component binaries accept exactly one argument; any index other
    /// than `0` is rejected.
    pub fn from_argv_nth(n: usize) -> Result<Self> {
        if n != 0 {
            return Err(Error::UnsupportedArgumentIndex(n));
        }
        Self::from_argv()
    }

    /// **Test-shim only.** Falls back to the named environment variable
    /// when argv has no configuration argument. The method is
    /// `#[doc(hidden)]` and named so reviewers can grep for
    /// `with_test_env_fallback` to flag production binaries that use it.
    ///
    /// Production binaries call [`from_argv`] instead.
    ///
    /// [`from_argv`]: ConfigurationSource::from_argv
    #[doc(hidden)]
    pub fn from_argv_with_test_env_fallback(env_var_name: &str) -> Result<Self> {
        let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
        Self::from_args_with_env_fallback(arguments, std::env::var_os(env_var_name))
    }

    /// The testable kernel of [`from_argv_with_test_env_fallback`].
    /// Accepts both argv and the env-var value as parameters so tests
    /// do not need to mutate process environment.
    ///
    /// [`from_argv_with_test_env_fallback`]: ConfigurationSource::from_argv_with_test_env_fallback
    pub fn from_args_with_env_fallback<I, S>(arguments: I, env_value: Option<OsString>) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let collected: Vec<OsString> = arguments.into_iter().map(|argument| argument.as_ref().to_owned()).collect();
        match Self::from_args(&collected) {
            Ok(source) => Ok(source),
            Err(Error::MissingArgument) => match env_value {
                Some(value) => Self::dispatch_single(&value),
                None => Err(Error::MissingArgument),
            },
            Err(other) => Err(other),
        }
    }

    /// Decode this source into a typed configuration record.
    pub fn decode<C: ConfigurationRecord>(self) -> Result<C> {
        match self {
            ConfigurationSource::InlineNota(text) => Self::decode_nota_text(&text),
            ConfigurationSource::NotaFile(path) => {
                let text = std::fs::read_to_string(&path)
                    .map_err(|source| Error::NotaFileRead { path: path.clone(), source })?;
                Self::decode_nota_text(&text)
            }
            ConfigurationSource::RkyvFile(path) => {
                let bytes =
                    std::fs::read(&path).map_err(|source| Error::RkyvFileRead { path: path.clone(), source })?;
                C::from_rkyv_bytes(&bytes)
            }
        }
    }

    fn decode_nota_text<C: NotaDecode>(text: &str) -> Result<C> {
        NotaSource::new(text).parse().map_err(Error::Nota)
    }

    fn dispatch_single(argument: &OsString) -> Result<Self> {
        let lossy = argument.to_string_lossy();
        if lossy.starts_with('(') {
            let text = argument.to_str().ok_or(Error::NonUtf8InlineArgument)?;
            return Ok(ConfigurationSource::InlineNota(text.to_owned()));
        }
        let path = PathBuf::from(argument);
        let extension = path.extension().and_then(OsStr::to_str).map(str::to_owned);
        match extension {
            Some(extension) if extension == "nota" => Ok(ConfigurationSource::NotaFile(path)),
            Some(extension) if extension == "rkyv" => Ok(ConfigurationSource::RkyvFile(path)),
            Some(extension) => Err(Error::UnknownExtension(path, extension)),
            None => Err(Error::ExtensionRequired(path)),
        }
    }
}
