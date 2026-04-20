// Shared chain-id literal consumed by both the `build.rs` compile-time
// validator (via `include!("src/chain_ids.rs")`) and the `src/` tree
// (via `mod chain_ids;`). Keep the literal declared in a single place so
// the registry validator and the runtime accessor can never drift.
//
// The eleven entries mirror `cow_sdk_core::SupportedChainId::ALL` in the
// same order; `build.rs` cannot take a dependency edge on `cow-sdk-core`
// without inviting a circular-build risk, so the authoritative chain set
// travels through this shared include instead.

#[allow(
    dead_code,
    reason = "consumed by build.rs via include!; the runtime path \
reaches the authoritative list through cow_sdk_core::SupportedChainId"
)]
const SUPPORTED_CHAIN_IDS: [u64; 11] = [
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
];
