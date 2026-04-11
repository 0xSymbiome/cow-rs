use cow_sdk_browser_wallet::{BrowserWallet, MockEip1193Transport, WalletEvent};
use cow_sdk_core::AsyncSigner;
use cow_sdk_core::{
    SupportedChainId, TypedDataDomain, TypedDataField, TypedDataPayload, TypedDataTypes,
};

#[tokio::test(flavor = "current_thread")]
async fn mock_wallet_connects_switches_chain_and_signs() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport(transport.clone());

    let session = wallet.connect().await.unwrap();
    assert!(session.connected);
    assert_eq!(session.chain_id, Some(u64::from(SupportedChainId::Sepolia)));
    assert_eq!(session.accounts.len(), 1);

    let signer = wallet.signer();
    assert_eq!(
        signer.sign_message(b"cow-rs").await.unwrap(),
        format!("0x{}1b", "11".repeat(64))
    );
    assert_eq!(
        signer
            .sign_typed_data(
                &TypedDataDomain {
                    name: "Gnosis Protocol".to_owned(),
                    version: "v2".to_owned(),
                    chain_id: u64::from(SupportedChainId::Sepolia),
                    verifying_contract: cow_sdk_core::Address::new(
                        "0x9008D19f58AAbD9eD0D60971565AA8510560ab41"
                    )
                    .unwrap(),
                },
                &[TypedDataField {
                    name: "sellToken".to_owned(),
                    kind: "address".to_owned(),
                }],
                r#"{"sellToken":"0x1111111111111111111111111111111111111111"}"#,
            )
            .await
            .unwrap(),
        format!("0x{}1c", "22".repeat(64))
    );

    let updated = wallet
        .switch_chain(SupportedChainId::Mainnet)
        .await
        .unwrap();
    assert_eq!(updated.chain_id, Some(u64::from(SupportedChainId::Mainnet)));
    assert!(
        wallet
            .take_events()
            .into_iter()
            .any(|event| matches!(event, WalletEvent::ChainChanged { chain_id } if chain_id == 1))
    );

    let reset = wallet.reset_session();
    assert!(!reset.connected);
    assert_eq!(reset.wallet_label, "Mock Wallet");
    assert!(reset.accounts.is_empty());
    assert_eq!(reset.chain_id, None);
    assert!(wallet.take_events().into_iter().any(
        |event| matches!(event, WalletEvent::SessionUpdated { current, .. } if !current.connected)
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn explicit_typed_data_payloads_preserve_custom_primary_types_and_nested_types() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport(transport.clone());

    wallet.connect().await.unwrap();
    let signer = wallet.signer();
    let mut types = TypedDataTypes::new();
    types.insert(
        "SmartHookAction".to_owned(),
        vec![
            TypedDataField {
                name: "actor".to_owned(),
                kind: "address".to_owned(),
            },
            TypedDataField {
                name: "config".to_owned(),
                kind: "HookConfig".to_owned(),
            },
        ],
    );
    types.insert(
        "HookConfig".to_owned(),
        vec![TypedDataField {
            name: "salt".to_owned(),
            kind: "bytes32".to_owned(),
        }],
    );
    types.insert(
        "EIP712Domain".to_owned(),
        vec![
            TypedDataField {
                name: "name".to_owned(),
                kind: "string".to_owned(),
            },
            TypedDataField {
                name: "version".to_owned(),
                kind: "string".to_owned(),
            },
            TypedDataField {
                name: "chainId".to_owned(),
                kind: "uint256".to_owned(),
            },
            TypedDataField {
                name: "verifyingContract".to_owned(),
                kind: "address".to_owned(),
            },
        ],
    );

    assert_eq!(
        signer
            .sign_typed_data_payload(&TypedDataPayload {
                domain: TypedDataDomain {
                    name: "Gnosis Protocol".to_owned(),
                    version: "v2".to_owned(),
                    chain_id: u64::from(SupportedChainId::Sepolia),
                    verifying_contract: cow_sdk_core::Address::new(
                        "0x9008D19f58AAbD9eD0D60971565AA8510560ab41"
                    )
                    .unwrap(),
                },
                primary_type: "SmartHookAction".to_owned(),
                types,
                message:
                    r#"{"actor":"0x1111111111111111111111111111111111111111","config":{"salt":"0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}"#
                        .to_owned(),
            })
            .await
            .unwrap(),
        format!("0x{}1c", "22".repeat(64))
    );

    let typed_data_request = transport
        .request_log()
        .into_iter()
        .find(|record| record.method == "eth_signTypedData_v4")
        .unwrap();
    let params = typed_data_request.params.unwrap();
    let typed_data = params.as_array().unwrap()[1].as_str().unwrap();
    let typed_data: serde_json::Value = serde_json::from_str(typed_data).unwrap();

    assert_eq!(
        typed_data["primaryType"],
        serde_json::json!("SmartHookAction")
    );
    assert_eq!(
        typed_data["types"]["SmartHookAction"][1]["type"],
        serde_json::json!("HookConfig")
    );
    assert_eq!(
        typed_data["types"]["HookConfig"][0]["name"],
        serde_json::json!("salt")
    );
    assert_eq!(
        typed_data["message"]["config"]["salt"],
        serde_json::json!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
}
