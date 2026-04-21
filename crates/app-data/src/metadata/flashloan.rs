//! Typed flash-loan hints consumed by the app-data metadata shape.
//!
//! The reviewed services authority carries a single flash-loan hint per order
//! with five required fields — `liquidityProvider`, `protocolAdapter`,
//! `receiver`, `token`, and `amount` — expressed on the wire as a camelCase
//! object. The bundled `flashloan/v0.2.0.json` sub-schema constrains
//! `amount` through the `bigPositiveNumber` definition (decimal strings
//! matching `^[1-9]\d*$`) and the address fields through the shared
//! `ethereumAddress` regex.
//!
//! [`FlashloanHints`] narrows that wire shape to a typed Rust struct whose
//! derived serde impls reproduce the wire form byte-identically. The typed
//! [`FlashloanHints::validate`] method matches the published basis-point
//! ruleset by rejecting a zero `amount` and any zero-address field at the
//! client before a document would fail the reviewed schema.

use cow_sdk_core::{Address, Amount, ValidationReason};
use serde::{Deserialize, Serialize};

use crate::AppDataError;

/// Typed flash-loan hint carried inside the app-data metadata envelope.
///
/// The wire form is a single camelCase object with five required fields.
/// The struct head is `#[non_exhaustive]` so future additions to the
/// reviewed schema may be introduced as a minor change without breaking
/// downstream exhaustive matches.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlashloanHints {
    /// Address of the contract the flash loan is drawn against.
    pub liquidity_provider: Address,
    /// Address of the adapter contract that executes the flash-loan call.
    pub protocol_adapter: Address,
    /// Address that receives the borrowed tokens.
    pub receiver: Address,
    /// Address of the token being borrowed.
    pub token: Address,
    /// Amount of the token to borrow, expressed as a decimal string in the
    /// token's smallest unit.
    pub amount: Amount,
}

impl FlashloanHints {
    /// Constructs a flash-loan hint after validating every field against the
    /// published bounds.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidFlashloanHints`] when `amount` is zero
    /// or when any address field is the zero address.
    pub fn new(
        liquidity_provider: Address,
        protocol_adapter: Address,
        receiver: Address,
        token: Address,
        amount: Amount,
    ) -> Result<Self, AppDataError> {
        let hints = Self {
            liquidity_provider,
            protocol_adapter,
            receiver,
            token,
            amount,
        };
        hints.validate()?;
        Ok(hints)
    }

    /// Validates this hint against the published flash-loan bounds.
    ///
    /// The reviewed schema requires every address field to be a non-zero
    /// 20-byte address, and the shared `bigPositiveNumber` definition
    /// rejects a zero `amount`.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidFlashloanHints`] on the first field
    /// that falls outside the documented bounds.
    pub fn validate(&self) -> Result<(), AppDataError> {
        if self.amount == Amount::zero() {
            return Err(AppDataError::InvalidFlashloanHints {
                field: "amount",
                reason: ValidationReason::OutOfRange {
                    details: "amount must be a positive decimal integer",
                },
            });
        }
        validate_non_zero_address("liquidityProvider", &self.liquidity_provider)?;
        validate_non_zero_address("protocolAdapter", &self.protocol_adapter)?;
        validate_non_zero_address("receiver", &self.receiver)?;
        validate_non_zero_address("token", &self.token)?;
        Ok(())
    }
}

fn validate_non_zero_address(field: &'static str, address: &Address) -> Result<(), AppDataError> {
    if address == &Address::from_bytes([0u8; 20]) {
        return Err(AppDataError::InvalidFlashloanHints {
            field,
            reason: ValidationReason::BadShape {
                details: "address must be a non-zero 20-byte Ethereum address",
            },
        });
    }
    Ok(())
}
