//! Custom EIP-1271 signing through the public `Eip1271SignatureProvider` seam.
//!
//! A smart-account integration supplies a pre-built EIP-1271 order signature
//! instead of signing locally: implement `Eip1271SignatureProvider`, then wire it
//! through `PostTradeAdditionalParams::with_signing_scheme(SigningScheme::Eip1271)`
//! plus `with_custom_eip1271_signature`, and post a limit order via
//! `Trading::post_limit_order` against the `cow_sdk::testing` doubles. Both knobs
//! are required; the posted order carries the provider's signature under the
//! `eip1271` scheme, and the order owner identifies the smart account.

use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use cow_sdk::core::OrderData;
use cow_sdk::orderbook::SigningScheme;
use cow_sdk::prelude::{SupportedChainId, Trading};
use cow_sdk::signing::eip1271::{Eip1271SignatureError, Eip1271SignatureProvider};
use cow_sdk::trading::{PostTradeAdditionalParams, TradeAdvancedSettings};

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{sample_limit_parameters, sample_owner, sample_quote_response};

/// A smart-account signer that returns a pre-built EIP-1271 signature blob.
struct SmartAccountSigner;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Eip1271SignatureProvider for SmartAccountSigner {
    async fn sign(&self, _order_to_sign: &OrderData) -> Result<String, Eip1271SignatureError> {
        // A real integration forwards the order to a smart account / multisig.
        Ok("0x7e57c0de".to_owned())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    let signer = MockSigner::builder().address(sample_owner()).build();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-native-examples")
        .orderbook_client(Arc::new(orderbook.clone()))
        .build()?;

    // Both knobs are required: the scheme selects the EIP-1271 path, and the
    // provider supplies the signature in place of the local signer. The order
    // owner (carried by the limit parameters) identifies the smart account.
    let advanced = TradeAdvancedSettings::new().with_additional_params(
        PostTradeAdditionalParams::new()
            .with_signing_scheme(SigningScheme::Eip1271)
            .with_custom_eip1271_signature(Arc::new(SmartAccountSigner)),
    );

    let post = trading
        .post_limit_order(sample_limit_parameters(), &signer, Some(&advanced))
        .await?;

    let sent = orderbook.recorded().sent_orders;
    let report = json!({
        "surface": "cow-sdk::signing::eip1271::Eip1271SignatureProvider",
        "mode": "simulated-transport",
        "signingScheme": format!("{:?}", post.signing_scheme),
        "orderSignature": post.signature,
        "postedOrderSignature": sent.first().map(|order| order.signature.clone()),
        "postedOrderCount": sent.len()
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
