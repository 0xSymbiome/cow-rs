use cow_sdk_core::{Address, ValidationReason};
use serde::de::{Deserializer, Error as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AppDataError;

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
    /// basis-point values fall outside the documented `[1..=9999]` range, or
    /// whose recipient address is the zero address.
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
        // SAFETY: partner-fee schema values are typed serde data owned by this
        // crate; serialization failure would mean the schema type stopped being
        // serializable.
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
    /// Returns [`AppDataError::Serialization`] when the JSON value does not match any
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
    /// outside the documented `[1..=9999]` range, or when `recipient` is the
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
    /// outside `[1..=9999]`, when `max_volume_bps` falls outside `[1..=9999]`,
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
    /// `max_volume_bps` falls outside `[1..=9999]`, or when `recipient` is the
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
    /// * `volumeBps` — integer in `[1..=9999]`
    /// * `surplusBps` — integer in `[1..=9999]`
    /// * `priceImprovementBps` — integer in `[1..=9999]`
    /// * `maxVolumeBps` — integer in `[1..=9999]`
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

// partnerFee schema v1.1.0 raised the volume cap to match the surplus cap. The two
// constants now coincide but stay distinct to mirror upstream's separate `maxVolumeBps`
// and `surplusBps` definitions, so a future divergence is a one-line change here.
const MAX_VOLUME_BPS: u16 = 9_999;
const MAX_SURPLUS_BPS: u16 = 9_999;

const fn validate_max_volume_bps(field: &'static str, value: u16) -> Result<(), AppDataError> {
    if value == 0 || value > MAX_VOLUME_BPS {
        return Err(AppDataError::InvalidPartnerFee {
            field,
            reason: ValidationReason::OutOfRange {
                details: "value must be an integer in the inclusive range [1, 9999]",
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
    if recipient.is_zero() {
        return Err(AppDataError::InvalidPartnerFee {
            field,
            reason: ValidationReason::Precondition {
                details: "recipient must not be the zero address",
            },
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partner_fee_roundtrips_single_and_array_shapes_and_exposes_first_volume_fee() {
        let recipient = Address::new("0x1111111111111111111111111111111111111111")
            .expect("test recipient must be valid");
        let fee = PartnerFee::from(vec![
            PartnerFeePolicy::surplus(250, 100, recipient).expect("surplus policy must validate"),
            PartnerFeePolicy::volume(42, recipient).expect("volume policy must validate"),
        ]);

        let value = fee.to_value();
        let reparsed = PartnerFee::from_value(value.clone()).expect("typed partner fee re-parses");

        assert_eq!(
            value,
            serde_json::json!([
                {
                    "surplusBps": 250,
                    "maxVolumeBps": 100,
                    "recipient": recipient.to_hex_string()
                },
                {
                    "volumeBps": 42,
                    "recipient": recipient.to_hex_string()
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
}
