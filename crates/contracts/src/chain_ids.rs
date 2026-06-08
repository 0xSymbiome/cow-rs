// Shared deployment chain-id literal consumed by the `build.rs` compile-time
// registry/coverage validator via `include!("src/chain_ids.rs")`. Declaring it
// here keeps the manifest validator's accepted chain set in one place: `build.rs`
// cannot take a dependency edge on `cow-sdk-core` without inviting a
// circular-build risk, so the chain set travels through this shared include
// rather than through `cow_sdk_core::SupportedChainId`.
//
// Deployment registry schema v2 tracks coverage-only and capability-only chains
// separately from the runtime-routable set, so this is the deployment superset
// (it includes Lens, which carries deployment rows but is not a runtime-routable
// chain in `cow_sdk_core::SupportedChainId`).

#[allow(
    dead_code,
    reason = "consumed by build.rs via include!; the runtime path reaches the \
authoritative chain list through cow_sdk_core::SupportedChainId"
)]
const DEPLOYMENT_CHAIN_IDS: [u64; 12] = [
    1,          // Mainnet
    56,         // Bnb
    100,        // GnosisChain
    137,        // Polygon
    8453,       // Base
    9745,       // Plasma
    42_161,     // ArbitrumOne
    43_114,     // Avalanche
    57_073,     // Ink
    59_144,     // Linea
    11_155_111, // Sepolia
    232,        // Lens
];
