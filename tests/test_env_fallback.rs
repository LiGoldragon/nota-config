//! Test-shim env-var fallback: opt-in only via
//! `from_args_with_env_fallback`. The argv path always wins when
//! an argument is present.

use std::ffi::OsString;

use nota_codec::NotaRecord;
use nota_config::{ConfigurationSource, impl_nota_only_configuration};

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct TinyConfig {
    pub label: String,
}

impl_nota_only_configuration!(TinyConfig);

#[test]
fn env_fallback_engages_when_argv_is_empty() {
    let arguments: [&str; 0] = [];
    let env_value: Option<OsString> = Some(OsString::from("(TinyConfig hello)"));
    let source = ConfigurationSource::from_args_with_env_fallback(arguments, env_value).unwrap();
    let configuration: TinyConfig = source.decode().unwrap();
    assert_eq!(configuration, TinyConfig { label: "hello".to_owned() });
}

#[test]
fn env_fallback_ignored_when_argv_has_argument() {
    let env_value: Option<OsString> = Some(OsString::from("(TinyConfig fromenv)"));
    let source = ConfigurationSource::from_args_with_env_fallback(["(TinyConfig fromargv)"], env_value).unwrap();
    let configuration: TinyConfig = source.decode().unwrap();
    assert_eq!(configuration, TinyConfig { label: "fromargv".to_owned() });
}

#[test]
fn env_fallback_missing_returns_typed_error() {
    let arguments: [&str; 0] = [];
    let err = ConfigurationSource::from_args_with_env_fallback(arguments, None).unwrap_err();
    assert!(matches!(err, nota_config::Error::MissingArgument), "got {err:?}");
}
