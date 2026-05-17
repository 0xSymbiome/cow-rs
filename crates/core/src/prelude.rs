//! Convenience prelude bringing the identity extension traits into
//! scope.
//!
//! The traits in this prelude
//! ([`AddressExt`](crate::types::identity_ext::AddressExt),
//! [`Hash32Ext`](crate::types::identity_ext::Hash32Ext),
//! [`HexDataExt`](crate::types::identity_ext::HexDataExt),
//! [`OrderUidExt`](crate::types::identity_ext::OrderUidExt)) expose the
//! cow-side identity accessors (`new`, `from_bytes`, `as_str`, etc.) on
//! the canonical [`alloy_primitives`] types. The prelude is the
//! forward-compatible foundation for the staged collapse of the cow
//! identity newtypes onto `alloy_primitives` per ADR 0052: once a
//! future stage retires the cow newtypes in favour of alloy type
//! aliases, callsites that already brought the prelude into scope
//! continue to resolve `Address::new(value)` style constructors against
//! the trait methods exposed here.
//!
//! Today the cow newtypes still keep their own inherent methods; the
//! prelude is therefore additive and does not change any existing
//! resolution path.

pub use crate::types::identity_ext::{AddressExt, Hash32Ext, HexDataExt, OrderUidExt};
