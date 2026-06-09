//! Cross-crate contract: the COW Shed `ExecuteHooks` typed-data payload — the
//! shape an owner signs through the owned `Signer` trait
//! (`sign_typed_data_payload`) — hashes to the exact same EIP-712 digest as the
//! macro-emitted `execute_hooks_signing_hash`.
//!
//! This is the load-bearing proof that `CowShedHooks::sign` produces a
//! signature the on-chain COW Shed proxy accepts: the dynamic typed-data path
//! (used by every SDK signer) and the canonical `sol!` digest path must agree
//! byte-for-byte. It runs deterministically with no signer and no network — it
//! only converts and hashes.

use alloy_primitives::{Bytes, U256, address, b256};
use cow_sdk_alloy_signer::__seam::cow_typed_data_payload_to_alloy;
use cow_sdk_contracts::DeploymentChainId;
use cow_sdk_contracts::cow_shed::{
    Call, CowShedVersion, cow_shed_eip712_domain, execute_hooks_signing_hash,
    execute_hooks_typed_data_payload, proxy_for,
};

#[test]
fn execute_hooks_payload_digest_matches_macro_digest() {
    for version in [CowShedVersion::V1_0_0, CowShedVersion::V1_0_1] {
        for chain in [DeploymentChainId::Mainnet, DeploymentChainId::GnosisChain] {
            assert_digests_agree(chain, version);
        }
    }
}

fn assert_digests_agree(chain: DeploymentChainId, version: CowShedVersion) {
    let user = address!("0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58");
    let proxy = proxy_for(chain, version, user);

    // A multi-call batch that exercises every `Call` field encoding: a plain
    // call, a value-bearing call with non-empty calldata and `allowFailure`,
    // and a `delegatecall`.
    let calls = vec![
        Call::new(
            address!("0x1111111111111111111111111111111111111111"),
            U256::ZERO,
            Bytes::new(),
        ),
        Call::new(
            address!("0x2222222222222222222222222222222222222222"),
            U256::from(7u64),
            Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]),
        )
        .allow_failure(),
        Call::new(
            address!("0x3333333333333333333333333333333333333333"),
            U256::from(1u64),
            Bytes::from(vec![0x01, 0x02]),
        )
        .delegate_call(),
    ];
    let nonce = b256!("0x00000000000000000000000000000000000000000000000000000000000004d2");
    let deadline = U256::from(1_900_000_000_u64);

    // Macro-emitted digest (already locked against reference vectors).
    let domain = cow_shed_eip712_domain(chain.as_u64(), version, proxy);
    let macro_digest = execute_hooks_signing_hash(&domain, &calls, nonce, deadline);

    // The dynamic typed-data payload an SDK `Signer` actually signs.
    let payload =
        execute_hooks_typed_data_payload(chain.as_u64(), version, proxy, &calls, nonce, deadline);
    let dynamic_digest = cow_typed_data_payload_to_alloy(&payload)
        .expect("ExecuteHooks payload converts to alloy typed-data")
        .eip712_signing_hash()
        .expect("alloy computes the typed-data signing hash");

    assert_eq!(
        dynamic_digest,
        macro_digest,
        "typed-data payload digest must equal the macro digest (chain {}, version {version})",
        chain.as_u64(),
    );
}
