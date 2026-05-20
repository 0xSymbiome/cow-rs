use cow_sdk_core::{Address, Amount, BuyTokenDestination, OrderKind, SellTokenSource};

use crate::{
    ContractsError,
    interaction::Interaction,
    order::{NormalizedOrder, Order},
    signature::{Signature, decode_signing_scheme, encode_eip1271_signature_data},
};

#[allow(
    clippy::wildcard_imports,
    reason = "settlement codec helpers intentionally share the parent sol! binding and DTO namespace"
)]
use super::*;

/// Encodes order flags into the compact settlement bitfield.
///
/// # Errors
///
/// This function currently uses a total flag mapping and does not return an error,
/// but it retains a fallible signature for API consistency with adjacent codecs.
///
/// # Panics
///
/// Panics only if a new balance enum variant reaches this codec before the
/// settlement flag mapping is updated.
pub fn encode_order_flags(flags: &OrderFlags) -> Result<u8, ContractsError> {
    let kind = match flags.kind {
        OrderKind::Sell => 0,
        OrderKind::Buy => 1,
    };
    let partial = if flags.partially_fillable { 0b10 } else { 0 };
    let sell = match flags.sell_token_balance {
        SellTokenSource::Erc20 => 0,
        SellTokenSource::External => 0b10 << 2,
        SellTokenSource::Internal => 0b11 << 2,
        // SAFETY: the supported settlement sell-token balance variants are
        // fully mapped above; a new variant must extend this bit layout.
        _ => unreachable!("SellTokenSource variants are exhaustively covered"),
    };
    let buy = match flags.buy_token_balance {
        BuyTokenDestination::Erc20 => 0,
        BuyTokenDestination::Internal => 0b1 << 4,
        // SAFETY: the supported settlement buy-token balance variants are
        // fully mapped above; a new variant must extend this bit layout.
        _ => unreachable!("BuyTokenDestination variants are exhaustively covered"),
    };
    Ok([kind, partial, sell, buy].into_iter().sum())
}

/// Decodes compact order flags from the settlement bitfield.
///
/// # Errors
///
/// Returns [`ContractsError::InvalidFlags`] if unsupported bits are set.
///
/// # Panics
///
/// Panics only if the local bit masks stop constraining decoded flag arms to
/// the enumerated values handled by this function.
pub fn decode_order_flags(encoded: u8) -> Result<OrderFlags, ContractsError> {
    if encoded & 0b1000_0000 != 0 {
        return Err(ContractsError::InvalidFlags(encoded));
    }

    let sell_bits = (encoded >> 2) & 0b11;
    let sell_token_balance = match sell_bits {
        0b00 | 0b01 => SellTokenSource::Erc20,
        0b10 => SellTokenSource::External,
        0b11 => SellTokenSource::Internal,
        // SAFETY: sell_bits is masked to two bits immediately above, so every
        // possible value is handled by the explicit arms.
        _ => unreachable!(),
    };

    let buy_token_balance = match (encoded >> 4) & 0b1 {
        0 => BuyTokenDestination::Erc20,
        1 => BuyTokenDestination::Internal,
        // SAFETY: the buy-token flag is masked to one bit immediately above,
        // so every possible value is handled by the explicit arms.
        _ => unreachable!(),
    };

    Ok(OrderFlags::new(
        if encoded & 0b1 == 0 {
            OrderKind::Sell
        } else {
            OrderKind::Buy
        },
        encoded & 0b10 != 0,
        sell_token_balance,
        buy_token_balance,
    ))
}

/// Encodes trade flags into the compact settlement bitfield.
///
/// # Errors
///
/// Returns any error from [`encode_order_flags`].
pub fn encode_trade_flags(flags: &TradeFlags) -> Result<u8, ContractsError> {
    let order_flags = encode_order_flags(&OrderFlags::new(
        flags.kind,
        flags.partially_fillable,
        flags.sell_token_balance,
        flags.buy_token_balance,
    ))?;
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
    Ok(TradeFlags::new(
        order.kind,
        order.partially_fillable,
        order.sell_token_balance,
        order.buy_token_balance,
        signing_scheme,
    ))
}

/// Encodes a signature into the settlement wire representation.
///
/// # Errors
///
/// Returns [`ContractsError`] if signature normalization or EIP-1271 encoding fails.
pub fn encode_signature_data(signature: &Signature) -> Result<String, ContractsError> {
    match signature {
        Signature::Ecdsa { data, .. } => normalize_signature_hex(data),
        Signature::Eip1271 { data } => encode_eip1271_signature_data(data),
        Signature::PreSign { owner } => Ok(owner.to_hex_string()),
    }
}

/// Decodes a `0x`-prefixed hex string and re-encodes it as canonical lowercase
/// hex so the wire form stays byte-identical regardless of input casing.
fn normalize_signature_hex(value: &str) -> Result<String, ContractsError> {
    let stripped = value
        .strip_prefix("0x")
        .ok_or(ContractsError::InvalidHexPrefix { field: "signature" })?;
    let bytes = hex::decode(stripped).map_err(|source| ContractsError::DecodeHex {
        field: "signature",
        source,
    })?;
    Ok(format!("0x{}", hex::encode(bytes)))
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
    Ok(Trade::new(
        tokens.index(&order.sell_token),
        tokens.index(&order.buy_token),
        order.receiver,
        order.sell_amount,
        order.buy_amount,
        order.valid_to,
        order.app_data,
        order.fee_amount,
        encode_trade_flags(&TradeFlags::new(
            order.kind,
            order.partially_fillable,
            order.sell_token_balance,
            order.buy_token_balance,
            signature.scheme(),
        ))?,
        execution.executed_amount,
        encode_signature_data(signature)?,
    ))
}

/// Builds the typed `settle(...)` call from encoded settlement components.
pub(super) fn encode_settle_call(
    tokens: &[Address],
    clearing_prices: &[Amount],
    trades: &[Trade],
    interactions: &[Vec<Interaction>; 3],
) -> Result<IGPv2Settlement::settleCall, ContractsError> {
    use alloy_sol_types::private::{Address as SolAddress, Bytes as SolBytes, FixedBytes, U256};

    // The cow `Amount` newtype is `#[repr(transparent)]` over
    // `alloy_primitives::U256` per ADR 0052, so the conversion to the
    // sol `U256` surface is an infallible deref of the inner U256 with
    // no intermediate bigint allocation. The historical
    // `amount_to_u256` overflow guard collapses to a no-op because
    // `Amount` cannot carry a value beyond `U256::MAX`.
    const fn amount_to_u256(amount: &Amount) -> U256 {
        *amount.as_u256()
    }

    const fn address_to_sol(address: &Address) -> SolAddress {
        *address.as_alloy()
    }

    fn hex_to_bytes(value: &str, field: &'static str) -> Result<SolBytes, ContractsError> {
        let stripped = value
            .strip_prefix("0x")
            .ok_or(ContractsError::InvalidHexPrefix { field })?;
        let bytes =
            hex::decode(stripped).map_err(|source| ContractsError::DecodeHex { field, source })?;
        Ok(SolBytes::from(bytes))
    }

    let sol_tokens: Vec<_> = tokens.iter().map(address_to_sol).collect();
    let sol_clearing_prices: Vec<_> = clearing_prices.iter().map(amount_to_u256).collect();

    let sol_trades = trades
        .iter()
        .map(
            |trade| -> Result<IGPv2Settlement::TradeData, ContractsError> {
                let app_data_bytes = trade.app_data.as_alloy().0;
                Ok(IGPv2Settlement::TradeData {
                    sellTokenIndex: U256::from(trade.sell_token_index),
                    buyTokenIndex: U256::from(trade.buy_token_index),
                    receiver: address_to_sol(&trade.receiver),
                    sellAmount: amount_to_u256(&trade.sell_amount),
                    buyAmount: amount_to_u256(&trade.buy_amount),
                    validTo: trade.valid_to,
                    appData: FixedBytes::from(app_data_bytes),
                    feeAmount: amount_to_u256(&trade.fee_amount),
                    flags: U256::from(trade.flags),
                    executedAmount: amount_to_u256(&trade.executed_amount),
                    signature: hex_to_bytes(&trade.signature, "signature")?,
                })
            },
        )
        .collect::<Result<Vec<_>, _>>()?;

    let mut sol_interactions: [Vec<IGPv2Settlement::InteractionData>; 3] =
        [Vec::new(), Vec::new(), Vec::new()];
    for (stage, stage_interactions) in interactions.iter().enumerate() {
        sol_interactions[stage] = stage_interactions
            .iter()
            .map(
                |interaction| -> Result<IGPv2Settlement::InteractionData, ContractsError> {
                    Ok(IGPv2Settlement::InteractionData {
                        target: address_to_sol(&interaction.target),
                        value: amount_to_u256(&interaction.value),
                        callData: SolBytes::copy_from_slice(&interaction.call_data),
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;
    }

    Ok(IGPv2Settlement::settleCall {
        tokens: sol_tokens,
        clearingPrices: sol_clearing_prices,
        trades: sol_trades,
        interactions: sol_interactions,
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
        let offending = trade.sell_token_index.max(trade.buy_token_index);
        return Err(ContractsError::InvalidTokenIndex {
            index: offending,
            registered: tokens.len(),
        });
    }
    let flags = decode_order_flags(trade.flags)?;
    Ok(Order::new(
        tokens[trade.sell_token_index],
        tokens[trade.buy_token_index],
        Some(trade.receiver),
        trade.sell_amount,
        trade.buy_amount,
        trade.valid_to,
        trade.app_data,
        trade.fee_amount,
        flags.kind,
        flags.partially_fillable,
        Some(flags.sell_token_balance),
        Some(flags.buy_token_balance),
    ))
}

#[cfg(test)]
mod tests {
    use cow_sdk_core::{
        Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource,
    };

    use crate::signature::{Eip1271SignatureData, SigningScheme};

    use super::*;

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
            SellTokenSource::Erc20 => 0,
            SellTokenSource::External => 0b10 << 2,
            SellTokenSource::Internal => 0b11 << 2,
            _ => unreachable!("SellTokenSource variants are exhaustively covered"),
        };
        let buy = match flags.buy_token_balance {
            BuyTokenDestination::Erc20 => 0,
            BuyTokenDestination::Internal => 0b1 << 4,
            _ => unreachable!("BuyTokenDestination variants are exhaustively covered"),
        };
        kind | partial | sell | buy
    }

    #[test]
    fn flag_codecs_match_the_manual_bit_layout_for_all_supported_combinations() {
        for kind in [OrderKind::Sell, OrderKind::Buy] {
            for partially_fillable in [false, true] {
                for sell_token_balance in [
                    SellTokenSource::Erc20,
                    SellTokenSource::External,
                    SellTokenSource::Internal,
                ] {
                    for buy_token_balance in
                        [BuyTokenDestination::Erc20, BuyTokenDestination::Internal]
                    {
                        let order_flags = OrderFlags::new(
                            kind,
                            partially_fillable,
                            sell_token_balance,
                            buy_token_balance,
                        );
                        let encoded = encode_order_flags(&order_flags).unwrap();

                        assert_eq!(encoded, manual_order_flags(&order_flags));
                        assert_eq!(decode_order_flags(encoded).unwrap(), order_flags);

                        for signing_scheme in [
                            SigningScheme::Eip712,
                            SigningScheme::EthSign,
                            SigningScheme::Eip1271,
                            SigningScheme::PreSign,
                        ] {
                            let trade_flags = TradeFlags::new(
                                kind,
                                partially_fillable,
                                sell_token_balance,
                                buy_token_balance,
                                signing_scheme,
                            );
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
    fn decode_order_rejects_each_invalid_index_independently() {
        let trade = Trade::new(
            0,
            1,
            Address::new("0x3333333333333333333333333333333333333333").unwrap(),
            Amount::new("10").unwrap(),
            Amount::new("20").unwrap(),
            123,
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
            Amount::new("1").unwrap(),
            0,
            Amount::zero(),
            "0x".to_owned(),
        );
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
            data: Eip1271SignatureData::new(
                Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
                "0x1234".to_owned(),
            ),
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

    #[test]
    fn settle_call_preserves_max_u256_amounts_and_signature_bytes() {
        // The cow `Amount` newtype is `#[repr(transparent)]` over
        // `alloy_primitives::U256` per ADR 0052, so the `uint256` ceiling
        // is enforced by the type system: an `Amount` greater than
        // `U256::MAX` is unconstructible, and the historical
        // `ContractsError::NumericOverflow` arm at the ABI encoding
        // boundary collapses into a compile-time impossibility.
        let tokens = [
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        ];
        let max_u256 = Amount::from_u256(alloy_primitives::U256::MAX);
        let trade = Trade::new(
            0,
            1,
            Address::new("0x3333333333333333333333333333333333333333").unwrap(),
            Amount::new("10").unwrap(),
            Amount::new("20").unwrap(),
            123,
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
            Amount::new("1").unwrap(),
            0,
            Amount::new("10").unwrap(),
            "0xabcdef".to_owned(),
        );

        let call = encode_settle_call(
            &tokens,
            &[max_u256],
            std::slice::from_ref(&trade),
            &[Vec::new(), Vec::new(), Vec::new()],
        )
        .expect("maximum uint256 clearing price must encode");

        assert_eq!(call.clearingPrices[0].to_be_bytes::<32>(), [0xff; 32]);
        assert_eq!(call.trades[0].signature.as_ref(), &[0xab, 0xcd, 0xef]);
    }
}
