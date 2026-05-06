//! Typestate builder for the native Alloy local-keystore signer.

use alloy_primitives::B256;
use alloy_signer_local::PrivateKeySigner;
use cow_sdk_core::{ChainId, SupportedChainId};
use thiserror::Error;

use crate::signer::LocalAlloyKeystoreSigner;

mod sealed {
    use alloy_signer_local::PrivateKeySigner;
    use cow_sdk_core::ChainId;

    /// Typestate marker indicating no key source has been selected.
    pub struct KeySourceUnset {
        _private: (),
    }

    impl KeySourceUnset {
        pub(super) const fn new() -> Self {
            Self { _private: () }
        }
    }

    /// Typestate marker indicating a private-key source has been selected.
    pub struct PrivateKeySource {
        pub(super) signer: PrivateKeySigner,
    }

    /// Typestate marker indicating no chain id has been selected.
    pub struct ChainUnset {
        _private: (),
    }

    impl ChainUnset {
        pub(super) const fn new() -> Self {
            Self { _private: () }
        }
    }

    /// Typestate marker indicating a chain id has been selected.
    pub struct ChainSet {
        pub(super) chain_id: ChainId,
    }

    pub trait SealedKeySource {}
    impl SealedKeySource for KeySourceUnset {}
    impl SealedKeySource for PrivateKeySource {}

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

/// Typestate builder for [`LocalAlloyKeystoreSigner`].
///
/// `K` tracks the key-source axis and `C` tracks the chain-id axis. The
/// [`build`](LocalAlloyKeystoreSignerBuilder::build) method is available only
/// after both axes have been selected.
#[must_use]
pub struct LocalAlloyKeystoreSignerBuilder<K = KeySourceUnset, C = ChainUnset>
where
    K: KeySourceState,
    C: ChainState,
{
    key: K,
    chain: C,
}

impl Default for LocalAlloyKeystoreSignerBuilder<KeySourceUnset, ChainUnset> {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalAlloyKeystoreSignerBuilder<KeySourceUnset, ChainUnset> {
    /// Creates a builder with neither key source nor chain id selected.
    pub const fn new() -> Self {
        Self {
            key: sealed::KeySourceUnset::new(),
            chain: sealed::ChainUnset::new(),
        }
    }
}

impl<C> LocalAlloyKeystoreSignerBuilder<KeySourceUnset, C>
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
    /// Returns [`LocalAlloyKeystoreSignerBuilderError::InvalidPrivateKey`]
    /// when the input is not a valid secp256k1 private key.
    pub fn private_key(
        self,
        hex: impl AsRef<str>,
    ) -> Result<
        LocalAlloyKeystoreSignerBuilder<PrivateKeySource, C>,
        LocalAlloyKeystoreSignerBuilderError,
    > {
        let signer = parse_private_key(hex.as_ref())?;
        Ok(LocalAlloyKeystoreSignerBuilder {
            key: sealed::PrivateKeySource { signer },
            chain: self.chain,
        })
    }

    /// Selects a private key from raw 32-byte key material.
    ///
    /// # Errors
    ///
    /// Returns [`LocalAlloyKeystoreSignerBuilderError::InvalidPrivateKey`]
    /// when the bytes do not encode a valid secp256k1 private key.
    pub fn private_key_bytes(
        self,
        bytes: [u8; 32],
    ) -> Result<
        LocalAlloyKeystoreSignerBuilder<PrivateKeySource, C>,
        LocalAlloyKeystoreSignerBuilderError,
    > {
        let signer = PrivateKeySigner::from_bytes(&B256::from(bytes))
            .map_err(|_| LocalAlloyKeystoreSignerBuilderError::InvalidPrivateKey)?;
        Ok(LocalAlloyKeystoreSignerBuilder {
            key: sealed::PrivateKeySource { signer },
            chain: self.chain,
        })
    }
}

impl<K> LocalAlloyKeystoreSignerBuilder<K, ChainUnset>
where
    K: KeySourceState,
{
    /// Selects the chain id bound to the signer.
    pub fn chain_id(
        self,
        chain_id: SupportedChainId,
    ) -> LocalAlloyKeystoreSignerBuilder<K, ChainSet> {
        LocalAlloyKeystoreSignerBuilder {
            key: self.key,
            chain: sealed::ChainSet {
                chain_id: ChainId::from(chain_id),
            },
        }
    }
}

impl LocalAlloyKeystoreSignerBuilder<PrivateKeySource, ChainSet> {
    /// Builds the signer after both key source and chain id have been selected.
    ///
    /// # Errors
    ///
    /// The current selected states are fully validated before this method is
    /// available, so this method cannot fail today. The `Result` preserves the
    /// builder error contract for future key-source additions.
    pub fn build(self) -> Result<LocalAlloyKeystoreSigner, LocalAlloyKeystoreSignerBuilderError> {
        Ok(LocalAlloyKeystoreSigner::from_parts(
            self.key.signer,
            self.chain.chain_id,
        ))
    }
}

/// Builder errors for [`LocalAlloyKeystoreSignerBuilder`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LocalAlloyKeystoreSignerBuilderError {
    /// The provided private key could not be parsed or was not a valid
    /// secp256k1 key.
    #[error("invalid private key")]
    InvalidPrivateKey,
}

fn parse_private_key(
    value: &str,
) -> Result<PrivateKeySigner, LocalAlloyKeystoreSignerBuilderError> {
    value
        .parse()
        .or_else(|_| value.strip_prefix("0x").unwrap_or(value).parse())
        .map_err(|_| LocalAlloyKeystoreSignerBuilderError::InvalidPrivateKey)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

    #[test]
    fn builder_default_starts_empty() {
        let _builder = LocalAlloyKeystoreSignerBuilder::default();
    }

    #[test]
    fn private_key_invalid_returns_error_no_leak() {
        let secret = "not-a-private-key";
        let Err(error) = LocalAlloyKeystoreSigner::builder().private_key(secret) else {
            panic!("invalid key must fail");
        };

        let rendered = format!("{error}");
        assert_eq!(rendered, "invalid private key");
        assert!(!rendered.contains(secret));
    }

    #[test]
    fn private_key_with_0x_prefix_succeeds() {
        let builder = LocalAlloyKeystoreSigner::builder()
            .private_key(TEST_KEY)
            .unwrap();
        let _builder = builder.chain_id(SupportedChainId::Sepolia);
    }

    #[test]
    fn private_key_without_0x_prefix_succeeds() {
        let builder = LocalAlloyKeystoreSigner::builder()
            .private_key(TEST_KEY.trim_start_matches("0x"))
            .unwrap();
        let _builder = builder.chain_id(SupportedChainId::Sepolia);
    }

    #[test]
    fn private_key_bytes_zero_returns_error() {
        let Err(error) = LocalAlloyKeystoreSigner::builder().private_key_bytes([0u8; 32]) else {
            panic!("zero key must fail");
        };

        assert!(matches!(
            error,
            LocalAlloyKeystoreSignerBuilderError::InvalidPrivateKey
        ));
    }

    #[test]
    fn chain_id_transitions_typestate() {
        let signer = LocalAlloyKeystoreSigner::builder()
            .private_key(TEST_KEY)
            .unwrap()
            .chain_id(SupportedChainId::Sepolia)
            .build()
            .unwrap();

        assert_eq!(signer.chain_id(), ChainId::from(SupportedChainId::Sepolia));
    }
}
