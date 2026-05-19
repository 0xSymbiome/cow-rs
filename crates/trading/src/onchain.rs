use std::str::FromStr;

use alloy_primitives::Bytes as AlloyBytes;
use alloy_sol_types::SolCall;
use cow_sdk_contracts::eth_flow::{
    EthFlowOrderData, encode_create_order_calldata, encode_invalidate_order_calldata,
};
use cow_sdk_contracts::settlement::IGPv2Settlement;
use cow_sdk_contracts::{ContractId, Registry};
use cow_sdk_core::{
    Address, Amount, AsyncSigner, HexData, ProtocolOptions, Signer, SupportedChainId,
    TransactionHash, TransactionRequest,
};
use cow_sdk_orderbook::Order;

use crate::slippage::{gas_with_margin, parse_integer};
use crate::{
    GAS_LIMIT_DEFAULT, OrderTraderParameters, PartialTraderParameters, TraderParameters,
    TradingError, calculate_unique_order_id, get_order_to_sign,
};

/// `EthFlow` transaction bundle returned by native-sell helper flows.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct EthFlowTransaction {
    /// Final unique order id.
    pub order_id: cow_sdk_core::OrderUid,
    /// Transaction request to submit.
    pub transaction: TransactionRequest,
    /// Unsigned order payload used to derive `order_id` and the transaction body.
    pub order_to_sign: cow_sdk_core::UnsignedOrder,
    /// Signer-derived owner resolved at transaction construction via
    /// [`AsyncSigner::get_address`].
    ///
    /// Downstream submission uses this value as `OrderCreation.from` for
    /// pre-HTTP validation — not `order_to_sign.receiver`, which is the
    /// payout recipient and may legitimately differ from the owner when the
    /// caller asks the proceeds to land at a separate address.
    pub from: cow_sdk_core::Address,
}

impl EthFlowTransaction {
    /// Creates an `EthFlow` transaction bundle from its component pieces.
    ///
    /// `from` is the signer-derived owner and is the identity downstream
    /// submission validates against. `order_to_sign.receiver` remains the
    /// payout recipient and is preserved unchanged.
    #[must_use]
    pub const fn new(
        order_id: cow_sdk_core::OrderUid,
        transaction: TransactionRequest,
        order_to_sign: cow_sdk_core::UnsignedOrder,
        from: cow_sdk_core::Address,
    ) -> Self {
        Self {
            order_id,
            transaction,
            order_to_sign,
            from,
        }
    }
}

/// Builds a pre-sign transaction using a sync signer.
///
/// When gas estimation fails, the helper falls back to the documented default
/// gas limit instead of failing closed.
///
/// ## Gas overhead
///
/// Successful gas estimates receive a 20% overhead using integer floor
/// division: `gas + (gas * 20) / 100`.
///
/// # Errors
///
/// Returns [`TradingError`] when ABI encoding or gas-margin conversion fails.
pub fn get_pre_sign_transaction<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order_uid: &cow_sdk_core::OrderUid,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionRequest, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let settlement = resolve_settlement_address(chain_id, options);
    let mut tx = TransactionRequest::new(
        Some(settlement),
        Some(HexData::new(encode_set_pre_signature(order_uid, true)?)?),
        Some(Amount::zero()),
        None,
    );
    let gas = signer
        .estimate_gas(&tx)
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string().into(),
        });
    let gas_limit = match gas {
        Ok(value) => gas_with_margin(&value)?,
        Err(_) => default_gas_limit(),
    };

    tx.gas_limit = Some(gas_limit);
    Ok(tx)
}

/// Builds a pre-sign transaction using an async signer.
///
/// When gas estimation fails, the helper falls back to the documented default
/// gas limit instead of failing closed.
///
/// ## Gas overhead
///
/// Successful gas estimates receive a 20% overhead using integer floor
/// division: `gas + (gas * 20) / 100`.
///
/// # Errors
///
/// Returns [`TradingError`] when ABI encoding or gas-margin conversion fails.
pub async fn get_pre_sign_transaction_async<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order_uid: &cow_sdk_core::OrderUid,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionRequest, TradingError>
where
    S: AsyncSigner,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let settlement = resolve_settlement_address(chain_id, options);
    let mut tx = TransactionRequest::new(
        Some(settlement),
        Some(HexData::new(encode_set_pre_signature(order_uid, true)?)?),
        Some(Amount::zero()),
        None,
    );
    let gas = signer
        .estimate_gas(&tx)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string().into(),
        });
    let gas_limit = match gas {
        Ok(value) => gas_with_margin(&value)?,
        Err(_) => default_gas_limit(),
    };

    tx.gas_limit = Some(gas_limit);
    Ok(tx)
}

/// Builds an `EthFlow` order-creation transaction using a sync signer.
///
/// ## Gas overhead
///
/// Successful gas estimates receive a 20% overhead using integer floor
/// division: `gas + (gas * 20) / 100`.
///
/// # Errors
///
/// Returns any error from [`get_eth_flow_transaction_async`].
pub async fn get_eth_flow_transaction<S>(
    app_data_keccak256: &cow_sdk_core::AppDataHash,
    params: &crate::LimitTradeParameters,
    chain_id: SupportedChainId,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
) -> Result<EthFlowTransaction, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    get_eth_flow_transaction_async(
        app_data_keccak256,
        params,
        chain_id,
        additional_params,
        trader,
        signer,
    )
    .await
}

/// Builds an `EthFlow` order-creation transaction using an async signer.
///
/// `EthFlow` order ids are generated against the wrapped-native sell token and
/// `MAX_VALID_TO_EPOCH`, then retried by decrementing buy amount until the
/// optional uniqueness checker reports a free id.
///
/// ## Gas overhead
///
/// Successful gas estimates receive a 20% overhead using integer floor
/// division: `gas + (gas * 20) / 100`.
///
/// # Errors
///
/// Returns [`TradingError`] when signer address resolution, transaction
/// encoding, unique-order-id generation, or gas-margin conversion fails.
pub async fn get_eth_flow_transaction_async<S>(
    app_data_keccak256: &cow_sdk_core::AppDataHash,
    params: &crate::LimitTradeParameters,
    chain_id: SupportedChainId,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
) -> Result<EthFlowTransaction, TradingError>
where
    S: AsyncSigner,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let from = signer
        .get_address()
        .await
        .map_err(|error| TradingError::Signer {
            operation: "get_address",
            message: error.to_string().into(),
        })?;
    let owner = from.clone();
    let mut adjusted = crate::adjust_ethflow_limit_parameters(chain_id, params);
    if adjusted.slippage_bps.is_none() {
        adjusted.slippage_bps = Some(crate::default_slippage_bps(chain_id, true));
    }

    let mut options = ProtocolOptions::new();
    if let Some(env) = adjusted.env.or(trader.env) {
        options = options.with_env(env);
    }
    if let Some(overrides) = adjusted
        .settlement_contract_override
        .clone()
        .or_else(|| trader.settlement_contract_override.clone())
    {
        options = options.with_settlement_contract_override(overrides);
    }
    if let Some(overrides) = adjusted
        .eth_flow_contract_override
        .clone()
        .or_else(|| trader.eth_flow_contract_override.clone())
    {
        options = options.with_eth_flow_contract_override(overrides);
    }
    let order_to_sign = get_order_to_sign(
        crate::order::OrderToSignParams {
            chain_id,
            from,
            is_ethflow: true,
            network_costs_amount: additional_params.network_costs_amount.clone(),
            apply_costs_slippage_and_fees: additional_params
                .apply_costs_slippage_and_fees
                .unwrap_or(true),
            protocol_fee_bps: None,
        },
        &adjusted,
        app_data_keccak256,
    )?;
    let generated = calculate_unique_order_id(
        chain_id,
        &order_to_sign,
        additional_params.check_eth_flow_order_exists.as_deref(),
        Some(&options),
    )
    .await?;
    let mut tx = TransactionRequest::new(
        Some(resolve_eth_flow_address(chain_id, Some(&options))),
        Some(HexData::new(encode_ethflow_create_order(
            &order_to_sign,
            adjusted
                .quote_id
                .ok_or(TradingError::MissingQuoteId("EthFlow transaction"))?,
        )?)?),
        Some(order_to_sign.sell_amount.clone()),
        None,
    );
    let gas = signer
        .estimate_gas(&tx)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string().into(),
        });
    let gas_limit = match gas {
        Ok(value) => gas_with_margin(&value)?,
        Err(_) => default_gas_limit(),
    };

    tx.gas_limit = Some(gas_limit);
    Ok(EthFlowTransaction {
        order_id: generated.order_id,
        order_to_sign,
        transaction: tx,
        from: owner,
    })
}

/// Builds an on-chain cancellation transaction using a sync signer.
///
/// Regular orders call the settlement contract. `EthFlow` orders call the
/// `EthFlow` contract. When gas estimation fails, the helper falls back to the
/// documented default gas limit.
///
/// # Errors
///
/// Returns [`TradingError`] when ABI encoding or gas conversion fails.
pub fn onchain_cancellation_transaction<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionRequest, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let mut tx = if order.ethflow_data.is_some() {
        TransactionRequest::new(
            Some(resolve_eth_flow_address(chain_id, options)),
            Some(HexData::new(encode_ethflow_invalidate_order(order)?)?),
            Some(Amount::zero()),
            None,
        )
    } else {
        TransactionRequest::new(
            Some(resolve_settlement_address(chain_id, options)),
            Some(HexData::new(encode_invalidate_order_uid(&order.uid)?)?),
            Some(Amount::zero()),
            None,
        )
    };
    let gas = signer
        .estimate_gas(&tx)
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string().into(),
        });
    tx.gas_limit = Some(match gas {
        Ok(value) => Amount::new(parse_integer("gas", &value.to_string())?.to_string())?,
        Err(_) => default_gas_limit(),
    });
    Ok(tx)
}

/// Builds an on-chain cancellation transaction using an async signer.
///
/// Regular orders call the settlement contract. `EthFlow` orders call the
/// `EthFlow` contract. When gas estimation fails, the helper falls back to the
/// documented default gas limit.
///
/// # Errors
///
/// Returns [`TradingError`] when ABI encoding or gas conversion fails.
pub async fn onchain_cancellation_transaction_async<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionRequest, TradingError>
where
    S: AsyncSigner,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let mut tx = if order.ethflow_data.is_some() {
        TransactionRequest::new(
            Some(resolve_eth_flow_address(chain_id, options)),
            Some(HexData::new(encode_ethflow_invalidate_order(order)?)?),
            Some(Amount::zero()),
            None,
        )
    } else {
        TransactionRequest::new(
            Some(resolve_settlement_address(chain_id, options)),
            Some(HexData::new(encode_invalidate_order_uid(&order.uid)?)?),
            Some(Amount::zero()),
            None,
        )
    };
    let gas = signer
        .estimate_gas(&tx)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string().into(),
        });
    tx.gas_limit = Some(match gas {
        Ok(value) => Amount::new(parse_integer("gas", &value.to_string())?.to_string())?,
        Err(_) => default_gas_limit(),
    });
    Ok(tx)
}

/// Cancels an order on-chain using a sync signer.
///
/// # Errors
///
/// Returns [`TradingError`] when transaction construction or submission fails.
pub fn cancel_order_onchain<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionHash, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let tx = onchain_cancellation_transaction(signer, chain_id, order, options)?;
    let broadcast = signer
        .send_transaction(&tx)
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string().into(),
        })?;
    Ok(broadcast.transaction_hash)
}

/// Cancels an order on-chain using an async signer.
///
/// # Errors
///
/// Returns [`TradingError`] when transaction construction or submission fails.
pub async fn cancel_order_onchain_async<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionHash, TradingError>
where
    S: AsyncSigner,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let tx = onchain_cancellation_transaction_async(signer, chain_id, order, options).await?;
    let broadcast = signer
        .send_transaction(&tx)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string().into(),
        })?;
    Ok(broadcast.transaction_hash)
}

/// Resolves protocol options for an order-level workflow.
///
/// Call-level order params take precedence over trader defaults for environment
/// and contract overrides.
#[must_use]
pub fn protocol_options_for_order(
    params: &OrderTraderParameters,
    trader: &TraderParameters,
) -> ProtocolOptions {
    protocol_options_for_partial_order(
        params,
        &PartialTraderParameters {
            chain_id: Some(trader.chain_id),
            app_code: Some(trader.app_code.clone()),
            owner: None,
            env: trader.env,
            settlement_contract_override: trader.settlement_contract_override.clone(),
            eth_flow_contract_override: trader.eth_flow_contract_override.clone(),
        },
    )
}

/// Resolves protocol options for an order-level workflow that only needs
/// chain-bound protocol context.
#[must_use]
pub(crate) fn protocol_options_for_partial_order(
    params: &OrderTraderParameters,
    trader: &PartialTraderParameters,
) -> ProtocolOptions {
    let mut options = ProtocolOptions::new();
    if let Some(env) = params.env.or(trader.env) {
        options = options.with_env(env);
    }
    if let Some(overrides) = params
        .settlement_contract_override
        .clone()
        .or_else(|| trader.settlement_contract_override.clone())
    {
        options = options.with_settlement_contract_override(overrides);
    }
    if let Some(overrides) = params
        .eth_flow_contract_override
        .clone()
        .or_else(|| trader.eth_flow_contract_override.clone())
    {
        options = options.with_eth_flow_contract_override(overrides);
    }
    options
}

/// Resolves the settlement address for on-chain helper calls.
///
/// # Panics
///
/// Panics only if the embedded deployment registry is missing the canonical
/// settlement entry for a supported chain/environment pair.
fn resolve_settlement_address(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Address {
    options
        .and_then(|opts| opts.settlement_contract_override.as_ref())
        .and_then(|override_map| override_map.get(&u64::from(chain_id)).cloned())
        .unwrap_or_else(|| {
            let env = options
                .and_then(|opts| opts.env)
                .unwrap_or(cow_sdk_core::CowEnv::Prod);
            // SAFETY: Registry::default parses the build-validated embedded
            // manifest, which must include settlement addresses for supported
            // chain/environment pairs.
            Registry::default()
                .address(ContractId::Settlement, chain_id, env)
                .expect("canonical settlement address is registered for every supported chain/env")
        })
}

/// Resolves the `EthFlow` address for on-chain helper calls.
///
/// # Panics
///
/// Panics only if the embedded deployment registry is missing the canonical
/// `EthFlow` entry for a supported chain/environment pair.
fn resolve_eth_flow_address(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Address {
    options
        .and_then(|opts| opts.eth_flow_contract_override.as_ref())
        .and_then(|override_map| override_map.get(&u64::from(chain_id)).cloned())
        .unwrap_or_else(|| {
            let env = options
                .and_then(|opts| opts.env)
                .unwrap_or(cow_sdk_core::CowEnv::Prod);
            // SAFETY: Registry::default parses the build-validated embedded
            // manifest, which must include EthFlow addresses for supported
            // chain/environment pairs.
            Registry::default()
                .address(ContractId::EthFlow, chain_id, env)
                .expect("canonical EthFlow address is registered for every supported chain/env")
        })
}

/// Returns the default on-chain helper gas limit as a typed amount.
///
/// # Panics
///
/// Panics only if the crate-owned decimal gas-limit literal stops fitting the
/// SDK amount validator.
fn default_gas_limit() -> Amount {
    // SAFETY: GAS_LIMIT_DEFAULT is a small static decimal literal that remains
    // within the supported amount range.
    Amount::new(GAS_LIMIT_DEFAULT.to_string()).expect("static gas limit literal must remain valid")
}

fn encode_set_pre_signature(
    order_uid: &cow_sdk_core::OrderUid,
    enabled: bool,
) -> Result<String, TradingError> {
    let call = IGPv2Settlement::setPreSignatureCall {
        orderUid: order_uid_bytes(order_uid)?,
        signed: enabled,
    };
    Ok(format!("0x{}", hex::encode(call.abi_encode())))
}

fn encode_invalidate_order_uid(order_uid: &cow_sdk_core::OrderUid) -> Result<String, TradingError> {
    let call = IGPv2Settlement::invalidateOrderCall {
        orderUid: order_uid_bytes(order_uid)?,
    };
    Ok(format!("0x{}", hex::encode(call.abi_encode())))
}

fn order_uid_bytes(order_uid: &cow_sdk_core::OrderUid) -> Result<AlloyBytes, TradingError> {
    AlloyBytes::from_str(order_uid.as_str()).map_err(|_| TradingError::InvalidNumeric {
        field: "orderUid",
        value: order_uid.as_str().to_owned().into(),
    })
}

fn encode_ethflow_create_order(
    order: &cow_sdk_core::UnsignedOrder,
    quote_id: i64,
) -> Result<String, TradingError> {
    let payload = EthFlowOrderData::from_unsigned_order(order, quote_id);
    let encoded = encode_create_order_calldata(&payload)?;
    Ok(format!("0x{}", hex::encode(encoded)))
}

fn encode_ethflow_invalidate_order(order: &Order) -> Result<String, TradingError> {
    let receiver = order
        .receiver
        .clone()
        .unwrap_or_else(|| order.owner.clone());
    let payload = EthFlowOrderData::new(
        order.buy_token.clone(),
        receiver,
        order.sell_amount.clone(),
        order.buy_amount.clone(),
        order.app_data.clone(),
        Amount::zero(),
        order.valid_to,
        false,
        0,
    );
    let encoded = encode_invalidate_order_calldata(&payload)?;
    Ok(format!("0x{}", hex::encode(encoded)))
}
