//! Inline NOTA round-trip: a typed configuration record decoded
//! from argv that begins with `(`.

use nota_codec::{NotaEnum, NotaRecord};
use nota_config::{ConfigurationSource, impl_nota_only_configuration};

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
fn argv_split_across_tokens_decodes_into_typed_record() {
    let arguments = ["(SmallConfig", "hello", "High)"];
    let source = ConfigurationSource::from_args(arguments).unwrap();
    let configuration: SmallConfig = source.decode().unwrap();
    assert_eq!(configuration, SmallConfig { label: "hello".to_owned(), level: Level::High });
}

#[test]
fn argv_single_token_decodes_into_typed_record() {
    let source = ConfigurationSource::from_args(["(SmallConfig hello High)"]).unwrap();
    let configuration: SmallConfig = source.decode().unwrap();
    assert_eq!(configuration, SmallConfig { label: "hello".to_owned(), level: Level::High });
}
