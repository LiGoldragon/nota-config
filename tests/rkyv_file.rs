//! rkyv-file round-trip: a dual-derived configuration record
//! decoded from a `.rkyv` archive file. Confirms the
//! `impl_rkyv_configuration!` macro wires the codec correctly,
//! and that the same dual-derived type still decodes inline NOTA.

use std::io::Write;

use nota_codec::NotaRecord;
use nota_config::{ConfigurationSource, impl_rkyv_configuration};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(NotaRecord, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct DualConfig {
    pub name: String,
    pub port: u64,
}

impl_rkyv_configuration!(DualConfig);

#[test]
fn rkyv_file_decodes_into_typed_record() {
    let original = DualConfig { name: "frontend".to_owned(), port: 8080 };
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&original).unwrap();
    let mut temporary = tempfile::Builder::new().suffix(".rkyv").tempfile().unwrap();
    temporary.write_all(&bytes).unwrap();
    let path = temporary.path().to_owned();
    let source = ConfigurationSource::from_args([path.as_os_str()]).unwrap();
    assert!(matches!(source, ConfigurationSource::RkyvFile(_)));
    let recovered: DualConfig = source.decode().unwrap();
    assert_eq!(recovered, original);
}

#[test]
fn dual_type_also_decodes_inline_nota() {
    let arguments = ["(DualConfig frontend 8080)"];
    let source = ConfigurationSource::from_args(arguments).unwrap();
    let configuration: DualConfig = source.decode().unwrap();
    assert_eq!(configuration, DualConfig { name: "frontend".to_owned(), port: 8080 });
}

#[test]
fn truncated_rkyv_file_returns_typed_error() {
    // A 1-byte file is too short to hold a `DualConfig` archive
    // (which contains at minimum an ArchivedString relative pointer
    // and a u64). `bytecheck` rejects the buffer at validation.
    let mut temporary = tempfile::Builder::new().suffix(".rkyv").tempfile().unwrap();
    temporary.write_all(b"x").unwrap();
    let path = temporary.path().to_owned();
    let source = ConfigurationSource::from_args([path.as_os_str()]).unwrap();
    let err = source.decode::<DualConfig>().unwrap_err();
    assert!(matches!(err, nota_config::Error::Rkyv(_)), "got {err:?}");
}
