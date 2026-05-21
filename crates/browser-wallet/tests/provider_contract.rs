#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_browser_wallet::{
    BrowserWallet, BrowserWalletError, Eip1193ProviderBuilder, MockEip1193Transport, Origin,
    WalletChainParameters,
};
use cow_sdk_core::AsyncProvider;
use cow_sdk_core::{Address, Amount, ContractCall, HexData, SupportedChainId, TransactionRequest};

#[tokio::test(flavor = "current_thread")]
async fn mock_provider_satisfies_async_provider_contracts() {
    let transport = MockEip1193Transport::sepolia();
    transport.set_connected(true);
    let wallet = BrowserWallet::from_transport_or_panic(transport.clone());
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
                Some(Amount::ZERO),
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

#[tokio::test(flavor = "current_thread")]
async fn wallet_add_chain_payload_urls_are_not_subject_to_external_host_policy() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport_or_panic(transport.clone());
    wallet.connect().await.unwrap();

    let chain = WalletChainParameters::for_supported_chain(SupportedChainId::Base)
        .try_with_rpc_url("https://rpc.private.example.invalid/base")
        .unwrap()
        .try_with_block_explorer_url("https://explorer.private.example.invalid/base")
        .unwrap();

    wallet.add_chain(&chain).await.unwrap();

    let add_chain_request = transport
        .request_log()
        .into_iter()
        .find(|record| record.method == "wallet_addEthereumChain")
        .expect("wallet_addEthereumChain request must be recorded");
    let payload = add_chain_request.params.unwrap();
    let request = payload.as_array().unwrap().first().unwrap();
    assert_eq!(
        request["rpcUrls"][0],
        serde_json::json!("https://rpc.private.example.invalid/base")
    );
    assert_eq!(
        request["blockExplorerUrls"][0],
        serde_json::json!("https://explorer.private.example.invalid/base")
    );
}

#[test]
fn trusted_origin_accepts_documented_schemes_and_rejects_others() {
    for accepted in [
        "https://wallet.example",
        "http://localhost:3000",
        "test://wallet/sepolia",
        "transport:Mock Wallet",
        "io.rabby",
    ] {
        assert!(Origin::new(accepted).is_ok(), "{accepted} must be accepted");
    }

    for rejected in [
        "javascript:alert(1)",
        "data:text/plain,wallet",
        "file:///tmp/wallet",
    ] {
        let error = Origin::new(rejected).unwrap_err();
        assert!(matches!(
            error,
            BrowserWalletError::InvalidProviderOrigin { .. }
        ));
    }
}
