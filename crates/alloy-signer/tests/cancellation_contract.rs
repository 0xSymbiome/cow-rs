#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy_signer::{LocalAlloySigner, SignerError};
use cow_sdk_core::{Cancellable, CancellationToken, Signer, SupportedChainId};

#[tokio::test]
async fn cancel_with_propagates_cancelled_through_question_mark() {
    let token = CancellationToken::new();
    token.cancel();

    let signer = LocalAlloySigner::builder()
        .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .unwrap();

    let result: Result<_, SignerError> = async {
        signer
            .sign_message(b"cancelled")
            .cancel_with(&token)
            .await?;
        Ok(())
    }
    .await;

    assert!(matches!(result, Err(SignerError::Cancelled)));
}
