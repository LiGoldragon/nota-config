//! Inline NOTA round-trip: a typed configuration record decoded
//! from argv that begins with `(`.

use nota_codec::{NotaEnum, NotaRecord};
use nota_config::{ConfigurationSource, Error, impl_nota_only_configuration};

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Low,
    High,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct SmallConfig {
    pub label: String,
    pub level: Level,
}

impl_nota_only_configuration!(SmallConfig);

#[test]
fn argv_split_across_tokens_is_rejected() {
    let arguments = ["([we're", "ready]", "High)"];
    let err = ConfigurationSource::from_args(arguments).unwrap_err();
    assert!(matches!(err, Error::MultipleArguments(3)), "got {err:?}");
}

#[test]
fn nota_argument_accepts_apostrophe_text_without_quote_delimiters() {
    let argument = "([we're ready] High)";
    assert!(!argument.contains('"'));

    let source = ConfigurationSource::from_args([argument]).unwrap();
    let configuration: SmallConfig = source.decode().unwrap();
    assert_eq!(configuration, SmallConfig { label: "we're ready".to_owned(), level: Level::High });
}
