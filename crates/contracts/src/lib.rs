//! Low-level `CoW` Protocol contract helpers for hashing, settlement encoding,
//! signature verification, and deployment metadata.

/// Deterministic deployment metadata and address derivation helpers.
pub mod deploy;
/// Contract crate error types.
pub mod errors;
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

mod primitives;

pub use deploy::{
    ContractAddresses, ContractName, DEPLOYER_CONTRACT, SALT, deployment_for_chain,
    deterministic_deployment_address,
};
pub use errors::ContractsError;
pub use interaction::{
    Interaction, InteractionLike, normalize_interaction, normalize_interactions,
};
pub use order::{
    BUY_ETH_ADDRESS, CANCELLATIONS_TYPE_FIELDS, NormalizedOrder, ORDER_TYPE_FIELDS,
    ORDER_TYPE_HASH, ORDER_UID_LENGTH, Order, OrderCancellations, OrderTypeField, OrderUidParams,
    compute_order_uid, extract_order_uid_params, hash_order, hash_order_cancellation,
    hash_order_cancellations, hash_order_for_contract, normalize_buy_token_balance,
    normalize_order, pack_order_uid_params, uid_for_contract,
};
pub use proxy::{
    EIP173_PROXY_ABI, IMPLEMENTATION_STORAGE_SLOT, OWNER_STORAGE_SLOT, implementation_address,
    owner_address, proxy_interface,
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
    EIP1271_MAGICVALUE, Eip1271SignatureData, Eip1271VerificationRequest, Signature, SigningScheme,
    decode_eip1271_signature_data, decode_signing_scheme, encode_eip1271_signature_data,
    encode_signing_scheme, function_magic_value, normalized_ecdsa_signature,
    verify_eip1271_signature, verify_eip1271_signature_async,
};
pub use swap::{BatchSwapStep, EncodedSwap, Swap, SwapEncoder, SwapExecution, encode_swap_step};
pub use vault::{
    GrantRoleCall, RequiredVaultRole, VAULT_INTERFACE, grant_required_roles,
    required_vault_role_calls, required_vault_roles,
};
