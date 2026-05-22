use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    errors::ValidationError,
    types::{Address, ChainId, TokenInfo},
};

use super::TOKEN_LIST_IMAGES_PATH;

/// Decodes a `0x`-prefixed 42-character hex literal into a 20-byte array at
/// compile time. Used only for the canonical wrapped-native-token address
/// constants declared in this module.
///
/// # Panics
///
/// Panics at compile time when the input is not exactly 42 characters, is
/// missing the `0x` prefix, or contains a non-hex character.
const fn hex_decode_20(hex: &str) -> [u8; 20] {
    // SAFETY: the only callers of this helper are the
    // `WRAPPED_NATIVE_*_BYTES` const declarations below, which are
    // repository-owned 42-character `0x`-prefixed hex literals. The
    // length, prefix, and per-nibble assertions therefore fire only as
    // compile-time guards if the constants are ever edited into an
    // invalid form.
    let bytes = hex.as_bytes();
    assert!(
        bytes.len() == 42,
        "hex_decode_20 requires a 42-character input"
    );
    assert!(
        bytes[0] == b'0' && bytes[1] == b'x',
        "hex_decode_20 requires a 0x prefix"
    );
    let mut out = [0u8; 20];
    let mut i = 0;
    while i < 20 {
        out[i] = (decode_nibble(bytes[2 + 2 * i]) << 4) | decode_nibble(bytes[2 + 2 * i + 1]);
        i += 1;
    }
    out
}

/// Decodes one ASCII hex nibble for the [`hex_decode_20`] compile-time helper.
///
/// # Panics
///
/// Panics when `c` is not an ASCII hex digit.
const fn decode_nibble(c: u8) -> u8 {
    // SAFETY: this helper is only reachable from `hex_decode_20` above,
    // which feeds it bytes drawn from the repository-owned wrapped-native
    // token hex constants. The non-hex fallback therefore acts as a
    // compile-time invariant guard rather than a runtime failure mode.
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => panic!("hex nibble must be 0-9, a-f, or A-F"),
    }
}

const WRAPPED_NATIVE_MAINNET_BYTES: [u8; 20] =
    hex_decode_20("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
const WRAPPED_NATIVE_GNOSIS_BYTES: [u8; 20] =
    hex_decode_20("0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d");
const WRAPPED_NATIVE_ARBITRUM_BYTES: [u8; 20] =
    hex_decode_20("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1");
const WRAPPED_NATIVE_BASE_INK_BYTES: [u8; 20] =
    hex_decode_20("0x4200000000000000000000000000000000000006");
const WRAPPED_NATIVE_SEPOLIA_BYTES: [u8; 20] =
    hex_decode_20("0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14");
const WRAPPED_NATIVE_POLYGON_BYTES: [u8; 20] =
    hex_decode_20("0x0d500b1d8e8ef31e21c99d1db9a6444d3adf1270");
const WRAPPED_NATIVE_AVALANCHE_BYTES: [u8; 20] =
    hex_decode_20("0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7");
const WRAPPED_NATIVE_BNB_BYTES: [u8; 20] =
    hex_decode_20("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c");
const WRAPPED_NATIVE_PLASMA_BYTES: [u8; 20] =
    hex_decode_20("0x6100e367285b01f48d07953803a2d8dca5d19873");
const WRAPPED_NATIVE_LINEA_BYTES: [u8; 20] =
    hex_decode_20("0xe5d7c2a44ffddf6b295a15c148167daaaf5cf34f");

/// Supported `CoW` Protocol chain ids with explicit API configuration.
///
/// Internal code in this crate can still use exhaustive `match` expressions
/// when implementing helpers like [`SupportedChainId::api_path`].
///
/// ```
/// use cow_sdk_core::SupportedChainId;
///
/// assert_eq!(SupportedChainId::Mainnet.api_path(), "mainnet");
/// ```
///
/// Downstream crates must include a wildcard arm when matching so future chain
/// additions remain semver-compatible.
///
/// ```compile_fail
/// use cow_sdk_core::SupportedChainId;
///
/// fn label(chain_id: SupportedChainId) -> &'static str {
///     match chain_id {
///         SupportedChainId::Mainnet => "mainnet",
///         SupportedChainId::Bnb => "bnb",
///         SupportedChainId::GnosisChain => "gnosis",
///         SupportedChainId::Polygon => "polygon",
///         SupportedChainId::Base => "base",
///         SupportedChainId::Plasma => "plasma",
///         SupportedChainId::ArbitrumOne => "arbitrum",
///         SupportedChainId::Avalanche => "avalanche",
///         SupportedChainId::Ink => "ink",
///         SupportedChainId::Linea => "linea",
///         SupportedChainId::Sepolia => "sepolia",
///     }
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u64)]
pub enum SupportedChainId {
    /// Ethereum mainnet.
    Mainnet = 1,
    /// BNB Smart Chain.
    Bnb = 56,
    /// Gnosis Chain.
    GnosisChain = 100,
    /// Polygon `PoS`.
    Polygon = 137,
    /// Base.
    Base = 8453,
    /// Plasma.
    Plasma = 9745,
    /// Arbitrum One.
    ArbitrumOne = 42161,
    /// Avalanche C-Chain.
    Avalanche = 43114,
    /// Ink.
    Ink = 57073,
    /// Linea.
    Linea = 59144,
    /// Ethereum Sepolia.
    Sepolia = 11_155_111,
}

impl SupportedChainId {
    /// Complete list of supported chain ids in deterministic iteration order.
    pub const ALL: [Self; 11] = [
        Self::Mainnet,
        Self::Bnb,
        Self::GnosisChain,
        Self::Polygon,
        Self::Base,
        Self::Plasma,
        Self::ArbitrumOne,
        Self::Avalanche,
        Self::Ink,
        Self::Linea,
        Self::Sepolia,
    ];

    /// Returns the path segment used by `CoW` API base URLs for this chain.
    #[must_use]
    pub const fn api_path(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Bnb => "bnb",
            Self::GnosisChain => "xdai",
            Self::Polygon => "polygon",
            Self::Base => "base",
            Self::Plasma => "plasma",
            Self::ArbitrumOne => "arbitrum_one",
            Self::Avalanche => "avalanche",
            Self::Ink => "ink",
            Self::Linea => "linea",
            Self::Sepolia => "sepolia",
        }
    }
}

impl TryFrom<ChainId> for SupportedChainId {
    type Error = ValidationError;

    fn try_from(value: ChainId) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Mainnet),
            56 => Ok(Self::Bnb),
            100 => Ok(Self::GnosisChain),
            137 => Ok(Self::Polygon),
            8453 => Ok(Self::Base),
            9745 => Ok(Self::Plasma),
            42161 => Ok(Self::ArbitrumOne),
            43114 => Ok(Self::Avalanche),
            57073 => Ok(Self::Ink),
            59144 => Ok(Self::Linea),
            11_155_111 => Ok(Self::Sepolia),
            chain_id => Err(ValidationError::UnsupportedChain { chain_id }),
        }
    }
}

impl From<SupportedChainId> for ChainId {
    fn from(value: SupportedChainId) -> Self {
        value as Self
    }
}

impl Serialize for SupportedChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64((*self).into())
    }
}

impl<'de> Deserialize<'de> for SupportedChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = ChainId::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

/// Returns wrapped-native token metadata for a supported chain.
#[must_use]
pub fn wrapped_native_token(chain_id: SupportedChainId) -> TokenInfo {
    let (address_bytes, decimals, name, symbol) = match chain_id {
        SupportedChainId::Mainnet => (WRAPPED_NATIVE_MAINNET_BYTES, 18, "Wrapped Ether", "WETH"),
        SupportedChainId::GnosisChain => (WRAPPED_NATIVE_GNOSIS_BYTES, 18, "Wrapped XDAI", "WXDAI"),
        SupportedChainId::ArbitrumOne => {
            (WRAPPED_NATIVE_ARBITRUM_BYTES, 18, "Wrapped Ether", "WETH")
        }
        SupportedChainId::Base | SupportedChainId::Ink => {
            (WRAPPED_NATIVE_BASE_INK_BYTES, 18, "Wrapped Ether", "WETH")
        }
        SupportedChainId::Sepolia => (WRAPPED_NATIVE_SEPOLIA_BYTES, 18, "Wrapped Ether", "WETH"),
        SupportedChainId::Polygon => (WRAPPED_NATIVE_POLYGON_BYTES, 18, "Wrapped POL", "WPOL"),
        SupportedChainId::Avalanche => {
            (WRAPPED_NATIVE_AVALANCHE_BYTES, 18, "Wrapped AVAX", "WAVAX")
        }
        SupportedChainId::Bnb => (WRAPPED_NATIVE_BNB_BYTES, 18, "Wrapped BNB", "WBNB"),
        SupportedChainId::Plasma => (WRAPPED_NATIVE_PLASMA_BYTES, 18, "Wrapped XPL", "WXPL"),
        SupportedChainId::Linea => (WRAPPED_NATIVE_LINEA_BYTES, 18, "Wrapped Ether", "WETH"),
    };

    let address = Address::from_bytes(address_bytes);

    TokenInfo {
        chain_id: chain_id.into(),
        logo_url: Some(format!(
            "{TOKEN_LIST_IMAGES_PATH}/{}/{}/logo.png",
            ChainId::from(chain_id),
            address.to_hex_string()
        )),
        address,
        decimals,
        name: name.to_owned(),
        symbol: symbol.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::SupportedChainId;

    #[test]
    fn supported_chain_id_still_matches_exhaustively_inside_core() {
        fn label(chain_id: SupportedChainId) -> &'static str {
            match chain_id {
                SupportedChainId::Mainnet => "mainnet",
                SupportedChainId::Bnb => "bnb",
                SupportedChainId::GnosisChain => "gnosis",
                SupportedChainId::Polygon => "polygon",
                SupportedChainId::Base => "base",
                SupportedChainId::Plasma => "plasma",
                SupportedChainId::ArbitrumOne => "arbitrum",
                SupportedChainId::Avalanche => "avalanche",
                SupportedChainId::Ink => "ink",
                SupportedChainId::Linea => "linea",
                SupportedChainId::Sepolia => "sepolia",
            }
        }

        assert_eq!(label(SupportedChainId::Mainnet), "mainnet");
        assert_eq!(label(SupportedChainId::Sepolia), "sepolia");
    }
}
