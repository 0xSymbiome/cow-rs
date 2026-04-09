use cow_sdk_core::{
    Address, AsyncSigner, ProtocolOptions, Signer, SupportedChainId, TransactionRequest,
    eth_flow_contract_address, settlement_contract_address,
};
use cow_sdk_orderbook::Order;

use crate::slippage::{gas_with_margin, parse_integer};
use crate::{
    GAS_LIMIT_DEFAULT, OrderTraderParameters, TraderParameters, TradingError,
    calculate_unique_order_id, get_order_to_sign,
};

pub struct EthFlowTransaction {
    pub order_id: cow_sdk_core::OrderUid,
    pub transaction: TransactionRequest,
    pub order_to_sign: cow_sdk_core::UnsignedOrder,
}

pub fn get_pre_sign_transaction<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order_uid: &cow_sdk_core::OrderUid,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionRequest, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display,
{
    let settlement = resolve_settlement_address(chain_id, options);
    let tx = TransactionRequest {
        to: Some(settlement),
        data: Some(encode_set_pre_signature(order_uid, true)?),
        value: Some("0".to_owned()),
        gas_limit: None,
    };
    let gas = signer
        .estimate_gas(&tx)
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string(),
        });
    let gas_limit = match gas {
        Ok(value) => gas_with_margin(&value)?,
        Err(_) => gas_with_margin(&GAS_LIMIT_DEFAULT.to_string())?,
    };

    Ok(TransactionRequest {
        gas_limit: Some(gas_limit),
        ..tx
    })
}

pub async fn get_pre_sign_transaction_async<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order_uid: &cow_sdk_core::OrderUid,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionRequest, TradingError>
where
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let settlement = resolve_settlement_address(chain_id, options);
    let tx = TransactionRequest {
        to: Some(settlement),
        data: Some(encode_set_pre_signature(order_uid, true)?),
        value: Some("0".to_owned()),
        gas_limit: None,
    };
    let gas = signer
        .estimate_gas(&tx)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string(),
        });
    let gas_limit = match gas {
        Ok(value) => gas_with_margin(&value)?,
        Err(_) => gas_with_margin(&GAS_LIMIT_DEFAULT.to_string())?,
    };

    Ok(TransactionRequest {
        gas_limit: Some(gas_limit),
        ..tx
    })
}

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
    S::Error: std::fmt::Display,
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
    S::Error: std::fmt::Display,
{
    let from = signer
        .get_address()
        .await
        .map_err(|error| TradingError::Signer {
            operation: "get_address",
            message: error.to_string(),
        })?;
    let mut adjusted = crate::adjust_ethflow_limit_parameters(chain_id, params);
    if adjusted.slippage_bps.is_none() {
        adjusted.slippage_bps = Some(crate::default_slippage_bps(chain_id, true));
    }

    let options = ProtocolOptions {
        env: adjusted.env.or(trader.env),
        settlement_contract_override: adjusted
            .settlement_contract_override
            .clone()
            .or_else(|| trader.settlement_contract_override.clone()),
        eth_flow_contract_override: adjusted
            .eth_flow_contract_override
            .clone()
            .or_else(|| trader.eth_flow_contract_override.clone()),
    };
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
    let tx = TransactionRequest {
        to: Some(resolve_eth_flow_address(chain_id, Some(&options))),
        data: Some(encode_ethflow_create_order(
            &order_to_sign,
            adjusted
                .quote_id
                .ok_or(TradingError::MissingQuoteId("EthFlow transaction"))?,
        )?),
        value: Some(order_to_sign.sell_amount.clone()),
        gas_limit: None,
    };
    let gas = signer
        .estimate_gas(&tx)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string(),
        });
    let gas_limit = match gas {
        Ok(value) => gas_with_margin(&value)?,
        Err(_) => gas_with_margin(&GAS_LIMIT_DEFAULT.to_string())?,
    };

    Ok(EthFlowTransaction {
        order_id: generated.order_id,
        order_to_sign,
        transaction: TransactionRequest {
            gas_limit: Some(gas_limit),
            ..tx
        },
    })
}

pub fn onchain_cancellation_transaction<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionRequest, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display,
{
    let mut tx = if order.ethflow_data.is_some() {
        TransactionRequest {
            to: Some(resolve_eth_flow_address(chain_id, options)),
            data: Some(encode_ethflow_invalidate_order(order)?),
            value: Some("0".to_owned()),
            gas_limit: None,
        }
    } else {
        TransactionRequest {
            to: Some(resolve_settlement_address(chain_id, options)),
            data: Some(encode_invalidate_order_uid(&order.uid)?),
            value: Some("0".to_owned()),
            gas_limit: None,
        }
    };
    let gas = signer
        .estimate_gas(&tx)
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string(),
        });
    tx.gas_limit = Some(match gas {
        Ok(value) => parse_integer("gas", &value)?.to_string(),
        Err(_) => GAS_LIMIT_DEFAULT.to_string(),
    });
    Ok(tx)
}

pub async fn onchain_cancellation_transaction_async<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<TransactionRequest, TradingError>
where
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let mut tx = if order.ethflow_data.is_some() {
        TransactionRequest {
            to: Some(resolve_eth_flow_address(chain_id, options)),
            data: Some(encode_ethflow_invalidate_order(order)?),
            value: Some("0".to_owned()),
            gas_limit: None,
        }
    } else {
        TransactionRequest {
            to: Some(resolve_settlement_address(chain_id, options)),
            data: Some(encode_invalidate_order_uid(&order.uid)?),
            value: Some("0".to_owned()),
            gas_limit: None,
        }
    };
    let gas = signer
        .estimate_gas(&tx)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "estimate_gas",
            message: error.to_string(),
        });
    tx.gas_limit = Some(match gas {
        Ok(value) => parse_integer("gas", &value)?.to_string(),
        Err(_) => GAS_LIMIT_DEFAULT.to_string(),
    });
    Ok(tx)
}

pub fn cancel_order_onchain<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<String, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display,
{
    let tx = onchain_cancellation_transaction(signer, chain_id, order, options)?;
    let receipt = signer
        .send_transaction(&tx)
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string(),
        })?;
    Ok(receipt.transaction_hash)
}

pub async fn cancel_order_onchain_async<S>(
    signer: &S,
    chain_id: SupportedChainId,
    order: &Order,
    options: Option<&ProtocolOptions>,
) -> Result<String, TradingError>
where
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let tx = onchain_cancellation_transaction_async(signer, chain_id, order, options).await?;
    let receipt = signer
        .send_transaction(&tx)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string(),
        })?;
    Ok(receipt.transaction_hash)
}

pub fn protocol_options_for_order(
    params: &OrderTraderParameters,
    trader: &TraderParameters,
) -> ProtocolOptions {
    ProtocolOptions {
        env: params.env.or(trader.env),
        settlement_contract_override: params
            .settlement_contract_override
            .clone()
            .or_else(|| trader.settlement_contract_override.clone()),
        eth_flow_contract_override: params
            .eth_flow_contract_override
            .clone()
            .or_else(|| trader.eth_flow_contract_override.clone()),
    }
}

fn resolve_settlement_address(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Address {
    options
        .and_then(|opts| opts.settlement_contract_override.as_ref())
        .and_then(|override_map| override_map.get(&u64::from(chain_id)).cloned())
        .unwrap_or_else(|| {
            settlement_contract_address(
                chain_id,
                options
                    .and_then(|opts| opts.env)
                    .unwrap_or(cow_sdk_core::CowEnv::Prod),
            )
        })
}

fn resolve_eth_flow_address(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Address {
    options
        .and_then(|opts| opts.eth_flow_contract_override.as_ref())
        .and_then(|override_map| override_map.get(&u64::from(chain_id)).cloned())
        .unwrap_or_else(|| {
            eth_flow_contract_address(
                chain_id,
                options
                    .and_then(|opts| opts.env)
                    .unwrap_or(cow_sdk_core::CowEnv::Prod),
            )
        })
}

fn encode_set_pre_signature(
    order_uid: &cow_sdk_core::OrderUid,
    enabled: bool,
) -> Result<String, TradingError> {
    encode_selector_and_dynamic_bytes_bool(
        "setPreSignature(bytes,bool)",
        order_uid.as_str(),
        enabled,
    )
}

fn encode_invalidate_order_uid(order_uid: &cow_sdk_core::OrderUid) -> Result<String, TradingError> {
    encode_selector_and_dynamic_bytes("invalidateOrder(bytes)", order_uid.as_str())
}

fn encode_ethflow_create_order(
    order: &cow_sdk_core::UnsignedOrder,
    quote_id: i64,
) -> Result<String, TradingError> {
    encode_ethflow_tuple_call("createOrder", order, quote_id)
}

fn encode_ethflow_invalidate_order(order: &Order) -> Result<String, TradingError> {
    let receiver = order
        .receiver
        .clone()
        .unwrap_or_else(|| order.owner.clone());
    encode_ethflow_tuple_static(
        "invalidateOrder",
        &EthFlowTupleData {
            buy_token: &order.buy_token,
            receiver: &receiver,
            sell_amount: &order.sell_amount,
            buy_amount: &order.buy_amount,
            fee_amount: &order.fee_amount,
            partially_fillable: false,
            quote_id: 0,
            app_data: order.app_data.as_str(),
            valid_to: order.valid_to,
        },
    )
}

fn encode_ethflow_tuple_call(
    method: &str,
    order: &cow_sdk_core::UnsignedOrder,
    quote_id: i64,
) -> Result<String, TradingError> {
    encode_ethflow_tuple_static(
        method,
        &EthFlowTupleData {
            buy_token: &order.buy_token,
            receiver: &order.receiver,
            sell_amount: &order.sell_amount,
            buy_amount: &order.buy_amount,
            fee_amount: &order.fee_amount,
            partially_fillable: order.partially_fillable,
            quote_id,
            app_data: order.app_data.as_str(),
            valid_to: order.valid_to,
        },
    )
}

struct EthFlowTupleData<'a> {
    buy_token: &'a Address,
    receiver: &'a Address,
    sell_amount: &'a str,
    buy_amount: &'a str,
    fee_amount: &'a str,
    partially_fillable: bool,
    quote_id: i64,
    app_data: &'a str,
    valid_to: u32,
}

fn encode_ethflow_tuple_static(
    method: &str,
    data: &EthFlowTupleData<'_>,
) -> Result<String, TradingError> {
    let selector = selector_bytes(&format!(
        "{method}((address,address,uint256,uint256,uint256,bool,uint256,bytes32,uint32))"
    ))?;
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&selector);
    encoded.extend_from_slice(&encode_address_word(data.buy_token)?);
    encoded.extend_from_slice(&encode_address_word(data.receiver)?);
    encoded.extend_from_slice(&encode_uint_word(data.sell_amount)?);
    encoded.extend_from_slice(&encode_uint_word(data.buy_amount)?);
    encoded.extend_from_slice(&encode_uint_word(data.fee_amount)?);
    encoded.extend_from_slice(&encode_bool_word(data.partially_fillable));
    encoded.extend_from_slice(&encode_uint_word(&data.quote_id.to_string())?);
    encoded.extend_from_slice(&encode_bytes32_word(data.app_data)?);
    encoded.extend_from_slice(&encode_uint_word(&data.valid_to.to_string())?);

    Ok(format!("0x{}", hex::encode(encoded)))
}

fn encode_selector_and_dynamic_bytes(
    signature: &str,
    bytes_hex: &str,
) -> Result<String, TradingError> {
    let selector = selector_bytes(signature)?;
    let bytes = decode_hex_field("bytes", bytes_hex)?;
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&selector);
    encoded.extend_from_slice(&encode_usize_word(32));
    encoded.extend_from_slice(&encode_usize_word(bytes.len()));
    encoded.extend_from_slice(&pad_to_word(bytes));
    Ok(format!("0x{}", hex::encode(encoded)))
}

fn encode_selector_and_dynamic_bytes_bool(
    signature: &str,
    bytes_hex: &str,
    flag: bool,
) -> Result<String, TradingError> {
    let selector = selector_bytes(signature)?;
    let bytes = decode_hex_field("bytes", bytes_hex)?;
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&selector);
    encoded.extend_from_slice(&encode_usize_word(64));
    encoded.extend_from_slice(&encode_bool_word(flag));
    encoded.extend_from_slice(&encode_usize_word(bytes.len()));
    encoded.extend_from_slice(&pad_to_word(bytes));
    Ok(format!("0x{}", hex::encode(encoded)))
}

fn selector_bytes(signature: &str) -> Result<[u8; 4], TradingError> {
    let selector = cow_sdk_contracts::function_magic_value(signature);
    let bytes = decode_hex_field("selector", &selector)?;
    let mut out = [0u8; 4];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_hex_field(field: &'static str, value: &str) -> Result<Vec<u8>, TradingError> {
    let Some(stripped) = value.strip_prefix("0x") else {
        return Err(TradingError::InvalidNumeric {
            field,
            value: value.to_owned(),
        });
    };
    hex::decode(stripped).map_err(|_| TradingError::InvalidNumeric {
        field,
        value: value.to_owned(),
    })
}

fn encode_address_word(address: &Address) -> Result<[u8; 32], TradingError> {
    let bytes = decode_hex_field("address", address.as_str())?;
    if bytes.len() != 20 {
        return Err(TradingError::InvalidNumeric {
            field: "address",
            value: address.as_str().to_owned(),
        });
    }
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(&bytes);
    Ok(out)
}

fn encode_uint_word(value: &str) -> Result<[u8; 32], TradingError> {
    let parsed = parse_integer("uint256", value)?;
    let bytes = parsed.to_signed_bytes_be();
    if bytes.len() > 32 {
        return Err(TradingError::NumericOverflow {
            field: "uint256",
            value: value.to_owned(),
        });
    }
    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(out)
}

fn encode_usize_word(value: usize) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&(value as u64).to_be_bytes());
    out
}

fn encode_bool_word(value: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = u8::from(value);
    out
}

fn encode_bytes32_word(value: &str) -> Result<[u8; 32], TradingError> {
    let bytes = decode_hex_field("bytes32", value)?;
    if bytes.len() != 32 {
        return Err(TradingError::InvalidNumeric {
            field: "bytes32",
            value: value.to_owned(),
        });
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn pad_to_word(mut bytes: Vec<u8>) -> Vec<u8> {
    let padding = (32 - (bytes.len() % 32)) % 32;
    bytes.extend(std::iter::repeat_n(0u8, padding));
    bytes
}
