//! Serde helpers that round-trip [`bytes::Bytes`] fields through the `CoW`
//! Protocol `0x`-prefixed hexadecimal wire format.
//!
//! Zero-copy byte buffers benefit every encoding pipeline that clones the same
//! encoded payload across multiple settlement candidates, while the wire format
//! remains the canonical hex string accepted by downstream consumers.

use bytes::Bytes;
use serde::{Deserialize, Deserializer, Serializer, de};

/// Serde helpers for required hex-prefixed byte fields.
pub(crate) mod hex_bytes {
    use super::{Bytes, Deserialize, Deserializer, Serializer, de, decode_hex};

    pub(crate) fn serialize<S: Serializer>(
        bytes: &Bytes,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let hex = format!("0x{}", hex::encode(bytes));
        serializer.serialize_str(&hex)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Bytes, D::Error> {
        let s = String::deserialize(deserializer)?;
        decode_hex(&s).map_err(de::Error::custom)
    }
}

/// Serde helpers for optional hex-prefixed byte fields.
pub(crate) mod option_hex_bytes {
    use super::{Bytes, Deserialize, Deserializer, Serializer, de, decode_hex};

    #[allow(
        clippy::ref_option,
        reason = "serde's #[serde(with = ...)] serialize hook requires the &Option<T> signature exactly"
    )]
    pub(crate) fn serialize<S: Serializer>(
        value: &Option<Bytes>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(bytes) => {
                let hex = format!("0x{}", hex::encode(bytes));
                serializer.serialize_str(&hex)
            }
            None => serializer.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Bytes>, D::Error> {
        let opt = Option::<String>::deserialize(deserializer)?;
        opt.map(|s| decode_hex(&s).map_err(de::Error::custom))
            .transpose()
    }
}

fn decode_hex(input: &str) -> Result<Bytes, String> {
    let stripped = input
        .strip_prefix("0x")
        .ok_or_else(|| "hex byte field must start with 0x".to_owned())?;
    hex::decode(stripped)
        .map_err(|err| err.to_string())
        .map(Bytes::from)
}
