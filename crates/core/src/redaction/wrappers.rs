use std::{collections::BTreeMap, fmt, iter::FromIterator};

use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::REDACTED_PLACEHOLDER;

/// Newtype wrapper that redacts the inner value in every non-explicit representation.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    /// Wraps a value in the redacted newtype.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Consumes the wrapper and returns the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Returns a borrow of the inner value for deliberate access.
    #[inline]
    pub const fn as_inner(&self) -> &T {
        &self.0
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_PLACEHOLDER)
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_PLACEHOLDER)
    }
}

impl<T> Serialize for Redacted<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(REDACTED_PLACEHOLDER)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Redacted<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self)
    }
}

impl<T> From<T> for Redacted<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// Redacting wrapper for chain or environment keyed URL maps.
///
/// Serialization is diagnostic-only: keys are preserved so logs and snapshots
/// can show which routes are configured, but every URL value serializes as
/// [`REDACTED_PLACEHOLDER`]. Deserializing expects the raw URL map shape so
/// persisted configuration can still be loaded before public output redacts it.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RedactedUrlMap<K>(BTreeMap<K, String>);

impl<K> RedactedUrlMap<K> {
    /// Creates an empty redacted URL map.
    #[must_use]
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Returns the raw map for deliberate dispatch-time use.
    #[must_use]
    pub const fn as_inner(&self) -> &BTreeMap<K, String> {
        &self.0
    }
}

impl<K: Ord> RedactedUrlMap<K> {
    /// Inserts a raw URL value under `key`.
    pub fn insert(&mut self, key: K, value: impl Into<String>) -> Option<String> {
        self.0.insert(key, value.into())
    }

    /// Returns the raw URL value for `key`, if one is configured.
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&String> {
        self.0.get(key)
    }
}

impl<K> Default for RedactedUrlMap<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> From<BTreeMap<K, String>> for RedactedUrlMap<K> {
    fn from(value: BTreeMap<K, String>) -> Self {
        Self(value)
    }
}

impl<K: Ord> FromIterator<(K, String)> for RedactedUrlMap<K> {
    fn from_iter<T: IntoIterator<Item = (K, String)>>(iter: T) -> Self {
        Self(BTreeMap::from_iter(iter))
    }
}

impl<K: Ord + fmt::Debug> fmt::Debug for RedactedUrlMap<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.keys().map(|key| (key, REDACTED_PLACEHOLDER)))
            .finish()
    }
}

impl<K: Ord + fmt::Debug> fmt::Display for RedactedUrlMap<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<K> Serialize for RedactedUrlMap<K>
where
    K: Ord + Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for key in self.0.keys() {
            map.serialize_entry(key, REDACTED_PLACEHOLDER)?;
        }
        map.end()
    }
}

impl<'de, K> Deserialize<'de> for RedactedUrlMap<K>
where
    K: Ord + Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        BTreeMap::<K, String>::deserialize(deserializer).map(Self)
    }
}

/// Redacting wrapper for URL maps where `None` marks unsupported chains.
///
/// Serialization is diagnostic-only: keys and `None` support markers are
/// preserved, while every configured URL value serializes as
/// [`REDACTED_PLACEHOLDER`]. Deserializing expects the raw optional URL map
/// shape so persisted configuration can still be loaded before public output
/// redacts it.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RedactedOptionalUrlMap<K>(BTreeMap<K, Option<String>>);

impl<K> RedactedOptionalUrlMap<K> {
    /// Creates an empty redacted optional URL map.
    #[must_use]
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Returns the raw map for deliberate dispatch-time use.
    #[must_use]
    pub const fn as_inner(&self) -> &BTreeMap<K, Option<String>> {
        &self.0
    }
}

impl<K: Ord> RedactedOptionalUrlMap<K> {
    /// Inserts an optional raw URL value under `key`.
    pub fn insert(&mut self, key: K, value: Option<String>) -> Option<Option<String>> {
        self.0.insert(key, value)
    }

    /// Returns the optional raw URL value for `key`, if the key is present.
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&Option<String>> {
        self.0.get(key)
    }
}

impl<K> Default for RedactedOptionalUrlMap<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> From<BTreeMap<K, Option<String>>> for RedactedOptionalUrlMap<K> {
    fn from(value: BTreeMap<K, Option<String>>) -> Self {
        Self(value)
    }
}

impl<K: Ord> FromIterator<(K, Option<String>)> for RedactedOptionalUrlMap<K> {
    fn from_iter<T: IntoIterator<Item = (K, Option<String>)>>(iter: T) -> Self {
        Self(BTreeMap::from_iter(iter))
    }
}

impl<K: Ord + fmt::Debug> fmt::Debug for RedactedOptionalUrlMap<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(key, value)| {
                (
                    key,
                    value
                        .as_ref()
                        .map(|_| REDACTED_PLACEHOLDER)
                        .as_ref()
                        .copied(),
                )
            }))
            .finish()
    }
}

impl<K: Ord + fmt::Debug> fmt::Display for RedactedOptionalUrlMap<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<K> Serialize for RedactedOptionalUrlMap<K>
where
    K: Ord + Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            match value {
                Some(_) => map.serialize_entry(key, REDACTED_PLACEHOLDER)?,
                None => map.serialize_entry(key, &Option::<&str>::None)?,
            }
        }
        map.end()
    }
}

impl<'de, K> Deserialize<'de> for RedactedOptionalUrlMap<K>
where
    K: Ord + Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        BTreeMap::<K, Option<String>>::deserialize(deserializer).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_and_display_emit_the_redacted_placeholder() {
        let secret = Redacted::new("super-secret-value".to_owned());
        assert_eq!(format!("{secret:?}"), REDACTED_PLACEHOLDER);
        assert_eq!(format!("{secret}"), REDACTED_PLACEHOLDER);
    }

    #[test]
    fn serialize_emits_the_redacted_placeholder_regardless_of_inner_value() {
        let secret = Redacted::new("another-secret".to_owned());
        let json = serde_json::to_string(&secret).unwrap();
        assert_eq!(json, format!("\"{REDACTED_PLACEHOLDER}\""));
    }

    #[test]
    fn into_inner_escapes_the_redaction_for_deliberate_use() {
        let secret = Redacted::new("explicit-unwrap".to_owned());
        assert_eq!(secret.into_inner(), "explicit-unwrap");
    }

    #[test]
    fn url_map_debug_display_and_serialize_redact_values_but_keep_keys() {
        let urls = RedactedUrlMap::from(BTreeMap::from([(
            1u64,
            "https://user:pass@example.test/path?key=secret".to_owned(),
        )]));

        let debug = format!("{urls:?}");
        let display = urls.to_string();
        let json = serde_json::to_value(&urls).unwrap();

        assert!(debug.contains(REDACTED_PLACEHOLDER));
        assert!(display.contains(REDACTED_PLACEHOLDER));
        assert_eq!(json["1"], REDACTED_PLACEHOLDER);
        for rendered in [debug, display, json.to_string()] {
            assert!(!rendered.contains("user:pass"));
            assert!(!rendered.contains("secret"));
            assert!(!rendered.contains("example.test"));
        }
        assert_eq!(
            urls.as_inner().get(&1).map(String::as_str),
            Some("https://user:pass@example.test/path?key=secret")
        );
    }

    #[test]
    fn optional_url_map_redacts_configured_values_and_preserves_none_markers() {
        let urls = RedactedOptionalUrlMap::from(BTreeMap::from([
            (
                1u64,
                Some("https://mainnet.example.test/path?token=secret".to_owned()),
            ),
            (100u64, None),
        ]));

        let debug = format!("{urls:#?}");
        let json = serde_json::to_value(&urls).unwrap();

        assert!(debug.contains(REDACTED_PLACEHOLDER));
        assert!(!debug.contains("token=secret"));
        assert_eq!(json["1"], REDACTED_PLACEHOLDER);
        assert_eq!(json["100"], serde_json::Value::Null);
        assert_eq!(
            urls.as_inner().get(&1).and_then(Option::as_deref),
            Some("https://mainnet.example.test/path?token=secret")
        );
    }
}
