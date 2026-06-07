//! The [`ConfigurationRecord`] trait — what every typed
//! configuration type implements.

use nota_next::NotaDecode;

use crate::error::Result;

/// A typed configuration record decoded from either NOTA text
/// (inline argv or file) or an rkyv archive.
///
/// Every configuration record invokes exactly **one** of:
///
/// - [`crate::impl_nota_only_configuration!`] — installs an impl
///   whose `from_rkyv_bytes` returns [`crate::Error::RkyvNotSupported`].
/// - [`crate::impl_rkyv_configuration!`] — installs an impl that
///   decodes through `rkyv::from_bytes`.
///
/// No blanket impl is provided. See `ARCHITECTURE.md`
/// §"Two macros, no blanket impl" for the rationale.
pub trait ConfigurationRecord: NotaDecode + Sized {
    /// Decode this record from rkyv bytes. Types installed via
    /// `impl_nota_only_configuration!` return
    /// [`crate::Error::RkyvNotSupported`]; types installed via
    /// `impl_rkyv_configuration!` decode through `rkyv::from_bytes`.
    fn from_rkyv_bytes(bytes: &[u8]) -> Result<Self>;
}
