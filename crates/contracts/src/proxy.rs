use alloy_primitives::{FixedBytes, fixed_bytes};
use alloy_sol_types::sol;
use cow_sdk_core::{Address, Provider};

use crate::ContractsError;

sol! {
    // Canonical EIP-173 ownership proxy interface used by cow-sdk consumers
    // to read the current owner, transfer ownership, and probe ERC-165
    // interface support. Signatures are reproduced verbatim from the
    // canonical EIP-173 specification (https://eips.ethereum.org/EIPS/eip-173).
    // The companion Solidity excerpt and the EIP-1967 storage-slot derivation
    // are committed under `crates/contracts/abi/eip1967/` for provenance.
    #[sol(rename_all = "camelcase")]
    interface IEip173Proxy {
        event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

        function owner() external view returns (address);

        function transferOwnership(address newOwner) external;

        function supportsInterface(bytes4 interfaceID) external view returns (bool);
    }
}

/// Strongly typed EIP-1967 storage slot selector.
///
/// Each variant carries the canonical 32-byte slot hash defined by
/// <https://eips.ethereum.org/EIPS/eip-1967>. The admin slot additionally
/// matches the value pinned in the upstream cowprotocol/contracts library
/// (`src/contracts/libraries/GPv2EIP1967.sol`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Eip1967Slot {
    /// `keccak256("eip1967.proxy.admin") - 1`.
    Admin,
    /// `keccak256("eip1967.proxy.implementation") - 1`.
    Implementation,
}

/// 32-byte representation of an EIP-1967 storage slot.
pub type SlotBytes = FixedBytes<32>;

const ADMIN_SLOT: SlotBytes =
    fixed_bytes!("0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103");
const IMPLEMENTATION_SLOT: SlotBytes =
    fixed_bytes!("0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc");

// The hex-form constants below are the same 32 bytes rendered as
// `0x`-prefixed lowercase hex. They exist on the
// `Provider::get_storage_at(&self, proxy: &Address, slot: &str)`
// trait-shape stability seam — the trait method takes the slot as a
// string at the cow-side `Provider` surface. The
// `eip1967_slot_hex_strings_match_their_byte_forms` round-trip test in
// `crates/contracts/tests/proxy_contract.rs` pins the relationship
// between the byte form and the string form.
const ADMIN_SLOT_HEX: &str = "0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103";
const IMPLEMENTATION_SLOT_HEX: &str =
    "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";

impl Eip1967Slot {
    /// Returns the 32-byte slot value as a typed [`SlotBytes`].
    #[must_use]
    pub const fn as_bytes(self) -> SlotBytes {
        match self {
            Self::Admin => ADMIN_SLOT,
            Self::Implementation => IMPLEMENTATION_SLOT,
        }
    }

    /// Returns the canonical `0x`-prefixed hex representation of the slot
    /// value as a static string reference.
    #[must_use]
    pub const fn as_hex_str(self) -> &'static str {
        match self {
            Self::Admin => ADMIN_SLOT_HEX,
            Self::Implementation => IMPLEMENTATION_SLOT_HEX,
        }
    }
}

/// Reads the implementation address from the proxy implementation slot.
///
/// # Errors
///
/// Returns [`ContractsError`] when the provider call fails or the storage value
/// cannot be decoded into an address.
pub fn implementation_address<P>(provider: &P, proxy: &Address) -> Result<Address, ContractsError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    read_address_slot(provider, proxy, Eip1967Slot::Implementation)
}

/// Reads the administrator address from the proxy admin slot.
///
/// # Errors
///
/// Returns [`ContractsError`] when the provider call fails or the storage value
/// cannot be decoded into an address.
pub fn admin_address<P>(provider: &P, proxy: &Address) -> Result<Address, ContractsError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    read_address_slot(provider, proxy, Eip1967Slot::Admin)
}

/// Reads the administrator address stored at the EIP-1967 admin slot.
///
/// Alias of [`admin_address`] retained for call-sites that read the slot as
/// the "owner" of an EIP-173 ownership proxy.
///
/// # Errors
///
/// Returns [`ContractsError`] when the provider call fails or the storage value
/// cannot be decoded into an address.
pub fn owner_address<P>(provider: &P, proxy: &Address) -> Result<Address, ContractsError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    admin_address(provider, proxy)
}

fn read_address_slot<P>(
    provider: &P,
    proxy: &Address,
    slot: Eip1967Slot,
) -> Result<Address, ContractsError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    // The `Provider::get_storage_at` contract below the typed surface takes
    // the slot selector as a `0x`-prefixed hex string; we pipe the typed
    // value through `Eip1967Slot::as_hex_str` so the call-site consumes the
    // typed slot instead of an ad-hoc literal.
    let word = provider
        .get_storage_at(proxy, slot.as_hex_str())
        .map_err(|error| ContractsError::Provider {
            operation: "get_storage_at",
            message: error.to_string().into(),
        })?;
    decode_storage_address(&word.to_hex_string())
}

fn decode_storage_address(value: &str) -> Result<Address, ContractsError> {
    use alloy_sol_types::private::{Address as SolAddress, FixedBytes};

    let stripped = value
        .strip_prefix("0x")
        .ok_or(ContractsError::InvalidHexPrefix {
            field: "storageSlot",
        })?;
    let bytes =
        alloy_primitives::hex::decode(stripped).map_err(|source| ContractsError::DecodeHex {
            field: "storageSlot",
            source,
        })?;
    let buf: [u8; 32] =
        bytes
            .try_into()
            .map_err(|raw: Vec<u8>| ContractsError::InvalidDecodedLength {
                field: "storageSlot",
                expected: 32,
                actual: raw.len(),
            })?;
    let word = FixedBytes::<32>::from(buf);
    Ok(Address::from_bytes(SolAddress::from_word(word).into()))
}
