//! High-level COW Shed hook orchestration.
//!
//! [`CowShedHooks`] bundles the deterministic primitives — proxy derivation,
//! EIP-712 domain + digest, owner signing, and factory calldata encoding —
//! into a single [`sign`](CowShedHooks::sign) call, mirroring the upstream
//! TypeScript `CowShedSdk.signCalls` ergonomics while staying provider-free:
//! it never owns an RPC client and never estimates gas. The low-level building
//! blocks ([`crate::proxy_of`], [`crate::execute_hooks_signing_hash`],
//! [`crate::encode_execute_hooks_calldata`]) remain public for advanced and
//! digest-only callers.

use alloy_primitives::{Address, B256, Bytes, U256};
use cow_sdk_app_data::Hook;
use cow_sdk_contracts::{DeploymentChainId, RecoverableSignature};
use cow_sdk_core::{Address as CoreAddress, HexData, Signer, TypedDataPayload};

use crate::address::{cow_shed_factory, proxy_for};
use crate::calls::encode_execute_hooks_calldata_signed;
use crate::eip712::execute_hooks_typed_data_payload;
use crate::errors::CowShedError;
use crate::types::Call;
use crate::version::CowShedVersion;

/// High-level COW Shed hook builder.
///
/// Construct one for a chain, optionally pin a [`CowShedVersion`], then call
/// [`sign`](Self::sign) with the hook `calls`, a `nonce`, and a `deadline`.
/// The builder is a small `Copy` value carrying only the chain and version;
/// the per-call inputs are passed explicitly so there is no hidden clock or
/// randomness and no "forgot to set the nonce" failure mode.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CowShedHooks {
    chain: DeploymentChainId,
    version: CowShedVersion,
}

impl CowShedHooks {
    /// Creates a builder for `chain` defaulting to the deployed
    /// [`CowShedVersion::V1_0_1`].
    ///
    /// `chain` accepts a [`cow_sdk_core::SupportedChainId`] (what a trading flow
    /// already holds) or a [`DeploymentChainId`] directly, matching the
    /// `cow-sdk-contracts` `Registry` idiom.
    #[must_use]
    pub fn new(chain: impl Into<DeploymentChainId>) -> Self {
        Self {
            chain: chain.into(),
            version: CowShedVersion::V1_0_1,
        }
    }

    /// Returns a copy pinned to an explicit [`CowShedVersion`].
    #[must_use]
    pub const fn with_version(mut self, version: CowShedVersion) -> Self {
        self.version = version;
        self
    }

    /// Returns the configured deployment chain.
    #[must_use]
    pub const fn chain(&self) -> DeploymentChainId {
        self.chain
    }

    /// Returns the configured COW Shed version.
    #[must_use]
    pub const fn version(&self) -> CowShedVersion {
        self.version
    }

    /// Returns the COW Shed factory address for the configured chain/version.
    #[must_use]
    pub fn factory(&self) -> Address {
        cow_shed_factory(self.chain, self.version)
    }

    /// Returns the deterministic proxy ("shed") account for `owner`.
    #[must_use]
    pub fn shed_account(&self, owner: Address) -> Address {
        proxy_for(self.chain, self.version, owner)
    }

    /// Builds the EIP-712 [`TypedDataPayload`] an owner signs to authorize
    /// `calls`.
    ///
    /// Use this for the manual signing path — sign it yourself via any
    /// [`Signer::sign_typed_data_payload`]. [`sign`](Self::sign) performs this
    /// step plus signature parsing and calldata encoding in one call.
    #[must_use]
    pub fn typed_data_payload(
        &self,
        owner: Address,
        calls: &[Call],
        nonce: B256,
        deadline: U256,
    ) -> TypedDataPayload {
        execute_hooks_typed_data_payload(
            self.chain.as_u64(),
            self.version,
            self.shed_account(owner),
            calls,
            nonce,
            deadline,
        )
    }

    /// Signs `calls` with `signer` and returns the encoded factory call.
    ///
    /// Resolves the owner from the signer, derives the proxy, builds and signs
    /// the `ExecuteHooks` EIP-712 payload through the owned [`Signer`] trait
    /// (`sign_typed_data_payload`), and encodes `factory.executeHooks`
    /// calldata. The returned [`SignedCowShedCall`] can be submitted directly
    /// or attached to a `CoW` order as a hook via
    /// [`SignedCowShedCall::to_app_data_hook`].
    ///
    /// `nonce` and `deadline` are caller-managed (the crate owns no clock or
    /// randomness): pick a unique `nonce` per authorization and a bounded
    /// `deadline` (Unix seconds) — `U256::MAX` is the non-expiring sentinel.
    ///
    /// This is the externally-owned-account path: it parses the signer's output
    /// as a 65-byte ECDSA signature. A smart-contract (EIP-1271) owner instead
    /// builds the payload with [`typed_data_payload`](Self::typed_data_payload),
    /// signs it with the owner's signer, and encodes the resulting blob with
    /// [`encode_execute_hooks_calldata_with_signature`](crate::encode_execute_hooks_calldata_with_signature).
    ///
    /// # Errors
    ///
    /// Returns [`CowShedError::Other`] if the signer cannot resolve its
    /// address, the typed-data signing fails, or the signer returns a value
    /// that is not a canonical 65-byte recoverable signature.
    pub async fn sign<S>(
        &self,
        signer: &S,
        calls: &[Call],
        nonce: B256,
        deadline: U256,
    ) -> Result<SignedCowShedCall, CowShedError>
    where
        S: Signer,
        S::Error: core::fmt::Display,
    {
        let owner = signer.address().await.map_err(|error| {
            CowShedError::Other(format!("cow-shed: resolve owner address: {error}").into())
        })?;
        let owner_alloy = owner.into_alloy();
        let shed = self.shed_account(owner_alloy);
        let payload = execute_hooks_typed_data_payload(
            self.chain.as_u64(),
            self.version,
            shed,
            calls,
            nonce,
            deadline,
        );
        let signature_hex = signer
            .sign_typed_data_payload(&payload)
            .await
            .map_err(|error| {
                CowShedError::Other(format!("cow-shed: sign ExecuteHooks payload: {error}").into())
            })?;
        let signature = RecoverableSignature::parse_hex(&signature_hex).map_err(|error| {
            CowShedError::Other(format!("cow-shed: parse signature: {error}").into())
        })?;
        let factory_calldata =
            encode_execute_hooks_calldata_signed(calls, nonce, deadline, owner_alloy, &signature);
        Ok(SignedCowShedCall {
            shed_account: shed,
            factory: self.factory(),
            factory_calldata,
        })
    }
}

/// A signed COW Shed hook bundle ready to submit or attach to an order.
///
/// Produced by [`CowShedHooks::sign`]. Submit `factory_calldata` to `factory`
/// as a transaction to execute the hooks directly, or call
/// [`to_app_data_hook`](Self::to_app_data_hook) to attach the bundle to a
/// `CoW` order's pre/post hooks.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedCowShedCall {
    /// The owner's deterministic COW Shed proxy ("shed") account.
    pub shed_account: Address,
    /// The COW Shed factory address the encoded call targets.
    pub factory: Address,
    /// ABI-encoded `factory.executeHooks(...)` calldata.
    pub factory_calldata: Bytes,
}

impl SignedCowShedCall {
    /// Wraps this signed call as a `CoW` Protocol app-data [`Hook`].
    ///
    /// The hook targets the COW Shed factory with the encoded `executeHooks`
    /// calldata and the supplied `gas_limit`, ready to set as a
    /// `metadata.hooks.pre[..]` or `.post[..]` entry on an order's app data.
    #[must_use]
    pub fn to_app_data_hook(&self, gas_limit: u64) -> Hook {
        Hook::new(
            CoreAddress::from_bytes(self.factory.into_array()),
            HexData::from_bytes(self.factory_calldata.to_vec()),
            gas_limit,
        )
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::address;

    use super::{Address, Bytes, CowShedHooks, CowShedVersion, SignedCowShedCall};
    use crate::address::{cow_shed_factory, proxy_for};

    #[test]
    fn builder_resolves_factory_and_shed_via_the_free_functions() {
        use cow_sdk_contracts::DeploymentChainId;

        let user = address!("0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58");
        let hooks = CowShedHooks::new(DeploymentChainId::GnosisChain);
        assert_eq!(hooks.version(), CowShedVersion::V1_0_1);
        assert_eq!(
            hooks.factory(),
            cow_shed_factory(DeploymentChainId::GnosisChain, CowShedVersion::V1_0_1)
        );
        assert_eq!(
            hooks.shed_account(user),
            proxy_for(DeploymentChainId::GnosisChain, CowShedVersion::V1_0_1, user)
        );

        let pinned = hooks.with_version(CowShedVersion::V1_0_0);
        assert_eq!(pinned.version(), CowShedVersion::V1_0_0);
        assert_eq!(
            pinned.factory(),
            cow_shed_factory(DeploymentChainId::GnosisChain, CowShedVersion::V1_0_0)
        );
    }

    #[test]
    fn to_app_data_hook_maps_factory_calldata_and_gas() {
        let factory: Address = address!("0x312f92fe5f1710408b20d52a374fa29e099cfa86");
        let calldata = Bytes::from(vec![0x13, 0xfb, 0x72, 0xc7, 0xde, 0xad]);
        let signed = SignedCowShedCall {
            shed_account: address!("0x66545b93a314e5bdec9e5ff9c4d2c7054e6afb04"),
            factory,
            factory_calldata: calldata.clone(),
        };

        let hook = signed.to_app_data_hook(500_000);

        assert_eq!(
            hook.target.into_alloy(),
            factory,
            "hook target is the factory"
        );
        assert_eq!(
            hook.call_data.as_alloy(),
            &calldata,
            "hook calldata is the encoded executeHooks call"
        );
        assert_eq!(hook.gas_limit, 500_000, "hook gas limit is preserved");
    }
}
