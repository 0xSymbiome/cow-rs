mod common;

use cow_sdk_contracts::{ContractId, Registry};
use cow_sdk_core::{Amount, CowEnv, SupportedChainId};
use cow_sdk_trading::{
    AllowanceParameters, ApprovalParameters, approval_transaction, approve_cow_protocol,
    cow_protocol_allowance,
};

use crate::common::{ALT_RECEIVER, COW, MockProvider, MockSigner, OWNER, address};

#[tokio::test]
async fn allowance_reads_use_runtime_chain_resolution_and_explicit_overrides() {
    let provider = MockProvider::default();
    let result = cow_protocol_allowance(
        &provider,
        &address(COW),
        &address(OWNER),
        SupportedChainId::Sepolia,
        CowEnv::Prod,
        None,
    )
    .await
    .expect("allowance read should succeed");
    let state = provider
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let expected_spender = Registry::default()
        .address(
            ContractId::VaultRelayer,
            SupportedChainId::Sepolia,
            CowEnv::Prod,
        )
        .expect("canonical vault-relayer address is registered on sepolia");

    assert_eq!(
        result,
        Amount::new("1000000000000000000").expect("test allowance literal must be valid")
    );
    let args: (String, String) = serde_json::from_str(
        &state
            .last_contract_call
            .expect("read contract call must be captured")
            .args_json,
    )
    .expect("args json must remain valid");
    assert_eq!(args.0, address(OWNER).to_hex_string());
    assert_eq!(args.1, expected_spender.to_hex_string());

    let custom = address(ALT_RECEIVER);
    let tx = approval_transaction(
        &ApprovalParameters::new(
            address(COW),
            Amount::new("123456").expect("test approval amount literal must be valid"),
        )
        .with_chain_id(SupportedChainId::Sepolia)
        .with_env(CowEnv::Prod)
        .with_vault_relayer_override(custom),
        SupportedChainId::Mainnet,
        CowEnv::Staging,
    )
    .expect("approval transaction should build");

    assert_eq!(tx.to, Some(address(COW)));
    let data_lower = tx
        .data
        .as_ref()
        .map(cow_sdk_core::HexData::to_hex_string)
        .unwrap_or_default()
        .to_lowercase();
    let custom_lower = custom.to_hex_string();
    let custom_inner = custom_lower.trim_start_matches("0x").to_lowercase();
    assert!(data_lower.contains(&custom_inner));
}

#[tokio::test]
async fn approval_submission_returns_transaction_hash() {
    let signer = MockSigner::default();
    let tx_hash = approve_cow_protocol(
        &signer,
        &ApprovalParameters::new(
            address(COW),
            Amount::new("1000").expect("test approval amount literal must be valid"),
        )
        .with_chain_id(SupportedChainId::Sepolia)
        .with_env(CowEnv::Prod),
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    )
    .await
    .expect("approval send should succeed");

    assert_eq!(tx_hash.to_hex_string(), crate::common::TX_HASH);
}

#[test]
fn approval_transaction_accepts_max_uint256_amount() {
    let tx = approval_transaction(
        &ApprovalParameters::new(
            address(COW),
            Amount::new(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935",
            )
            .expect("max uint256 literal must be valid"),
        )
        .with_chain_id(SupportedChainId::Sepolia)
        .with_env(CowEnv::Prod),
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    )
    .expect("max uint256 approval amount should encode");

    assert!(
        tx.data
            .as_ref()
            .map(cow_sdk_core::HexData::to_hex_string)
            .unwrap_or_default()
            .ends_with(&"f".repeat(64))
    );
}

#[test]
fn parameter_structs_preserve_call_level_chain_and_override_values() {
    let allowance = AllowanceParameters::new(address(COW), address(OWNER))
        .with_chain_id(SupportedChainId::Mainnet)
        .with_env(CowEnv::Staging)
        .with_vault_relayer_override(address(ALT_RECEIVER));
    let approval = ApprovalParameters::new(
        address(COW),
        Amount::new("42").expect("test approval amount literal must be valid"),
    )
    .with_chain_id(SupportedChainId::Mainnet)
    .with_env(CowEnv::Staging)
    .with_vault_relayer_override(address(ALT_RECEIVER));

    assert_eq!(allowance.chain_id, Some(SupportedChainId::Mainnet));
    assert_eq!(approval.chain_id, Some(SupportedChainId::Mainnet));
    assert_eq!(
        allowance.vault_relayer_override,
        Some(address(ALT_RECEIVER))
    );
    assert_eq!(approval.vault_relayer_override, Some(address(ALT_RECEIVER)));
}
