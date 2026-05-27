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
    let second =
        domain_separator_for(&domain).expect("domain_separator_for must remain deterministic");
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
        stripped
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
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
    TypedDataDomain::new(
        bounded_ascii(input.name_seed, input.name_len),
        bounded_ascii(input.version_seed, input.version_len),
        ChainId::from(input.chain_id),
        Address::from_bytes(input.verifying_contract),
    )
}

fn mutate_domain(input: &FuzzInput) -> TypedDataDomain {
    let mut mutated = build_domain(input);
    match input.mutation_selector % 4 {
        0 => {
            mutated.name = format!("{}X", mutated.name);
        }
        1 => {
            mutated.version = format!("{}Y", mutated.version);
        }
        2 => {
            mutated.chain_id = mutated.chain_id.wrapping_add(1);
        }
        _ => {
            let mut bytes = input.verifying_contract;
            bytes[0] = bytes[0].wrapping_add(1);
            mutated.verifying_contract = Address::from_bytes(bytes);
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
