//! Orderbook app-data wire routing shared across the request DTOs.
//!
//! The `(full, hash)` pair maps onto the services `OrderCreationAppData` forms
//! through a single canonical routing helper so the signed `OrderCreation`
//! payload and the `OrderQuoteRequest` quote payload can never silently diverge
//! on this security-relevant boundary.

use serde::{Deserialize, Deserializer, Serialize, ser::SerializeMap};

use cow_sdk_core::AppDataHash;

/// Canonical entry point for the orderbook app-data wire routing, shared by
/// [`OrderCreation`](super::OrderCreation) and [`QuoteAppData`]. Single source of
/// truth for the services `OrderCreationAppData` mapping.
///
/// | (full, hash)       | wire shape                                  | services variant |
/// | ------------------ | ------------------------------------------- | ---------------- |
/// | (None, None)       | (both omitted)                              | none             |
/// | (Some(f), None)    | `{"appData": f}`                            | `Full`           |
/// | (None, Some(h))    | `{"appData": "0x<h>"}` (hash under appData) | `Hash`           |
/// | (Some(f), Some(h)) | `{"appData": f, "appDataHash": "0x<h>"}`    | `Both`           |
pub(super) fn serialize_app_data_pair<M>(
    map: &mut M,
    app_data: Option<&str>,
    app_data_hash: Option<&AppDataHash>,
) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    match (app_data, app_data_hash) {
        (None, None) => {}
        (Some(full), None) => map.serialize_entry("appData", full)?,
        // services `Hash` form: the hash hex string lives under the `appData` key.
        (None, Some(hash)) => map.serialize_entry("appData", hash)?,
        (Some(full), Some(hash)) => {
            map.serialize_entry("appData", full)?;
            map.serialize_entry("appDataHash", hash)?;
        }
    }
    Ok(())
}

/// App-data on a quote request: the `(full document, hash)` pair.
///
/// This is a field pair rather than a `Hash`/`Full`/`Both` enum, matching how
/// the signed [`OrderCreation`](super::OrderCreation) payload models app-data,
/// and it is serialized through the shared app-data wire routing so
/// every combination — including hash-only — produces a wire shape the orderbook
/// accepts. In particular, a hash-only request serializes the hash under the
/// `appData` key (the services `Hash` form), never as an `appDataHash`-only body
/// that the orderbook rejects.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct QuoteAppData {
    /// Crate-internal: set through the `OrderQuoteRequest` builders; the public
    /// surface is the [`QuoteAppData::full`]/[`hash`](QuoteAppData::hash)/
    /// [`both`](QuoteAppData::both) constructors and the accessors.
    pub(crate) full: Option<String>,
    pub(crate) hash: Option<AppDataHash>,
}

impl QuoteAppData {
    /// App-data given as the full document (JSON) string.
    #[must_use]
    pub fn full(document: impl Into<String>) -> Self {
        Self {
            full: Some(document.into()),
            hash: None,
        }
    }

    /// App-data given as a hash only.
    #[must_use]
    pub const fn hash(hash: AppDataHash) -> Self {
        Self {
            full: None,
            hash: Some(hash),
        }
    }

    /// App-data given as the full document plus its expected hash.
    #[must_use]
    pub fn both(document: impl Into<String>, hash: AppDataHash) -> Self {
        Self {
            full: Some(document.into()),
            hash: Some(hash),
        }
    }

    /// Returns the full app-data document string, if present.
    #[must_use]
    pub fn full_app_data(&self) -> Option<&str> {
        self.full.as_deref()
    }

    /// Returns the explicit app-data hash, if present.
    #[must_use]
    pub const fn app_data_hash(&self) -> Option<AppDataHash> {
        self.hash
    }

    /// Returns the effective app-data hash: the explicit hash when present,
    /// otherwise the hash parsed from a hash-form `appData` string.
    #[must_use]
    pub fn resolved_hash(&self) -> Option<AppDataHash> {
        if let Some(hash) = self.hash {
            return Some(hash);
        }
        self.full
            .as_deref()
            .and_then(|value| AppDataHash::new(value).ok())
    }
}

impl Serialize for QuoteAppData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        serialize_app_data_pair(&mut map, self.full.as_deref(), self.hash.as_ref())?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for QuoteAppData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Wire {
            app_data: Option<String>,
            app_data_hash: Option<AppDataHash>,
        }

        let wire = Wire::deserialize(deserializer)?;
        // Mirror the orderbook's own app-data parsing: a lone `appData` that is
        // itself a 32-byte hash is the `Hash` form, so it resolves into the hash
        // slot rather than being mistaken for a full document. This keeps the
        // wire <-> struct mapping round-trip stable for every form, and keeps
        // the [`QuoteAppData::full_app_data`] / [`QuoteAppData::app_data_hash`]
        // accessors honest for a decoded request.
        Ok(match (wire.app_data, wire.app_data_hash) {
            (Some(app_data), None) => AppDataHash::new(&app_data).map_or_else(
                |_| Self {
                    full: Some(app_data),
                    hash: None,
                },
                |hash| Self {
                    full: None,
                    hash: Some(hash),
                },
            ),
            (full, hash) => Self { full, hash },
        })
    }
}
