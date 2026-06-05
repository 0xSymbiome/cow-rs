//! Order and cancellation signing with typed-payload inspection.
//!
//! Signs an order (`sign_order`) and a cancellation (`sign_order_cancellation`)
//! through a `cow_sdk::testing::MockSigner`, and inspects the EIP-712 typed data
//! (`order_typed_data`), the derived order id (`generate_order_id`), and the
//! EIP-1271 payload (`eip1271_signature_payload`). No network and no real key.

use std::error::Error;

use serde_json::json;

use cow_sdk::prelude::SupportedChainId;
use cow_sdk::signing::{
    eip1271_signature_payload, generate_order_id, order_typed_data, sign_order,
    sign_order_cancellation,
};

use cow_sdk::testing::MockSigner;
use cow_sdk_examples_native::support::{
    sample_order_uid, sample_owner, sample_unsigned_order, text_preview,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let chain_id = SupportedChainId::Sepolia;
    let signer = MockSigner::new();
    let order = sample_unsigned_order();

    // Sign the order (EIP-712 by default through the signer).
    let signed_order = sign_order(&order, chain_id, &signer, None).await?;

    // Derive the typed data and the order id without signing — both are pure
    // functions of the order, chain, and owner.
    let typed_order = order_typed_data(chain_id, &order, None)?;
    let generated = generate_order_id(chain_id, &order, &sample_owner(), None)?;

    // Sign an order cancellation: a separate signed message keyed by the order uid.
    let cancellation =
        sign_order_cancellation(&sample_order_uid(), chain_id, &signer, None).await?;

    // Build the EIP-1271 payload a smart-contract wallet would verify on-chain.
    let eip1271_payload = eip1271_signature_payload(&order, &signed_order.signature)?;

    let report = json!({
        "surface": "cow-sdk::signing",
        "mode": "deterministic",
        "order": {
            "primaryType": typed_order.primary_type,
            "digest": generated.order_digest,
            "orderId": generated.order_id.to_hex_string(),
            "signature": signed_order.signature,
            "scheme": format!("{:?}", signed_order.signing_scheme),
            "eip1271PayloadPrefix": text_preview(&eip1271_payload, 18)
        },
        "cancellation": {
            "orderUid": sample_order_uid().to_hex_string(),
            "signature": cancellation.signature,
            "scheme": format!("{:?}", cancellation.signing_scheme)
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
