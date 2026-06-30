//! Safe pre-sign order placement and its bundled on-chain activation (ADR 0073).
//!
//! A smart-contract wallet cannot produce an ECDSA signature, so it authorizes an
//! order through an on-chain pre-sign instead. This scenario carries that mode as
//! one value: quote with `Trading::quote_only`, then place with
//! `Trading::place_swap` under `Authorization::pre_sign()` and an explicit Safe
//! owner — the same call shape an EOA order uses. The scheme statically selects
//! the result arm, so the placement returns `OrderPlacement::PendingActivation`
//! and the order UID is reachable only by matching the arm that also yields the
//! `SafeActivation`. That makes the on-chain obligation un-droppable: a pre-sign
//! order that is posted but never activated would sit inert until it expires.
//!
//! The activation bundles the two calls the Safe batches from its own account —
//! the ERC-20 `approve` granting the vault relayer the sell-token allowance, then
//! the settlement `setPreSignature(uid, true)` that makes the order fillable. The
//! SDK returns them as ordered `UnsignedTransaction` values rather than bare
//! calldata, and the bundle is transport-neutral: a single-owner Safe can send the
//! calls directly, while a higher-threshold Safe proposes them to its transaction
//! service for the owners to co-sign. Deterministic against the `cow_sdk::testing`
//! doubles — no network and no private key.

use std::error::Error;

use serde_json::json;

use cow_sdk::core::{Address, SupportedChainId, address};
use cow_sdk::orderbook::SigningScheme;
use cow_sdk::trading::{Authorization, OrderPlacement, Trading};

use cow_sdk::testing::MockOrderbook;
use cow_sdk_examples_native::support::{call_data_prefix, sample_trade_parameters};

// The smart-contract wallet that owns the order. Distinct from the EOA signer the
// other scenarios sign as: a pre-sign order consults no ECDSA signer, so the owner
// is supplied explicitly and identifies the Safe.
const SAFE_OWNER: Address = address!("0x974caa59e49682cda0ad2bbe82983419a2ecc400");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Deterministic, transport-mocked orderbook. The pre-sign path consults no
    // signer, so none is constructed.
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(cow_sdk_examples_native::support::sample_quote_response())
        .build();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-presign-order")
        .orderbook(orderbook.clone())
        .build()?;

    // Quote first: `quote_only` returns an owned `QuoteResults` that needs no
    // signer, since the price is what authorizes nothing — only the later
    // activation does.
    let quote = trading.quote_only(sample_trade_parameters(), None).await?;

    // Place under `Authorization::pre_sign()` with the explicit Safe owner. The
    // same `place_swap` call would take `Authorization::ecdsa(&signer)` for an EOA
    // and resolve to `OrderPlacement::Live`; the pre-sign scheme selects the
    // pending-activation arm instead.
    let placement = trading
        .place_swap(&quote, SAFE_OWNER, Authorization::pre_sign(), None)
        .await?;

    // The order UID is reachable only through the pending-activation arm, which
    // also yields the activation the Safe still owes — the type forbids posting a
    // pre-sign order and dropping its on-chain step.
    let OrderPlacement::PendingActivation {
        order_uid,
        activation,
    } = placement
    else {
        return Err("a pre-sign placement must resolve to PendingActivation".into());
    };

    // The activation is the ordered approve-then-set-pre-signature pair, each a
    // gas-free zero-value call for one smart-account batch.
    let [approve, set_pre_signature] = activation.calls.as_slice() else {
        return Err(
            "the activation must carry exactly the approve and setPreSignature calls".into(),
        );
    };

    // The posted order carries the pre-sign scheme; the orderbook accepts it with
    // no balance or allowance, which is why the approve is part of the activation
    // rather than the placement.
    let posted = orderbook
        .recorded()
        .sent_orders
        .last()
        .cloned()
        .expect("a pre-sign order must be recorded");

    let report = json!({
        "surface": "cow_sdk::trading::Trading::place_swap + Authorization::pre_sign",
        "mode": "simulated-transport",
        "owner": SAFE_OWNER.to_hex_string(),
        "placement": "PendingActivation",
        "orderUid": order_uid.to_hex_string(),
        "postedOrder": {
            "signingScheme": posted.signing_scheme,
            "isPreSign": posted.signing_scheme == SigningScheme::PreSign
        },
        // The two calls the Safe batches from its own account, in order.
        "activation": {
            "callCount": activation.calls.len(),
            "approve": {
                "step": "ERC-20 approve (vault relayer allowance)",
                "to": approve.to.to_hex_string(),
                "value": approve.value.to_string(),
                "callDataPrefix": call_data_prefix(&approve.data)
            },
            "setPreSignature": {
                "step": "settlement setPreSignature(uid, true)",
                "to": set_pre_signature.to.to_hex_string(),
                "value": set_pre_signature.value.to_string(),
                "callDataPrefix": call_data_prefix(&set_pre_signature.data)
            }
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
