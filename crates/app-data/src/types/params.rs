use cow_sdk_core::Address;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AppDataError;

use super::MetadataMap;

/// Inputs used to build an app-data document.
///
/// The typed sub-metadata fields `signer`, `flashloan`, and `hooks` sit
/// alongside the open-ended `metadata` slot. On the wire each typed field
/// lands inside the nested `metadata` object in its reviewed camelCase
/// position. The `hooks` value also remains readable through
/// `metadata["hooks"]` after deserialization so existing open-ended metadata
/// consumers can migrate to the typed slot on their own schedule.
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
        let hooks = match metadata.get("hooks").cloned() {
            Some(value) => Some(
                serde_json::from_value::<crate::metadata::HookList>(value)
                    .map_err(serde::de::Error::custom)?,
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
    /// Creates app-data parameters with the source-compatible constructor shape.
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
            hooks: None,
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
    pub const fn with_signer(mut self, signer: Address) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Returns a copy with typed flash-loan hint metadata.
    #[must_use]
    pub fn with_flashloan(mut self, flashloan: crate::metadata::FlashloanHints) -> Self {
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
