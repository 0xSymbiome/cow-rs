#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]

//! Low-level `CoW` Protocol contract helpers for hashing, settlement encoding,
//! signature verification, and deployment metadata.

#![warn(missing_docs)]
#![allow(
    clippy::redundant_pub_crate,
    reason = "the remaining items inside the private `primitives` module (`ORDER_UID_LENGTH_BYTES`, `function_selector`) carry explicit `pub(crate)` markers as cross-module use documentation and as defensive scoping if the module is ever promoted to `pub mod`"
)]

/// Deterministic deployment metadata and address derivation helpers.
pub mod deploy;
/// Chain-keyed registry of canonical CoW Protocol contract deployments.
pub mod deployments;
/// Typed ABI binding for the EIP-1271 standard signature-validation
/// interface, generated via the `alloy::sol!` macro.
pub mod eip1271;
/// Typed ERC-20 and EIP-2612 Permit bindings generated from the upstream
/// Solidity surface via the `alloy::sol!` macro.
pub mod erc20;
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
/// Order hashing, UID packing, and normalization helpers.
pub mod order;
/// Proxy inspection helpers for EIP-173 style ownership proxies.
pub mod proxy;
/// Reader helpers for allow-list, settlement, and trade-simulation contracts.
pub mod reader;
/// Settlement encoding helpers and flag codecs.
pub mod settlement;
/// Signature codecs and EIP-1271 verification helpers.
pub mod signature;
/// Batch-swap encoding helpers.
pub mod swap;
/// Vault authorization role helpers.
pub mod vault;
/// Cache-aware EIP-1271 signature verification path.
pub mod verify;

mod chain_ids;
mod primitives;

pub use primitives::{buy_balance_name, encode_address_word, order_kind_name, sell_balance_name};

pub use deploy::{
    ContractAddresses, ContractName, DEPLOYER_CONTRACT, SALT, deployment_address_hash_input,
    deployment_for_chain, deterministic_deployment_address,
};
pub use deployments::{
    ContractId, DeploymentChainId, DeploymentCoverage, DeploymentCoverageError,
    DeploymentCoverageStatus, DeploymentEnv, DeploymentVerificationStatus, Registry, RegistryError,
};
pub use eip1271::IERC1271;
pub use erc20::{IERC20, IERC20Permit, PERMIT_TYPE_HASH, permit_typed_data_hash};
pub use errors::ContractsError;
pub use eth_flow::{
    EthFlowOrderData, encode_create_order_calldata, encode_invalidate_order_calldata,
};
pub use interaction::{
    Interaction, InteractionLike, normalize_interaction, normalize_interactions,
};
pub use order::{
    BUY_ETH_ADDRESS, CANCELLATIONS_TYPE_FIELDS, GPv2Order, GPv2OrderCancellations, NormalizedOrder,
    ORDER_TYPE_FIELDS, ORDER_UID_LENGTH, Order, OrderCancellations, OrderTypeField, OrderUidParams,
    compute_order_uid, extract_order_uid_params, hash_order, hash_order_cancellation,
    hash_order_cancellations, normalize_order, pack_order_uid_params,
};
pub use proxy::{
    Eip1967Slot, IEip173Proxy, SlotBytes, admin_address, implementation_address, owner_address,
};
pub use reader::{
    AllowListReader, SettlementReader, TradeSimulation, TradeSimulationBalanceDelta,
    TradeSimulationResult, TradeSimulator,
};
pub use settlement::{
    EncodedSettlement, InteractionStage, OrderFlags, OrderRefunds, Prices, SettlementEncoder,
    TokenRegistry, Trade, TradeExecution, TradeFlags, decode_order, decode_order_flags,
    decode_trade_flags, encode_order_flags, encode_signature_data, encode_trade,
    encode_trade_flags,
};
pub use signature::{
    Eip1271SignatureData, Eip1271VerificationRequest, RecoverableSignature, Signature,
    SigningScheme, decode_eip1271_signature_data, decode_signing_scheme,
    encode_eip1271_signature_data, encode_signing_scheme, verify_eip1271_signature,
};
pub use swap::{BatchSwapStep, EncodedSwap, Swap, SwapEncoder, SwapExecution, encode_swap_step};
pub use vault::{
    GrantRoleCall, RequiredVaultRole, VAULT_INTERFACE, grant_required_roles,
    required_vault_role_calls, required_vault_roles,
};
pub use verify::{Eip1271VerificationCache, verify_eip1271_signature_cached};
