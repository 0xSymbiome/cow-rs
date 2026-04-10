mod common;

use cow_sdk_core::{Amount, CowEnv, SupportedChainId, vault_relayer_address};
use cow_sdk_trading::{
    AllowanceParameters, ApprovalParameters, approval_transaction, approve_cow_protocol,
    get_cow_protocol_allowance,
};

use crate::common::{ALT_RECEIVER, COW, MockProvider, MockSigner, OWNER, address};

#[test]
fn allowance_reads_use_runtime_chain_resolution_and_explicit_overrides() {
    let provider = MockProvider::default();
    let result = get_cow_protocol_allowance(
        &provider,
        &address(COW),
        &address(OWNER),
        SupportedChainId::Sepolia,
        CowEnv::Prod,
        None,
    )
    .expect("allowance read should succeed");
    let state = provider
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let expected_spender = vault_relayer_address(SupportedChainId::Sepolia, CowEnv::Prod);

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
    assert_eq!(args.0, address(OWNER).as_str());
    assert_eq!(args.1, expected_spender.as_str());

    let custom = address(ALT_RECEIVER);
    let tx = approval_transaction(
        &ApprovalParameters {
            token_address: address(COW),
            amount: Amount::new("123456").expect("test approval amount literal must be valid"),
            chain_id: Some(SupportedChainId::Sepolia),
            env: Some(CowEnv::Prod),
            vault_relayer_address: Some(custom.clone()),
        },
        SupportedChainId::Mainnet,
        CowEnv::Staging,
    )
    .expect("approval transaction should build");

    assert_eq!(tx.to, Some(address(COW)));
    assert!(
        tx.data
            .as_ref()
            .map(|value| value.as_str())
            .unwrap_or_default()
            .to_lowercase()
            .contains(
                custom
                    .as_str()
                    .trim_start_matches("0x")
                    .to_lowercase()
                    .as_str()
            )
    );
}

#[test]
fn approval_submission_returns_transaction_hash() {
    let signer = MockSigner::default();
    let tx_hash = approve_cow_protocol(
        &signer,
        &ApprovalParameters {
            token_address: address(COW),
            amount: Amount::new("1000").expect("test approval amount literal must be valid"),
            chain_id: Some(SupportedChainId::Sepolia),
            env: Some(CowEnv::Prod),
            vault_relayer_address: None,
        },
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    )
    .expect("approval send should succeed");

    assert_eq!(tx_hash.as_str(), crate::common::TX_HASH);
}

#[test]
fn approval_transaction_accepts_max_uint256_amount() {
    let tx = approval_transaction(
        &ApprovalParameters {
            token_address: address(COW),
            amount: Amount::new(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935",
            )
            .expect("max uint256 literal must be valid"),
            chain_id: Some(SupportedChainId::Sepolia),
            env: Some(CowEnv::Prod),
            vault_relayer_address: None,
        },
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    )
    .expect("max uint256 approval amount should encode");

    assert!(
        tx.data
            .as_ref()
            .map(|value| value.as_str())
            .unwrap_or_default()
            .ends_with(&"f".repeat(64))
    );
}

#[test]
fn parameter_structs_preserve_call_level_chain_and_override_values() {
    let allowance = AllowanceParameters {
        token_address: address(COW),
        owner: address(OWNER),
        chain_id: Some(SupportedChainId::Mainnet),
        env: Some(CowEnv::Staging),
        vault_relayer_address: Some(address(ALT_RECEIVER)),
    };
    let approval = ApprovalParameters {
        token_address: address(COW),
        amount: Amount::new("42").expect("test approval amount literal must be valid"),
        chain_id: Some(SupportedChainId::Mainnet),
        env: Some(CowEnv::Staging),
        vault_relayer_address: Some(address(ALT_RECEIVER)),
    };

    assert_eq!(allowance.chain_id, Some(SupportedChainId::Mainnet));
    assert_eq!(approval.chain_id, Some(SupportedChainId::Mainnet));
    assert_eq!(allowance.vault_relayer_address, Some(address(ALT_RECEIVER)));
    assert_eq!(approval.vault_relayer_address, Some(address(ALT_RECEIVER)));
}
