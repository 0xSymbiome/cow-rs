#![no_main]

//! Fuzz target for the EIP-2612 `permit_typed_data_hash` envelope.
//!
//! **Property:** `PROP-CON-017`.
//! Drives arbitrary `Eip712Domain` and `IERC20Permit::Permit` shapes
//! through the public `permit_typed_data_hash` helper and compares the
//! result against a hand-computed reference digest built from the
//! canonical `\x19\x01 || domain_separator || struct_hash` envelope.
//! The invariant proves the helper composes the envelope exactly as
//! specified by EIP-712, independent of any internal ordering or
//! padding assumption.
//!
//! Inputs are derived via [`arbitrary::Arbitrary`] on the documented
//! domain fields (optional name, optional version, optional chain id,
//! optional verifying contract, optional salt) plus the five
//! `Permit` struct fields (`owner`, `spender`, `value`, `nonce`,
//! `deadline`). `String::from_utf8_lossy` is applied so arbitrary byte
//! sequences always parse into valid `Cow<str>` domain fields.

use alloy_sol_types::{
    Eip712Domain, SolStruct,
    private::{Address, Cow, FixedBytes, U256},
};
use cow_sdk_contracts::{IERC20Permit, permit_typed_data_hash};
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};
use sha3::{Digest, Keccak256};

/// Maximum byte width for the domain `name` seed. EIP-712 domains in
/// practice carry short product names; 32 bytes is more than enough to
/// cover boundary shapes.
const MAX_NAME_BYTES: usize = 32;
/// Maximum byte width for the domain `version` seed. Domain versions
/// are short semver-style strings; 16 bytes exceeds every production
/// shape we know of.
const MAX_VERSION_BYTES: usize = 16;

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    has_name: bool,
    name_bytes: [u8; MAX_NAME_BYTES],
    name_len: u8,
    has_version: bool,
    version_bytes: [u8; MAX_VERSION_BYTES],
    version_len: u8,
    has_chain_id: bool,
    chain_id: u64,
    has_verifying_contract: bool,
    verifying_contract: [u8; 20],
    has_salt: bool,
    salt: [u8; 32],
    owner: [u8; 20],
    spender: [u8; 20],
    value: [u8; 32],
    nonce: [u8; 32],
    deadline: [u8; 32],
}

fuzz_target!(|input: FuzzInput| {
    let name = input.has_name.then(|| {
        let end = usize::from(input.name_len) % (MAX_NAME_BYTES + 1);
        let slice = &input.name_bytes[..end];
        Cow::Owned(String::from_utf8_lossy(slice).into_owned())
    });
    let version = input.has_version.then(|| {
        let end = usize::from(input.version_len) % (MAX_VERSION_BYTES + 1);
        let slice = &input.version_bytes[..end];
        Cow::Owned(String::from_utf8_lossy(slice).into_owned())
    });
    let chain_id = input.has_chain_id.then(|| U256::from(input.chain_id));
    let verifying_contract = input
        .has_verifying_contract
        .then(|| Address::from(input.verifying_contract));
    let salt = input.has_salt.then(|| FixedBytes::<32>::from(input.salt));

    let domain = Eip712Domain::new(name, version, chain_id, verifying_contract, salt);

    let permit = IERC20Permit::Permit {
        owner: Address::from(input.owner),
        spender: Address::from(input.spender),
        value: U256::from_be_bytes(input.value),
        nonce: U256::from_be_bytes(input.nonce),
        deadline: U256::from_be_bytes(input.deadline),
    };

    let digest = permit_typed_data_hash(&domain, &permit);

    let domain_separator: [u8; 32] = domain.separator().into();
    let struct_hash: [u8; 32] = permit.eip712_hash_struct().into();
    let mut hasher = Keccak256::new();
    hasher.update([0x19, 0x01]);
    hasher.update(domain_separator);
    hasher.update(struct_hash);
    let reference: [u8; 32] = hasher.finalize().into();

    assert_eq!(
        digest, reference,
        "permit_typed_data_hash must match keccak256(0x1901 || domain_separator || struct_hash)",
    );
});
