use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use cow_sdk_core::{
    Address, Amount, AppDataHash, HexData, OrderBalance, OrderKind, OrderUid, TypedDataDomain,
};

use crate::{
    ContractsError,
    interaction::{Interaction, InteractionLike, normalize_interaction},
    order::{NormalizedOrder, Order, OrderUidParams, extract_order_uid_params, normalize_order},
    primitives::{abi_encode_bytes_array, function_selector, normalize_hex_payload, zero_address},
    signature::{Signature, SigningScheme, decode_signing_scheme, encode_eip1271_signature_data},
};

/// Settlement interaction stage.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum InteractionStage {
    /// Interactions executed before trades.
    Pre = 0,
    /// Interactions executed between trade processing steps.
    Intra = 1,
    /// Interactions executed after trades.
    Post = 2,
}

/// Compact order-flag inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFlags {
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order is partially fillable.
    pub partially_fillable: bool,
    /// Sell-token balance source.
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance source.
    pub buy_token_balance: OrderBalance,
}

/// Compact trade-flag inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeFlags {
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order is partially fillable.
    pub partially_fillable: bool,
    /// Sell-token balance source.
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance source.
    pub buy_token_balance: OrderBalance,
    /// Signing scheme used for the signature.
    pub signing_scheme: SigningScheme,
}

/// Trade execution override used while encoding settlements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeExecution {
    /// Executed amount recorded in the encoded trade.
    pub executed_amount: Amount,
}

/// Order-refund payload used for settlement post-interactions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRefunds {
    /// Filled-amount storage entries to clear.
    pub filled_amounts: Vec<OrderUid>,
    /// Pre-signature storage entries to clear.
    pub pre_signatures: Vec<OrderUid>,
}

/// Clearing prices keyed by token address.
pub type Prices = BTreeMap<Address, Amount>;

/// Encoded settlement trade payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Sell token index in the token registry.
    pub sell_token_index: usize,
    /// Buy token index in the token registry.
    pub buy_token_index: usize,
    /// Receiver address.
    pub receiver: Address,
    /// Sell amount.
    pub sell_amount: Amount,
    /// Buy amount.
    pub buy_amount: Amount,
    /// Expiration timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: AppDataHash,
    /// Fee amount.
    pub fee_amount: Amount,
    /// Encoded trade flags.
    pub flags: u8,
    /// Executed amount.
    pub executed_amount: Amount,
    /// Encoded signature payload.
    pub signature: String,
}

/// Fully encoded settlement payload.
pub type EncodedSettlement = (Vec<Address>, Vec<Amount>, Vec<Trade>, [Vec<Interaction>; 3]);

/// Registry that assigns stable indexes to token addresses.
#[derive(Debug, Clone, Default)]
pub struct TokenRegistry {
    tokens: Vec<Address>,
    token_map: BTreeMap<String, usize>,
}

impl TokenRegistry {
    /// Creates an empty token registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns registered token addresses in index order.
    #[must_use]
    pub fn addresses(&self) -> Vec<Address> {
        self.tokens.clone()
    }

    /// Returns the stable index for `token`, inserting it if needed.
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

/// Stateful settlement encoder.
#[derive(Debug, Clone)]
pub struct SettlementEncoder {
    /// Typed-data domain used for the settlement.
    pub domain: TypedDataDomain,
    tokens: TokenRegistry,
    trades: Vec<Trade>,
    interactions: [Vec<Interaction>; 3],
    order_refunds: OrderRefunds,
}

impl SettlementEncoder {
    /// Creates a new settlement encoder.
    #[must_use]
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

    /// Returns the encoded token registry in index order.
    #[must_use]
    pub fn tokens(&self) -> Vec<Address> {
        self.tokens.addresses()
    }

    /// Returns the encoded trades.
    #[must_use]
    pub fn trades(&self) -> Vec<Trade> {
        self.trades.clone()
    }

    /// Returns the encoded interactions grouped by stage.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if post-stage order-refund interactions cannot
    /// be encoded.
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

    /// Returns the encoded post-interactions used to clear refund storage.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if a stored order UID cannot be decoded.
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
                value: Amount::zero(),
                call_data: HexData::new(format!("0x{}", hex::encode(call_data)))?,
            });
        }
        Ok(interactions)
    }

    /// Returns clearing prices aligned to the encoder's token registry.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::MissingClearingPrice`] if any registered token
    /// is absent from `prices`.
    pub fn clearing_prices(&self, prices: &Prices) -> Result<Vec<Amount>, ContractsError> {
        let normalized: BTreeMap<String, Amount> = prices
            .iter()
            .map(|(token, price)| (token.normalized_key(), price.clone()))
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

    /// Encodes and appends a trade.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if order normalization fails, if execution is
    /// missing for a partially fillable order, or if trade encoding fails.
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
                executed_amount: Amount::zero(),
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

    /// Encodes and appends an interaction in the requested stage.
    pub fn encode_interaction(&mut self, interaction: &InteractionLike, stage: InteractionStage) {
        self.interactions[stage as usize].push(normalize_interaction(interaction));
    }

    /// Appends order-refund storage-clearing requests.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if any supplied UID cannot be decoded.
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

    /// Returns the fully encoded settlement tuple.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if clearing prices or interactions cannot be encoded.
    pub fn encoded_settlement(&self, prices: &Prices) -> Result<EncodedSettlement, ContractsError> {
        Ok((
            self.tokens(),
            self.clearing_prices(prices)?,
            self.trades(),
            self.interactions()?,
        ))
    }

    /// Returns an interaction-only settlement setup payload.
    #[must_use]
    pub fn encoded_setup(interactions: &[InteractionLike]) -> EncodedSettlement {
        let mut encoder = Self::new(TypedDataDomain {
            name: "unused".to_owned(),
            version: "unused".to_owned(),
            chain_id: 0,
            verifying_contract: zero_address(),
        });
        for interaction in interactions {
            encoder.encode_interaction(interaction, InteractionStage::Intra);
        }
        (
            encoder.tokens(),
            Vec::new(),
            encoder.trades(),
            [
                encoder.interactions[InteractionStage::Pre as usize].clone(),
                encoder.interactions[InteractionStage::Intra as usize].clone(),
                encoder.interactions[InteractionStage::Post as usize].clone(),
            ],
        )
    }
}

/// Encodes order flags into the compact settlement bitfield.
///
/// # Errors
///
/// This function currently uses a total flag mapping and does not return an error,
/// but it retains a fallible signature for API consistency with adjacent codecs.
pub fn encode_order_flags(flags: &OrderFlags) -> Result<u8, ContractsError> {
    let kind = match flags.kind {
        OrderKind::Sell => 0,
        OrderKind::Buy => 1,
    };
    let partial = if flags.partially_fillable { 0b10 } else { 0 };
    let sell = match flags.sell_token_balance {
        OrderBalance::Erc20 => 0,
        OrderBalance::External => 0b10 << 2,
        OrderBalance::Internal => 0b11 << 2,
    };
    let buy = match flags.buy_token_balance {
        OrderBalance::Erc20 | OrderBalance::External => 0,
        OrderBalance::Internal => 0b1 << 4,
    };
    Ok([kind, partial, sell, buy].into_iter().sum())
}

/// Decodes compact order flags from the settlement bitfield.
///
/// # Errors
///
/// Returns [`ContractsError::InvalidFlags`] if unsupported bits are set.
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

/// Encodes trade flags into the compact settlement bitfield.
///
/// # Errors
///
/// Returns any error from [`encode_order_flags`].
pub fn encode_trade_flags(flags: &TradeFlags) -> Result<u8, ContractsError> {
    let order_flags = encode_order_flags(&OrderFlags {
        kind: flags.kind,
        partially_fillable: flags.partially_fillable,
        sell_token_balance: flags.sell_token_balance,
        buy_token_balance: flags.buy_token_balance,
    })?;
    let signing_scheme = flags.signing_scheme.as_u8() << 5;
    // Keep trade encoding aligned with the order codec: each field owns a disjoint bit range.
    Ok(order_flags + signing_scheme)
}

/// Decodes trade flags from the compact settlement bitfield.
///
/// # Errors
///
/// Returns [`ContractsError`] if the order flags or signing scheme are invalid.
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

/// Encodes a signature into the settlement wire representation.
///
/// # Errors
///
/// Returns [`ContractsError`] if signature normalization or EIP-1271 encoding fails.
pub fn encode_signature_data(signature: &Signature) -> Result<String, ContractsError> {
    match signature {
        Signature::Ecdsa { data, .. } => normalize_hex_payload(data, "signature"),
        Signature::Eip1271 { data } => encode_eip1271_signature_data(data),
        Signature::PreSign { owner } => Ok(owner.as_str().to_owned()),
    }
}

/// Encodes a normalized order, signature, and execution into a settlement trade.
///
/// # Errors
///
/// Returns [`ContractsError`] if flags or signature encoding fails.
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

/// Decodes an encoded trade back into the contract-order representation.
///
/// # Errors
///
/// Returns [`ContractsError`] if token indexes are out of range or trade flags
/// cannot be decoded.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_order(partially_fillable: bool) -> Order {
        Order {
            sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            receiver: Some(Address::new("0x3333333333333333333333333333333333333333").unwrap()),
            sell_amount: Amount::new("10").unwrap(),
            buy_amount: Amount::new("20").unwrap(),
            valid_to: 123,
            app_data: AppDataHash::new(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            )
            .unwrap(),
            fee_amount: Amount::new("1").unwrap(),
            kind: OrderKind::Buy,
            partially_fillable,
            sell_token_balance: Some(OrderBalance::Internal),
            buy_token_balance: Some(OrderBalance::External),
        }
    }

    fn sample_signature() -> Signature {
        Signature::PreSign {
            owner: Address::new("0x4444444444444444444444444444444444444444").unwrap(),
        }
    }

    fn manual_order_flags(flags: &OrderFlags) -> u8 {
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
        kind | partial | sell | buy
    }

    #[test]
    fn flag_codecs_match_the_manual_bit_layout_for_all_supported_combinations() {
        for kind in [OrderKind::Sell, OrderKind::Buy] {
            for partially_fillable in [false, true] {
                for sell_token_balance in [
                    OrderBalance::Erc20,
                    OrderBalance::External,
                    OrderBalance::Internal,
                ] {
                    for buy_token_balance in [OrderBalance::Erc20, OrderBalance::Internal] {
                        let order_flags = OrderFlags {
                            kind,
                            partially_fillable,
                            sell_token_balance,
                            buy_token_balance,
                        };
                        let encoded = encode_order_flags(&order_flags).unwrap();

                        assert_eq!(encoded, manual_order_flags(&order_flags));
                        assert_eq!(decode_order_flags(encoded).unwrap(), order_flags);

                        for signing_scheme in [
                            SigningScheme::Eip712,
                            SigningScheme::EthSign,
                            SigningScheme::Eip1271,
                            SigningScheme::PreSign,
                        ] {
                            let trade_flags = TradeFlags {
                                kind,
                                partially_fillable,
                                sell_token_balance,
                                buy_token_balance,
                                signing_scheme,
                            };
                            let encoded_trade = encode_trade_flags(&trade_flags).unwrap();
                            assert_eq!(
                                encoded_trade,
                                manual_order_flags(&order_flags) | (signing_scheme.as_u8() << 5)
                            );
                            assert_eq!(decode_trade_flags(encoded_trade).unwrap(), trade_flags);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fill_or_kill_orders_default_the_executed_amount_to_zero() {
        let domain = TypedDataDomain {
            name: "Gnosis Protocol".to_owned(),
            version: "v2".to_owned(),
            chain_id: 1,
            verifying_contract: Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
        };
        let mut encoder = SettlementEncoder::new(domain);

        encoder
            .encode_trade(&sample_order(false), &sample_signature(), None)
            .unwrap();

        assert_eq!(encoder.trades()[0].executed_amount, Amount::zero());
    }

    #[test]
    fn decode_order_rejects_each_invalid_index_independently() {
        let trade = Trade {
            sell_token_index: 0,
            buy_token_index: 1,
            receiver: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
            sell_amount: Amount::new("10").unwrap(),
            buy_amount: Amount::new("20").unwrap(),
            valid_to: 123,
            app_data: AppDataHash::new(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            )
            .unwrap(),
            fee_amount: Amount::new("1").unwrap(),
            flags: 0,
            executed_amount: Amount::zero(),
            signature: "0x".to_owned(),
        };
        let tokens = vec![
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        ];

        let mut sell_invalid = trade.clone();
        sell_invalid.sell_token_index = 2;
        assert!(decode_order(&sell_invalid, &tokens).is_err());

        let mut buy_invalid = trade;
        buy_invalid.buy_token_index = 2;
        assert!(decode_order(&buy_invalid, &tokens).is_err());
    }

    #[test]
    fn signature_encoding_preserves_each_supported_signature_variant() {
        let ecdsa = Signature::Ecdsa {
            scheme: SigningScheme::Eip712,
            data: "0xABCD".to_owned(),
        };
        let eip1271 = Signature::Eip1271 {
            data: crate::signature::Eip1271SignatureData {
                verifier: Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
                signature: "0x1234".to_owned(),
            },
        };
        let presign = sample_signature();

        assert_eq!(encode_signature_data(&ecdsa).unwrap(), "0xabcd");
        assert_eq!(
            encode_signature_data(&eip1271).unwrap(),
            "0x9008d19f58aabd9ed0d60971565aa8510560ab411234"
        );
        assert_eq!(
            encode_signature_data(&presign).unwrap(),
            "0x4444444444444444444444444444444444444444"
        );
    }
}
