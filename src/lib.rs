//! `nota-config` — typed configuration input for Persona-stack binaries.
//!
//! See `README.md` for the high-level shape and `ARCHITECTURE.md`
//! for the per-module roles, the detection algorithm, and the
//! two-macro pattern. The originating design is
//! `reports/designer/183-typed-configuration-input-pattern.md` in
//! the primary workspace.

pub mod configuration;
pub mod error;
pub mod source;

pub use configuration::ConfigurationRecord;
pub use error::{Error, Result};
pub use source::ConfigurationSource;

/// Install a [`ConfigurationRecord`] impl that rejects rkyv input.
/// Use for configuration types that travel only as NOTA.
///
/// ```ignore
/// use nota_next::{NotaDecode, NotaEncode};
/// use nota_config::impl_nota_only_configuration;
///
/// #[derive(NotaEncode, NotaDecode, Debug, Clone, PartialEq)]
/// pub struct SimpleConfig {
///     pub label: String,
/// }
///
/// impl_nota_only_configuration!(SimpleConfig);
/// ```
#[macro_export]
macro_rules! impl_nota_only_configuration {
    ($t:ty) => {
        impl $crate::ConfigurationRecord for $t {
            fn from_rkyv_bytes(_bytes: &[u8]) -> $crate::Result<Self> {
                Err($crate::Error::RkyvNotSupported(std::any::type_name::<$t>()))
            }
        }
    };
}

/// Install a [`ConfigurationRecord`] impl that decodes rkyv input
/// through `rkyv::from_bytes`. Use for configuration types that
/// also derive `Archive + RkyvSerialize + RkyvDeserialize`.
///
/// ```ignore
/// use nota_next::{NotaDecode, NotaEncode};
/// use nota_config::impl_rkyv_configuration;
/// use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
///
/// #[derive(NotaEncode, NotaDecode, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq)]
/// pub struct DualConfig {
///     pub label: String,
///     pub port: u64,
/// }
///
/// impl_rkyv_configuration!(DualConfig);
/// ```
#[macro_export]
macro_rules! impl_rkyv_configuration {
    ($t:ty) => {
        impl $crate::ConfigurationRecord for $t {
            fn from_rkyv_bytes(bytes: &[u8]) -> $crate::Result<Self> {
                rkyv::from_bytes::<$t, rkyv::rancor::Error>(bytes).map_err(|err| $crate::Error::Rkyv(err.to_string()))
            }
        }
    };
}
