#![no_main]

//! Fuzz target for the explicit-domain `EIP712Domain` separator helper.
//!
//! **Surface:** `cow_sdk_signing::domain_separator_for(&TypedDataDomain)`.
//! **Property:** `PROP-SIG-001`.
//! **Seed contract:** corpus inputs cover the canonical mainnet
//! `Gnosis Protocol v2` domain, an empty-name boundary, a maximum-length
//! ASCII name, a saturated `chain_id = u64::MAX` boundary, an all-`0xff`
//! verifying-contract boundary, and an adversarial mutation seed whose
//! version byte differs by a single bit from the canonical domain.
//! **Corpus README:** `../corpus/fuzz_signing_domain_separator/README.md`.
//!
//! The target asserts the public `domain_separator_for` helper:
//!
//! * Never panics for any constructible `TypedDataDomain`.
//! * Returns a `0x`-prefixed 32-byte (64 hex char) digest.
//! * Is deterministic across two calls on identical input.
//! * Mutating any single field of the domain (name, version, chain id,
//!   or verifying contract) produces a different separator.

use cow_sdk_core::{Address, ChainId, TypedDataDomain};
use cow_sdk_signing::domain_separator_for;
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};

const BOUNDED_NAME_WINDOW: usize = 33;

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    name_seed: u8,
    name_len: u8,
    version_seed: u8,
    version_len: u8,
    chain_id: u64,
    verifying_contract: [u8; 20],
    mutation_selector: u8,
}

fuzz_target!(|input: FuzzInput| {
    let domain = build_domain(&input);

    let first = domain_separator_for(&domain)
        .expect("domain_separator_for must accept any byte-constructed domain");
    let second = domain_separator_for(&domain)
        .expect("domain_separator_for must remain deterministic");
    assert_eq!(
        first, second,
        "domain_separator_for must produce the same digest for identical inputs",
    );

    assert!(
        first.starts_with("0x"),
        "domain separator must keep the 0x prefix, got {first:?}",
    );
    let stripped = &first[2..];
    assert_eq!(
        stripped.len(),
        64,
        "domain separator hex body must be 64 ASCII chars (32 bytes), got {} chars",
        stripped.len(),
    );
    assert!(
        stripped.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "domain separator must be lowercase hex, got {first:?}",
    );

    // Mutation-resistance check: change exactly one field of the domain
    // and assert the separator differs from the baseline. The selector
    // partitions the mutation across the four domain fields so every
    // round of fuzzing visits a deterministic mutation class.
    let mutated = mutate_domain(&input);
    if mutated != domain {
        let mutated_separator = domain_separator_for(&mutated)
            .expect("mutated domain_separator_for must remain deterministic");
        assert_ne!(
            first, mutated_separator,
            "mutating any single domain field must change the separator (baseline={domain:?}, mutated={mutated:?})",
        );
    }
});

fn build_domain(input: &FuzzInput) -> TypedDataDomain {
    TypedDataDomain {
        name: Some(bounded_ascii(input.name_seed, input.name_len).into()),
        version: Some(bounded_ascii(input.version_seed, input.version_len).into()),
        chain_id: Some(alloy_primitives::U256::from(ChainId::from(input.chain_id))),
        verifying_contract: Some(*Address::from_bytes(input.verifying_contract).as_alloy()),
        salt: None,
    }
}

fn mutate_domain(input: &FuzzInput) -> TypedDataDomain {
    let mut mutated = build_domain(input);
    match input.mutation_selector % 4 {
        0 => {
            let current = mutated.name.as_deref().unwrap_or_default().to_owned();
            mutated.name = Some(format!("{current}X").into());
        }
        1 => {
            let current = mutated.version.as_deref().unwrap_or_default().to_owned();
            mutated.version = Some(format!("{current}Y").into());
        }
        2 => {
            let current = mutated
                .chain_id
                .unwrap_or(alloy_primitives::U256::ZERO);
            mutated.chain_id = Some(current.wrapping_add(alloy_primitives::U256::from(1u64)));
        }
        _ => {
            let mut bytes = input.verifying_contract;
            bytes[0] = bytes[0].wrapping_add(1);
            mutated.verifying_contract = Some(*Address::from_bytes(bytes).as_alloy());
        }
    }
    mutated
}

/// Builds a bounded ASCII string from a seed byte and a length byte.
///
/// The length is clamped to a short window and the characters map the
/// seed through a printable-ASCII window so the resulting string never
/// violates `TypedDataDomain` expectations.
fn bounded_ascii(seed: u8, len_byte: u8) -> String {
    let len = usize::from(len_byte) % BOUNDED_NAME_WINDOW;
    if len == 0 {
        return String::new();
    }
    (0..len)
        .map(|offset| char::from(b'A' + (((seed as usize + offset) as u8) % 26)))
        .collect()
}
