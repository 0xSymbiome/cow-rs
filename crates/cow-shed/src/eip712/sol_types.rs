//! Canonical COW Shed sol! type definitions.
//!
//! This module is the single source of truth for the macro-emitted Rust
//! types that back the COW Shed wire surface. The `alloy_sol_types::sol!`
//! macro requires every referenced struct to be declared in the same macro
//! scope, so the EIP-712 typed-data structs (`Call`, `ExecuteHooks`) and
//! the on-chain ABI interfaces (`COWShed`, `COWShedFactory`) are
//! co-located in one block here. The [`bindings`](crate::bindings) module
//! re-exports the interface types under
//! `cow_sdk_cow_shed::bindings::shed::COWShed` and
//! `cow_sdk_cow_shed::bindings::factory::COWShedFactory`, and
//! [`types::Call`](crate::types::Call) re-exports the canonical `Call` as
//! the crate-level `cow_sdk_cow_shed::Call` alias.
//!
//! One sol! block covers the typed-data hashing path, the ABI calldata
//! builders, and both proxy and factory interfaces, so every
//! `Call[]`-bearing function signature references the same generated Rust
//! type. The [`calls`](crate::calls) module passes the input slice
//! straight through to the macro-emitted `executeHooksCall` and
//! `executePreSignedHooksCall` structs without a field-by-field
//! converter.
//!
//! The macro emits `<Call as alloy_sol_types::SolStruct>` and
//! `<ExecuteHooks as alloy_sol_types::SolStruct>` implementations whose
//! `eip712_root_type()`, `eip712_encode_type()`, `eip712_type_hash()`,
//! `eip712_hash_struct()`, and `eip712_signing_hash(&domain)` accessors
//! are the canonical source of truth for the COW Shed EIP-712 surface.
//! The macro emits the canonical EIP-712 type-string literals
//! (`"Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)"`
//! and the composite `ExecuteHooks(...)Call(...)`) at macro expansion
//! time with no whitespace between commas and in declaration order,
//! matching the protocol specification.
//!
//! The macro also emits the `SolCall` implementations for every
//! interface method (e.g. `COWShed::executeHooksCall`,
//! `COWShedFactory::executeHooksCall`,
//! `COWShed::executePreSignedHooksCall`) whose `abi_encode()` and
//! `abi_decode()` accessors back the ABI calldata builders in
//! [`crate::calls`]. The
//! `parity/fixtures/cow_shed/execute_hooks_calldata.json` rows gate the
//! wire-byte-identity contract for those builders.

alloy_sol_types::sol! {
    /// EIP-712 `Call` struct as encoded by the COW Shed `executeHooks`
    /// signing path and reused as the on-chain ABI tuple for every
    /// `Call[]`-bearing function in the COW Shed proxy and factory
    /// interfaces below.
    #[derive(Debug, Default, PartialEq, Eq)]
    struct Call {
        address target;
        uint256 value;
        bytes callData;
        bool allowFailure;
        bool isDelegateCall;
    }

    /// EIP-712 `ExecuteHooks` envelope hashed under the COW Shed per-proxy
    /// domain when signing a hook bundle.
    #[derive(Debug, Default, PartialEq, Eq)]
    struct ExecuteHooks {
        Call[] calls;
        bytes32 nonce;
        uint256 deadline;
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
