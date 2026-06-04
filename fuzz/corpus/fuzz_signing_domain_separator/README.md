# `fuzz_signing_domain_separator` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_signing_domain_separator.rs`.
Each seed is a deterministic `Arbitrary` payload that drives
`cow_sdk_signing::domain_separator_for(&TypedDataDomain)` and the
target's adjacent mutation-resistance check. The domain field shape is
pinned by the focused `crates/signing/tests/domain_contract.rs` contract
test (`name`, `version`, `chainId`, `verifyingContract`).

## Seed contract

The `Arbitrary`-derived input layout is, in order:

- `name_seed: u8`
- `name_len: u8`
- `version_seed: u8`
- `version_len: u8`
- `chain_id: u64` (little-endian)
- `verifying_contract: [u8; 20]`
- `mutation_selector: u8` (drives the post-baseline mutation across the
  four domain fields)

Total size: 33 bytes.

## Seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-mainnet.bin` | canonical | Mainnet `Gnosis Protocol v2` shape with the canonical settlement verifying contract; the field set matches `crates/signing/tests/domain_contract.rs`. |
| `seed-01-boundary-empty-name.bin` | boundary | Zero-length name and version with a zero verifying contract, exercising the minimum-input typed-domain shape. |
| `seed-02-boundary-max-name.bin` | boundary | Maximum-length ASCII name and version (`len = 32`), exercising the upper input window for the bounded-ASCII generator. |
| `seed-03-boundary-chain-id-max.bin` | boundary | `chain_id = u64::MAX`, exercising the saturated 256-bit chain-id word in the EIP-712 domain encoding. |
| `seed-04-boundary-ff-verifier.bin` | boundary | All-`0xff` verifying contract, exercising the boundary address-encoding word. |
| `seed-05-adversarial-bit-flip-verifier.bin` | adversarial | Canonical mainnet domain with a single-bit flip on the first verifying-contract byte, exercising the mutation-resistance assertion that any single-field change must shift the digest. |
