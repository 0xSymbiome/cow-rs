#![no_main]

//! Fuzz target for `GPv2Settlement.settle(...)` ABI encoding.
//!
//! **Property:** `PROP-CON-013`.
//! Drives arbitrary `(tokens, clearingPrices, trades, interactions)`
//! tuples through the `alloy::sol!`-generated
//! `IGPv2Settlement::settleCall` encoder and asserts:
//!
//! * The 4-byte selector prefix equals
//!   `keccak256("settle(address[],uint256[],(uint256,uint256,address,uint256,uint256,uint32,bytes32,uint256,uint256,uint256,bytes)[],(address,uint256,bytes)[][3])")[0..4]`.
//! * The encoded length is at least the selector plus four
//!   dynamic-argument offset words (`4 + 4 * 32`). `settle` has four
//!   top-level dynamic arguments, so every valid encoding starts with
//!   that head.
//! * Encoding is panic-free on every arbitrary input.
//!
//! Structured input is derived via [`arbitrary::Arbitrary`] on a local
//! struct; each collection is capped at a small maximum so the run
//! stays bounded.

use alloy_sol_types::{
    SolCall,
    private::{Address, Bytes, FixedBytes, U256},
    sol,
};
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};
use sha3::{Digest, Keccak256};

sol! {
    interface IGPv2Settlement {
        struct TradeData {
            uint256 sellTokenIndex;
            uint256 buyTokenIndex;
            address receiver;
            uint256 sellAmount;
            uint256 buyAmount;
            uint32 validTo;
            bytes32 appData;
            uint256 feeAmount;
            uint256 flags;
            uint256 executedAmount;
            bytes signature;
        }

        struct InteractionData {
            address target;
            uint256 value;
            bytes callData;
        }

        function settle(
            address[] tokens,
            uint256[] clearingPrices,
            TradeData[] trades,
            InteractionData[][3] interactions
        ) external;
    }
}

const MAX_TOKENS: usize = 6;
const MAX_TRADES: usize = 3;
const MAX_INTERACTIONS_PER_STAGE: usize = 3;
const MAX_SIGNATURE_BYTES: usize = 128;
const MAX_CALL_DATA_BYTES: usize = 128;

#[derive(Debug, Arbitrary)]
struct FuzzInteraction {
    target: [u8; 20],
    value: u128,
    call_data: Vec<u8>,
}

#[derive(Debug, Arbitrary)]
struct FuzzTrade {
    sell_token_index: u32,
    buy_token_index: u32,
    receiver: [u8; 20],
    sell_amount: u128,
    buy_amount: u128,
    valid_to: u32,
    app_data: [u8; 32],
    fee_amount: u128,
    flags: u128,
    executed_amount: u128,
    signature: Vec<u8>,
}

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    tokens: Vec<[u8; 20]>,
    prices: Vec<u128>,
    trades: Vec<FuzzTrade>,
    pre_interactions: Vec<FuzzInteraction>,
    intra_interactions: Vec<FuzzInteraction>,
    post_interactions: Vec<FuzzInteraction>,
}

fuzz_target!(|input: FuzzInput| {
    let tokens: Vec<Address> = input
        .tokens
        .into_iter()
        .take(MAX_TOKENS)
        .map(Address::from)
        .collect();
    let clearing_prices: Vec<U256> = input
        .prices
        .into_iter()
        .take(MAX_TOKENS)
        .map(U256::from)
        .collect();
    let trades: Vec<IGPv2Settlement::TradeData> = input
        .trades
        .into_iter()
        .take(MAX_TRADES)
        .map(|t| IGPv2Settlement::TradeData {
            sellTokenIndex: U256::from(t.sell_token_index),
            buyTokenIndex: U256::from(t.buy_token_index),
            receiver: Address::from(t.receiver),
            sellAmount: U256::from(t.sell_amount),
            buyAmount: U256::from(t.buy_amount),
            validTo: t.valid_to,
            appData: FixedBytes::from(t.app_data),
            feeAmount: U256::from(t.fee_amount),
            flags: U256::from(t.flags),
            executedAmount: U256::from(t.executed_amount),
            signature: Bytes::from(
                t.signature
                    .into_iter()
                    .take(MAX_SIGNATURE_BYTES)
                    .collect::<Vec<u8>>(),
            ),
        })
        .collect();

    let stage = |stage: Vec<FuzzInteraction>| -> Vec<IGPv2Settlement::InteractionData> {
        stage
            .into_iter()
            .take(MAX_INTERACTIONS_PER_STAGE)
            .map(|i| IGPv2Settlement::InteractionData {
                target: Address::from(i.target),
                value: U256::from(i.value),
                callData: Bytes::from(
                    i.call_data
                        .into_iter()
                        .take(MAX_CALL_DATA_BYTES)
                        .collect::<Vec<u8>>(),
                ),
            })
            .collect()
    };
    let interactions = [
        stage(input.pre_interactions),
        stage(input.intra_interactions),
        stage(input.post_interactions),
    ];

    let call = IGPv2Settlement::settleCall {
        tokens: tokens.clone(),
        clearingPrices: clearing_prices.clone(),
        trades: trades.clone(),
        interactions: interactions.clone(),
    };
    let encoded = call.abi_encode();

    let canonical_selector: [u8; 4] = {
        let signature = "settle(address[],uint256[],(uint256,uint256,address,uint256,uint256,uint32,bytes32,uint256,uint256,uint256,bytes)[],(address,uint256,bytes)[][3])";
        let digest = Keccak256::digest(signature.as_bytes());
        [digest[0], digest[1], digest[2], digest[3]]
    };
    assert_eq!(
        &encoded[..4],
        &canonical_selector,
        "settle selector must match keccak256 of the canonical ABI signature",
    );

    assert!(
        encoded.len() >= 4 + 4 * 32,
        "settle call-data must be at least selector + four dynamic-argument offsets",
    );

    // Decoder round-trip: the encoded bytes must decode back through the matching
    // decoder and re-encoding the decoded value must produce byte-identical
    // call-data. This proves the ABI encoder and decoder are inverses on every
    // well-typed input shape, and that the encoded representation is canonical.
    let decoded = IGPv2Settlement::settleCall::abi_decode_validate(&encoded)
        .expect("settle call-data must round-trip through the matching decoder");
    let re_encoded = decoded.abi_encode();
    assert_eq!(
        encoded, re_encoded,
        "settle encoder must produce byte-identical call-data after a decode/encode round-trip",
    );

    // Encoder determinism: re-encoding the original tuple a second time must
    // also produce the same bytes — the canonical ABI encoding is a function
    // of the typed input only.
    let original_re_encoded = IGPv2Settlement::settleCall {
        tokens,
        clearingPrices: clearing_prices,
        trades,
        interactions,
    }
    .abi_encode();
    assert_eq!(
        encoded, original_re_encoded,
        "settle encoder must be deterministic for identical typed input",
    );
    let _ = call;
});
