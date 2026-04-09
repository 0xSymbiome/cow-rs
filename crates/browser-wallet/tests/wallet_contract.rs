use cow_sdk_browser_wallet::{BrowserWallet, MockEip1193Transport, WalletEvent};
use cow_sdk_core::AsyncSigner;
use cow_sdk_core::{SupportedChainId, TypedDataDomain, TypedDataField};

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
