use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, OrderBalance, OrderKind, OrderUid, TypedDataDomain};

use crate::{
    ContractsError,
    interaction::{Interaction, InteractionLike, normalize_interaction},
    order::{NormalizedOrder, Order, OrderUidParams, extract_order_uid_params, normalize_order},
    primitives::{abi_encode_bytes_array, function_selector, normalize_hex_payload},
    signature::{Signature, SigningScheme, decode_signing_scheme, encode_eip1271_signature_data},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum InteractionStage {
    Pre = 0,
    Intra = 1,
    Post = 2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFlags {
    pub kind: OrderKind,
    pub partially_fillable: bool,
    pub sell_token_balance: OrderBalance,
    pub buy_token_balance: OrderBalance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeFlags {
    pub kind: OrderKind,
    pub partially_fillable: bool,
    pub sell_token_balance: OrderBalance,
    pub buy_token_balance: OrderBalance,
    pub signing_scheme: SigningScheme,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeExecution {
    pub executed_amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRefunds {
    pub filled_amounts: Vec<OrderUid>,
    pub pre_signatures: Vec<OrderUid>,
}

pub type Prices = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub sell_token_index: usize,
    pub buy_token_index: usize,
    pub receiver: Address,
    pub sell_amount: String,
    pub buy_amount: String,
    pub valid_to: u32,
    pub app_data: cow_sdk_core::AppDataHash,
    pub fee_amount: String,
    pub flags: u8,
    pub executed_amount: String,
    pub signature: String,
}

pub type EncodedSettlement = (Vec<Address>, Vec<String>, Vec<Trade>, [Vec<Interaction>; 3]);

#[derive(Debug, Clone, Default)]
pub struct TokenRegistry {
    tokens: Vec<Address>,
    token_map: BTreeMap<String, usize>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn addresses(&self) -> Vec<Address> {
        self.tokens.clone()
    }

    pub fn index(&mut self, token: &Address) -> usize {
        let key = token.normalized_key();
        if let Some(index) = self.token_map.get(&key) {
            return *index;
        }
        let index = self.tokens.len();
        self.tokens.push(token.clone());
        self.token_map.insert(key, index);
        index
    }
}

#[derive(Debug, Clone)]
pub struct SettlementEncoder {
    pub domain: TypedDataDomain,
    tokens: TokenRegistry,
    trades: Vec<Trade>,
    interactions: [Vec<Interaction>; 3],
    order_refunds: OrderRefunds,
}

impl SettlementEncoder {
    pub fn new(domain: TypedDataDomain) -> Self {
        Self {
            domain,
            tokens: TokenRegistry::new(),
            trades: Vec::new(),
            interactions: [Vec::new(), Vec::new(), Vec::new()],
            order_refunds: OrderRefunds {
                filled_amounts: Vec::new(),
                pre_signatures: Vec::new(),
            },
        }
    }

    pub fn tokens(&self) -> Vec<Address> {
        self.tokens.addresses()
    }

    pub fn trades(&self) -> Vec<Trade> {
        self.trades.clone()
    }

    pub fn interactions(&self) -> Result<[Vec<Interaction>; 3], ContractsError> {
        Ok([
            self.interactions[InteractionStage::Pre as usize].clone(),
            self.interactions[InteractionStage::Intra as usize].clone(),
            {
                let mut post = self.interactions[InteractionStage::Post as usize].clone();
                post.extend(self.encoded_order_refunds()?);
                post
            },
        ])
    }

    pub fn encoded_order_refunds(&self) -> Result<Vec<Interaction>, ContractsError> {
        let mut interactions = Vec::new();
        for (method, order_uids) in [
            (
                "freeFilledAmountStorage(bytes[])",
                &self.order_refunds.filled_amounts,
            ),
            (
                "freePreSignatureStorage(bytes[])",
                &self.order_refunds.pre_signatures,
            ),
        ] {
            if order_uids.is_empty() {
                continue;
            }
            let selector = function_selector(method);
            let encoded_items = order_uids
                .iter()
                .map(|uid| {
                    crate::primitives::parse_hex_exact(
                        uid.as_str(),
                        "orderUid",
                        crate::order::ORDER_UID_LENGTH,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut call_data = Vec::new();
            call_data.extend_from_slice(&selector);
            call_data.extend_from_slice(&abi_encode_bytes_array(&encoded_items));
            interactions.push(Interaction {
                target: self.domain.verifying_contract.clone(),
                value: "0".to_owned(),
                call_data: format!("0x{}", hex::encode(call_data)),
            });
        }
        Ok(interactions)
    }

    pub fn clearing_prices(&self, prices: &Prices) -> Result<Vec<String>, ContractsError> {
        let normalized: BTreeMap<String, String> = prices
            .iter()
            .map(|(token, price)| (token.to_ascii_lowercase(), price.clone()))
            .collect();

        self.tokens
            .addresses()
            .iter()
            .map(|token| {
                normalized
                    .get(&token.normalized_key())
                    .cloned()
                    .ok_or_else(|| ContractsError::MissingClearingPrice(token.as_str().to_owned()))
            })
            .collect()
    }

    pub fn encode_trade(
        &mut self,
        order: &Order,
        signature: &Signature,
        execution: Option<TradeExecution>,
    ) -> Result<(), ContractsError> {
        let order = normalize_order(order)?;
        let execution = match execution {
            Some(execution) => execution,
            None if order.partially_fillable => return Err(ContractsError::MissingExecutedAmount),
            None => TradeExecution {
                executed_amount: "0".to_owned(),
            },
        };
        self.trades.push(encode_trade(
            &mut self.tokens,
            &order,
            signature,
            &execution,
        )?);
        Ok(())
    }

    pub fn encode_interaction(&mut self, interaction: &InteractionLike, stage: InteractionStage) {
        self.interactions[stage as usize].push(normalize_interaction(interaction));
    }

    pub fn encode_order_refunds(&mut self, refunds: &OrderRefunds) -> Result<(), ContractsError> {
        for uid in refunds
            .filled_amounts
            .iter()
            .chain(refunds.pre_signatures.iter())
        {
            let params = extract_order_uid_params(uid)?;
            let _ = OrderUidParams {
                order_digest: params.order_digest,
                owner: params.owner,
                valid_to: params.valid_to,
            };
        }
        self.order_refunds
            .filled_amounts
            .extend(refunds.filled_amounts.clone());
        self.order_refunds
            .pre_signatures
            .extend(refunds.pre_signatures.clone());
        Ok(())
    }

    pub fn encoded_settlement(&self, prices: &Prices) -> Result<EncodedSettlement, ContractsError> {
        Ok((
            self.tokens(),
            self.clearing_prices(prices)?,
            self.trades(),
            self.interactions()?,
        ))
    }

    pub fn encoded_setup(interactions: &[InteractionLike]) -> EncodedSettlement {
        let mut encoder = Self::new(TypedDataDomain {
            name: "unused".to_owned(),
            version: "unused".to_owned(),
            chain_id: 0,
            verifying_contract: Address::new(crate::primitives::ZERO_ADDRESS)
                .expect("static zero address remains valid"),
        });
        for interaction in interactions {
            encoder.encode_interaction(interaction, InteractionStage::Intra);
        }
        (
            encoder.tokens(),
            Vec::new(),
            encoder.trades(),
            encoder.interactions().unwrap(),
        )
    }
}

pub fn encode_order_flags(flags: &OrderFlags) -> Result<u8, ContractsError> {
    let kind = match flags.kind {
        OrderKind::Sell => 0,
        OrderKind::Buy => 1,
    };
    let partial = u8::from(flags.partially_fillable) << 1;
    let sell = match flags.sell_token_balance {
        OrderBalance::Erc20 => 0,
        OrderBalance::External => 0b10 << 2,
        OrderBalance::Internal => 0b11 << 2,
    };
    let buy = match flags.buy_token_balance {
        OrderBalance::Erc20 | OrderBalance::External => 0,
        OrderBalance::Internal => 0b1 << 4,
    };
    Ok(kind | partial | sell | buy)
}

pub fn decode_order_flags(encoded: u8) -> Result<OrderFlags, ContractsError> {
    if encoded & 0b1000_0000 != 0 {
        return Err(ContractsError::InvalidFlags(encoded));
    }

    let sell_bits = (encoded >> 2) & 0b11;
    let sell_token_balance = match sell_bits {
        0b00 | 0b01 => OrderBalance::Erc20,
        0b10 => OrderBalance::External,
        0b11 => OrderBalance::Internal,
        _ => unreachable!(),
    };

    let buy_token_balance = match (encoded >> 4) & 0b1 {
        0 => OrderBalance::Erc20,
        1 => OrderBalance::Internal,
        _ => unreachable!(),
    };

    Ok(OrderFlags {
        kind: if encoded & 0b1 == 0 {
            OrderKind::Sell
        } else {
            OrderKind::Buy
        },
        partially_fillable: encoded & 0b10 != 0,
        sell_token_balance,
        buy_token_balance,
    })
}

pub fn encode_trade_flags(flags: &TradeFlags) -> Result<u8, ContractsError> {
    Ok(encode_order_flags(&OrderFlags {
        kind: flags.kind,
        partially_fillable: flags.partially_fillable,
        sell_token_balance: flags.sell_token_balance,
        buy_token_balance: flags.buy_token_balance,
    })? | (flags.signing_scheme.as_u8() << 5))
}

pub fn decode_trade_flags(encoded: u8) -> Result<TradeFlags, ContractsError> {
    let order = decode_order_flags(encoded)?;
    let signing_scheme = decode_signing_scheme((encoded >> 5) & 0b11)?;
    Ok(TradeFlags {
        kind: order.kind,
        partially_fillable: order.partially_fillable,
        sell_token_balance: order.sell_token_balance,
        buy_token_balance: order.buy_token_balance,
        signing_scheme,
    })
}

pub fn encode_signature_data(signature: &Signature) -> Result<String, ContractsError> {
    match signature {
        Signature::Ecdsa { data, .. } => normalize_hex_payload(data, "signature"),
        Signature::Eip1271 { data } => encode_eip1271_signature_data(data),
        Signature::PreSign { owner } => Ok(owner.as_str().to_owned()),
    }
}

pub fn encode_trade(
    tokens: &mut TokenRegistry,
    order: &NormalizedOrder,
    signature: &Signature,
    execution: &TradeExecution,
) -> Result<Trade, ContractsError> {
    Ok(Trade {
        sell_token_index: tokens.index(&order.sell_token),
        buy_token_index: tokens.index(&order.buy_token),
        receiver: order.receiver.clone(),
        sell_amount: order.sell_amount.clone(),
        buy_amount: order.buy_amount.clone(),
        valid_to: order.valid_to,
        app_data: order.app_data.clone(),
        fee_amount: order.fee_amount.clone(),
        flags: encode_trade_flags(&TradeFlags {
            kind: order.kind,
            partially_fillable: order.partially_fillable,
            sell_token_balance: order.sell_token_balance,
            buy_token_balance: order.buy_token_balance,
            signing_scheme: signature.scheme(),
        })?,
        executed_amount: execution.executed_amount.clone(),
        signature: encode_signature_data(signature)?,
    })
}

pub fn decode_order(trade: &Trade, tokens: &[Address]) -> Result<Order, ContractsError> {
    if trade.sell_token_index >= tokens.len() || trade.buy_token_index >= tokens.len() {
        return Err(ContractsError::Decode("Invalid trade".to_owned()));
    }
    let flags = decode_order_flags(trade.flags)?;
    Ok(Order {
        sell_token: tokens[trade.sell_token_index].clone(),
        buy_token: tokens[trade.buy_token_index].clone(),
        receiver: Some(trade.receiver.clone()),
        sell_amount: trade.sell_amount.clone(),
        buy_amount: trade.buy_amount.clone(),
        valid_to: trade.valid_to,
        app_data: trade.app_data.clone(),
        fee_amount: trade.fee_amount.clone(),
        kind: flags.kind,
        partially_fillable: flags.partially_fillable,
        sell_token_balance: Some(flags.sell_token_balance),
        buy_token_balance: Some(flags.buy_token_balance),
    })
}
