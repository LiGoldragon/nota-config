//! Detection tests — the `from_args` algorithm picks the right
//! `ConfigurationSource` variant for each argv shape, and returns
//! typed errors when the shape is unrecognised.

use nota_config::{ConfigurationSource, Error};

#[test]
fn inline_nota_single_arg_starts_with_paren() {
    let source = ConfigurationSource::from_args(["(SmallConfig label High)"]).unwrap();
    assert_eq!(source, ConfigurationSource::InlineNota("(SmallConfig label High)".to_owned()));
}

#[test]
fn multiple_args_are_rejected_instead_of_joined() {
    let err = ConfigurationSource::from_args(["(SmallConfig", "label", "High)"]).unwrap_err();
    assert!(matches!(err, Error::MultipleArguments(3)), "got {err:?}");
}

#[test]
fn nota_file_extension_is_recognised() {
    let source = ConfigurationSource::from_args(["/path/to/config.nota"]).unwrap();
    assert_eq!(source, ConfigurationSource::NotaFile("/path/to/config.nota".into()));
}

#[test]
fn rkyv_file_extension_is_recognised() {
    let source = ConfigurationSource::from_args(["/path/to/config.rkyv"]).unwrap();
    assert_eq!(source, ConfigurationSource::RkyvFile("/path/to/config.rkyv".into()));
}

#[test]
fn missing_argument_returns_typed_error() {
    let arguments: [&str; 0] = [];
    let err = ConfigurationSource::from_args(arguments).unwrap_err();
    assert!(matches!(err, Error::MissingArgument), "got {err:?}");
}

#[test]
fn unsupported_argument_index_returns_typed_error() {
    let err = ConfigurationSource::from_argv_nth(1).unwrap_err();
    assert!(matches!(err, Error::UnsupportedArgumentIndex(1)), "got {err:?}");
}

#[test]
fn unknown_extension_returns_typed_error() {
    let err = ConfigurationSource::from_args(["/path/to/config.yaml"]).unwrap_err();
    assert!(matches!(err, Error::UnknownExtension(_, _)), "got {err:?}");
}

#[test]
fn extension_required_when_path_has_none() {
    let err = ConfigurationSource::from_args(["/path/to/config"]).unwrap_err();
    assert!(matches!(err, Error::ExtensionRequired(_)), "got {err:?}");
}
