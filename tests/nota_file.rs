//! NOTA-file round-trip: a typed configuration record decoded
//! from a `.nota` file referenced on argv.

use std::io::Write;

use nota_config::{ConfigurationSource, impl_nota_only_configuration};
use nota_next::{NotaDecode, NotaEncode};

#[derive(NotaEncode, NotaDecode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Low,
    High,
}

#[derive(NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq)]
pub struct SmallConfig {
    pub label: String,
    pub level: Level,
}

impl_nota_only_configuration!(SmallConfig);

#[test]
fn nota_file_decodes_into_typed_record() {
    let mut temporary = tempfile::Builder::new().suffix(".nota").tempfile().unwrap();
    temporary.write_all(b"([we're ready] Low)").unwrap();
    let path = temporary.path().to_owned();
    let source = ConfigurationSource::from_args([path.as_os_str()]).unwrap();
    assert!(matches!(source, ConfigurationSource::NotaFile(_)));
    let configuration: SmallConfig = source.decode().unwrap();
    assert_eq!(configuration, SmallConfig { label: "we're ready".to_owned(), level: Level::Low });
}

#[test]
fn nota_only_type_rejects_rkyv_file() {
    let temporary = tempfile::Builder::new().suffix(".rkyv").tempfile().unwrap();
    let path = temporary.path().to_owned();
    let source = ConfigurationSource::from_args([path.as_os_str()]).unwrap();
    let err = source.decode::<SmallConfig>().unwrap_err();
    assert!(matches!(err, nota_config::Error::RkyvNotSupported(_)), "got {err:?}");
}

#[test]
fn nota_file_missing_returns_typed_error() {
    let source = ConfigurationSource::NotaFile("/nonexistent/path/config.nota".into());
    let err = source.decode::<SmallConfig>().unwrap_err();
    assert!(matches!(err, nota_config::Error::NotaFileRead { .. }), "got {err:?}");
}
