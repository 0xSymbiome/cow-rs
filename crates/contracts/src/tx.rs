//! Gas-free on-chain transaction builders over the canonical contract bindings.
//!
//! Each builder resolves its target deployment — honoring a caller override and
//! otherwise the embedded [`Registry`] — encodes the call-data from the typed
//! bindings, and returns a gas-free [`UnsignedTransaction`] (`to` / `data` /
//! `value`, no gas limit), mirroring the upstream services `eth::Tx` shape. The
//! caller estimates gas, signs, and submits: `cow-sdk-trading` wraps these with
//! signer-bound gas estimation and submission, while `cow-sdk-wasm` surfaces them
//! directly. Resolution is fail-closed — a chain/environment with no registered
//! deployment yields [`ContractsError::DeploymentNotFound`] rather than a panic.

use cow_sdk_core::{
    Address, Amount, CowEnv, HexData, OrderData, OrderUid, ProtocolOptions, SupportedChainId,
    TransactionRequest,
};

use crate::eth_flow::{
    EthFlowOrderData, encode_create_order_calldata, encode_invalidate_order_calldata,
};
use crate::settlement::{encode_invalidate_order, encode_set_pre_signature};
use crate::{ContractId, ContractsError, Registry};

/// A gas-free unsigned transaction produced by the contract tx builders.
///
/// Carries the resolved `to`, the encoded `data`, and the native `value`, with no
/// gas limit — mirroring the upstream services `eth::Tx` shape. The caller
/// estimates gas, signs, and submits. Convert to the optional-field
/// [`TransactionRequest`] wire shape with `.into()`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnsignedTransaction {
    /// Destination contract address.
    pub to: Address,
    /// Encoded call-data payload.
    pub data: HexData,
    /// Native token value to transfer.
    pub value: Amount,
}

impl UnsignedTransaction {
    /// Creates a gas-free unsigned transaction from its parts.
    #[must_use]
    pub const fn new(to: Address, data: HexData, value: Amount) -> Self {
        Self { to, data, value }
    }
}

impl From<UnsignedTransaction> for TransactionRequest {
    fn from(tx: UnsignedTransaction) -> Self {
        Self::new(Some(tx.to), Some(tx.data), Some(tx.value), None)
    }
}

impl From<&UnsignedTransaction> for TransactionRequest {
    fn from(tx: &UnsignedTransaction) -> Self {
        Self::new(Some(tx.to), Some(tx.data.clone()), Some(tx.value), None)
    }
}

/// Resolves a registry-backed contract address, honoring a caller override and
/// otherwise reading the canonical deployment from the embedded [`Registry`].
///
/// Returns `None` when no deployment of `contract_id` is registered for the
/// chain/environment pair. The single source for the override-then-registry
/// fallback used across the SDK.
#[must_use]
pub fn resolve_contract_address(
    contract_id: ContractId,
    override_address: Option<Address>,
    chain_id: SupportedChainId,
    env: CowEnv,
) -> Option<Address> {
    override_address.or_else(|| Registry::default().address(contract_id, chain_id, env))
}

/// Reads the per-chain override for `contract_id` from `options`, if present.
fn contract_override(
    contract_id: ContractId,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Option<Address> {
    let map = options.and_then(|opts| match contract_id {
        ContractId::Settlement => opts.settlement_contract_override.as_ref(),
        ContractId::EthFlow => opts.eth_flow_contract_override.as_ref(),
        _ => None,
    })?;
    map.get(&u64::from(chain_id)).copied()
}

/// Resolves the environment from `options`, defaulting to production.
fn resolved_env(options: Option<&ProtocolOptions>) -> CowEnv {
    options.and_then(|opts| opts.env).unwrap_or(CowEnv::Prod)
}

/// Resolves the settlement contract address for `chain_id`, honoring any
/// settlement override carried in `options`.
///
/// Returns `None` when no settlement deployment is registered for the
/// chain/environment.
#[must_use]
pub fn resolve_settlement_address(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Option<Address> {
    resolve_contract_address(
        ContractId::Settlement,
        contract_override(ContractId::Settlement, chain_id, options),
        chain_id,
        resolved_env(options),
    )
}

/// Resolves the `CoWSwapEthFlow` contract address for `chain_id`, honoring any
/// eth-flow override carried in `options`.
///
/// Returns `None` when no eth-flow deployment is registered for the
/// chain/environment.
#[must_use]
pub fn resolve_eth_flow_address(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Option<Address> {
    resolve_contract_address(
        ContractId::EthFlow,
        contract_override(ContractId::EthFlow, chain_id, options),
        chain_id,
        resolved_env(options),
    )
}

/// Builds the gas-free settlement pre-sign transaction.
///
/// Targets the settlement contract with `setPreSignature(orderUid, true)`
/// call-data and zero native value.
///
/// # Errors
///
/// Returns [`ContractsError::DeploymentNotFound`] when no settlement deployment
/// is registered for the chain/environment.
pub fn pre_sign_transaction(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<UnsignedTransaction, ContractsError> {
    let to = resolve_settlement_address(chain_id, options).ok_or(
        ContractsError::DeploymentNotFound {
            contract: "settlement",
            chain_id: u64::from(chain_id),
        },
    )?;
    Ok(UnsignedTransaction::new(
        to,
        HexData::from_bytes(encode_set_pre_signature(order_uid, true)),
        Amount::ZERO,
    ))
}

/// Builds the gas-free settlement on-chain cancellation transaction.
///
/// Targets the settlement contract with `invalidateOrder(orderUid)` call-data and
/// zero native value.
///
/// # Errors
///
/// Returns [`ContractsError::DeploymentNotFound`] when no settlement deployment
/// is registered for the chain/environment.
pub fn invalidate_order_transaction(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<UnsignedTransaction, ContractsError> {
    let to = resolve_settlement_address(chain_id, options).ok_or(
        ContractsError::DeploymentNotFound {
            contract: "settlement",
            chain_id: u64::from(chain_id),
        },
    )?;
    Ok(UnsignedTransaction::new(
        to,
        HexData::from_bytes(encode_invalidate_order(order_uid)),
        Amount::ZERO,
    ))
}

/// Builds the gas-free `CoWSwapEthFlow` create-order transaction for a native
/// sell.
///
/// Targets the eth-flow contract with `createOrder(EthFlowOrderData)` call-data
/// and a native `value` equal to the order's sell amount.
///
/// # Errors
///
/// Returns [`ContractsError::DeploymentNotFound`] when no eth-flow deployment is
/// registered for the chain/environment, or [`ContractsError::ZeroReceiver`] when
/// the order receiver is the zero address (matching
/// [`EthFlowOrderData::from_unsigned_order`]).
pub fn ethflow_create_order_transaction(
    order: &OrderData,
    quote_id: i64,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<UnsignedTransaction, ContractsError> {
    let to =
        resolve_eth_flow_address(chain_id, options).ok_or(ContractsError::DeploymentNotFound {
            contract: "eth-flow",
            chain_id: u64::from(chain_id),
        })?;
    let payload = EthFlowOrderData::from_unsigned_order(order, quote_id)?;
    Ok(UnsignedTransaction::new(
        to,
        HexData::from_bytes(encode_create_order_calldata(&payload)),
        order.sell_amount,
    ))
}

/// Builds the gas-free `CoWSwapEthFlow` on-chain order-cancellation transaction.
///
/// Targets the eth-flow contract with `invalidateOrder(EthFlowOrderData)`
/// call-data and zero native value. This is distinct from the settlement-level
/// [`invalidate_order_transaction`], which cancels a regular order by its packed
/// UID; eth-flow cancellation takes the full order payload back.
///
/// # Errors
///
/// Returns [`ContractsError::DeploymentNotFound`] when no eth-flow deployment is
/// registered for the chain/environment, or [`ContractsError::ZeroReceiver`] when
/// the order receiver is the zero address (matching
/// [`EthFlowOrderData::from_unsigned_order`]).
pub fn ethflow_invalidate_order_transaction(
    order: &OrderData,
    quote_id: i64,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<UnsignedTransaction, ContractsError> {
    let to =
        resolve_eth_flow_address(chain_id, options).ok_or(ContractsError::DeploymentNotFound {
            contract: "eth-flow",
            chain_id: u64::from(chain_id),
        })?;
    let payload = EthFlowOrderData::from_unsigned_order(order, quote_id)?;
    Ok(UnsignedTransaction::new(
        to,
        HexData::from_bytes(encode_invalidate_order_calldata(&payload)),
        Amount::ZERO,
    ))
}

#[cfg(test)]
mod tests {
    use super::{ethflow_invalidate_order_transaction, resolve_eth_flow_address};
    use crate::eth_flow::{EthFlowOrderData, encode_invalidate_order_calldata};
    use cow_sdk_core::{Amount, HexData, SupportedChainId};

    #[test]
    fn ethflow_invalidate_targets_eth_flow_with_zero_value_and_invalidate_calldata() {
        let order = cow_sdk_test_utils::builders::OrderBuilder::weth_dai()
            .receiver("0x2222222222222222222222222222222222222222")
            .build();
        let quote_id = 1_234_567_i64;

        let tx =
            ethflow_invalidate_order_transaction(&order, quote_id, SupportedChainId::Mainnet, None)
                .expect("eth-flow is deployed on mainnet");

        let eth_flow = resolve_eth_flow_address(SupportedChainId::Mainnet, None)
            .expect("eth-flow is deployed on mainnet");
        let payload =
            EthFlowOrderData::from_unsigned_order(&order, quote_id).expect("non-zero receiver");

        assert_eq!(tx.to, eth_flow);
        assert_eq!(tx.value, Amount::ZERO);
        assert_eq!(
            tx.data,
            HexData::from_bytes(encode_invalidate_order_calldata(&payload))
        );
    }
}
