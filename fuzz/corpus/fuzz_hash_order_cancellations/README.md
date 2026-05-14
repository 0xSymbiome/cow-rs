# `fuzz_hash_order_cancellations` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_hash_order_cancellations.rs`.
Each seed is a deterministic `Arbitrary` payload that drives the typed
`EIP712` order-cancellation digest pipeline through
`cow_sdk_contracts::hash_order_cancellations`. The cancellation
type-field shape is fixed by the parity fixture
`parity/fixtures/contracts.json::contracts-cancellation-type-fields`,
which pins the single `orderUids` field consumed by every seed in this
directory.

## Seed contract

The `Arbitrary`-derived input layout is, in order:

- `domain_name_seed: u8`
- `domain_name_len: u8`
- `domain_version_seed: u8`
- `domain_version_len: u8`
- `chain_id: u64` (little-endian)
- `verifying_contract: [u8; 20]`
- `uid_count: u8` (capped at 16 inside the target)
- `seed_digest: [u8; 32]`
- `seed_owner: [u8; 20]`
- `seed_valid_to: u32` (little-endian)
- `rotation_seed: u8`

Total size: 90 bytes. Seeds shorter than that consume the available
prefix and rely on the documented `Arbitrary` defaults for the
remaining fields.

## Seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-single-uid.bin` | canonical | Mainnet `Gnosis Protocol v2` domain with the canonical settlement verifying contract and a single deterministically packed UID; the type-field shape matches `parity/fixtures/contracts.json::contracts-cancellation-type-fields`. |
| `seed-01-boundary-empty.bin` | boundary | All-zero prefix that forces `uid_count = 0` and exercises the empty-batch hashing path. |
| `seed-02-boundary-zero-uid.bin` | boundary | `uid_count = 1` with an all-zero digest, owner, and `valid_to`, exercising the minimum-value canonical UID. |
| `seed-03-boundary-ff-uid.bin` | boundary | `uid_count = 1` with all-`0xff` bytes, exercising the maximum-byte canonical UID and the saturated `valid_to = u32::MAX`. |
| `seed-04-boundary-valid-to-max.bin` | boundary | Single UID with `valid_to = u32::MAX` paired with a structured non-trivial domain. |
| `seed-05-boundary-max-batch.bin` | boundary | `uid_count = 16` (the in-target cap), exercising the maximum-length cancellation batch. |
| `seed-06-adversarial-collision-rotate.bin` | adversarial | `uid_count = 3` with `rotation_seed = 0` so the per-index UID rotation only perturbs `digest[0]`, exercising the collision-vs-determinism boundary on near-identical UIDs. |
