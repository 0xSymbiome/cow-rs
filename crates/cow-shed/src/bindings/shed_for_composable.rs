//! Gnosis-only COW Shed forwarder ABI binding.

use alloy_sol_types::sol;

sol! {
    /// Gnosis-only COW Shed forwarder for composable orders.
    interface COWShedForComposableCoW {
        /// ERC-1271 signature validation.
        function isValidSignature(bytes32 hash, bytes signature) external view returns (bytes4);
    }
}
