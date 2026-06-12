//! Canonical COW Shed `sol!` ABI bindings.
//!
//! This module is the single source of truth for the macro-emitted Rust types
//! that back the COW Shed wire surface. The `alloy_sol_types::sol!` macro
//! requires every referenced struct to be declared in the same macro scope, so
//! the EIP-712 typed-data structs ([`Call`], [`ExecuteHooks`]) and the on-chain
//! ABI interfaces (`COWShed`, `COWShedFactory`) are co-located in one block.
//! [`types::Call`](crate::cow_shed::types::Call) re-exports the canonical `Call`
//! as the `cow_sdk_contracts::cow_shed::Call` alias.
//!
//! One sol! block covers the typed-data hashing path, the ABI calldata
//! builders, and both proxy and factory interfaces, so every `Call[]`-bearing
//! function signature references the same generated Rust type. The
//! [`calls`](crate::cow_shed::calls) module passes the input slice straight
//! through to the macro-emitted `executeHooksCall` structs without a
//! field-by-field converter.
//!
//! The macro emits `<Call as alloy_sol_types::SolStruct>` and
//! `<ExecuteHooks as alloy_sol_types::SolStruct>` implementations whose
//! `eip712_*` accessors are the canonical source of truth for the COW Shed
//! EIP-712 surface, and the `SolCall` implementations whose `abi_encode()` /
//! `abi_decode()` back the calldata builders in [`crate::cow_shed::calls`]. The
//! `parity/fixtures/cow_shed/*.json` rows gate the wire-byte-identity contract.
//!
//! # Mirrored generation
//!
//! The interfaces mirror the **deployed v1.0.x generation** — upstream
//! `cowdao-grants/cow-shed` at the `v1.0.1` tag, pinned by commit in
//! `parity/source-lock.yaml` (per ADR 0012) and cross-checked against the
//! deployed-runtime factory ABI the TypeScript arbiter ships
//! (`cow-sdk/packages/cow-shed/src/abi/CowShedFactoryAbi.ts`). The v2.x
//! source generations (ENS-purged 1-arg `initializeProxy`, the pre-sign
//! family, the `COWShedForComposableCoW` forwarder) are deliberately not
//! bound: per ADR 0049 the SDK binds deployed reality, and v2.x is deployed
//! only as the Gnosis chain-100 redeploy whose EIP-712 domain version is
//! `"2.0.0"` — outside the supported [`CowShedVersion`] family.
//!
//! # Subset scope
//!
//! Every bound function, event, and error exists byte-for-byte in the
//! deployed v1.0.x runtime, but the mirror is a deliberate subset: the
//! factory's inherited ENS resolver read surface (`initializeEns`, `addr`,
//! `name`, `baseName`, `baseNode`, the resolution-node getters,
//! `supportsInterface`) and the constructor-only `NoCodeAtImplementation`
//! error are out of scope for hook execution and proxy discovery.
//!
//! [`CowShedVersion`]: crate::cow_shed::CowShedVersion

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

    /// COW Shed proxy ABI (deployed v1.0.x implementation surface).
    interface COWShed {
        /// Signature validation failed.
        error InvalidSignature();
        /// The hook bundle deadline has elapsed (raised through `executeHooks`
        /// by the authentication library).
        error DeadlineElapsed();
        /// The hook nonce was already consumed (raised through the execute
        /// paths by the inherited storage mixin).
        error NonceAlreadyUsed();
        /// Caller is not the trusted executor.
        error OnlyTrustedExecutor();
        /// Caller is not the proxy itself.
        error OnlySelf();
        /// Proxy was already initialized.
        error AlreadyInitialized();
        /// Caller is not the admin.
        error OnlyAdmin();
        /// Caller is none of admin, trusted executor, or the proxy itself.
        error OnlyAdminOrTrustedExecutorOrSelf();

        /// Trusted executor changed.
        event TrustedExecutorChanged(address previousExecutor, address newExecutor);
        /// Implementation changed.
        event Upgraded(address indexed implementation);

        /// Initialize proxy state (called by the factory at deployment; the
        /// selector is the init guard embedded in the proxy creation code).
        function initialize(address factory, bool claimResolver) external;
        /// Execute signed hooks on the proxy.
        function executeHooks(
            Call[] calls,
            bytes32 nonce,
            uint256 deadline,
            bytes signature
        ) external;
        /// Execute hooks from the trusted executor path.
        function trustedExecuteHooks(Call[] calls) external;
        /// Set the reverse-resolution resolver (ENS opt-out recovery path).
        function claimWithResolver(address resolver) external;
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
        /// Deployed implementation version — the EIP-712 domain version
        /// string mirrored by `CowShedVersion::version_str`.
        function VERSION() external view returns (string);
    }

    /// Deployed COW Shed factory ABI (v1.0.x generation).
    interface COWShedFactory {
        /// Signature validation failed.
        error InvalidSignature();
        /// The hook bundle deadline has elapsed (bubbled from the proxy).
        error DeadlineElapsed();
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
        /// Deploy and initialize the user's proxy. `withEns = true` registers
        /// ENS records and reverts off-mainnet; the `executeHooks` auto-deploy
        /// path ignores ENS failures, so hook execution works on every chain.
        function initializeProxy(address user, bool withEns) external;
        /// Reverse lookup from proxy to owner.
        function ownerOf(address proxy) external view returns (address);
        /// Deterministic proxy address for a user.
        function proxyOf(address who) external view returns (address);
    }
}
