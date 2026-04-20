use num_bigint::BigInt;

use cow_sdk_core::{
    Address, Amount, AppDataHash, AtomAmount, CowEnv, EVM_NATIVE_CURRENCY_ADDRESS,
    MAX_VALID_TO_EPOCH, ProtocolOptions, SupportedChainId, UnsignedOrder, ValidTo,
    eth_flow_contract_address, wrapped_native_token,
};
use cow_sdk_orderbook::OrderQuoteResponse;
use cow_sdk_signing::{GeneratedOrderId, generate_order_id};

use crate::slippage::parse_integer;
use crate::{
    DEFAULT_QUOTE_VALIDITY, EthFlowOrderExistsChecker, LimitTradeParameters, TradeParameters,
    TradingError, calculate_quote_amounts_and_costs, default_slippage_bps, partner_fee_bps,
};

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_arch = "wasm32")]
use web_time::{SystemTime, UNIX_EPOCH};

/// Inputs that control how an unsigned order is derived for signing or posting.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct OrderToSignParams {
    /// Active chain id.
    pub chain_id: SupportedChainId,
    /// Effective owner.
    pub from: Address,
    /// Whether the flow is building an `EthFlow` order.
    pub is_ethflow: bool,
    /// Optional network cost amount folded into amount calculations.
    pub network_costs_amount: Option<Amount>,
    /// Whether costs, slippage, and fees should be applied to the final order payload.
    pub apply_costs_slippage_and_fees: bool,
    /// Optional protocol-fee value used during amount calculation.
    pub protocol_fee_bps: Option<f64>,
}

impl OrderToSignParams {
    /// Creates an order-signing input with the required identity fields.
    #[must_use]
    pub const fn new(chain_id: SupportedChainId, from: Address, is_ethflow: bool) -> Self {
        Self {
            chain_id,
            from,
            is_ethflow,
            network_costs_amount: None,
            apply_costs_slippage_and_fees: false,
            protocol_fee_bps: None,
        }
    }

    /// Returns a copy with an explicit network-cost amount.
    #[must_use]
    pub fn with_network_costs_amount(mut self, amount: Amount) -> Self {
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

impl LimitTradeParameters {
    /// Returns the sell amount as a typed [`AtomAmount`].
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InvalidInput`] when the stored wire-format
    /// sell amount cannot be parsed into the supported `uint256` range.
    pub fn sell_atom_amount(&self) -> Result<AtomAmount, TradingError> {
        AtomAmount::try_from(&self.sell_amount)
            .map_err(|err| TradingError::InvalidInput(err.to_string()))
    }

    /// Returns the buy amount as a typed [`AtomAmount`].
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InvalidInput`] when the stored wire-format
    /// buy amount cannot be parsed into the supported `uint256` range.
    pub fn buy_atom_amount(&self) -> Result<AtomAmount, TradingError> {
        AtomAmount::try_from(&self.buy_amount)
            .map_err(|err| TradingError::InvalidInput(err.to_string()))
    }

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
                .map_err(|err| TradingError::InvalidInput(err.to_string()))
        })
    }
}

/// Returns `true` when `sell_token` is the protocol native-asset sentinel address.
#[must_use]
pub fn is_ethflow_order(sell_token: &Address) -> bool {
    sell_token
        .as_str()
        .eq_ignore_ascii_case(EVM_NATIVE_CURRENCY_ADDRESS)
}

/// Rewrites a swap trade to use the wrapped-native token for `EthFlow` quoting.
#[must_use]
pub fn adjust_ethflow_trade_parameters(
    chain_id: SupportedChainId,
    trade_parameters: &TradeParameters,
) -> TradeParameters {
    let mut adjusted = trade_parameters.clone();
    adjusted.sell_token = wrapped_native_token(chain_id).address;
    adjusted
}

/// Rewrites a limit-order request to use the wrapped-native token for `EthFlow` posting.
#[must_use]
pub fn adjust_ethflow_limit_parameters(
    chain_id: SupportedChainId,
    limit_parameters: &LimitTradeParameters,
) -> LimitTradeParameters {
    let mut adjusted = limit_parameters.clone();
    adjusted.sell_token = wrapped_native_token(chain_id).address;
    adjusted
}

/// Converts swap-style trade params plus a quote response into limit-order params.
///
/// # Errors
///
/// Returns [`TradingError`] when quoted string amounts cannot be converted into
/// typed [`Amount`] values.
pub fn swap_params_to_limit_order_params(
    trade_parameters: &TradeParameters,
    quote_response: &OrderQuoteResponse,
) -> Result<LimitTradeParameters, TradingError> {
    Ok(LimitTradeParameters {
        kind: trade_parameters.kind,
        owner: trade_parameters.owner.clone(),
        sell_token: trade_parameters.sell_token.clone(),
        sell_token_decimals: trade_parameters.sell_token_decimals,
        buy_token: trade_parameters.buy_token.clone(),
        buy_token_decimals: trade_parameters.buy_token_decimals,
        sell_amount: Amount::new(quote_response.quote.sell_amount.clone())?,
        buy_amount: Amount::new(quote_response.quote.buy_amount.clone())?,
        quote_id: quote_response.id,
        env: trade_parameters.env,
        settlement_contract_override: trade_parameters.settlement_contract_override.clone(),
        eth_flow_contract_override: trade_parameters.eth_flow_contract_override.clone(),
        partially_fillable: trade_parameters.partially_fillable,
        sell_token_balance: quote_response.quote.sell_token_balance,
        buy_token_balance: quote_response.quote.buy_token_balance,
        slippage_bps: trade_parameters.slippage_bps,
        receiver: trade_parameters.receiver.clone(),
        valid_for: trade_parameters.valid_for,
        valid_to: trade_parameters.valid_to,
        partner_fee: trade_parameters.partner_fee.clone(),
    })
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
pub fn get_order_to_sign(
    params: OrderToSignParams,
    limit_parameters: &LimitTradeParameters,
    app_data_keccak256: &AppDataHash,
) -> Result<UnsignedOrder, TradingError> {
    let network_costs_amount = params.network_costs_amount.unwrap_or_else(Amount::zero);
    let receiver = limit_parameters
        .receiver
        .clone()
        .unwrap_or_else(|| params.from.clone());
    let valid_to = if let Some(valid_to) = limit_parameters.valid_to {
        valid_to
    } else {
        let valid_for = limit_parameters.valid_for.unwrap_or(DEFAULT_QUOTE_VALIDITY);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| {
                TradingError::InvalidInput(format!(
                    "system time must not be earlier than the unix epoch: {error}"
                ))
            })?
            .as_secs();
        let clamped_valid_to = now
            .saturating_add(u64::from(valid_for))
            .min(u64::from(MAX_VALID_TO_EPOCH));
        u32::try_from(clamped_valid_to)
            .expect("validity timestamp is clamped to the supported `u32` range")
    };

    let slippage_bps = limit_parameters
        .slippage_bps
        .unwrap_or_else(|| default_slippage_bps(params.chain_id, params.is_ethflow));
    let (sell_amount_to_use, buy_amount_to_use) = if params.apply_costs_slippage_and_fees {
        let quote = cow_sdk_orderbook::QuoteData::new(
            limit_parameters.sell_token.clone(),
            limit_parameters.buy_token.clone(),
            limit_parameters.sell_amount.as_str().to_owned(),
            limit_parameters.buy_amount.as_str().to_owned(),
            valid_to,
            app_data_keccak256.clone(),
            limit_parameters.kind,
        )
        .with_network_cost_amount(network_costs_amount.as_str().to_owned())
        .with_receiver(receiver.clone())
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
        (
            limit_parameters.sell_amount.clone(),
            limit_parameters.buy_amount.clone(),
        )
    };

    Ok(UnsignedOrder {
        sell_token: limit_parameters.sell_token.clone(),
        buy_token: limit_parameters.buy_token.clone(),
        receiver,
        sell_amount: sell_amount_to_use,
        buy_amount: buy_amount_to_use,
        valid_to,
        app_data: app_data_keccak256.clone(),
        fee_amount: Amount::zero(),
        kind: limit_parameters.kind,
        partially_fillable: limit_parameters.partially_fillable,
        sell_token_balance: limit_parameters.sell_token_balance,
        buy_token_balance: limit_parameters.buy_token_balance,
    })
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
pub async fn calculate_unique_order_id(
    chain_id: SupportedChainId,
    order: &UnsignedOrder,
    checker: Option<&dyn EthFlowOrderExistsChecker>,
    options: Option<&ProtocolOptions>,
) -> Result<GeneratedOrderId, TradingError> {
    let owner = options
        .and_then(|opts| opts.eth_flow_contract_override.as_ref())
        .and_then(|override_map| override_map.get(&u64::from(chain_id)).cloned())
        .unwrap_or_else(|| {
            eth_flow_contract_address(
                chain_id,
                options.and_then(|opts| opts.env).unwrap_or(CowEnv::Prod),
            )
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
    let amount = parse_integer("buyAmount", value.as_str())?;
    if amount <= BigInt::from(0) {
        return Err(TradingError::InvalidInput(format!(
            "buyAmount must be greater than 0: {amount}"
        )));
    }
    Amount::new((amount - BigInt::from(1)).to_string()).map_err(Into::into)
}
