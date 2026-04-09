use num_bigint::BigInt;

use cow_sdk_core::{
    Address, AppDataHash, CowEnv, EVM_NATIVE_CURRENCY_ADDRESS, MAX_VALID_TO_EPOCH, OrderBalance,
    ProtocolOptions, SupportedChainId, UnsignedOrder, addresses_equal, eth_flow_contract_address,
    wrapped_native_token,
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

#[derive(Debug, Clone)]
pub struct OrderToSignParams {
    pub chain_id: SupportedChainId,
    pub from: Address,
    pub is_ethflow: bool,
    pub network_costs_amount: Option<String>,
    pub apply_costs_slippage_and_fees: bool,
    pub protocol_fee_bps: Option<f64>,
}

pub fn is_ethflow_order(sell_token: &Address) -> bool {
    addresses_equal(
        sell_token,
        &Address::new(EVM_NATIVE_CURRENCY_ADDRESS).expect("native token literal remains valid"),
    )
}

pub fn adjust_ethflow_trade_parameters(
    chain_id: SupportedChainId,
    trade_parameters: &TradeParameters,
) -> TradeParameters {
    let mut adjusted = trade_parameters.clone();
    adjusted.sell_token = wrapped_native_token(chain_id).address;
    adjusted
}

pub fn adjust_ethflow_limit_parameters(
    chain_id: SupportedChainId,
    limit_parameters: &LimitTradeParameters,
) -> LimitTradeParameters {
    let mut adjusted = limit_parameters.clone();
    adjusted.sell_token = wrapped_native_token(chain_id).address;
    adjusted
}

pub fn swap_params_to_limit_order_params(
    trade_parameters: &TradeParameters,
    quote_response: &OrderQuoteResponse,
) -> LimitTradeParameters {
    LimitTradeParameters {
        kind: trade_parameters.kind,
        owner: trade_parameters.owner.clone(),
        sell_token: trade_parameters.sell_token.clone(),
        sell_token_decimals: trade_parameters.sell_token_decimals,
        buy_token: trade_parameters.buy_token.clone(),
        buy_token_decimals: trade_parameters.buy_token_decimals,
        sell_amount: quote_response.quote.sell_amount.clone(),
        buy_amount: quote_response.quote.buy_amount.clone(),
        quote_id: quote_response.id,
        env: trade_parameters.env,
        settlement_contract_override: trade_parameters.settlement_contract_override.clone(),
        eth_flow_contract_override: trade_parameters.eth_flow_contract_override.clone(),
        partially_fillable: trade_parameters.partially_fillable,
        slippage_bps: trade_parameters.slippage_bps,
        receiver: trade_parameters.receiver.clone(),
        valid_for: trade_parameters.valid_for,
        valid_to: trade_parameters.valid_to,
        partner_fee: trade_parameters.partner_fee.clone(),
    }
}

pub fn get_order_to_sign(
    params: OrderToSignParams,
    limit_parameters: &LimitTradeParameters,
    app_data_keccak256: &AppDataHash,
) -> Result<UnsignedOrder, TradingError> {
    let network_costs_amount = params
        .network_costs_amount
        .unwrap_or_else(|| "0".to_owned());
    let receiver = limit_parameters
        .receiver
        .clone()
        .unwrap_or_else(|| params.from.clone());
    let valid_to = limit_parameters.valid_to.unwrap_or_else(|| {
        let valid_for = limit_parameters.valid_for.unwrap_or(DEFAULT_QUOTE_VALIDITY);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix epoch remains earlier than current time")
            .as_secs();
        (now + u64::from(valid_for)) as u32
    });

    let slippage_bps = limit_parameters
        .slippage_bps
        .unwrap_or_else(|| default_slippage_bps(params.chain_id, params.is_ethflow));
    let mut sell_amount_to_use = limit_parameters.sell_amount.clone();
    let mut buy_amount_to_use = limit_parameters.buy_amount.clone();

    if params.apply_costs_slippage_and_fees {
        let quote = cow_sdk_orderbook::QuoteData {
            sell_token: limit_parameters.sell_token.clone(),
            buy_token: limit_parameters.buy_token.clone(),
            receiver: Some(receiver.clone()),
            sell_amount: limit_parameters.sell_amount.clone(),
            buy_amount: limit_parameters.buy_amount.clone(),
            valid_to,
            app_data: app_data_keccak256.clone(),
            fee_amount: network_costs_amount.clone(),
            kind: limit_parameters.kind,
            partially_fillable: limit_parameters.partially_fillable,
            sell_token_balance: OrderBalance::Erc20,
            buy_token_balance: OrderBalance::Erc20,
        };
        let amounts = calculate_quote_amounts_and_costs(
            &quote,
            slippage_bps,
            partner_fee_bps(limit_parameters.partner_fee.as_ref()),
            params.protocol_fee_bps,
        )?;
        sell_amount_to_use = if amounts.is_sell {
            amounts.before_all_fees.sell_amount
        } else {
            amounts.after_slippage.sell_amount
        };
        buy_amount_to_use = if amounts.is_sell {
            amounts.after_slippage.buy_amount
        } else {
            amounts.before_all_fees.buy_amount
        };
    }

    Ok(UnsignedOrder {
        sell_token: limit_parameters.sell_token.clone(),
        buy_token: limit_parameters.buy_token.clone(),
        receiver,
        sell_amount: sell_amount_to_use,
        buy_amount: buy_amount_to_use,
        valid_to,
        app_data: app_data_keccak256.clone(),
        fee_amount: "0".to_owned(),
        kind: limit_parameters.kind,
        partially_fillable: limit_parameters.partially_fillable,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
    })
}

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

    loop {
        let mut order_for_id = current.clone();
        order_for_id.valid_to = MAX_VALID_TO_EPOCH;
        order_for_id.sell_token = wrapped_native_token(chain_id).address;

        let generated = generate_order_id(chain_id, &order_for_id, &owner, options)?;
        let exists = match checker {
            Some(checker) => {
                checker
                    .order_exists(&generated.order_id, &generated.order_digest)
                    .await?
            }
            None => false,
        };

        if !exists {
            return Ok(generated);
        }

        current.buy_amount = adjust_buy_amount(&current.buy_amount)?;
    }
}

fn adjust_buy_amount(value: &str) -> Result<String, TradingError> {
    let amount = parse_integer("buyAmount", value)?;
    if amount <= BigInt::from(0) {
        return Err(TradingError::InvalidInput(format!(
            "buyAmount must be greater than 0: {amount}"
        )));
    }
    Ok((amount - BigInt::from(1)).to_string())
}
