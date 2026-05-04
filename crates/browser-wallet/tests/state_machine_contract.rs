#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_browser_wallet::{
    BrowserWallet, BrowserWalletError, Eip1193Transport, MockEip1193Transport, WalletEvent,
};
use cow_sdk_core::{Address, SupportedChainId};
use serde_json::{Value, json};

#[tokio::test(flavor = "current_thread")]
async fn wallet_session_state_machine_keeps_reset_and_refresh_boundaries_explicit() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport_or_panic(transport);

    let initial = wallet.session();
    assert!(!initial.connected);
    assert!(initial.accounts.is_empty());
    assert!(initial.selected_account.is_none());
    assert_eq!(initial.chain_id, None);

    let passive = wallet
        .refresh_session()
        .await
        .expect("passive refresh should succeed deterministically");
    assert!(!passive.connected);
    assert!(passive.accounts.is_empty());
    assert!(passive.selected_account.is_none());
    assert_eq!(passive.chain_id, Some(u64::from(SupportedChainId::Sepolia)));

    let connected = wallet
        .connect()
        .await
        .expect("connect should populate the deterministic session");
    assert!(connected.connected);
    assert_eq!(connected.accounts.len(), 1);
    assert_eq!(
        connected.selected_account,
        connected.accounts.first().cloned()
    );
    assert_eq!(
        connected.chain_id,
        Some(u64::from(SupportedChainId::Sepolia))
    );

    let reset = wallet.reset_session();
    assert!(!reset.connected);
    assert!(reset.accounts.is_empty());
    assert!(reset.selected_account.is_none());
    assert_eq!(reset.chain_id, None);
    assert_eq!(reset.wallet_label, "Mock Wallet");

    let restored = wallet
        .refresh_session()
        .await
        .expect("refresh should restore session state from the transport");
    assert!(restored.connected);
    assert_eq!(restored.accounts.len(), 1);
    assert_eq!(
        restored.selected_account,
        restored.accounts.first().cloned()
    );
    assert_eq!(
        restored.chain_id,
        Some(u64::from(SupportedChainId::Sepolia))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn wallet_event_state_machine_tracks_disconnect_and_explicit_reconnect() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport_or_panic(transport.clone());
    let alternate = Address::new("0x5555555555555555555555555555555555555555").unwrap();

    wallet
        .connect()
        .await
        .expect("initial connect should succeed deterministically");
    transport.emit_accounts_changed(vec![alternate.clone()]);
    transport.emit_chain_changed(u64::from(SupportedChainId::Mainnet));

    let updated = wallet.session();
    assert!(updated.connected);
    assert_eq!(updated.accounts, vec![alternate.clone()]);
    assert_eq!(updated.selected_account, Some(alternate.clone()));
    assert_eq!(updated.chain_id, Some(u64::from(SupportedChainId::Mainnet)));

    transport.emit_disconnected(Some("provider disconnected".to_owned()));

    let disconnected = wallet.session();
    assert!(!disconnected.connected);
    assert!(disconnected.accounts.is_empty());
    assert!(disconnected.selected_account.is_none());
    assert_eq!(disconnected.chain_id, None);

    let reconnected = wallet
        .connect()
        .await
        .expect("explicit reconnect should restore transport-owned state");
    assert!(reconnected.connected);
    assert_eq!(reconnected.selected_account, Some(alternate));
    assert_eq!(
        reconnected.chain_id,
        Some(u64::from(SupportedChainId::Mainnet))
    );

    let events = wallet.take_events();
    assert!(events.iter().any(
        |event| matches!(event, WalletEvent::Disconnected { message } if message.as_deref() == Some("provider disconnected"))
    ));
    assert!(events.iter().any(
        |event| matches!(event, WalletEvent::SessionUpdated { current, .. } if current.connected && current.chain_id == Some(u64::from(SupportedChainId::Mainnet)))
    ));
}

#[derive(Debug)]
struct MalformedChainIdTransport;

#[async_trait::async_trait(?Send)]
impl Eip1193Transport for MalformedChainIdTransport {
    fn label(&self) -> &'static str {
        "Malformed Fixture Wallet"
    }

    async fn request(
        &self,
        method: &str,
        _params: Option<Value>,
    ) -> Result<Value, BrowserWalletError> {
        match method {
            "eth_accounts" => Ok(json!(["0x4444444444444444444444444444444444444444"])),
            "eth_chainId" => Ok(json!({ "not": "a chain id" })),
            other => Err(BrowserWalletError::UnsupportedRpcMethod {
                method: other.to_owned().into(),
                message: "fixture supports only session refresh methods"
                    .to_owned()
                    .into(),
            }),
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_json_rpc_error_classifies_as_typed_provider_error() {
    let wallet = BrowserWallet::from_transport_or_panic(MalformedChainIdTransport);

    let error = wallet
        .refresh_session()
        .await
        .expect_err("malformed eth_chainId response must fail");

    match error {
        BrowserWalletError::MalformedResponse { method, message } => {
            assert_eq!(method.as_inner(), "eth_chainId");
            assert!(
                message
                    .as_inner()
                    .contains("expected string or number chain id"),
                "malformed-response message must retain chain-id parse context: {message}",
            );
        }
        other => panic!("expected MalformedResponse, got {other:?}"),
    }
}
