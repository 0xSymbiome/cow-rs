//! Chain and deployment boundary shapes for the TypeScript-callable surface.

use serde::{Deserialize, Serialize};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use tsify::Tsify;

/// Deployment address output.
///
/// A default-flavour boundary construct built by the leaf's host-safe `helpers`
/// from the chain deployment registry and surfaced by `deploymentAddresses`. The
/// shape is always defined so the host-side `helpers` can build it; only the
/// TypeScript declaration derive is scoped to the wasm-bindgen target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), derive(Tsify))]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentAddresses {
    /// Settlement contract.
    pub settlement: String,
    /// Vault relayer contract.
    pub vault_relayer: String,
    /// `EthFlow` contract.
    pub eth_flow: String,
}

/// Wrapped-native token metadata.
///
/// A default-flavour boundary construct built by the leaf's host-safe `helpers`
/// from the native wrapped-native lookup and surfaced by `wrappedNativeToken`.
/// The shape is always defined; only the TypeScript declaration derive is scoped
/// to the wasm-bindgen target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), derive(Tsify))]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct WrappedNativeToken {
    /// Wrapped-native token contract address.
    pub address: String,
    /// Token symbol, such as `WETH` or `WXDAI`.
    pub symbol: String,
    /// Token decimals.
    pub decimals: u8,
}

/// Generated order UID output.
///
/// A default-flavour boundary projection that renames the native signing crate's
/// generated-order-id fields to the `{orderUid, orderDigest}` the boundary
/// surfaces (from `computeOrderUid` / `orderDigest`). The rename helper that
/// builds it from the native type lives in the leaf's host-safe `helpers`; this
/// module carries only the boundary shape. The shape is always defined; only the
/// TypeScript declaration derive is scoped to the wasm-bindgen target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), derive(Tsify))]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedOrderUid {
    /// Compact order UID.
    #[serde(rename = "orderUid")]
    pub order_uid: String,
    /// Underlying order digest.
    pub order_digest: String,
}
