//! COW Shed factory ABI bindings.

use alloy_sol_types::sol;

sol! {
    /// Hook call tuple used by COW Shed factory methods.
    #[derive(Debug, PartialEq, Eq)]
    struct Call {
        address target;
        uint256 value;
        bytes callData;
        bool allowFailure;
        bool isDelegateCall;
    }

    /// Deployed COW Shed factory ABI.
    interface COWShedFactory {
        /// Signature validation failed.
        error InvalidSignature();
        /// Hook nonce was already used.
        error NonceAlreadyUsed();
        /// ENS setup failed.
        error SettingEnsRecordsFailed();

        /// Emitted when a user proxy is deployed.
        event COWShedBuilt(address user, address shed);

        /// Execute hooks on a user's proxy, deploying it first if needed.
        function executeHooks(
            Call[] calls,
            bytes32 nonce,
            uint256 deadline,
            address user,
            bytes signature
        ) external;

        /// Factory implementation address.
        function implementation() external view returns (address);
        /// Deploy and initialize the user's proxy.
        function initializeProxy(address user, bool withEns) external;
        /// Reverse lookup from proxy to owner.
        function ownerOf(address proxy) external view returns (address);
        /// Deterministic proxy address for a user.
        function proxyOf(address who) external view returns (address);
    }
}

#[cfg(feature = "cow-shed-ens")]
sol! {
    /// ENS-oriented factory ABI extension.
    interface COWShedFactoryEns {
        /// Forward ENS node resolution.
        function addr(bytes32 node) external view returns (address);
        /// Base ENS name.
        function baseName() external view returns (string);
        /// Base ENS node.
        function baseNode() external view returns (bytes32);
        /// Forward resolution node.
        function forwardResolutionNodeToAddress(bytes32 node) external view returns (address);
        /// Initialize ENS records for a user.
        function initializeEns(address user) external;
        /// Reverse ENS node name.
        function name(bytes32 node) external view returns (string);
        /// Reverse resolution node.
        function reverseResolutionNodeToAddress(bytes32 node) external view returns (address);
        /// ERC-165 support query.
        function supportsInterface(bytes4 interfaceId) external pure returns (bool);
    }
}
