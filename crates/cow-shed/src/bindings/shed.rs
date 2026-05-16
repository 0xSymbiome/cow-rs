//! COW Shed proxy ABI bindings.

use alloy_sol_types::sol;

sol! {
    /// Hook call tuple used by COW Shed proxy methods.
    #[derive(Debug, PartialEq, Eq)]
    struct Call {
        address target;
        uint256 value;
        bytes callData;
        bool allowFailure;
        bool isDelegateCall;
    }

    /// COW Shed proxy ABI.
    interface COWShed {
        /// Signature validation failed.
        error InvalidSignature();
        /// Caller is not the trusted role.
        error OnlyTrustedRole();
        /// Caller is not the proxy itself.
        error OnlySelf();
        /// Proxy was already initialized.
        error AlreadyInitialized();
        /// Caller is not the admin.
        error OnlyAdmin();
        /// Hook payload was not pre-signed.
        error NotPreSigned();

        /// Trusted executor changed.
        event TrustedExecutorChanged(address previousExecutor, address newExecutor);
        /// Implementation changed.
        event Upgraded(address indexed implementation);
        /// Pre-sign storage changed.
        event PreSignStorageChanged(address indexed newStorage);

        /// Initialize proxy state.
        function initialize(address factory) external;
        /// Execute signed hooks on the proxy.
        function executeHooks(
            Call[] calls,
            bytes32 nonce,
            uint256 deadline,
            bytes signature
        ) external;
        /// Execute hooks that were pre-signed on-chain.
        function executePreSignedHooks(Call[] calls, bytes32 nonce, uint256 deadline) external;
        /// Query whether hooks are pre-signed.
        function isPreSignedHooks(Call[] calls, bytes32 nonce, uint256 deadline) external view returns (bool);
        /// Set pre-sign status for hooks.
        function preSignHooks(Call[] calls, bytes32 nonce, uint256 deadline, bool signed) external;
        /// Current pre-sign storage contract.
        function preSignStorage() external view returns (address);
        /// Reset pre-sign storage.
        function resetPreSignStorage() external returns (address);
        /// Set pre-sign storage.
        function setPreSignStorage(address storageContract) external returns (address);
        /// Execute hooks from the trusted executor path.
        function trustedExecuteHooks(Call[] calls) external;
        /// Update trusted executor.
        function updateTrustedExecutor(address who) external;
        /// Update proxy implementation.
        function updateImplementation(address newImplementation) external;
        /// Revoke a nonce.
        function revokeNonce(bytes32 nonce) external;
        /// Query nonce usage.
        function nonces(bytes32 nonce) external view returns (bool);
        /// Proxy EIP-712 domain separator.
        function domainSeparator() external view returns (bytes32);
        /// Current trusted executor.
        function trustedExecutor() external view returns (address);
    }
}
