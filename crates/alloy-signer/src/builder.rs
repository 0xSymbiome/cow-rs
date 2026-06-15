//! Typestate builder for the native Alloy local private-key signer.

use alloy_primitives::B256;
use alloy_signer_local::PrivateKeySigner;
use cow_sdk_core::{ChainId, SupportedChainId};
use thiserror::Error;

use crate::signer::LocalAlloySigner;

mod sealed {
    use alloy_signer_local::PrivateKeySigner;
    use cow_sdk_core::ChainId;

    /// Typestate marker indicating no key source has been selected.
    #[derive(Debug)]
    pub struct KeySourceUnset {
        _private: (),
    }

    impl KeySourceUnset {
        pub(super) const fn new() -> Self {
            Self { _private: () }
        }
    }

    /// Typestate marker indicating a private-key source has been selected.
    #[derive(Debug)]
    pub struct PrivateKeySource {
        pub(super) signer: PrivateKeySigner,
    }

    /// Typestate marker indicating no chain id has been selected.
    #[derive(Debug)]
    pub struct ChainUnset {
        _private: (),
    }

    impl ChainUnset {
        pub(super) const fn new() -> Self {
            Self { _private: () }
        }
    }

    /// Typestate marker indicating a chain id has been selected.
    #[derive(Debug)]
    pub struct ChainSet {
        pub(super) chain_id: ChainId,
    }

    #[allow(
        unnameable_types,
        reason = "Sealed trait pattern intentionally hides the marker; downstream impls are gated by orphan rules."
    )]
    pub trait SealedKeySource {}
    impl SealedKeySource for KeySourceUnset {}
    impl SealedKeySource for PrivateKeySource {}

    #[allow(
        unnameable_types,
        reason = "Sealed trait pattern intentionally hides the marker; downstream impls are gated by orphan rules."
    )]
    pub trait SealedChain {}
    impl SealedChain for ChainUnset {}
    impl SealedChain for ChainSet {}
}

pub use sealed::{ChainSet, ChainUnset, KeySourceUnset, PrivateKeySource};

/// Sealed marker trait for key-source builder states.
pub trait KeySourceState: sealed::SealedKeySource {}
impl KeySourceState for KeySourceUnset {}
impl KeySourceState for PrivateKeySource {}

/// Sealed marker trait for chain-id builder states.
pub trait ChainState: sealed::SealedChain {}
impl ChainState for ChainUnset {}
impl ChainState for ChainSet {}

/// Typestate builder for [`LocalAlloySigner`].
///
/// `K` tracks the key-source axis and `C` tracks the chain-id axis. The
/// [`build`](LocalAlloySignerBuilder::build) method is available only
/// after both axes have been selected.
#[derive(Debug)]
#[must_use]
pub struct LocalAlloySignerBuilder<K = KeySourceUnset, C = ChainUnset>
where
    K: KeySourceState,
    C: ChainState,
{
    key: K,
    chain: C,
}

impl Default for LocalAlloySignerBuilder<KeySourceUnset, ChainUnset> {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalAlloySignerBuilder<KeySourceUnset, ChainUnset> {
    /// Creates a builder with neither key source nor chain id selected.
    pub const fn new() -> Self {
        Self {
            key: sealed::KeySourceUnset::new(),
            chain: sealed::ChainUnset::new(),
        }
    }
}

impl<C> LocalAlloySignerBuilder<KeySourceUnset, C>
where
    C: ChainState,
{
    /// Selects a private key from a 32-byte hex string.
    ///
    /// The input may include or omit the `0x` prefix. Invalid key material
    /// returns an error that carries no key bytes.
    ///
    /// # Errors
    ///
    /// Returns [`LocalAlloySignerBuilderError::InvalidPrivateKey`]
    /// when the input is not a valid secp256k1 private key.
    pub fn private_key(
        self,
        hex: impl AsRef<str>,
    ) -> Result<LocalAlloySignerBuilder<PrivateKeySource, C>, LocalAlloySignerBuilderError> {
        let signer = parse_private_key(hex.as_ref())?;
        Ok(LocalAlloySignerBuilder {
            key: sealed::PrivateKeySource { signer },
            chain: self.chain,
        })
    }

    /// Selects a private key from raw 32-byte key material.
    ///
    /// # Errors
    ///
    /// Returns [`LocalAlloySignerBuilderError::InvalidPrivateKey`]
    /// when the bytes do not encode a valid secp256k1 private key.
    pub fn private_key_bytes(
        self,
        bytes: [u8; 32],
    ) -> Result<LocalAlloySignerBuilder<PrivateKeySource, C>, LocalAlloySignerBuilderError> {
        let signer = PrivateKeySigner::from_bytes(&B256::from(bytes))
            .map_err(|_| LocalAlloySignerBuilderError::InvalidPrivateKey)?;
        Ok(LocalAlloySignerBuilder {
            key: sealed::PrivateKeySource { signer },
            chain: self.chain,
        })
    }
}

impl<K> LocalAlloySignerBuilder<K, ChainUnset>
where
    K: KeySourceState,
{
    /// Selects the chain id bound to the signer.
    pub fn chain_id(self, chain_id: SupportedChainId) -> LocalAlloySignerBuilder<K, ChainSet> {
        LocalAlloySignerBuilder {
            key: self.key,
            chain: sealed::ChainSet {
                chain_id: ChainId::from(chain_id),
            },
        }
    }
}

impl LocalAlloySignerBuilder<PrivateKeySource, ChainSet> {
    /// Builds the signer after both key source and chain id have been selected.
    ///
    /// # Errors
    ///
    /// The current selected states are fully validated before this method is
    /// available, so this method cannot fail today. The `Result` preserves the
    /// builder error contract for future key-source additions.
    pub fn build(self) -> Result<LocalAlloySigner, LocalAlloySignerBuilderError> {
        Ok(LocalAlloySigner::from_parts(
            self.key.signer,
            self.chain.chain_id,
        ))
    }
}

/// Builder errors for [`LocalAlloySignerBuilder`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LocalAlloySignerBuilderError {
    /// The provided private key could not be parsed or was not a valid
    /// secp256k1 key.
    #[error("invalid private key")]
    InvalidPrivateKey,
}

fn parse_private_key(value: &str) -> Result<PrivateKeySigner, LocalAlloySignerBuilderError> {
    parse_private_key_signer(value).ok_or(LocalAlloySignerBuilderError::InvalidPrivateKey)
}

/// Parses a [`PrivateKeySigner`] from a hex string, accepting an optional `0x`
/// prefix. Shared with the umbrella builder through the crate `__seam` so the
/// key-handling rule lives in exactly one place.
#[must_use]
pub fn parse_private_key_signer(value: &str) -> Option<PrivateKeySigner> {
    value
        .parse()
        .ok()
        .or_else(|| value.strip_prefix("0x").unwrap_or(value).parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

    #[test]
    fn builder_default_starts_empty() {
        let _builder = LocalAlloySignerBuilder::default();
    }

    #[test]
    fn private_key_invalid_returns_error_no_leak() {
        let secret = "not-a-private-key";
        let Err(error) = LocalAlloySigner::builder().private_key(secret) else {
            panic!("invalid key must fail");
        };

        let rendered = format!("{error}");
        assert_eq!(rendered, "invalid private key");
        assert!(!rendered.contains(secret));
    }

    #[test]
    fn private_key_with_0x_prefix_succeeds() {
        let builder = LocalAlloySigner::builder().private_key(TEST_KEY).unwrap();
        let _builder = builder.chain_id(SupportedChainId::Sepolia);
    }

    #[test]
    fn private_key_without_0x_prefix_succeeds() {
        let builder = LocalAlloySigner::builder()
            .private_key(TEST_KEY.trim_start_matches("0x"))
            .unwrap();
        let _builder = builder.chain_id(SupportedChainId::Sepolia);
    }

    #[test]
    fn private_key_bytes_zero_returns_error() {
        let Err(error) = LocalAlloySigner::builder().private_key_bytes([0u8; 32]) else {
            panic!("zero key must fail");
        };

        assert!(matches!(
            error,
            LocalAlloySignerBuilderError::InvalidPrivateKey
        ));
    }

    #[test]
    fn chain_id_transitions_typestate() {
        let signer = LocalAlloySigner::builder()
            .private_key(TEST_KEY)
            .unwrap()
            .chain_id(SupportedChainId::Sepolia)
            .build()
            .unwrap();

        assert_eq!(signer.chain_id(), ChainId::from(SupportedChainId::Sepolia));
    }
}
