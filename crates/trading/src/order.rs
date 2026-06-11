use alloy_primitives::aliases::I512;

use cow_sdk_contracts::{ContractId, Registry};
use cow_sdk_core::{
    Address, Amount, AppDataHash, CowEnv, MAX_VALID_TO_EPOCH, NATIVE_CURRENCY_ADDRESS, OrderData,
    ProtocolOptions, SupportedChainId, ValidTo, wrapped_native_token,
};
use cow_sdk_orderbook::OrderQuoteResponse;
use cow_sdk_signing::{GeneratedOrderId, generate_order_id};

use crate::slippage::parse_integer;
use crate::{
    DEFAULT_QUOTE_VALIDITY, EthFlowOrderExistsChecker, LimitTradeParams, LimitTradeParamsFromQuote,
    TradeParams, TradingError, calculate_quote_amounts_and_costs, default_slippage_bps,
    partner_fee_bps,
};

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_arch = "wasm32")]
use web_time::{SystemTime, UNIX_EPOCH};

/// Inputs that control how an unsigned order is derived for signing or posting.
///
/// Every field is `Copy`-bounded (`SupportedChainId`, `Address`, the cow
/// `Amount` newtype, `bool`, `f64`, and the `Option` wrappers thereof),
/// so the struct is `Copy` itself. The public helper
/// [`order_to_sign`] therefore takes the typed bag by value without
/// the usual pass-by-reference dance — calling code composes the struct
/// literal at the call site and the by-value move is bit-for-bit free.
#[derive(Debug, Clone, Copy)]
pub struct OrderToSignParams {
    /// Active chain id.
    pub chain_id: SupportedChainId,
    /// Effective owner.
    pub from: Address,
    /// Whether the flow is building an `EthFlow` order.
    pub is_eth_flow: bool,
    /// Optional network cost amount folded into amount calculations.
    pub network_costs_amount: Option<Amount>,
    /// Whether costs, slippage, and fees should be applied to the final order payload.
    pub apply_costs_slippage_and_fees: bool,
    /// Optional protocol-fee value used during amount calculation.
    pub protocol_fee_bps: Option<f64>,
}

impl OrderToSignParams {
    /// Creates an order-signing input with the required identity fields.
    ///
    /// `apply_costs_slippage_and_fees` defaults to `true` so the public helper
    /// folds cost, slippage, partner-fee, and protocol-fee adjustments into the
    /// unsigned order amounts in the same shape the internal quote and
    /// submission flows produce. Callers that want raw-amount payloads must
    /// opt out explicitly by calling
    /// [`OrderToSignParams::with_apply_costs_slippage_and_fees`] with `false`.
    #[must_use]
    pub const fn new(chain_id: SupportedChainId, from: Address, is_eth_flow: bool) -> Self {
        Self {
            chain_id,
            from,
            is_eth_flow,
            network_costs_amount: None,
            apply_costs_slippage_and_fees: true,
            protocol_fee_bps: None,
        }
    }

    /// Returns a copy with an explicit network-cost amount.
    #[must_use]
    pub const fn with_network_costs_amount(mut self, amount: Amount) -> Self {
        self.network_costs_amount = Some(amount);
        self
    }

    /// Returns a copy with the cost/slippage/fee application flag set.
    #[must_use]
    pub const fn with_apply_costs_slippage_and_fees(mut self, apply: bool) -> Self {
        self.apply_costs_slippage_and_fees = apply;
        self
    }

    /// Returns a copy with an explicit protocol-fee value.
    #[must_use]
    pub const fn with_protocol_fee_bps(mut self, protocol_fee_bps: f64) -> Self {
        self.protocol_fee_bps = Some(protocol_fee_bps);
        self
    }
}

impl LimitTradeParams {
    /// Resolves the order expiration into a typed [`ValidTo`].
    ///
    /// `valid_to` wins when present; otherwise `valid_for` is combined with
    /// the supplied `now_epoch_seconds` through [`ValidTo::relative`]. Returns
    /// `Ok(None)` when neither field is configured so callers can apply their
    /// own default before signing.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InvalidInput`] when a relative `valid_for`
    /// value falls outside the supported duration window.
    pub fn valid_to_typed(&self, now_epoch_seconds: u64) -> Result<Option<ValidTo>, TradingError> {
        if let Some(absolute) = self.valid_to {
            return Ok(Some(ValidTo::absolute(absolute)));
        }
        self.valid_for.map_or(Ok(None), |duration| {
            ValidTo::relative(now_epoch_seconds, u64::from(duration))
                .map(Some)
                .map_err(|_| TradingError::InvalidInput {
                    field: "validFor",
                    reason: cow_sdk_core::ValidationReason::OutOfRange {
                        details: "relative valid_for window must sit inside the supported bounds",
                    },
                })
        })
    }
}

/// Returns `true` when `sell_token` is the protocol native-asset sentinel address.
#[must_use]
pub fn is_eth_flow_order(sell_token: &Address) -> bool {
    *sell_token == NATIVE_CURRENCY_ADDRESS
}

/// Rewrites a swap trade to use the wrapped-native token for `EthFlow` quoting.
#[must_use]
pub fn adjust_eth_flow_trade_params(
    chain_id: SupportedChainId,
    trade_parameters: &TradeParams,
) -> TradeParams {
    let mut adjusted = trade_parameters.clone();
    adjusted.sell_token = wrapped_native_token(chain_id).address;
    adjusted
}

/// Rewrites a limit-order request to use the wrapped-native token for `EthFlow` posting.
#[must_use]
pub fn adjust_eth_flow_limit_params(
    chain_id: SupportedChainId,
    limit_parameters: &LimitTradeParams,
) -> LimitTradeParams {
    let mut adjusted = limit_parameters.clone();
    adjusted.sell_token = wrapped_native_token(chain_id).address;
    adjusted
}

/// Converts swap-style trade params plus a quote response into the
/// from-quote limit-order shape.
///
/// The returned [`LimitTradeParamsFromQuote`] is the typed
/// guarantee that the `quote_id` field is present; downstream
/// `EthFlow` entries require this newtype on their public boundary.
///
/// # Errors
///
/// Returns [`TradingError::MissingQuoteId`] when the orderbook quote
/// response does not carry an identifier.
pub fn swap_params_to_limit_order_params(
    trade_parameters: &TradeParams,
    quote_response: &OrderQuoteResponse,
) -> Result<LimitTradeParamsFromQuote, TradingError> {
    let inner = LimitTradeParams {
        kind: trade_parameters.kind,
        owner: trade_parameters.owner,
        sell_token: trade_parameters.sell_token,
        buy_token: trade_parameters.buy_token,
        sell_amount: quote_response.quote.sell_amount,
        buy_amount: quote_response.quote.buy_amount,
        quote_id: quote_response.id,
        env: trade_parameters.env,
        settlement_contract_override: trade_parameters.settlement_contract_override.clone(),
        eth_flow_contract_override: trade_parameters.eth_flow_contract_override.clone(),
        partially_fillable: trade_parameters.partially_fillable,
        // The signed order binds the balance sources the caller requested, not
        // the response echo. `OrderbookApi::quote` already proved the response
        // echoed these (ADR 0058), so this is byte-identical under an honest
        // orderbook and the authority is the caller's request rather than the
        // wire.
        sell_token_balance: trade_parameters.sell_token_balance,
        buy_token_balance: trade_parameters.buy_token_balance,
        slippage_bps: trade_parameters.slippage_bps,
        receiver: trade_parameters.receiver,
        valid_for: trade_parameters.valid_for,
        valid_to: trade_parameters.valid_to,
        partner_fee: trade_parameters.partner_fee.clone(),
    };
    LimitTradeParamsFromQuote::try_from_limit(inner)
}

/// Builds the unsigned order payload used for signing or on-chain helpers.
///
/// Relative validity uses `DEFAULT_QUOTE_VALIDITY` when neither `valid_for` nor
/// `valid_to` is provided. When `apply_costs_slippage_and_fees` is enabled, the
/// helper recomputes amounts from the public fee/slippage contract before
/// building the final order.
///
/// # Errors
///
/// Returns [`TradingError`] when amount calculation, local time resolution, or typed value
/// conversion fails.
///
/// # Panics
///
/// Panics only if the internally clamped validity timestamp no longer fits into `u32`.
/// The implementation clamps it to the supported `u32` range before conversion.
pub fn order_to_sign(
    params: OrderToSignParams,
    limit_parameters: &LimitTradeParams,
    app_data_keccak256: &AppDataHash,
) -> Result<OrderData, TradingError> {
    let network_costs_amount = params.network_costs_amount.unwrap_or(Amount::ZERO);
    let receiver = limit_parameters
        .receiver
        .filter(|receiver| !is_zero_address(receiver))
        .unwrap_or(params.from);
    let valid_to = if let Some(valid_to) = limit_parameters.valid_to {
        valid_to
    } else {
        let valid_for = limit_parameters.valid_for.unwrap_or(DEFAULT_QUOTE_VALIDITY);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TradingError::InvalidInput {
                field: "systemTime",
                reason: cow_sdk_core::ValidationReason::Precondition {
                    details: "system time must not be earlier than the unix epoch",
                },
            })?
            .as_secs();
        let clamped_valid_to = now
            .saturating_add(u64::from(valid_for))
            .min(u64::from(MAX_VALID_TO_EPOCH));
        // SAFETY: clamped_valid_to is explicitly capped at MAX_VALID_TO_EPOCH,
        // which is a u32 value.
        u32::try_from(clamped_valid_to)
            .expect("validity timestamp is clamped to the supported `u32` range")
    };

    let slippage_bps = limit_parameters
        .slippage_bps
        .unwrap_or_else(|| default_slippage_bps(params.chain_id, params.is_eth_flow));
    let (sell_amount_to_use, buy_amount_to_use) = if params.apply_costs_slippage_and_fees {
        let quote = cow_sdk_orderbook::QuoteData::new(
            limit_parameters.sell_token,
            limit_parameters.buy_token,
            limit_parameters.sell_amount,
            limit_parameters.buy_amount,
            valid_to,
            *app_data_keccak256,
            limit_parameters.kind,
        )
        .with_network_cost_amount(network_costs_amount)
        .with_receiver(receiver)
        .with_partially_fillable(limit_parameters.partially_fillable)
        .with_sell_token_balance(limit_parameters.sell_token_balance)
        .with_buy_token_balance(limit_parameters.buy_token_balance);
        let amounts = calculate_quote_amounts_and_costs(
            &quote,
            slippage_bps,
            partner_fee_bps(limit_parameters.partner_fee.as_ref()),
            params.protocol_fee_bps,
        )?;
        let sell_amount = if amounts.is_sell {
            amounts.before_all_fees.sell_amount
        } else {
            amounts.after_slippage.sell_amount
        };
        let buy_amount = if amounts.is_sell {
            amounts.after_slippage.buy_amount
        } else {
            amounts.before_all_fees.buy_amount
        };
        (sell_amount, buy_amount)
    } else {
        (limit_parameters.sell_amount, limit_parameters.buy_amount)
    };

    Ok(OrderData::new(
        limit_parameters.sell_token,
        limit_parameters.buy_token,
        receiver,
        sell_amount_to_use,
        buy_amount_to_use,
        valid_to,
        *app_data_keccak256,
        Amount::ZERO,
        limit_parameters.kind,
        limit_parameters.partially_fillable,
        limit_parameters.sell_token_balance,
        limit_parameters.buy_token_balance,
    ))
}

/// Generates a unique `EthFlow` order id, retrying by decrementing buy amount.
///
/// The helper normalizes the order for `EthFlow` id generation by fixing
/// `valid_to` to `MAX_VALID_TO_EPOCH` and replacing the sell token with the
/// wrapped-native token for `chain_id`.
///
/// # Errors
///
/// Returns [`TradingError`] when id generation fails, the optional checker
/// fails, or the buy amount can no longer be decremented safely.
///
/// # Panics
///
/// Panics if the embedded deployment registry is missing the canonical
/// `EthFlow` contract entry for the resolved chain and environment. The
/// shipped registry manifest is validated at compile time, so this panic
/// cannot be reached from an unmodified binary.
pub async fn calculate_unique_order_id(
    chain_id: SupportedChainId,
    order: &OrderData,
    checker: Option<&dyn EthFlowOrderExistsChecker>,
    options: Option<&ProtocolOptions>,
) -> Result<GeneratedOrderId, TradingError> {
    let owner = options
        .and_then(|opts| opts.eth_flow_contract_override.as_ref())
        .and_then(|override_map| override_map.get(&u64::from(chain_id)).copied())
        .unwrap_or_else(|| {
            let env = options.and_then(|opts| opts.env).unwrap_or(CowEnv::Prod);
            // SAFETY: Registry::default parses the build-validated embedded
            // manifest, which must include EthFlow addresses for supported
            // chain/environment pairs.
            Registry::default()
                .address(ContractId::EthFlow, chain_id, env)
                .expect("canonical EthFlow address is registered for every supported chain/env")
        });
    let mut current = order.clone();

    let Some(checker) = checker else {
        let mut order_for_id = current;
        order_for_id.valid_to = MAX_VALID_TO_EPOCH;
        order_for_id.sell_token = wrapped_native_token(chain_id).address;
        return generate_order_id(chain_id, &order_for_id, &owner, options).map_err(Into::into);
    };

    loop {
        let mut order_for_id = current.clone();
        order_for_id.valid_to = MAX_VALID_TO_EPOCH;
        order_for_id.sell_token = wrapped_native_token(chain_id).address;

        let generated = generate_order_id(chain_id, &order_for_id, &owner, options)?;
        if checker
            .order_exists(&generated.order_id, &generated.order_digest)
            .await?
        {
            current.buy_amount = adjust_buy_amount(&current.buy_amount)?;
            continue;
        }

        return Ok(generated);
    }
}

fn adjust_buy_amount(value: &Amount) -> Result<Amount, TradingError> {
    let amount = parse_integer("buyAmount", &value.to_string())?;
    if amount <= I512::ZERO {
        return Err(TradingError::InvalidInput {
            field: "buyAmount",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "buyAmount must be greater than 0",
            },
        });
    }
    Amount::new((amount - I512::ONE).to_string()).map_err(Into::into)
}

fn is_zero_address(address: &Address) -> bool {
    address.is_zero()
}
