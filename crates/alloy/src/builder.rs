//! Typestate builder for the native composed Alloy client.

use alloy_primitives::B256;
use alloy_signer_local::PrivateKeySigner;
use cow_sdk_alloy_provider::RetryConfig;
use cow_sdk_core::{ChainId, Provider, Redacted, SupportedChainId};
use thiserror::Error;

use crate::{client::AlloyClient, error::AlloyClientError};

mod sealed {
    use alloy_signer_local::PrivateKeySigner;
    use cow_sdk_core::{ChainId, Redacted};

    /// Typestate marker indicating no transport has been selected.
    pub struct TransportUnset {
        pub(super) _private: (),
    }

    /// Typestate marker indicating HTTP transport has been selected.
    pub struct HttpTransport {
        pub(super) url: Redacted<reqwest::Url>,
    }

    /// Typestate marker indicating no key source has been selected.
    pub struct KeySourceUnset {
        pub(super) _private: (),
    }

    /// Typestate marker indicating a private-key source has been selected.
    pub struct PrivateKeySource {
        pub(super) signer: PrivateKeySigner,
    }

    /// Typestate marker indicating no chain id has been selected.
    pub struct ChainUnset {
        pub(super) _private: (),
    }

    /// Typestate marker indicating a chain id has been selected.
    pub struct ChainSet {
        pub(super) chain_id: ChainId,
    }

    #[allow(
        unnameable_types,
        reason = "Sealed trait pattern intentionally hides the marker; downstream impls are gated by orphan rules."
    )]
    pub trait SealedTransport {}
    impl SealedTransport for TransportUnset {}
    impl SealedTransport for HttpTransport {}

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

pub use sealed::{
    ChainSet, ChainUnset, HttpTransport, KeySourceUnset, PrivateKeySource, TransportUnset,
};

/// Sealed marker trait for transport builder states.
pub trait TransportState: sealed::SealedTransport {}
impl TransportState for TransportUnset {}
impl TransportState for HttpTransport {}

/// Sealed marker trait for key-source builder states.
pub trait KeySourceState: sealed::SealedKeySource {}
impl KeySourceState for KeySourceUnset {}
impl KeySourceState for PrivateKeySource {}

/// Sealed marker trait for chain-id builder states.
pub trait ChainState: sealed::SealedChain {}
impl ChainState for ChainUnset {}
impl ChainState for ChainSet {}

/// Typestate builder for [`AlloyClient`].
///
/// The generic parameters track transport, private-key source, and chain-id
/// selection. [`build`](AlloyClientBuilder::build) is available only once all
/// three axes are configured.
#[must_use]
pub struct AlloyClientBuilder<T = TransportUnset, K = KeySourceUnset, C = ChainUnset>
where
    T: TransportState,
    K: KeySourceState,
    C: ChainState,
{
    transport: T,
    key: K,
    chain: C,
    retry: Option<RetryConfig>,
}

impl Default for AlloyClientBuilder<TransportUnset, KeySourceUnset, ChainUnset> {
    fn default() -> Self {
        Self::new()
    }
}

impl AlloyClientBuilder<TransportUnset, KeySourceUnset, ChainUnset> {
    /// Creates a builder with no transport, key source, or chain id selected.
    pub const fn new() -> Self {
        Self {
            transport: TransportUnset { _private: () },
            key: KeySourceUnset { _private: () },
            chain: ChainUnset { _private: () },
            retry: None,
        }
    }
}

impl<T, K, C> AlloyClientBuilder<T, K, C>
where
    T: TransportState,
    K: KeySourceState,
    C: ChainState,
{
    /// Opts into bounded exponential backoff for transient, rate-limited RPC
    /// requests issued by the composed client.
    ///
    /// Without this setter each request is issued once and a transient transport
    /// failure (such as a public-endpoint `429`) is surfaced directly to the
    /// caller — the runtime-neutral default. Supplying a [`RetryConfig`] wraps
    /// the composed client's JSON-RPC transport in the same retry layer the
    /// read-only provider leaf uses.
    pub const fn retry(mut self, retry: RetryConfig) -> Self {
        self.retry = Some(retry);
        self
    }
}

impl<K, C> AlloyClientBuilder<TransportUnset, K, C>
where
    K: KeySourceState,
    C: ChainState,
{
    /// Selects native HTTP transport for the composed provider.
    ///
    /// # Errors
    ///
    /// Returns [`AlloyClientBuilderError::InvalidUrl`] if the input is not a
    /// valid URL. The invalid URL value is never echoed.
    pub fn http(
        self,
        rpc_url: impl AsRef<str>,
    ) -> Result<AlloyClientBuilder<HttpTransport, K, C>, AlloyClientBuilderError> {
        let url = reqwest::Url::parse(rpc_url.as_ref())
            .map_err(|_| AlloyClientBuilderError::InvalidUrl)?;
        Ok(AlloyClientBuilder {
            transport: HttpTransport {
                url: Redacted::new(url),
            },
            key: self.key,
            chain: self.chain,
            retry: self.retry,
        })
    }
}

impl<T, C> AlloyClientBuilder<T, KeySourceUnset, C>
where
    T: TransportState,
    C: ChainState,
{
    /// Selects a private key from a 32-byte hex string.
    ///
    /// The input may include or omit the `0x` prefix. Invalid key material
    /// returns an error that carries no key bytes.
    ///
    /// # Errors
    ///
    /// Returns [`AlloyClientBuilderError::InvalidPrivateKey`] when the input
    /// is not a valid secp256k1 private key.
    pub fn private_key(
        self,
        hex: impl AsRef<str>,
    ) -> Result<AlloyClientBuilder<T, PrivateKeySource, C>, AlloyClientBuilderError> {
        let signer = parse_private_key(hex.as_ref())?;
        Ok(AlloyClientBuilder {
            transport: self.transport,
            key: PrivateKeySource { signer },
            chain: self.chain,
            retry: self.retry,
        })
    }

    /// Selects a private key from raw 32-byte key material.
    ///
    /// # Errors
    ///
    /// Returns [`AlloyClientBuilderError::InvalidPrivateKey`] when the bytes
    /// do not encode a valid secp256k1 private key.
    pub fn private_key_bytes(
        self,
        bytes: [u8; 32],
    ) -> Result<AlloyClientBuilder<T, PrivateKeySource, C>, AlloyClientBuilderError> {
        let signer = PrivateKeySigner::from_bytes(&B256::from(bytes))
            .map_err(|_| AlloyClientBuilderError::InvalidPrivateKey)?;
        Ok(AlloyClientBuilder {
            transport: self.transport,
            key: PrivateKeySource { signer },
            chain: self.chain,
            retry: self.retry,
        })
    }
}

impl<T, K> AlloyClientBuilder<T, K, ChainUnset>
where
    T: TransportState,
    K: KeySourceState,
{
    /// Selects the chain id bound to the provider and signer.
    pub fn chain_id(self, chain_id: SupportedChainId) -> AlloyClientBuilder<T, K, ChainSet> {
        AlloyClientBuilder {
            transport: self.transport,
            key: self.key,
            chain: ChainSet {
                chain_id: ChainId::from(chain_id),
            },
            retry: self.retry,
        }
    }
}

impl AlloyClientBuilder<HttpTransport, PrivateKeySource, ChainSet> {
    /// Builds the composed client after all required inputs are selected.
    ///
    /// # Errors
    ///
    /// The selected states are fully validated before this method is
    /// available, so this method cannot fail today. The `Result` preserves the
    /// builder error contract for future transport and key-source additions.
    #[allow(
        clippy::unused_async,
        reason = "builder remains async-compatible with provider setup and public examples"
    )]
    pub async fn build(self) -> Result<AlloyClient, AlloyClientBuilderError> {
        Ok(AlloyClient::from_parts(
            self.transport.url.into_inner(),
            self.key.signer,
            self.chain.chain_id,
            self.retry,
        ))
    }

    /// Builds the composed client and verifies the configured chain id against
    /// the remote `eth_chainId`.
    ///
    /// This method dispatches one `eth_chainId` call. Use it for trading flows
    /// that require the configured chain to agree with the remote endpoint
    /// before any transaction is signed or submitted. Workflows that prefer a
    /// custom verification cadence can call [`Self::build`] and then
    /// [`AlloyClient::verify_chain_id`] explicitly.
    ///
    /// # Errors
    ///
    /// Returns [`AlloyClientBuilderError::ChainMismatch`] when the configured
    /// chain id does not match the remote endpoint. Returns
    /// [`AlloyClientBuilderError::Client`] for transport, decoding, or internal
    /// client failures encountered while resolving `eth_chainId`.
    pub async fn build_checked(self) -> Result<AlloyClient, AlloyClientBuilderError> {
        let configured = self.chain.chain_id;
        let client = self.build().await?;
        let remote = client
            .get_chain_id()
            .await
            .map_err(|error| AlloyClientBuilderError::Client(Box::new(error)))?;
        if remote != configured {
            return Err(AlloyClientBuilderError::ChainMismatch { configured, remote });
        }
        Ok(client)
    }
}

impl std::fmt::Debug for AlloyClientBuilder<TransportUnset, KeySourceUnset, ChainUnset> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlloyClientBuilder")
            .field("transport", &"unset")
            .field("key", &"unset")
            .field("chain", &"unset")
            .finish()
    }
}

impl std::fmt::Debug for AlloyClientBuilder<HttpTransport, PrivateKeySource, ChainSet> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlloyClientBuilder")
            .field("transport", &self.transport)
            .field("key", &"[redacted]")
            .field("chain_id", &self.chain.chain_id)
            .field("retry", &self.retry)
            .finish()
    }
}

impl std::fmt::Debug for TransportUnset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TransportUnset")
    }
}

impl std::fmt::Debug for HttpTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpTransport")
            .field("url", &self.url)
            .finish()
    }
}

impl std::fmt::Debug for KeySourceUnset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KeySourceUnset")
    }
}

impl std::fmt::Debug for PrivateKeySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PrivateKeySource([redacted])")
    }
}

impl std::fmt::Debug for ChainUnset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ChainUnset")
    }
}

impl std::fmt::Debug for ChainSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainSet")
            .field("chain_id", &self.chain_id)
            .finish()
    }
}

/// Errors returned while constructing [`AlloyClient`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AlloyClientBuilderError {
    /// The configured RPC URL could not be parsed.
    #[error("rpc url failed to parse")]
    InvalidUrl,
    /// The provided private key could not be parsed or was not a valid
    /// secp256k1 key.
    #[error("invalid private key")]
    InvalidPrivateKey,
    /// The configured chain id did not match the remote `eth_chainId`.
    ///
    /// Surfaced only by [`AlloyClientBuilder::build_checked`]; the default
    /// [`AlloyClientBuilder::build`] path remains free of network I/O.
    #[error("configured chain id `{configured}` does not match remote `eth_chainId` `{remote}`")]
    ChainMismatch {
        /// Chain id configured on the builder via
        /// [`AlloyClientBuilder::chain_id`].
        configured: ChainId,
        /// Chain id reported by the configured RPC endpoint.
        remote: ChainId,
    },
    /// Client-level error encountered by the checked build path.
    #[error(transparent)]
    Client(Box<AlloyClientError>),
}

impl From<AlloyClientError> for AlloyClientBuilderError {
    fn from(error: AlloyClientError) -> Self {
        Self::Client(Box::new(error))
    }
}

fn parse_private_key(value: &str) -> Result<PrivateKeySigner, AlloyClientBuilderError> {
    cow_sdk_alloy_signer::__seam::parse_private_key_signer(value)
        .ok_or(AlloyClientBuilderError::InvalidPrivateKey)
}
