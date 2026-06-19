//! Fixture-anchored parity for the directly-supported EVM network set and each
//! chain's wrapped-native token.
//!
//! `parity/fixtures/chains/supported_networks.json` transcribes the EVM members
//! of the upstream cow-sdk `SupportedChainId` enum and their
//! `WRAPPED_NATIVE_CURRENCIES` entries at the source-lock commit. The non-EVM
//! `SOLANA` member of the upstream enum is out of scope for this EVM SDK.
//!
//! `cargo xtask parity drift` watches the two cited cow-sdk paths, so a chain
//! added or removed upstream — or a changed wrapped-native address — is surfaced
//! for review, and the fixture's freshness ratchet fails on the next pin bump
//! until it is re-verified. This test proves the value half: `SupportedChainId`
//! and `wrapped_native_token` reproduce the pinned set byte-for-byte.

use cow_sdk_core::{Address, SupportedChainId, wrapped_native_token};

const FIXTURE: &str = include_str!("../../../parity/fixtures/chains/supported_networks.json");

#[test]
fn supported_network_set_and_wrapped_native_tokens_match_pinned_upstream() {
    let fixture: serde_json::Value =
        serde_json::from_str(FIXTURE).expect("supported-networks fixture parses");
    let rows = fixture["rows"]
        .as_array()
        .expect("fixture carries a rows array");

    // Set parity in both directions: an equal count plus an injective `try_from`
    // over every upstream chain id means the supported set is exactly the pinned
    // EVM set. A chain added or removed upstream — or in `SupportedChainId::ALL`
    // — breaks the count or the resolution and fails here.
    assert_eq!(
        rows.len(),
        SupportedChainId::ALL.len(),
        "the supported EVM network count must match the pinned cow-sdk config",
    );

    for row in rows {
        let chain_id = row["chain_id"]
            .as_u64()
            .expect("row carries a numeric chain_id");
        let chain = SupportedChainId::try_from(chain_id)
            .unwrap_or_else(|_| panic!("upstream chain {chain_id} must be a supported variant"));
        let token = wrapped_native_token(chain);

        let expected_address = row["wrapped_native_address"]
            .as_str()
            .expect("row carries a wrapped_native_address");
        assert_eq!(
            token.address,
            Address::new(expected_address).expect("fixture wrapped-native address parses"),
            "wrapped-native address must match the pinned config for chain {chain_id}",
        );
        assert_eq!(
            token.name,
            row["wrapped_native_name"].as_str().expect("name"),
            "wrapped-native name must match the pinned config for chain {chain_id}",
        );
        assert_eq!(
            token.symbol,
            row["wrapped_native_symbol"].as_str().expect("symbol"),
            "wrapped-native symbol must match the pinned config for chain {chain_id}",
        );
        assert_eq!(
            u64::from(token.decimals),
            row["wrapped_native_decimals"].as_u64().expect("decimals"),
            "wrapped-native decimals must match the pinned config for chain {chain_id}",
        );
    }
}
