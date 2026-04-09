use cow_sdk_browser_wallet::{BrowserWallet, MockEip1193Transport};
use cow_sdk_core::AsyncProvider;
use cow_sdk_core::{Address, ContractCall, SupportedChainId};

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
            .read_contract(&ContractCall {
                address: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
                method: "allowance".to_owned(),
                abi_json: r#"[{"type":"function","name":"allowance","inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#.to_owned(),
                args_json: serde_json::json!([
                    "0x4444444444444444444444444444444444444444",
                    "0x5555555555555555555555555555555555555555"
                ])
                .to_string(),
            })
            .await
            .unwrap(),
        "\"42\""
    );
    assert_eq!(
        provider
            .call(&cow_sdk_core::TransactionRequest {
                to: Some(Address::new("0x1111111111111111111111111111111111111111").unwrap()),
                data: Some("0x1234".to_owned()),
                value: Some("0".to_owned()),
                gas_limit: Some("21000".to_owned()),
            })
            .await
            .unwrap(),
        format!("0x{}2a", "0".repeat(62))
    );
}
