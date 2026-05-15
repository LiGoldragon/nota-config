//! [`ConfigurationSource`] — the three argv-derived transports
//! (inline NOTA, NOTA file, rkyv file) and the dispatch from
//! each into a typed configuration record.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use nota_codec::{Decoder, NotaDecode};

use crate::configuration::ConfigurationRecord;
use crate::error::{Error, Result};

/// One argv-derived configuration source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationSource {
    /// One or more argv tokens that, when joined with single spaces,
    /// form a complete NOTA record. The first token begins with `(`.
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
        let collected: Vec<OsString> = args.into_iter().map(|argument| argument.as_ref().to_owned()).collect();
        let first = collected.first().ok_or(Error::MissingArgument)?;
        Self::dispatch(first, &collected)
    }

    /// Pick the Nth positional configuration argument from argv (0-indexed,
    /// skipping `argv[0]`). Useful when a binary takes more than one
    /// configuration path on argv (e.g. daemon config + spawn envelope).
    pub fn from_argv_nth(n: usize) -> Result<Self> {
        let collected: Vec<OsString> = std::env::args_os().skip(1).collect();
        let argument = collected.get(n).ok_or(Error::MissingArgument)?;
        Self::dispatch_single(argument)
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
                let text =
                    std::fs::read_to_string(&path).map_err(|source| Error::NotaFileRead { path: path.clone(), source })?;
                Self::decode_nota_text(&text)
            }
            ConfigurationSource::RkyvFile(path) => {
                let bytes = std::fs::read(&path).map_err(|source| Error::RkyvFileRead { path: path.clone(), source })?;
                C::from_rkyv_bytes(&bytes)
            }
        }
    }

    fn decode_nota_text<C: NotaDecode>(text: &str) -> Result<C> {
        let mut decoder = Decoder::new(text);
        let record = C::decode(&mut decoder)?;
        Ok(record)
    }

    fn dispatch(first: &OsString, all: &[OsString]) -> Result<Self> {
        let lossy = first.to_string_lossy();
        if lossy.starts_with('(') {
            let mut parts: Vec<String> = Vec::with_capacity(all.len());
            for argument in all {
                let text = argument.to_str().ok_or(Error::NonUtf8InlineArgument)?;
                parts.push(text.to_owned());
            }
            return Ok(ConfigurationSource::InlineNota(parts.join(" ")));
        }
        Self::dispatch_single(first)
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
