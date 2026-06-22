#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Low-level `CoW` Protocol contract helpers for order hashing, signature
//! codecs and on-chain verification, ABI bindings, fail-closed event decoding,
//! and deployment metadata.

#![warn(missing_docs)]
#![allow(
    clippy::redundant_pub_crate,
    reason = "the cross-module helpers inside the private `primitives` module (`check_topics`, `order_uid_from_bytes`) are `pub(crate)` by design: `pub(crate)` keeps them crate-internal under `unreachable_pub` and documents the cross-module use, so the `redundant_pub_crate` pedantic lint is suppressed crate-wide rather than widening the items to `pub`"
)]

/// COW Shed account-abstraction proxy, EIP-712, and hook-signing helpers.
///
/// Gated behind the off-by-default `cow-shed` feature so the default
/// `cow-sdk-contracts` surface and dependency closure stay lean.
#[cfg(feature = "cow-shed")]
#[cfg_attr(docsrs, doc(cfg(feature = "cow-shed")))]
pub mod cow_shed;
/// Chain-keyed registry of canonical CoW Protocol contract deployments.
pub mod deployments;
/// Contract crate error types.
pub mod errors;
/// Typed `CoWSwapEthFlow` call-data encoders generated from the upstream
/// Solidity surface via the `alloy::sol!` macro.
pub mod eth_flow;
/// Hex decode helpers for `0x`-prefixed payloads inside the contracts
/// boundary.
///
/// The module raises typed `ContractsError` variants with a
/// `&'static str` `field` discriminator on every failure mode.
pub mod hex_field;
/// Typed interaction models and normalization helpers.
pub mod interaction;
/// Typed `CoWSwapOnchainOrders` event bindings and a fail-closed log decoder.
pub mod onchain_orders;
/// Order hashing, UID packing, and normalization helpers.
pub mod order;
/// Typed `GPv2Settlement` ABI binding and a fail-closed settlement event-log decoder.
pub mod settlement;
/// Signature codecs and EIP-1271 verification helpers.
pub mod signature;
/// Typed ERC-20 and wrapped-native (WETH9-family) token bindings and wrap /
/// unwrap interaction helpers.
pub mod tokens;
/// Gas-free on-chain transaction builders with override-or-registry resolution.
pub mod tx;
/// Cache-aware EIP-1271 signature verification path.
pub mod verify;

mod primitives;

pub use primitives::{
    buy_balance_from_marker, buy_balance_name, order_kind_from_marker, order_kind_name,
    sell_balance_from_marker, sell_balance_name,
};

pub use deployments::{ContractId, DeploymentChainId, DeploymentEnv, Registry};
pub use errors::ContractsError;
pub use eth_flow::{
    EthFlowEvent, EthFlowOnchainData, EthFlowOrderData, ICoWSwapEthFlowEvents, OnchainOrderRefund,
    decode_eth_flow_log, decode_order_refund, encode_create_order_calldata,
    encode_invalidate_order_calldata, parse_eth_flow_onchain_data,
};
pub use interaction::{
    Interaction, InteractionLike, normalize_interaction, normalize_interactions,
};
pub use onchain_orders::{
    ICoWSwapOnchainOrders, OnchainOrderInvalidation, OnchainOrderPlacement, OnchainSigningScheme,
    decode_order_invalidation, decode_order_placement,
};
pub use order::{
    BUY_ETH_ADDRESS, CANCELLATIONS_TYPE_FIELDS, GPv2OrderCancellations, ORDER_TYPE_FIELDS,
    ORDER_UID_LENGTH, OrderCancellations, OrderTypeField, OrderUidParams, compute_order_uid,
    extract_order_uid_params, hash_order, hash_order_cancellation, hash_order_cancellations,
    order_eip712_type_hash, pack_order_uid_params,
};
pub use settlement::{
    IGPv2SettlementEvents, SettlementEvent, decode_settlement_log, encode_invalidate_order,
    encode_set_pre_signature,
};
pub use signature::{
    Eip1271SignatureData, Eip1271VerificationRequest, IERC1271, MAX_SIGNATURE_HEX_BYTES,
    RecoverableSignature, Signature, SigningScheme, decode_eip1271_signature_data,
    decode_signing_scheme, encode_eip1271_signature_data, encode_signing_scheme,
    verify_eip1271_signature,
};
pub use tokens::{
    IERC20, IWrappedNativeToken, unwrap_interaction, unwrap_transaction, wrap_interaction,
    wrap_transaction,
};
pub use tx::{
    UnsignedTransaction, ethflow_create_order_transaction, invalidate_order_transaction,
    pre_sign_transaction, resolve_contract_address, resolve_eth_flow_address,
    resolve_settlement_address,
};
pub use verify::{Eip1271Cache, NoopEip1271Cache, verify_eip1271_signature_cached};
