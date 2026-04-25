use std::{fmt, str::FromStr};

use cow_sdk_core::{Address, REDACTED_PLACEHOLDER, Redacted, ValidationReason};
use serde::de::{Deserializer, Error as _};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::AppDataError;

/// Default `appCode` value inserted by [`crate::generate_app_data_doc`].
pub const DEFAULT_APP_CODE: &str = "CoW Swap";
/// Default public IPFS gateway used for read operations.
pub const DEFAULT_IPFS_READ_URI: &str = "https://cloudflare-ipfs.com/ipfs";
/// Default Pinata base URI used for write operations.
pub const DEFAULT_IPFS_WRITE_URI: &str = "https://api.pinata.cloud";
/// Latest bundled app-data schema version.
pub const LATEST_APP_DATA_VERSION: &str = "1.14.0";
/// Alias for the latest bundled schema version.
pub const LATEST_SCHEMA_VERSION: &str = LATEST_APP_DATA_VERSION;
/// Latest supported quote metadata schema version.
pub const LATEST_QUOTE_METADATA_VERSION: &str = "1.1.0";
/// Latest supported referrer metadata schema version.
pub const LATEST_REFERRER_METADATA_VERSION: &str = "1.0.0";
/// Latest supported order-class metadata schema version.
pub const LATEST_ORDER_CLASS_METADATA_VERSION: &str = "0.3.0";
/// Latest supported UTM metadata schema version.
pub const LATEST_UTM_METADATA_VERSION: &str = "0.3.0";
/// Latest supported hooks metadata schema version.
pub const LATEST_HOOKS_METADATA_VERSION: &str = "0.2.0";
/// Latest supported signer metadata schema version.
pub const LATEST_SIGNER_METADATA_VERSION: &str = "0.1.0";
/// Latest supported widget metadata schema version.
pub const LATEST_WIDGET_METADATA_VERSION: &str = "0.1.0";
/// Latest supported partner-fee metadata schema version.
pub const LATEST_PARTNER_FEE_METADATA_VERSION: &str = "1.0.0";
/// Latest supported replaced-order metadata schema version.
pub const LATEST_REPLACED_ORDER_METADATA_VERSION: &str = "0.1.0";
/// Latest supported wrappers metadata schema version.
pub const LATEST_WRAPPERS_METADATA_VERSION: &str = "0.2.0";
/// Latest supported user-consents metadata schema version.
pub const LATEST_USER_CONSENTS_METADATA_VERSION: &str = "0.1.0";

/// Parsed app-data JSON document.
pub type AppDataDoc = Value;
/// Mutable JSON object used for nested `metadata` sections.
pub type MetadataMap = Map<String, Value>;

/// Semantic version for bundled app-data schemas.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(String);

impl SchemaVersion {
    /// Creates a validated schema version string.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidSchemaVersion`] when the value is not a
    /// three-part numeric semantic version.
    pub fn new(version: impl Into<String>) -> Result<Self, AppDataError> {
        let version = version.into();
        if is_semver(&version) {
            Ok(Self(version))
        } else {
            Err(AppDataError::InvalidSchemaVersion(version))
        }
    }

    /// Returns the latest bundled schema version.
    #[must_use]
    pub fn latest() -> Self {
        Self(LATEST_APP_DATA_VERSION.to_string())
    }

    /// Returns the inner schema version string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for SchemaVersion {
    type Err = AppDataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Inputs used to build an app-data document.
///
/// The two typed sub-metadata fields `signer` and `flashloan` sit alongside
/// the open-ended `metadata` slot. On the wire both typed fields land inside
/// the nested `metadata` object in their reviewed camelCase positions, and
/// any key other than `signer` or `flashloan` continues to flow through the
/// untyped [`MetadataMap`] slot so open-ended sub-objects remain supported.
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `metadata: MetadataMap` field is a `serde_json::Map<String, serde_json::Value>` alias, and `serde_json::Value` does not implement `Eq`"
)]
#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub struct AppDataParams {
    /// Optional application name written to the `appCode` field.
    pub app_code: Option<String>,
    /// Optional environment label for distinguishing deployments.
    pub environment: Option<String>,
    /// Declared signer carried as `metadata.signer` on the wire, read by the
    /// submission-seam validator that enforces the reviewed
    /// `AppdataFromMismatch` invariant.
    pub signer: Option<Address>,
    /// Typed flash-loan hint carried as `metadata.flashloan` on the wire.
    pub flashloan: Option<crate::metadata::FlashloanHints>,
    /// Arbitrary application metadata merged into the document. The two
    /// typed sub-metadata fields above leave this slot; every other
    /// open-ended sub-object continues to live inside the map.
    pub metadata: MetadataMap,
}

impl Serialize for AppDataParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut state = serializer.serialize_map(None)?;
        if let Some(app_code) = &self.app_code {
            state.serialize_entry("appCode", app_code)?;
        }
        if let Some(environment) = &self.environment {
            state.serialize_entry("environment", environment)?;
        }
        let metadata_value = self
            .metadata_wire_value()
            .map_err(serde::ser::Error::custom)?;
        state.serialize_entry("metadata", &metadata_value)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for AppDataParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(default, rename = "appCode")]
            app_code: Option<String>,
            #[serde(default)]
            environment: Option<String>,
            #[serde(default)]
            metadata: MetadataMap,
        }

        let Wire {
            app_code,
            environment,
            mut metadata,
        } = Wire::deserialize(deserializer)?;

        let signer = match metadata.remove("signer") {
            Some(value) => {
                Some(serde_json::from_value::<Address>(value).map_err(serde::de::Error::custom)?)
            }
            None => None,
        };
        let flashloan = match metadata.remove("flashloan") {
            Some(value) => Some(
                serde_json::from_value::<crate::metadata::FlashloanHints>(value)
                    .map_err(serde::de::Error::custom)?,
            ),
            None => None,
        };

        Ok(Self {
            app_code,
            environment,
            signer,
            flashloan,
            metadata,
        })
    }
}

impl AppDataParams {
    /// Creates app-data parameters with the current full field shape.
    #[must_use]
    pub const fn new(
        app_code: Option<String>,
        environment: Option<String>,
        signer: Option<Address>,
        flashloan: Option<crate::metadata::FlashloanHints>,
        metadata: MetadataMap,
    ) -> Self {
        Self {
            app_code,
            environment,
            signer,
            flashloan,
            metadata,
        }
    }

    /// Returns a copy with an explicit `appCode` value.
    #[must_use]
    pub fn with_app_code(mut self, app_code: impl Into<String>) -> Self {
        self.app_code = Some(app_code.into());
        self
    }

    /// Returns a copy with an explicit environment label.
    #[must_use]
    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    /// Returns a copy with a typed signer metadata value.
    #[must_use]
    pub fn with_signer(mut self, signer: Address) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Returns a copy with typed flash-loan hint metadata.
    #[must_use]
    pub fn with_flashloan(mut self, flashloan: crate::metadata::FlashloanHints) -> Self {
        self.flashloan = Some(flashloan);
        self
    }

    /// Returns a copy with explicit open-ended metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: MetadataMap) -> Self {
        self.metadata = metadata;
        self
    }

    /// Returns the canonical metadata [`Value`] merged from the typed
    /// sub-fields and the open-ended [`MetadataMap`] slot.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::Json`] when the typed `flashloan` sub-field
    /// fails to serialize — which cannot happen for values produced through
    /// the public constructors and is surfaced only for the defensive path.
    pub fn metadata_wire_value(&self) -> Result<Value, AppDataError> {
        let mut metadata = self.metadata.clone();
        if let Some(signer) = &self.signer {
            metadata.insert(
                "signer".to_owned(),
                Value::String(signer.as_str().to_owned()),
            );
        }
        if let Some(flashloan) = &self.flashloan {
            metadata.insert(
                "flashloan".to_owned(),
                serde_json::to_value(flashloan).map_err(AppDataError::from)?,
            );
        }
        Ok(Value::Object(metadata))
    }
}

/// Derived identifiers for a validated app-data document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AppDataInfo {
    /// CID representation of the document.
    pub cid: String,
    /// Serialized JSON content used to derive the digest.
    #[serde(rename = "appDataContent")]
    pub app_data_content: String,
    /// `0x`-prefixed app-data digest.
    #[serde(rename = "appDataHex")]
    pub app_data_hex: String,
}

impl AppDataInfo {
    /// Creates derived identifiers for a validated app-data document.
    #[must_use]
    pub fn new(
        cid: impl Into<String>,
        app_data_content: impl Into<String>,
        app_data_hex: impl Into<String>,
    ) -> Self {
        Self {
            cid: cid.into(),
            app_data_content: app_data_content.into(),
            app_data_hex: app_data_hex.into(),
        }
    }
}

/// Schema validation result returned by [`crate::validate_app_data_doc`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ValidationResult {
    /// Whether validation succeeded.
    pub success: bool,
    /// Rendered validation errors when `success` is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

impl ValidationResult {
    /// Creates a schema validation result.
    #[must_use]
    pub const fn new(success: bool, errors: Option<String>) -> Self {
        Self { success, errors }
    }
}

/// Typed partner-fee metadata accepted by app-data and trading helpers.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PartnerFee {
    /// Single fee policy object.
    Single(PartnerFeePolicy),
    /// Ordered fee policy list.
    Multiple(Vec<PartnerFeePolicy>),
}

impl PartnerFee {
    /// Returns the first supported volume-basis-point fee in this value, if one exists.
    #[must_use]
    pub fn volume_bps(&self) -> Option<u16> {
        match self {
            Self::Single(policy) => policy.volume_bps(),
            Self::Multiple(policies) => policies.iter().find_map(PartnerFeePolicy::volume_bps),
        }
    }

    /// Validates every policy carried by this payload against the published
    /// bounds for the partner-fee schema.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidPartnerFee`] on the first policy whose
    /// basis-point values fall outside the documented `[1..=9999]` /
    /// `[1..=100]` ranges, or whose recipient address is the zero address.
    pub fn validate(&self) -> Result<(), AppDataError> {
        match self {
            Self::Single(policy) => policy.validate(),
            Self::Multiple(policies) => policies.iter().try_for_each(PartnerFeePolicy::validate),
        }
    }

    /// Serializes this typed partner-fee payload into the app-data metadata shape.
    ///
    /// # Panics
    ///
    /// Panics only if the compile-time partner-fee schema types stop being
    /// serializable to JSON.
    #[must_use]
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).expect("partner-fee schema types must remain serializable")
    }

    /// Parses partner-fee metadata from an app-data metadata value.
    ///
    /// Accepts every in-scope shape — `Volume { volumeBps, recipient }`,
    /// `Surplus { surplusBps, maxVolumeBps, recipient }`,
    /// `PriceImprovement { priceImprovementBps, maxVolumeBps, recipient }`,
    /// arrays of the above — and the legacy `{ bps, recipient }` object which
    /// is promoted to a `Volume` policy for wire parity with the reviewed
    /// services parser.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::Json`] when the JSON value does not match any
    /// supported partner-fee schema shape. Bounds validation is not performed
    /// here — call [`PartnerFee::validate`] on the parsed value to enforce the
    /// documented basis-point ranges.
    pub fn from_value(value: Value) -> Result<Self, AppDataError> {
        serde_json::from_value(value).map_err(AppDataError::from)
    }
}

impl From<PartnerFeePolicy> for PartnerFee {
    fn from(value: PartnerFeePolicy) -> Self {
        Self::Single(value)
    }
}

impl From<Vec<PartnerFeePolicy>> for PartnerFee {
    fn from(value: Vec<PartnerFeePolicy>) -> Self {
        Self::Multiple(value)
    }
}

/// One typed partner-fee policy object.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum PartnerFeePolicy {
    /// Fee paid from traded volume.
    Volume {
        /// Fee paid in basis points of volume.
        #[serde(rename = "volumeBps")]
        volume_bps: u16,
        /// Recipient of the partner fee.
        recipient: Address,
    },
    /// Fee paid from surplus, capped by volume.
    Surplus {
        /// Fee paid in basis points of surplus.
        #[serde(rename = "surplusBps")]
        surplus_bps: u16,
        /// Maximum fee paid in basis points of volume.
        #[serde(rename = "maxVolumeBps")]
        max_volume_bps: u16,
        /// Recipient of the partner fee.
        recipient: Address,
    },
    /// Fee paid from price improvement, capped by volume.
    PriceImprovement {
        /// Fee paid in basis points of price improvement.
        #[serde(rename = "priceImprovementBps")]
        price_improvement_bps: u16,
        /// Maximum fee paid in basis points of volume.
        #[serde(rename = "maxVolumeBps")]
        max_volume_bps: u16,
        /// Recipient of the partner fee.
        recipient: Address,
    },
}

impl PartnerFeePolicy {
    /// Creates a volume-based partner-fee policy after validating the
    /// supplied basis-point value and recipient against the published
    /// partner-fee bounds.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidPartnerFee`] when `volume_bps` falls
    /// outside the documented `[1..=100]` range, or when `recipient` is the
    /// zero address.
    pub fn volume(volume_bps: u16, recipient: Address) -> Result<Self, AppDataError> {
        let policy = Self::Volume {
            volume_bps,
            recipient,
        };
        policy.validate()?;
        Ok(policy)
    }

    /// Creates a surplus-based partner-fee policy after validating the
    /// supplied basis-point values and recipient against the published
    /// partner-fee bounds.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidPartnerFee`] when `surplus_bps` falls
    /// outside `[1..=9999]`, when `max_volume_bps` falls outside `[1..=100]`,
    /// or when `recipient` is the zero address.
    pub fn surplus(
        surplus_bps: u16,
        max_volume_bps: u16,
        recipient: Address,
    ) -> Result<Self, AppDataError> {
        let policy = Self::Surplus {
            surplus_bps,
            max_volume_bps,
            recipient,
        };
        policy.validate()?;
        Ok(policy)
    }

    /// Creates a price-improvement-based partner-fee policy after validating
    /// the supplied basis-point values and recipient against the published
    /// partner-fee bounds.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidPartnerFee`] when
    /// `price_improvement_bps` falls outside `[1..=9999]`, when
    /// `max_volume_bps` falls outside `[1..=100]`, or when `recipient` is the
    /// zero address.
    pub fn price_improvement(
        price_improvement_bps: u16,
        max_volume_bps: u16,
        recipient: Address,
    ) -> Result<Self, AppDataError> {
        let policy = Self::PriceImprovement {
            price_improvement_bps,
            max_volume_bps,
            recipient,
        };
        policy.validate()?;
        Ok(policy)
    }

    /// Returns the volume-basis-point fee when this policy uses the volume shape.
    #[must_use]
    pub const fn volume_bps(&self) -> Option<u16> {
        match self {
            Self::Volume { volume_bps, .. } => Some(*volume_bps),
            Self::Surplus { .. } | Self::PriceImprovement { .. } => None,
        }
    }

    /// Validates this policy against the published partner-fee schema bounds.
    ///
    /// The bounds the reviewed schema applies:
    ///
    /// * `volumeBps` — integer in `[1..=100]`
    /// * `surplusBps` — integer in `[1..=9999]`
    /// * `priceImprovementBps` — integer in `[1..=9999]`
    /// * `maxVolumeBps` — integer in `[1..=100]`
    /// * `recipient` — non-zero 20-byte address
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidPartnerFee`] on the first field that
    /// falls outside the documented bounds, or when `recipient` is the zero
    /// address.
    pub fn validate(&self) -> Result<(), AppDataError> {
        match self {
            Self::Volume {
                volume_bps,
                recipient,
            } => {
                validate_max_volume_bps("partnerFee.volumeBps", *volume_bps)?;
                validate_recipient("partnerFee.recipient", recipient)?;
            }
            Self::Surplus {
                surplus_bps,
                max_volume_bps,
                recipient,
            } => {
                validate_surplus_bps("partnerFee.surplusBps", *surplus_bps)?;
                validate_max_volume_bps("partnerFee.maxVolumeBps", *max_volume_bps)?;
                validate_recipient("partnerFee.recipient", recipient)?;
            }
            Self::PriceImprovement {
                price_improvement_bps,
                max_volume_bps,
                recipient,
            } => {
                validate_surplus_bps("partnerFee.priceImprovementBps", *price_improvement_bps)?;
                validate_max_volume_bps("partnerFee.maxVolumeBps", *max_volume_bps)?;
                validate_recipient("partnerFee.recipient", recipient)?;
            }
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for PartnerFeePolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Fields {
            #[serde(default, rename = "volumeBps")]
            volume_bps: Option<u16>,
            #[serde(default, rename = "surplusBps")]
            surplus_bps: Option<u16>,
            #[serde(default, rename = "priceImprovementBps")]
            price_improvement_bps: Option<u16>,
            #[serde(default, rename = "maxVolumeBps")]
            max_volume_bps: Option<u16>,
            #[serde(default)]
            bps: Option<u16>,
            recipient: Address,
        }

        let fields = Fields::deserialize(deserializer)?;
        match (
            fields.volume_bps,
            fields.surplus_bps,
            fields.price_improvement_bps,
            fields.max_volume_bps,
            fields.bps,
        ) {
            (Some(volume_bps), None, None, None, None) => Ok(Self::Volume {
                volume_bps,
                recipient: fields.recipient,
            }),
            (None, Some(surplus_bps), None, Some(max_volume_bps), None) => Ok(Self::Surplus {
                surplus_bps,
                max_volume_bps,
                recipient: fields.recipient,
            }),
            (None, None, Some(price_improvement_bps), Some(max_volume_bps), None) => {
                Ok(Self::PriceImprovement {
                    price_improvement_bps,
                    max_volume_bps,
                    recipient: fields.recipient,
                })
            }
            (None, None, None, None, Some(bps)) => Ok(Self::Volume {
                volume_bps: bps,
                recipient: fields.recipient,
            }),
            _ => Err(D::Error::custom("unknown partner fee policy format")),
        }
    }
}

const MAX_VOLUME_BPS: u16 = 100;
const MAX_SURPLUS_BPS: u16 = 9_999;

const fn validate_max_volume_bps(field: &'static str, value: u16) -> Result<(), AppDataError> {
    if value == 0 || value > MAX_VOLUME_BPS {
        return Err(AppDataError::InvalidPartnerFee {
            field,
            reason: ValidationReason::OutOfRange {
                details: "value must be an integer in the inclusive range [1, 100]",
            },
        });
    }
    Ok(())
}

const fn validate_surplus_bps(field: &'static str, value: u16) -> Result<(), AppDataError> {
    if value == 0 || value > MAX_SURPLUS_BPS {
        return Err(AppDataError::InvalidPartnerFee {
            field,
            reason: ValidationReason::OutOfRange {
                details: "value must be an integer in the inclusive range [1, 9999]",
            },
        });
    }
    Ok(())
}

fn validate_recipient(field: &'static str, recipient: &Address) -> Result<(), AppDataError> {
    if recipient == &address_zero() {
        return Err(AppDataError::InvalidPartnerFee {
            field,
            reason: ValidationReason::Precondition {
                details: "recipient must not be the zero address",
            },
        });
    }
    Ok(())
}

fn address_zero() -> Address {
    Address::from_bytes([0u8; 20])
}

/// IPFS configuration used by fetch and upload helpers.
#[derive(Clone, PartialEq, Eq, Deserialize, Default)]
pub struct IpfsConfig {
    /// Legacy shared base URI used when `read_uri` is absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Base URI used for Pinata-style write requests.
    #[serde(default, rename = "writeUri", skip_serializing_if = "Option::is_none")]
    pub write_uri: Option<String>,
    /// Base URI used for IPFS read requests.
    #[serde(default, rename = "readUri", skip_serializing_if = "Option::is_none")]
    pub read_uri: Option<String>,
    /// Pinata API key used by upload helpers.
    #[serde(
        default,
        rename = "pinataApiKey",
        skip_serializing_if = "Option::is_none"
    )]
    pub pinata_api_key: Option<Redacted<String>>,
    /// Pinata API secret used by upload helpers.
    #[serde(
        default,
        rename = "pinataApiSecret",
        skip_serializing_if = "Option::is_none"
    )]
    pub pinata_api_secret: Option<Redacted<String>>,
}

impl fmt::Debug for IpfsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpfsConfig")
            .field("uri", &self.uri)
            .field("write_uri", &self.write_uri)
            .field("read_uri", &self.read_uri)
            .field(
                "pinata_api_key",
                &self.pinata_api_key.as_ref().map(|_| REDACTED_PLACEHOLDER),
            )
            .field(
                "pinata_api_secret",
                &self
                    .pinata_api_secret
                    .as_ref()
                    .map(|_| REDACTED_PLACEHOLDER),
            )
            .finish()
    }
}

impl Serialize for IpfsConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("IpfsConfig", 5)?;

        if let Some(uri) = &self.uri {
            state.serialize_field("uri", uri)?;
        }
        if let Some(write_uri) = &self.write_uri {
            state.serialize_field("writeUri", write_uri)?;
        }
        if let Some(read_uri) = &self.read_uri {
            state.serialize_field("readUri", read_uri)?;
        }
        if self.pinata_api_key.is_some() {
            state.serialize_field("pinataApiKey", REDACTED_PLACEHOLDER)?;
        }
        if self.pinata_api_secret.is_some() {
            state.serialize_field("pinataApiSecret", REDACTED_PLACEHOLDER)?;
        }

        state.end()
    }
}

/// Raw HTTP response returned by app-data transport seams.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TransportResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response body text.
    pub body: String,
}

impl TransportResponse {
    /// Creates a raw HTTP response for app-data transport seams.
    #[must_use]
    pub fn new(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }
}

/// Result returned by Pinata upload helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IpfsUploadResult {
    /// `0x`-prefixed app-data digest derived from the returned CID.
    #[serde(rename = "appData")]
    pub app_data: String,
    /// CID returned by the upload backend.
    pub cid: String,
}

impl IpfsUploadResult {
    /// Creates a Pinata upload result.
    #[must_use]
    pub fn new(app_data: impl Into<String>, cid: impl Into<String>) -> Self {
        Self {
            app_data: app_data.into(),
            cid: cid.into(),
        }
    }
}

fn is_semver(version: &str) -> bool {
    let mut parts = version.split('.');
    let major = parts.next().is_some_and(is_non_empty_digits);
    let minor = parts.next().is_some_and(is_non_empty_digits);
    let patch = parts.next().is_some_and(is_non_empty_digits);
    major && minor && patch && parts.next().is_none()
}

fn is_non_empty_digits(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_latest_default_display_and_parse_stay_aligned() {
        let latest = SchemaVersion::latest();

        assert_eq!(latest.as_str(), LATEST_APP_DATA_VERSION);
        assert_eq!(latest.to_string(), LATEST_APP_DATA_VERSION);
        assert_eq!(
            LATEST_APP_DATA_VERSION.parse::<SchemaVersion>().unwrap(),
            latest
        );
    }

    #[test]
    fn schema_version_validation_accepts_triplets_and_rejects_non_semver_inputs() {
        for valid in ["0.1.0", "1.14.0", "999.0.42"] {
            assert!(is_semver(valid), "{valid}");
            assert_eq!(SchemaVersion::new(valid).unwrap().as_str(), valid);
        }

        for invalid in ["1.0", "1.0.0.1", "v1.0.0", "1.two.3", "", "1..3"] {
            assert!(!is_semver(invalid), "{invalid}");
            let error = SchemaVersion::new(invalid).unwrap_err();
            match error {
                AppDataError::InvalidSchemaVersion(ref message) => {
                    assert_eq!(message, invalid);
                }
                other => panic!("expected InvalidSchemaVersion, got {other:?}"),
            }
        }

        assert!(is_non_empty_digits("123456"));
        assert!(!is_non_empty_digits(""));
        assert!(!is_non_empty_digits("12a45"));
    }

    #[test]
    fn schema_version_from_str_fails_closed_for_invalid_inputs() {
        for invalid in ["1.0", "1.0.0.1", "v1.0.0", "1.two.3", "", "1..3"] {
            let error = invalid.parse::<SchemaVersion>().unwrap_err();
            match error {
                AppDataError::InvalidSchemaVersion(ref message) => {
                    assert_eq!(message, invalid);
                }
                other => panic!("expected InvalidSchemaVersion, got {other:?}"),
            }
        }
    }

    #[test]
    fn partner_fee_roundtrips_single_and_array_shapes_and_exposes_first_volume_fee() {
        let recipient = Address::new("0x1111111111111111111111111111111111111111")
            .expect("test recipient must be valid");
        let fee = PartnerFee::from(vec![
            PartnerFeePolicy::surplus(250, 100, recipient.clone())
                .expect("surplus policy must validate"),
            PartnerFeePolicy::volume(42, recipient.clone()).expect("volume policy must validate"),
        ]);

        let value = fee.to_value();
        let reparsed = PartnerFee::from_value(value.clone()).expect("typed partner fee re-parses");

        assert_eq!(
            value,
            serde_json::json!([
                {
                    "surplusBps": 250,
                    "maxVolumeBps": 100,
                    "recipient": recipient.as_str()
                },
                {
                    "volumeBps": 42,
                    "recipient": recipient.as_str()
                }
            ])
        );
        assert_eq!(reparsed, fee);
        assert_eq!(fee.volume_bps(), Some(42));
        assert_eq!(
            PartnerFee::from(
                PartnerFeePolicy::price_improvement(25, 100, recipient)
                    .expect("price-improvement policy must validate")
            )
            .volume_bps(),
            None
        );
    }

    #[test]
    fn ipfs_config_debug_and_serialize_redact_pinata_credentials() {
        let config = IpfsConfig {
            uri: Some("https://ipfs.example".to_owned()),
            write_uri: Some("https://pinata.example".to_owned()),
            read_uri: Some("https://read.example".to_owned()),
            pinata_api_key: Some("pinata-key".to_owned().into()),
            pinata_api_secret: Some("pinata-secret".to_owned().into()),
        };

        let debug = format!("{config:?}");
        let json = serde_json::to_value(&config).expect("ipfs config serializes");

        assert!(debug.contains("IpfsConfig"));
        assert!(debug.contains(REDACTED_PLACEHOLDER));
        assert!(!debug.contains("pinata-key"));
        assert!(!debug.contains("pinata-secret"));
        assert_eq!(
            json["pinataApiKey"],
            serde_json::json!(REDACTED_PLACEHOLDER)
        );
        assert_eq!(
            json["pinataApiSecret"],
            serde_json::json!(REDACTED_PLACEHOLDER)
        );
    }
}
