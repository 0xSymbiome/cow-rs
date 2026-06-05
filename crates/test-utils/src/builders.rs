//! Canonical order / domain / signature fixtures for the workspace test suites.
//!
//! `OrderData` is assembled as JSON and deserialized rather than
//! hand-constructed, and enum overrides are routed through the types' own
//! `Serialize` impls to avoid hard-coding wire spellings.

use cow_sdk_core::{
    Address, BuyTokenDestination, OrderData, OrderKind, SellTokenSource, TypedDataDomain,
};
use serde_json::{Value, json};

/// The `GPv2` Settlement contract (Ethereum mainnet). Tests should obtain the
/// domain via [`sample_domain`] or source the address from
/// `cow_sdk_contracts::Registry` rather than re-typing this literal.
fn mainnet_settlement() -> Address {
    Address::new("0x9008d19f58aabd9ed0d60971565aa8510560ab41").expect("valid settlement address")
}

/// Constructs an [`Address`] from a `0x`-hex string literal, canonicalising the
/// casing per ADR 0052. The shared home for the per-test address constructor the
/// suites would otherwise each re-declare.
///
/// # Panics
/// Panics if `value` is not a valid address literal.
#[must_use]
pub fn address(value: &str) -> Address {
    Address::new(value).expect("test address literal must be valid")
}

/// Builder for the canonical `OrderData` test vector.
#[derive(Clone, Debug)]
pub struct OrderBuilder {
    value: Value,
}

impl Default for OrderBuilder {
    fn default() -> Self {
        Self::upstream_signing()
    }
}

impl OrderBuilder {
    /// The upstream signing vector preset — the canonical order shared by the
    /// signing and alloy test suites.
    #[must_use]
    pub fn upstream_signing() -> Self {
        Self {
            value: json!({
                "sellToken": "0xd057b63f5e69cf1b929b356b579cba08d7688048",
                "buyToken":  "0x7b878668cd1a3adf89764d3a331e0a7bb832192d",
                "receiver":  "0xa6ddbd0de6b310819b49f680f65871bee85f517e",
                "sellAmount": "500000000000000",
                "buyAmount": "23000020000",
                "validTo": 5_000_222,
                "appData": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "feeAmount": "2300000",
                "kind": "sell",
                "partiallyFillable": true,
                "sellTokenBalance": "erc20",
                "buyTokenBalance": "erc20"
            }),
        }
    }

    /// The WETH/DAI order preset used by the contracts swap, settlement, and
    /// order tests; the receiver is the zero address and the balances default
    /// to `erc20`.
    #[must_use]
    pub fn weth_dai() -> Self {
        Self {
            value: json!({
                "sellToken": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                "buyToken": "0x6b175474e89094c44da98b954eedeac495271d0f",
                "receiver": "0x0000000000000000000000000000000000000000",
                "sellAmount": "1000000000000000000",
                "buyAmount": "2000000000000000000000",
                "validTo": 1_709_990_000,
                "appData": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "feeAmount": "5000000000000000",
                "kind": "sell",
                "partiallyFillable": false,
                "sellTokenBalance": "erc20",
                "buyTokenBalance": "erc20"
            }),
        }
    }

    /// Override the receiver address (0x-prefixed hex).
    #[must_use]
    pub fn receiver(mut self, receiver: &str) -> Self {
        self.value["receiver"] = json!(receiver);
        self
    }

    /// Override the order kind.
    ///
    /// # Panics
    /// Panics if `kind` fails to serialize, which the enum never does.
    #[must_use]
    pub fn kind(mut self, kind: OrderKind) -> Self {
        self.value["kind"] = serde_json::to_value(kind).expect("OrderKind serializes");
        self
    }

    /// Override partial-fillability.
    #[must_use]
    pub fn partially_fillable(mut self, value: bool) -> Self {
        self.value["partiallyFillable"] = json!(value);
        self
    }

    /// Override the sell-token balance source.
    ///
    /// # Panics
    /// Panics if `source` fails to serialize, which the enum never does.
    #[must_use]
    pub fn sell_balance(mut self, source: SellTokenSource) -> Self {
        self.value["sellTokenBalance"] =
            serde_json::to_value(source).expect("SellTokenSource serializes");
        self
    }

    /// Override the buy-token balance destination.
    ///
    /// # Panics
    /// Panics if `destination` fails to serialize, which the enum never does.
    #[must_use]
    pub fn buy_balance(mut self, destination: BuyTokenDestination) -> Self {
        self.value["buyTokenBalance"] =
            serde_json::to_value(destination).expect("BuyTokenDestination serializes");
        self
    }

    /// Build the `OrderData`.
    ///
    /// # Panics
    /// Panics if the assembled JSON is not a valid `OrderData`.
    #[must_use]
    pub fn build(self) -> OrderData {
        serde_json::from_value(self.value).expect("OrderBuilder must produce a valid OrderData")
    }
}

/// The canonical `GPv2` EIP-712 domain (mainnet settlement).
#[must_use]
pub fn sample_domain() -> TypedDataDomain {
    sample_domain_with(mainnet_settlement())
}

/// The canonical `GPv2` EIP-712 domain with an explicit verifying contract.
#[must_use]
pub fn sample_domain_with(verifying_contract: Address) -> TypedDataDomain {
    TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        1,
        verifying_contract,
    )
}

/// A deterministic 65-byte ECDSA signature hex string filled with `fill_byte`
/// (recovery byte `0x1b`).
#[must_use]
pub fn sample_signature_hex(fill_byte: u8) -> String {
    format!("0x{}1b", format!("{fill_byte:02x}").repeat(64))
}
