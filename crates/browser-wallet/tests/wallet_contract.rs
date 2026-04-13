#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_browser_wallet::{
    BrowserWallet, BrowserWalletError, InjectedWalletDetectionOptions, MockEip1193Transport,
    WalletChainChangeKind, WalletChainParameters, WalletEvent, WalletNativeCurrency,
};
use cow_sdk_core::AsyncSigner;
use cow_sdk_core::{
    SupportedChainId, TypedDataDomain, TypedDataField, TypedDataPayload, TypedDataTypes,
};

fn supported_domain(chain_id: SupportedChainId) -> TypedDataDomain {
    TypedDataDomain {
        name: "Gnosis Protocol".to_owned(),
        version: "v2".to_owned(),
        chain_id: u64::from(chain_id),
        verifying_contract: cow_sdk_core::Address::new(
            "0x9008D19f58AAbD9eD0D60971565AA8510560ab41",
        )
        .unwrap(),
    }
}

fn order_typed_fields() -> Vec<TypedDataField> {
    vec![
        TypedDataField {
            name: "sellToken".to_owned(),
            kind: "address".to_owned(),
        },
        TypedDataField {
            name: "buyToken".to_owned(),
            kind: "address".to_owned(),
        },
        TypedDataField {
            name: "receiver".to_owned(),
            kind: "address".to_owned(),
        },
        TypedDataField {
            name: "sellAmount".to_owned(),
            kind: "uint256".to_owned(),
        },
        TypedDataField {
            name: "buyAmount".to_owned(),
            kind: "uint256".to_owned(),
        },
        TypedDataField {
            name: "validTo".to_owned(),
            kind: "uint32".to_owned(),
        },
        TypedDataField {
            name: "appData".to_owned(),
            kind: "bytes32".to_owned(),
        },
        TypedDataField {
            name: "feeAmount".to_owned(),
            kind: "uint256".to_owned(),
        },
        TypedDataField {
            name: "kind".to_owned(),
            kind: "string".to_owned(),
        },
        TypedDataField {
            name: "partiallyFillable".to_owned(),
            kind: "bool".to_owned(),
        },
        TypedDataField {
            name: "sellTokenBalance".to_owned(),
            kind: "string".to_owned(),
        },
        TypedDataField {
            name: "buyTokenBalance".to_owned(),
            kind: "string".to_owned(),
        },
    ]
}

fn eip712_domain_fields() -> Vec<TypedDataField> {
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
    ]
}

fn legacy_order_message() -> &'static str {
    r#"{"sellToken":"0x1111111111111111111111111111111111111111","buyToken":"0x2222222222222222222222222222222222222222","receiver":"0x3333333333333333333333333333333333333333","sellAmount":"1","buyAmount":"2","validTo":1,"appData":"0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","feeAmount":"0","kind":"sell","partiallyFillable":false,"sellTokenBalance":"erc20","buyTokenBalance":"erc20"}"#
}

fn legacy_cancellation_fields() -> Vec<TypedDataField> {
    vec![TypedDataField {
        name: "orderUids".to_owned(),
        kind: "bytes[]".to_owned(),
    }]
}

fn legacy_cancellation_message() -> &'static str {
    r#"{"orderUids":["0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"]}"#
}

fn order_payload(chain_id: SupportedChainId) -> TypedDataPayload {
    let mut order_types = TypedDataTypes::new();
    order_types.insert("Order".to_owned(), order_typed_fields());
    order_types.insert("EIP712Domain".to_owned(), eip712_domain_fields());

    TypedDataPayload {
        domain: supported_domain(chain_id),
        primary_type: "Order".to_owned(),
        types: order_types,
        message: legacy_order_message().to_owned(),
    }
}

fn smart_hook_payload(chain_id: SupportedChainId) -> TypedDataPayload {
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
    types.insert("EIP712Domain".to_owned(), eip712_domain_fields());

    TypedDataPayload {
        domain: supported_domain(chain_id),
        primary_type: "SmartHookAction".to_owned(),
        types,
        message:
            r#"{"actor":"0x1111111111111111111111111111111111111111","config":{"salt":"0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}"#
                .to_owned(),
    }
}

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
            .sign_typed_data_payload(&order_payload(SupportedChainId::Sepolia))
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

    assert_eq!(
        signer
            .sign_typed_data_payload(&smart_hook_payload(SupportedChainId::Sepolia))
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

#[tokio::test(flavor = "current_thread")]
async fn legacy_typed_data_compatibility_is_limited_to_order_and_cancellation_shapes() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport(transport.clone());
    wallet.connect().await.unwrap();
    let signer = wallet.signer();
    let domain = supported_domain(SupportedChainId::Sepolia);
    let order_fields = order_typed_fields();
    let cancellation_fields = legacy_cancellation_fields();

    signer
        .sign_typed_data(&domain, &order_fields, legacy_order_message())
        .await
        .unwrap();

    signer
        .sign_typed_data(&domain, &cancellation_fields, legacy_cancellation_message())
        .await
        .unwrap();

    let typed_requests = transport
        .request_log()
        .into_iter()
        .filter(|record| record.method == "eth_signTypedData_v4")
        .collect::<Vec<_>>();
    assert_eq!(typed_requests.len(), 2);

    let first: serde_json::Value = serde_json::from_str(
        typed_requests[0]
            .params
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()[1]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let second: serde_json::Value = serde_json::from_str(
        typed_requests[1]
            .params
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()[1]
            .as_str()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(first["primaryType"], serde_json::json!("Order"));
    assert_eq!(
        second["primaryType"],
        serde_json::json!("OrderCancellations")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn legacy_typed_data_compatibility_rejects_unknown_primary_type_shapes() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport(transport);
    wallet.connect().await.unwrap();
    let signer = wallet.signer();

    let error = signer
        .sign_typed_data(
            &TypedDataDomain {
                name: "Gnosis Protocol".to_owned(),
                version: "v2".to_owned(),
                chain_id: u64::from(SupportedChainId::Sepolia),
                verifying_contract: cow_sdk_core::Address::new(
                    "0x9008D19f58AAbD9eD0D60971565AA8510560ab41",
                )
                .unwrap(),
            },
            &[TypedDataField {
                name: "actor".to_owned(),
                kind: "address".to_owned(),
            }],
            r#"{"actor":"0x1111111111111111111111111111111111111111"}"#,
        )
        .await
        .unwrap_err();

    assert_eq!(
        error,
        BrowserWalletError::Serialization {
            message: "legacy sign_typed_data compatibility supports only CoW order and order cancellation payloads; use sign_typed_data_payload for explicit primary types".to_owned(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn injected_discovery_keeps_bounded_timeout_contract_off_wasm() {
    let discovery = BrowserWallet::discover_with(InjectedWalletDetectionOptions::new(750))
        .await
        .unwrap();

    assert!(discovery.is_empty());
    assert_eq!(discovery.timeout_ms(), 750);
    assert!(!discovery.used_legacy_fallback());
    assert_eq!(discovery.wallets(), Vec::new());
}

#[tokio::test(flavor = "current_thread")]
async fn transport_events_keep_wallet_session_synchronized() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport(transport.clone());
    let alternate =
        cow_sdk_core::Address::new("0x5555555555555555555555555555555555555555").unwrap();

    transport.emit_connected(Some(u64::from(SupportedChainId::Sepolia)));
    transport.emit_accounts_changed(vec![alternate.clone()]);
    transport.emit_chain_changed(u64::from(SupportedChainId::Mainnet));

    let session = wallet.session();
    assert!(session.connected);
    assert_eq!(session.selected_account, Some(alternate.clone()));
    assert_eq!(session.accounts, vec![alternate.clone()]);
    assert_eq!(session.chain_id, Some(u64::from(SupportedChainId::Mainnet)));

    transport.emit_disconnected(Some("provider disconnected".to_owned()));

    let session = wallet.session();
    assert!(!session.connected);
    assert!(session.accounts.is_empty());
    assert!(session.selected_account.is_none());
    assert_eq!(session.chain_id, None);

    let events = wallet.take_events();
    assert!(events.iter().any(
        |event| matches!(event, WalletEvent::Connected { chain_id } if *chain_id == Some(u64::from(SupportedChainId::Sepolia)))
    ));
    assert!(events.iter().any(
        |event| matches!(event, WalletEvent::AccountsChanged { accounts } if accounts == &vec![alternate.clone()])
    ));
    assert!(events.iter().any(
        |event| matches!(event, WalletEvent::ChainChanged { chain_id } if *chain_id == u64::from(SupportedChainId::Mainnet))
    ));
    assert!(events.iter().any(
        |event| matches!(event, WalletEvent::Disconnected { message } if message.as_deref() == Some("provider disconnected"))
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn listener_lifetime_follows_wallet_and_provider_values() {
    let transport = MockEip1193Transport::sepolia();
    assert_eq!(transport.listener_count(), 0);

    let wallet = BrowserWallet::from_transport(transport.clone());
    assert_eq!(transport.listener_count(), 1);

    let provider = wallet.provider();
    let wallet_clone = wallet.clone();

    drop(wallet);
    assert_eq!(transport.listener_count(), 1);

    drop(provider);
    assert_eq!(transport.listener_count(), 1);

    drop(wallet_clone);
    assert_eq!(transport.listener_count(), 0);
}

#[tokio::test(flavor = "current_thread")]
async fn add_chain_uses_typed_chain_parameters_and_keeps_request_shape_explicit() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport(transport.clone());
    wallet.connect().await.unwrap();

    let chain = WalletChainParameters::for_supported_chain(SupportedChainId::Base)
        .try_with_rpc_url("https://base.example.invalid/rpc")
        .unwrap()
        .try_with_block_explorer_url("https://base.example.invalid/explorer")
        .unwrap();

    let result = wallet.add_chain(&chain).await.unwrap();
    assert_eq!(result.kind, WalletChainChangeKind::Added);
    assert_eq!(result.requested_chain_id, SupportedChainId::Base);
    assert_eq!(
        result.session.chain_id,
        Some(u64::from(SupportedChainId::Sepolia))
    );

    let add_chain_request = transport
        .request_log()
        .into_iter()
        .find(|record| record.method == "wallet_addEthereumChain")
        .unwrap();
    let payload = add_chain_request.params.unwrap();
    let request = payload.as_array().unwrap().first().unwrap();

    assert_eq!(request["chainName"], serde_json::json!("Base"));
    assert_eq!(request["chainId"], serde_json::json!("0x2105"));
    assert_eq!(
        request["nativeCurrency"]["symbol"],
        serde_json::json!("ETH")
    );
    assert_eq!(
        request["rpcUrls"][0],
        serde_json::json!("https://base.example.invalid/rpc")
    );
    assert_eq!(
        request["blockExplorerUrls"][0],
        serde_json::json!("https://base.example.invalid/explorer")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn switch_or_add_chain_adds_then_switches_when_chain_is_not_present() {
    let transport = MockEip1193Transport::sepolia();
    transport.set_added_chains(vec![SupportedChainId::Sepolia]);
    let wallet = BrowserWallet::from_transport(transport.clone());
    wallet.connect().await.unwrap();

    let chain = WalletChainParameters::for_supported_chain(SupportedChainId::Base)
        .try_with_rpc_url("https://base.example.invalid/rpc")
        .unwrap();

    let result = wallet.switch_or_add_chain(&chain).await.unwrap();

    assert_eq!(result.kind, WalletChainChangeKind::AddedThenSwitched);
    assert_eq!(result.requested_chain_id, SupportedChainId::Base);
    assert_eq!(
        result.session.chain_id,
        Some(u64::from(SupportedChainId::Base))
    );

    let request_log = transport.request_log();
    let methods = request_log
        .iter()
        .map(|record| record.method.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        vec![
            "eth_requestAccounts",
            "eth_chainId",
            "wallet_switchEthereumChain",
            "wallet_addEthereumChain",
            "wallet_switchEthereumChain",
            "eth_accounts",
            "eth_chainId",
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn switch_or_add_chain_does_not_add_when_chain_not_added_targets_a_different_chain() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport(transport.clone());
    wallet.connect().await.unwrap();
    transport.fail_method(
        "wallet_switchEthereumChain",
        BrowserWalletError::ChainNotAdded {
            chain_id: u64::from(SupportedChainId::Mainnet),
            method: "wallet_switchEthereumChain".to_owned(),
            code: 4902,
            message: "mock wallet does not know chain 1".to_owned(),
        },
    );

    let chain = WalletChainParameters::for_supported_chain(SupportedChainId::Base)
        .try_with_rpc_url("https://base.example.invalid/rpc")
        .unwrap();

    let error = wallet.switch_or_add_chain(&chain).await.unwrap_err();
    assert_eq!(
        error,
        BrowserWalletError::ChainNotAdded {
            chain_id: u64::from(SupportedChainId::Mainnet),
            method: "wallet_switchEthereumChain".to_owned(),
            code: 4902,
            message: "mock wallet does not know chain 1".to_owned(),
        }
    );

    let methods = transport
        .request_log()
        .into_iter()
        .map(|record| record.method)
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        vec![
            "eth_requestAccounts".to_owned(),
            "eth_chainId".to_owned(),
            "wallet_switchEthereumChain".to_owned(),
        ]
    );
}

#[test]
fn chain_configuration_validation_rejects_invalid_inputs_before_rpc() {
    let invalid = WalletChainParameters::for_supported_chain(SupportedChainId::Base);
    assert_eq!(
        invalid.validate().unwrap_err(),
        BrowserWalletError::InvalidChainConfiguration {
            chain_id: u64::from(SupportedChainId::Base),
            message: "wallet add-chain requires at least one RPC URL".to_owned(),
        }
    );

    let invalid_url = WalletChainParameters::for_supported_chain(SupportedChainId::Base)
        .try_with_rpc_url("wss://base.example.invalid/rpc")
        .unwrap_err();
    assert_eq!(
        invalid_url,
        BrowserWalletError::InvalidChainConfiguration {
            chain_id: u64::from(SupportedChainId::Base),
            message: "RPC URL must use an http or https URL".to_owned(),
        }
    );

    let invalid_currency = WalletNativeCurrency::new("", "ETH", 18).unwrap_err();
    assert_eq!(
        invalid_currency,
        BrowserWalletError::InvalidChainConfiguration {
            chain_id: 0,
            message: "native currency name must not be empty".to_owned(),
        }
    );
}
