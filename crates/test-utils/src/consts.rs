//! Canonical test constants shared across the workspace test suites.
//!
//! Two classes:
//! - **Synthetic placeholders** (`ADDR_A`..) — pure test fillers.
//! - **Real canonical values** (`tokens`, `ANVIL_*`, the CID vectors) — kept
//!   here with a provenance note. The settlement address is intentionally NOT
//!   here: source it from `cow_sdk_contracts::Registry` instead.
//!
//! Real addresses are written lowercase so the `address!` macro accepts them
//! without an EIP-55 checksum (their canonical checksummed forms differ only
//! in case).

use alloy_primitives::{Address, address};

// --- Class A: synthetic placeholder addresses ---
/// Synthetic placeholder address `0x1111…1111`.
pub const ADDR_A: Address = address!("1111111111111111111111111111111111111111");
/// Synthetic placeholder address `0x2222…2222`.
pub const ADDR_B: Address = address!("2222222222222222222222222222222222222222");
/// Synthetic placeholder address `0x3333…3333`.
pub const ADDR_C: Address = address!("3333333333333333333333333333333333333333");
/// Synthetic placeholder address `0x4444…4444`.
pub const ADDR_D: Address = address!("4444444444444444444444444444444444444444");
/// Synthetic placeholder address `0x5555…5555`.
pub const ADDR_E: Address = address!("5555555555555555555555555555555555555555");

// --- Class B: real canonical token addresses (provenance: well-known mainnet/sepolia) ---
/// Real canonical token addresses. Lowercase so `address!` skips checksum.
pub mod tokens {
    use super::{Address, address};

    /// WETH (Ethereum mainnet).
    pub const WETH_MAINNET: Address = address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
    /// DAI (Ethereum mainnet).
    pub const DAI_MAINNET: Address = address!("6b175474e89094c44da98b954eedeac495271d0f");
    /// USDC (Ethereum mainnet).
    pub const USDC_MAINNET: Address = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
    /// WETH (Sepolia testnet).
    pub const WETH_SEPOLIA: Address = address!("fff9976782d46cc05630d1f6ebab18b2324d6b14");
}

/// Well-known Anvil/Hardhat account #1 **private key** (a public test key, not a secret).
pub const ANVIL_KEY_1: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
/// The address derived from [`ANVIL_KEY_1`].
pub const ANVIL_ADDR_1: Address = address!("70997970c51812dc3a010c7d01b50e0d17dc79c8");

// --- App-data CID vectors. source: cowprotocol/app-data schemas + cowprotocol/services app-data hashing (keccak256 + CIDv1) ---
/// Upstream app-data hash vector #1.
pub const APP_DATA_HEX_1: &str =
    "0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";
/// Upstream `CIDv1` for [`APP_DATA_HEX_1`].
pub const CID_1: &str = "f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";
/// Upstream app-data hash vector #2.
pub const APP_DATA_HEX_2: &str =
    "0x8af4e8c9973577b08ac21d17d331aade86c11ebcc5124744d621ca8365ec9424";
/// Upstream `CIDv1` for [`APP_DATA_HEX_2`].
pub const CID_2: &str = "f01551b208af4e8c9973577b08ac21d17d331aade86c11ebcc5124744d621ca8365ec9424";

/// A relative `validTo` fixture value used across order tests.
pub const VALID_TO: u32 = 1_735_689_600;
/// Mainnet chain id.
pub const CHAIN_MAINNET: u32 = 1;
/// Gnosis Chain id.
pub const CHAIN_GNOSIS: u32 = 100;
