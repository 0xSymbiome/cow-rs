#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_browser_wallet::{
    BrowserWallet, BrowserWalletError, Eip1193ProviderBuilder, MockEip1193Transport, Origin,
};
use cow_sdk_core::AsyncProvider;
use cow_sdk_core::{Address, Amount, ContractCall, HexData, SupportedChainId, TransactionRequest};

#[tokio::test(flavor = "current_thread")]
async fn mock_provider_satisfies_async_provider_contracts() {
    let transport = MockEip1193Transport::sepolia();
    transport.set_connected(true);
    let wallet = BrowserWallet::from_transport(transport.clone());
    wallet.connect().await.unwrap();

    let provider = wallet.provider();
    assert_eq!(
        provider.get_chain_id().await.unwrap(),
        u64::from(SupportedChainId::Sepolia)
    );
    assert_eq!(
        provider
            .read_contract(&ContractCall::new(
                Address::new("0x1111111111111111111111111111111111111111").unwrap(),
                "allowance".to_owned(),
                r#"[{"type":"function","name":"allowance","inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#.to_owned(),
                serde_json::json!([
                    "0x4444444444444444444444444444444444444444",
                    "0x5555555555555555555555555555555555555555"
                ])
                .to_string(),
            ))
            .await
            .unwrap(),
        "\"42\""
    );
    assert_eq!(
        provider
            .call(&TransactionRequest::new(
                Some(Address::new("0x1111111111111111111111111111111111111111").unwrap()),
                Some(HexData::new("0x1234").unwrap()),
                Some(Amount::zero()),
                Some(Amount::from(21_000u32)),
            ))
            .await
            .unwrap(),
        HexData::new(format!("0x{}2a", "0".repeat(62))).unwrap()
    );
}

#[test]
fn anonymous_provider_builder_requires_trusted_origin() {
    let error = Eip1193ProviderBuilder::new(MockEip1193Transport::sepolia())
        .build()
        .unwrap_err();

    assert!(matches!(
        error,
        BrowserWalletError::UntrustedProviderOrigin { .. }
    ));
    let rendered = error.to_string();
    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sepolia"));
}

#[test]
fn provider_builder_accepts_explicit_trusted_origin() {
    let provider = Eip1193ProviderBuilder::new(MockEip1193Transport::sepolia())
        .with_trusted_origin(Origin::new("test://wallet/sepolia").unwrap())
        .build()
        .unwrap();

    assert_eq!(provider.session().wallet_label, "Mock Wallet");
    assert_eq!(
        provider.origin().map(Origin::as_str),
        Some("test://wallet/sepolia")
    );
}
