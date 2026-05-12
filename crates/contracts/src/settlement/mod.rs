//! Settlement ABI bindings, DTOs, and public encoder/codec re-exports.

mod codec;
mod encoder;

pub use self::{codec::*, encoder::*};

use std::collections::BTreeMap;

use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};

use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, OrderUid, SellTokenSource,
};

use crate::{interaction::Interaction, signature::SigningScheme};
sol! {
    // Canonical GPv2Settlement ABI surface used by this crate for call-data
    // encoding. Signatures are reproduced verbatim from the mainnet-deployed
    // GPv2Settlement contract at 0x9008D19f58AAbD9eD0D60971565AA8510560ab41
    // (upstream source at https://github.com/cowprotocol/contracts —
    // src/contracts/GPv2Settlement.sol plus libraries/GPv2Trade.sol and
    // libraries/GPv2Interaction.sol). The Solidity excerpt used to author this
    // binding is committed under `crates/contracts/abi/settlement/` for
    // provenance.
    #[sol(rename_all = "camelcase")]
    interface IGPv2Settlement {
        struct TradeData {
            uint256 sellTokenIndex;
            uint256 buyTokenIndex;
            address receiver;
            uint256 sellAmount;
            uint256 buyAmount;
            uint32 validTo;
            bytes32 appData;
            uint256 feeAmount;
            uint256 flags;
            uint256 executedAmount;
            bytes signature;
        }

        struct InteractionData {
            address target;
            uint256 value;
            bytes callData;
        }

        function settle(
            address[] calldata tokens,
            uint256[] calldata clearingPrices,
            TradeData[] calldata trades,
            InteractionData[][3] calldata interactions
        ) external;

        function invalidateOrder(bytes calldata orderUid) external;

        function setPreSignature(bytes calldata orderUid, bool signed) external;

        function freeFilledAmountStorage(bytes[] calldata orderUids) external;

        function freePreSignatureStorage(bytes[] calldata orderUids) external;
    }
}

/// Settlement interaction stage.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum InteractionStage {
    /// Interactions executed before trades.
    Pre = 0,
    /// Interactions executed between trade processing steps.
    Intra = 1,
    /// Interactions executed after trades.
    Post = 2,
}

#[derive(Clone, Copy)]
enum OrderRefundKind {
    FilledAmount,
    PreSignature,
}

/// Compact order-flag inputs.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFlags {
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order is partially fillable.
    pub partially_fillable: bool,
    /// Sell-token balance source.
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination.
    pub buy_token_balance: BuyTokenDestination,
}

/// Compact trade-flag inputs.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeFlags {
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order is partially fillable.
    pub partially_fillable: bool,
    /// Sell-token balance source.
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination.
    pub buy_token_balance: BuyTokenDestination,
    /// Signing scheme used for the signature.
    pub signing_scheme: SigningScheme,
}

/// Trade execution override used while encoding settlements.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeExecution {
    /// Executed amount recorded in the encoded trade.
    pub executed_amount: Amount,
}

/// Order-refund payload used for settlement post-interactions.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRefunds {
    /// Filled-amount storage entries to clear.
    pub filled_amounts: Vec<OrderUid>,
    /// Pre-signature storage entries to clear.
    pub pre_signatures: Vec<OrderUid>,
}

/// Clearing prices keyed by token address.
pub type Prices = BTreeMap<Address, Amount>;

/// Encoded settlement trade payload.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Sell token index in the token registry.
    pub sell_token_index: usize,
    /// Buy token index in the token registry.
    pub buy_token_index: usize,
    /// Receiver address.
    pub receiver: Address,
    /// Sell amount.
    pub sell_amount: Amount,
    /// Buy amount.
    pub buy_amount: Amount,
    /// Expiration timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: AppDataHash,
    /// Fee amount.
    pub fee_amount: Amount,
    /// Encoded trade flags.
    pub flags: u8,
    /// Executed amount.
    pub executed_amount: Amount,
    /// Encoded signature payload.
    pub signature: String,
}

impl OrderFlags {
    /// Creates compact order-flag inputs.
    #[must_use]
    pub const fn new(
        kind: OrderKind,
        partially_fillable: bool,
        sell_token_balance: SellTokenSource,
        buy_token_balance: BuyTokenDestination,
    ) -> Self {
        Self {
            kind,
            partially_fillable,
            sell_token_balance,
            buy_token_balance,
        }
    }
}

impl TradeFlags {
    /// Creates compact trade-flag inputs.
    #[must_use]
    pub const fn new(
        kind: OrderKind,
        partially_fillable: bool,
        sell_token_balance: SellTokenSource,
        buy_token_balance: BuyTokenDestination,
        signing_scheme: SigningScheme,
    ) -> Self {
        Self {
            kind,
            partially_fillable,
            sell_token_balance,
            buy_token_balance,
            signing_scheme,
        }
    }
}

impl TradeExecution {
    /// Creates a trade execution override.
    #[must_use]
    pub const fn new(executed_amount: Amount) -> Self {
        Self { executed_amount }
    }
}

impl OrderRefunds {
    /// Creates an order-refund payload.
    #[must_use]
    pub const fn new(filled_amounts: Vec<OrderUid>, pre_signatures: Vec<OrderUid>) -> Self {
        Self {
            filled_amounts,
            pre_signatures,
        }
    }
}

impl Trade {
    /// Creates an encoded settlement trade payload.
    #[must_use]
    // Mirrors the full current public field set so callers can migrate off
    // struct literals without losing explicit control over any wire field.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        sell_token_index: usize,
        buy_token_index: usize,
        receiver: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        app_data: AppDataHash,
        fee_amount: Amount,
        flags: u8,
        executed_amount: Amount,
        signature: String,
    ) -> Self {
        Self {
            sell_token_index,
            buy_token_index,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount,
            flags,
            executed_amount,
            signature,
        }
    }
}

/// Fully encoded settlement payload.
pub type EncodedSettlement = (Vec<Address>, Vec<Amount>, Vec<Trade>, [Vec<Interaction>; 3]);

/// Registry that assigns stable indexes to token addresses.
#[derive(Debug, Clone, Default)]
pub struct TokenRegistry {
    tokens: Vec<Address>,
    token_map: BTreeMap<String, usize>,
}

impl TokenRegistry {
    /// Creates an empty token registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns registered token addresses in index order.
    #[must_use]
    pub fn addresses(&self) -> Vec<Address> {
        self.tokens.clone()
    }

    /// Returns the stable index for `token`, inserting it if needed.
    pub fn index(&mut self, token: &Address) -> usize {
        let key = token.normalized_key();
        if let Some(index) = self.token_map.get(&key) {
            return *index;
        }
        let index = self.tokens.len();
        self.tokens.push(token.clone());
        self.token_map.insert(key, index);
        index
    }
}
