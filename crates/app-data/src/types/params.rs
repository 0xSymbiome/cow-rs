use cow_sdk_core::{Address, AppCode};
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    AppDataDoc, AppDataError, AppDataValidated, info::get_app_data_info,
    schema::generate_app_data_doc,
};

use super::MetadataMap;

/// Inputs used to build an app-data document.
///
/// The typed sub-metadata fields `signer`, `flashloan`, and `hooks` sit
/// alongside the open-ended `metadata` slot. On the wire each typed field
/// lands inside the nested `metadata` object in its reviewed camelCase
/// position. The `hooks` value also remains readable through
/// `metadata["hooks"]` after deserialization so existing open-ended metadata
/// consumers can migrate to the typed slot on their own schedule.
///
/// # Quick start
///
/// Build a minimal SDK-attribution document tagged with a validated
/// [`AppCode`], compute its keccak digest, and produce a payload ready for
/// `PUT /api/v1/app_data/{hash}`:
///
/// ```
/// use cow_sdk_core::AppCode;
/// use cow_sdk_app_data::AppDataParams;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let code = AppCode::new("my-app")?;
/// let validated = AppDataParams::new(code).into_validated()?;
///
/// // validated.info.app_data_content — canonical JSON for PUT /app_data
/// // validated.info.app_data_hex     — 0x-prefixed keccak256 digest
/// # Ok(())
/// # }
/// ```
///
/// Chain `with_*` setters for environment, signer, hooks, or open-ended
/// metadata before either terminal:
///
/// ```
/// use cow_sdk_core::AppCode;
/// use cow_sdk_app_data::AppDataParams;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let code = AppCode::new("my-app")?;
/// let signer_addr =
///     "0x0000000000000000000000000000000000000001".parse::<cow_sdk_core::Address>()?;
///
/// let validated = AppDataParams::new(code)
///     .with_environment("production")
///     .with_signer(signer_addr)
///     .into_validated()?;
/// # Ok(())
/// # }
/// ```
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `metadata: MetadataMap` field is a `serde_json::Map<String, serde_json::Value>` alias, and `serde_json::Value` does not implement `Eq`"
)]
#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub struct AppDataParams {
    /// Optional validated application identifier written to the `appCode`
    /// field on the wire.
    pub app_code: Option<AppCode>,
    /// Optional environment label for distinguishing deployments.
    pub environment: Option<String>,
    /// Declared signer carried as `metadata.signer` on the wire, read by the
    /// submission-seam validator that enforces the reviewed
    /// `AppdataFromMismatch` invariant.
    pub signer: Option<Address>,
    /// Typed flash-loan hint carried as `metadata.flashloan` on the wire.
    pub flashloan: Option<crate::metadata::FlashloanHints>,
    /// Typed hooks envelope carried as `metadata.hooks` on the wire while
    /// preserving open-ended `metadata["hooks"]` access.
    pub hooks: Option<crate::metadata::HookList>,
    /// Arbitrary application metadata merged into the document. The two
    /// signer and flash-loan fields above leave this slot; hooks remain in
    /// the map for compatibility, and every other open-ended sub-object
    /// continues to live inside the map.
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
            app_code: Option<AppCode>,
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

        // The typed sub-metadata values come from the caller's own app-data
        // document. The inner `serde_json::Error` rendering echoes the
        // offending key or value, so each failure is mapped to a fixed,
        // field-tagged message that names only the public wire key and never
        // the caller-supplied bytes (ADR 0025). The structural detail is
        // intentionally dropped rather than folded into the deserializer
        // error message.
        let signer = match metadata.remove("signer") {
            Some(value) => Some(
                serde_json::from_value::<Address>(value).map_err(|_| {
                    <D::Error as serde::de::Error>::custom("metadata.signer is not a valid address")
                })?,
            ),
            None => None,
        };
        let flashloan = match metadata.remove("flashloan") {
            Some(value) => Some(
                serde_json::from_value::<crate::metadata::FlashloanHints>(value).map_err(|_| {
                    <D::Error as serde::de::Error>::custom(
                        "metadata.flashloan failed flash-loan hints validation",
                    )
                })?,
            ),
            None => None,
        };
        let hooks = match metadata.get("hooks").cloned() {
            Some(value) => Some(
                serde_json::from_value::<crate::metadata::HookList>(value).map_err(|_| {
                    <D::Error as serde::de::Error>::custom("metadata.hooks failed hooks validation")
                })?,
            ),
            None => None,
        };

        Ok(Self {
            app_code,
            environment,
            signer,
            flashloan,
            hooks,
            metadata,
        })
    }
}

impl AppDataParams {
    /// Creates parameters tagged with a validated [`AppCode`].
    ///
    /// All other fields default to their empty/unset state — chain `.with_*`
    /// setters to add environment, signer, flashloan hints, hooks, or
    /// open-ended metadata, then call [`AppDataParams::into_doc`] or
    /// [`AppDataParams::into_validated`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cow_sdk_core::AppCode;
    /// use cow_sdk_app_data::AppDataParams;
    ///
    /// # fn main() -> Result<(), cow_sdk_core::AppCodeError> {
    /// let code = AppCode::new("my-app")?;
    /// let params = AppDataParams::new(code).with_environment("production");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new(app_code: AppCode) -> Self {
        Self {
            app_code: Some(app_code),
            environment: None,
            signer: None,
            flashloan: None,
            hooks: None,
            metadata: MetadataMap::new(),
        }
    }

    /// Returns a copy with an explicit environment label.
    #[must_use]
    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    /// Returns a copy with a typed signer metadata value.
    #[must_use]
    pub const fn with_signer(mut self, signer: Address) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Returns a copy with typed flash-loan hint metadata.
    #[must_use]
    pub const fn with_flashloan(mut self, flashloan: crate::metadata::FlashloanHints) -> Self {
        self.flashloan = Some(flashloan);
        self
    }

    /// Returns a copy with typed hooks metadata.
    #[must_use]
    pub fn with_hooks(mut self, hooks: crate::metadata::HookList) -> Self {
        self.hooks = Some(hooks);
        self
    }

    /// Returns a copy with explicit open-ended metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: MetadataMap) -> Self {
        self.metadata = metadata;
        self
    }

    /// Generates the canonical app-data JSON document from these parameters.
    ///
    /// Fluent terminal equivalent of
    /// [`generate_app_data_doc(self)`](crate::generate_app_data_doc). Does
    /// **not** validate the doc against the embedded JSON schema or compute
    /// its keccak digest — use [`AppDataParams::into_validated`] for the
    /// full upload-ready tuple.
    ///
    /// # Examples
    ///
    /// ```
    /// use cow_sdk_core::AppCode;
    /// use cow_sdk_app_data::AppDataParams;
    ///
    /// # fn main() -> Result<(), cow_sdk_core::AppCodeError> {
    /// let code = AppCode::new("my-app")?;
    /// let doc = AppDataParams::new(code).into_doc();
    /// assert_eq!(doc["appCode"], "my-app");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn into_doc(self) -> AppDataDoc {
        generate_app_data_doc(self)
    }

    /// Generates the doc, validates it against the bundled JSON schema, and
    /// computes the [`AppDataValidated`] tuple (CID, canonical JSON content,
    /// keccak256 hex digest, and size warnings) in a single call.
    ///
    /// Recommended entry point for the full-cycle "tag, hash, upload" flow.
    /// The returned [`AppDataValidated::info`] carries the digest and
    /// canonical content ready to feed into `OrderBookApi::upload_app_data`.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError`] when the generated document fails the
    /// embedded JSON schema validation, exceeds
    /// [`crate::APP_DATA_MAX_BYTES`], or fails CID derivation.
    ///
    /// # Examples
    ///
    /// ```
    /// use cow_sdk_core::AppCode;
    /// use cow_sdk_app_data::AppDataParams;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let code = AppCode::new("my-app")?;
    /// let validated = AppDataParams::new(code).into_validated()?;
    ///
    /// // Ready to upload:
    /// //   PUT /api/v1/app_data/<validated.info.app_data_hex>
    /// //   body: validated.info.app_data_content
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_validated(self) -> Result<AppDataValidated, AppDataError> {
        get_app_data_info(self.into_doc())
    }

    /// Returns the canonical metadata [`Value`] merged from the typed
    /// sub-fields and the open-ended [`MetadataMap`] slot.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::Json`] when a typed `flashloan` or `hooks`
    /// sub-field fails to serialize — which cannot happen for values produced
    /// through the public constructors and is surfaced only for the defensive
    /// path.
    pub fn metadata_wire_value(&self) -> Result<Value, AppDataError> {
        let mut metadata = self.metadata.clone();
        if let Some(signer) = &self.signer {
            metadata.insert("signer".to_owned(), Value::String(signer.to_hex_string()));
        }
        if let Some(flashloan) = &self.flashloan {
            metadata.insert(
                "flashloan".to_owned(),
                serde_json::to_value(flashloan).map_err(AppDataError::from)?,
            );
        }
        if let Some(hooks) = &self.hooks {
            metadata.insert(
                "hooks".to_owned(),
                serde_json::to_value(hooks).map_err(AppDataError::from)?,
            );
        }
        Ok(Value::Object(metadata))
    }
}
