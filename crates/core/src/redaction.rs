//! Typed redaction wrapper for secret-bearing configuration fields.
//!
//! `Redacted<T>` and the URL-map wrappers keep the inner value available for
//! deliberate use while preventing accidental leakage through
//! [`std::fmt::Debug`], [`std::fmt::Display`], and [`serde::Serialize`].
//! Every redacted value representation emits the literal string `"[redacted]"`
//! regardless of the wrapped payload. [`Redacted::into_inner`],
//! [`Redacted::as_inner`], [`RedactedUrlMap::as_inner`], and
//! [`RedactedOptionalUrlMap::as_inner`] surface the underlying value when the
//! caller has explicit intent.

use std::{collections::BTreeMap, fmt, iter::FromIterator};

use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The placeholder emitted in every redacted representation.
pub const REDACTED_PLACEHOLDER: &str = "[redacted]";
/// Maximum number of sanitized response-body bytes retained before appending
/// [`RESPONSE_BODY_TRUNCATION_MARKER`].
pub const REDACTED_RESPONSE_BODY_MAX_BYTES: usize = 256;
/// Marker appended when [`redact_response_body`] truncates a sanitized body.
pub const RESPONSE_BODY_TRUNCATION_MARKER: &str = "...[truncated]";

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

    /// Returns a mutable borrow of the inner value for deliberate mutation.
    #[inline]
    pub const fn as_inner_mut(&mut self) -> &mut T {
        &mut self.0
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

/// Strips credential-shaped tokens from a partner response body and bounds the
/// retained diagnostic text.
///
/// Redaction runs before truncation so credentials after the byte cap cannot be
/// preserved by a too-early slice. The scanner is dependency-free and covers
/// common header, URL-query, JSON-string, and JWT-shaped credential echoes.
#[must_use]
pub fn redact_response_body(input: &str) -> String {
    truncate_sanitized_body(&strip_credential_tokens(input))
}

fn truncate_sanitized_body(input: &str) -> String {
    if input.len() <= REDACTED_RESPONSE_BODY_MAX_BYTES {
        return input.to_owned();
    }

    let mut boundary = REDACTED_RESPONSE_BODY_MAX_BYTES;
    while !input.is_char_boundary(boundary) {
        boundary -= 1;
    }

    let mut output = input[..boundary].to_owned();
    output.push_str(RESPONSE_BODY_TRUNCATION_MARKER);
    output
}

fn strip_credential_tokens(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut offset = 0;

    while offset < input.len() {
        if let Some(redaction) = url_userinfo_span(input, offset) {
            output.push_str(&input[offset..redaction.value_start]);
            output.push_str(REDACTED_PLACEHOLDER);
            offset = redaction.value_end;
            continue;
        }

        if let Some(end) = jwt_token_end(input, offset) {
            output.push_str(REDACTED_PLACEHOLDER);
            offset = end;
            continue;
        }

        if let Some(redaction) = credential_value_span(input, offset) {
            output.push_str(&input[offset..redaction.value_start]);
            output.push_str(REDACTED_PLACEHOLDER);
            offset = redaction.value_end;
            continue;
        }

        let next = input[offset..]
            .chars()
            .next()
            .expect("offset is always within the string");
        output.push(next);
        offset += next.len_utf8();
    }

    output
}

struct ValueRedaction {
    value_start: usize,
    value_end: usize,
}

fn url_userinfo_span(input: &str, offset: usize) -> Option<ValueRedaction> {
    if offset > 0 && is_url_scheme_char(input.as_bytes()[offset - 1]) {
        return None;
    }

    let scheme_end = input[offset..].find("://")? + offset;
    if scheme_end == offset || !input[offset..scheme_end].bytes().all(is_url_scheme_char) {
        return None;
    }

    let authority_start = scheme_end + 3;
    let mut authority_end = authority_start;
    while let Some(byte) = input.as_bytes().get(authority_end).copied() {
        if byte.is_ascii_whitespace() || matches!(byte, b'/' | b'?' | b'#' | b'"' | b'\'') {
            break;
        }
        authority_end += 1;
    }

    let at = input[authority_start..authority_end].find('@')? + authority_start;
    (at > authority_start).then_some(ValueRedaction {
        value_start: authority_start,
        value_end: at,
    })
}

fn credential_value_span(input: &str, offset: usize) -> Option<ValueRedaction> {
    let parsed = parse_key(input, offset)?;
    if !is_credential_key(parsed.key) {
        return None;
    }

    let delimiter = skip_ascii_whitespace(input, parsed.after_key);
    let delimiter_byte = *input.as_bytes().get(delimiter)?;
    if !matches!(delimiter_byte, b':' | b'=') {
        return None;
    }

    let value_prefix = skip_ascii_whitespace(input, delimiter + 1);
    let value_start = if normalized_key(parsed.key) == "authorization" {
        skip_authorization_scheme(input, value_prefix)
    } else {
        value_prefix
    };
    let (value_start, value_end) = value_span(input, value_start)?;

    Some(ValueRedaction {
        value_start,
        value_end,
    })
}

struct ParsedKey<'a> {
    key: &'a str,
    after_key: usize,
}

fn parse_key(input: &str, offset: usize) -> Option<ParsedKey<'_>> {
    let first = *input.as_bytes().get(offset)?;
    if matches!(first, b'"' | b'\'') {
        let quote = first;
        let key_start = offset + 1;
        let key_end = find_unescaped_byte(input, key_start, quote)?;
        return Some(ParsedKey {
            key: &input[key_start..key_end],
            after_key: key_end + 1,
        });
    }

    if offset > 0 && is_key_char(input.as_bytes()[offset - 1]) {
        return None;
    }

    let mut end = offset;
    while let Some(byte) = input.as_bytes().get(end).copied() {
        if !is_key_char(byte) {
            break;
        }
        end += 1;
    }

    if end == offset {
        return None;
    }

    Some(ParsedKey {
        key: &input[offset..end],
        after_key: end,
    })
}

fn is_credential_key(key: &str) -> bool {
    let normalized = normalized_key(key);
    normalized == "authorization"
        || normalized == "apikey"
        || normalized == "xapikey"
        || normalized == "token"
        || normalized == "secret"
        || normalized.contains("apikey")
        || normalized.contains("token")
        || normalized.contains("secret")
}

fn normalized_key(key: &str) -> String {
    key.bytes()
        .filter(|byte| !matches!(byte, b'-' | b'_'))
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect()
}

fn value_span(input: &str, offset: usize) -> Option<(usize, usize)> {
    let quote = input.as_bytes().get(offset).copied();
    if matches!(quote, Some(b'"' | b'\'')) {
        let value_start = offset + 1;
        let value_end = find_unescaped_byte(input, value_start, quote?)?;
        return Some((value_start, value_end));
    }

    let mut end = offset;
    while let Some(byte) = input.as_bytes().get(end).copied() {
        if !is_credential_value_char(byte) {
            break;
        }
        end += 1;
    }

    (end > offset).then_some((offset, end))
}

fn skip_authorization_scheme(input: &str, offset: usize) -> usize {
    let Some(after_bearer) = advance_ascii_case_insensitive(input, offset, "bearer") else {
        return offset;
    };

    let after_spaces = skip_ascii_whitespace(input, after_bearer);
    if after_spaces == after_bearer {
        offset
    } else {
        after_spaces
    }
}

fn jwt_token_end(input: &str, offset: usize) -> Option<usize> {
    if offset > 0 && is_credential_value_char(input.as_bytes()[offset - 1]) {
        return None;
    }
    if !input[offset..].starts_with("eyJ") {
        return None;
    }

    let mut end = offset;
    while let Some(byte) = input.as_bytes().get(end).copied() {
        if !is_credential_value_char(byte) {
            break;
        }
        end += 1;
    }

    (end - offset >= 23).then_some(end)
}

fn find_unescaped_byte(input: &str, offset: usize, target: u8) -> Option<usize> {
    let mut current = offset;
    let mut escaped = false;

    while let Some(byte) = input.as_bytes().get(current).copied() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == target {
            return Some(current);
        }
        current += 1;
    }

    None
}

fn advance_ascii_case_insensitive(input: &str, offset: usize, needle: &str) -> Option<usize> {
    let end = offset.checked_add(needle.len())?;
    let candidate = input.get(offset..end)?;
    candidate.eq_ignore_ascii_case(needle).then_some(end)
}

fn skip_ascii_whitespace(input: &str, mut offset: usize) -> usize {
    while let Some(byte) = input.as_bytes().get(offset).copied() {
        if !byte.is_ascii_whitespace() {
            break;
        }
        offset += 1;
    }
    offset
}

const fn is_key_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}

const fn is_url_scheme_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-')
}

const fn is_credential_value_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
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

impl<K> PartialEq<BTreeMap<K, String>> for RedactedUrlMap<K>
where
    K: Ord + PartialEq,
{
    fn eq(&self, other: &BTreeMap<K, String>) -> bool {
        self.0 == *other
    }
}

impl<K> PartialEq<RedactedUrlMap<K>> for BTreeMap<K, String>
where
    K: Ord + PartialEq,
{
    fn eq(&self, other: &RedactedUrlMap<K>) -> bool {
        *self == other.0
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

impl<K> PartialEq<BTreeMap<K, Option<String>>> for RedactedOptionalUrlMap<K>
where
    K: Ord + PartialEq,
{
    fn eq(&self, other: &BTreeMap<K, Option<String>>) -> bool {
        self.0 == *other
    }
}

impl<K> PartialEq<RedactedOptionalUrlMap<K>> for BTreeMap<K, Option<String>>
where
    K: Ord + PartialEq,
{
    fn eq(&self, other: &RedactedOptionalUrlMap<K>) -> bool {
        *self == other.0
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
