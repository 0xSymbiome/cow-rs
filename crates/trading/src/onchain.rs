use cow_sdk_contracts::eth_flow::{EthFlowOrderData, encode_invalidate_order_calldata};
use cow_sdk_core::{
    Address, AddressPerChain, Amount, HexData, ProtocolOptions, Signer, SupportedChainId,
    TransactionHash, TransactionRequest,
};
use cow_sdk_orderbook::Order;

use crate::slippage::gas_with_margin;
use crate::{
    DEFAULT_GAS_LIMIT, OrderTraderParams, PartialTraderParams, TraderParams, TradingError,
    calculate_unique_order_id, order_to_sign,
};

/// Fully populated transaction produced by the on-chain helper flows.
///
/// Unlike [`TransactionRequest`] — the optional-field wire shape accepted by
/// [`Signer`] backends — every field here is unconditionally set by the
/// producing helper, so consumers read `to`, `data`, `value`, and `gas_limit`
/// directly instead of unwrapping SDK output. Convert with `.into()` when
/// handing the transaction to a submission seam such as
/// [`Signer::send_transaction`] or
/// [`submit_and_wait_for_receipt`](crate::submit_and_wait_for_receipt).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PreparedTransaction {
    /// Destination contract address.
    pub to: Address,
    /// Hex-encoded calldata payload.
    pub data: HexData,
    /// Native token value to transfer.
    pub value: Amount,
    /// Gas limit, either margin-adjusted from an estimate or the documented
    /// default fallback.
    pub gas_limit: Amount,
}

impl PreparedTransaction {
    /// Creates a prepared transaction from its component fields.
    #[must_use]
    pub const fn new(to: Address, data: HexData, value: Amount, gas_limit: Amount) -> Self {
        Self {
            to,
            data,
            value,
            gas_limit,
        }
    }
}

impl From<PreparedTransaction> for TransactionRequest {
    fn from(prepared: PreparedTransaction) -> Self {
        Self::new(
            Some(prepared.to),
            Some(prepared.data),
            Some(prepared.value),
            Some(prepared.gas_limit),
        )
    }
}

/// `EthFlow` transaction bundle returned by native-sell helper flows.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct EthFlowTransaction {
    /// Final unique order id.
    pub order_id: cow_sdk_core::OrderUid,
    /// Prepared transaction to submit.
    pub transaction: PreparedTransaction,
    /// Unsigned order payload used to derive `order_id` and the transaction body.
    pub order_to_sign: cow_sdk_core::OrderData,
    /// Signer-derived owner resolved at transaction construction via
    /// [`Signer::address`].
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
        transaction: PreparedTransaction,
        order_to_sign: cow_sdk_core::OrderData,
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

/// Builds a pre-sign transaction.
///
/// The returned [`PreparedTransaction`] targets the settlement contract with
/// a `setPreSignature` calldata payload and zero native value. When gas
/// estimation fails, the helper falls back to the documented default gas
/// limit instead of failing closed.
///
/// ## Gas overhead
///
/// Successful gas estimates receive a 20% overhead using integer floor
/// division: `gas + (gas * 20) / 100`.
///
/// # Errors
///
/// Returns [`TradingError`] when ABI encoding or gas-margin conversion fails.
pub async fn pre_sign_transaction<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order_uid: &cow_sdk_core::OrderUid,
    options: Option<&ProtocolOptions>,
) -> Result<PreparedTransaction, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
{
    let unsigned = cow_sdk_contracts::pre_sign_transaction(order_uid, chain_id, options)?;
    let gas_limit =
        gas_limit_with_margin_or_default(signer, &TransactionRequest::from(&unsigned)).await?;

    Ok(PreparedTransaction::new(
        unsigned.to,
        unsigned.data,
        unsigned.value,
        gas_limit,
    ))
}

/// Estimates gas for `request` and applies the documented 20% floor-division
/// margin, falling back to the crate default gas limit when estimation fails.
async fn gas_limit_with_margin_or_default<S>(
    signer: &S,
    request: &TransactionRequest,
) -> Result<Amount, TradingError>
where
    S: Signer,
{
    signer.estimate_gas(request).await.map_or_else(
        |_| Ok(default_gas_limit()),
        |estimate| gas_with_margin(&estimate),
    )
}

/// Builds an `EthFlow` order-creation transaction.
///
/// Chain authority comes from [`TraderParams::chain_id`]; the trader value is
/// the single source of truth for chain resolution in this helper.
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
pub async fn eth_flow_transaction<S>(
    app_data_keccak256: &cow_sdk_core::AppDataHash,
    params: &crate::LimitTradeParamsFromQuote,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParams,
    signer: &S,
) -> Result<EthFlowTransaction, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
{
    let chain_id = trader.chain_id;
    let from = signer
        .address()
        .await
        .map_err(|error| TradingError::Signer {
            operation: "address",
            message: error.to_string().into(),
        })?;
    let quote_id = params.quote_id();
    let mut adjusted = crate::adjust_eth_flow_limit_params(chain_id, params.as_limit());
    if adjusted.slippage_bps.is_none() {
        adjusted.slippage_bps = Some(crate::default_slippage_bps(chain_id, true));
    }

    let options = protocol_options(
        adjusted.env.or(trader.env),
        adjusted.settlement_contract_override.as_ref(),
        trader.settlement_contract_override.as_ref(),
        adjusted.eth_flow_contract_override.as_ref(),
        trader.eth_flow_contract_override.as_ref(),
    );
    let order_to_sign = order_to_sign(
        crate::order::OrderToSignParams {
            chain_id,
            from,
            is_eth_flow: true,
            network_costs_amount: additional_params.network_costs_amount,
            apply_costs_slippage_and_fees: additional_params
                .apply_costs_slippage_and_fees
                .unwrap_or(true),
            protocol_fee_bps: additional_params.protocol_fee_bps,
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
    let unsigned = cow_sdk_contracts::ethflow_create_order_transaction(
        &order_to_sign,
        quote_id,
        chain_id,
        Some(&options),
    )?;
    let gas_limit =
        gas_limit_with_margin_or_default(signer, &TransactionRequest::from(&unsigned)).await?;

    Ok(EthFlowTransaction {
        order_id: generated.order_id,
        order_to_sign,
        transaction: PreparedTransaction::new(
            unsigned.to,
            unsigned.data,
            unsigned.value,
            gas_limit,
        ),
        from,
    })
}

/// Builds an on-chain cancellation transaction.
///
/// Regular orders call the settlement contract. `EthFlow` orders call the
/// `EthFlow` contract. When gas estimation fails, the helper falls back to the
/// documented default gas limit.
///
/// # Errors
///
/// Returns [`TradingError`] when ABI encoding or gas conversion fails.
pub async fn onchain_cancellation_transaction<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionRequest, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
{
    let mut tx = if order.ethflow_data.is_some() {
        let eth_flow = cow_sdk_contracts::resolve_eth_flow_address(chain_id, options).ok_or(
            cow_sdk_contracts::ContractsError::DeploymentNotFound {
                contract: "eth-flow",
                chain_id: u64::from(chain_id),
            },
        )?;
        TransactionRequest::new(
            Some(eth_flow),
            Some(HexData::new(encode_ethflow_invalidate_order(order)?)?),
            Some(Amount::ZERO),
            None,
        )
    } else {
        cow_sdk_contracts::invalidate_order_transaction(&order.uid, chain_id, options)?.into()
    };
    tx.gas_limit = Some(
        signer
            .estimate_gas(&tx)
            .await
            .ok()
            .unwrap_or_else(default_gas_limit),
    );
    Ok(tx)
}

/// Cancels an order on-chain.
///
/// # Errors
///
/// Returns [`TradingError`] when transaction construction or submission fails.
pub async fn onchain_cancel_order<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionHash, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
{
    let tx = onchain_cancellation_transaction(signer, chain_id, order, options).await?;
    let broadcast = signer
        .send_transaction(&tx)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string().into(),
        })?;
    Ok(broadcast.transaction_hash)
}

/// Assembles [`ProtocolOptions`] from an already-resolved environment and a
/// call-level / trader override pair for each contract, applying call-level
/// precedence: the call value wins and the trader value is the fallback.
///
/// Callers resolve the environment themselves — some pass the canonical
/// orderbook environment, some fall a call-level value back to the trader
/// default — so this helper owns only the contract-override precedence rule
/// shared across the quote, post, eth-flow, and cancellation lanes.
pub(crate) fn protocol_options(
    env: Option<cow_sdk_core::CowEnv>,
    settlement_primary: Option<&AddressPerChain>,
    settlement_fallback: Option<&AddressPerChain>,
    eth_flow_primary: Option<&AddressPerChain>,
    eth_flow_fallback: Option<&AddressPerChain>,
) -> ProtocolOptions {
    let mut options = ProtocolOptions::new();
    if let Some(env) = env {
        options = options.with_env(env);
    }
    if let Some(overrides) = settlement_primary.or(settlement_fallback) {
        options = options.with_settlement_contract_override(overrides.clone());
    }
    if let Some(overrides) = eth_flow_primary.or(eth_flow_fallback) {
        options = options.with_eth_flow_contract_override(overrides.clone());
    }
    options
}

/// Resolves protocol options for an order-level workflow that only needs
/// chain-bound protocol context.
#[must_use]
pub(crate) fn protocol_options_for_partial_order(
    params: &OrderTraderParams,
    trader: &PartialTraderParams,
) -> ProtocolOptions {
    protocol_options(
        params.env.or(trader.env),
        params.settlement_contract_override.as_ref(),
        trader.settlement_contract_override.as_ref(),
        params.eth_flow_contract_override.as_ref(),
        trader.eth_flow_contract_override.as_ref(),
    )
}

/// Returns the default on-chain helper gas limit as a typed amount.
///
/// # Panics
///
/// Panics only if the crate-owned decimal gas-limit literal stops fitting the
/// SDK amount validator.
fn default_gas_limit() -> Amount {
    // SAFETY: DEFAULT_GAS_LIMIT is a small static decimal literal that remains
    // within the supported amount range.
    Amount::new(DEFAULT_GAS_LIMIT.to_string()).expect("static gas limit literal must remain valid")
}

fn encode_ethflow_invalidate_order(order: &Order) -> Result<String, TradingError> {
    let receiver = order.receiver.unwrap_or(order.owner);
    let payload = EthFlowOrderData::new(
        order.buy_token,
        receiver,
        order.sell_amount,
        order.buy_amount,
        order.app_data,
        Amount::ZERO,
        order.valid_to,
        false,
        0,
    )?;
    Ok(alloy_primitives::hex::encode_prefixed(
        encode_invalidate_order_calldata(&payload),
    ))
}
