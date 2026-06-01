# `fuzz_recoverable_signature_differential` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_recoverable_signature_differential.rs`.
The target drives a fixed 65-byte signature payload through both
`cow_sdk_contracts::RecoverableSignature::parse_bytes` and
`alloy_primitives::Signature::from_raw`, asserting that the cow accept set is a
strict subset of alloy's, that an agreed rejection cites the same trailing
recovery byte through `alloy_primitives::SignatureError::InvalidParity`, and
that the canonical 65-byte output is byte-identical when both surfaces accept.

Each seed file is a 65-byte payload: 32 bytes `r`, 32 bytes `s`, then one
trailing recovery byte `v`. The trailing byte is the discriminating input — the
cow accept set is exactly `{0, 1, 27, 28}` and the canonical form normalizes to
`{27, 28}`.

Seed classes:

- canonical: `seed-canonical-v27.bin` and `seed-canonical-v28.bin` carry the two
  canonical recovery bytes both surfaces accept; `seed-canonical-v0.bin` and
  `seed-canonical-v1.bin` carry the legacy `{0, 1}` parities cow accepts and
  normalizes to `{27, 28}`.
- boundary: `seed-boundary-zero.bin` (all-zero `r`/`s` with `v = 27`) and
  `seed-boundary-max.bin` (all-`0xff` `r`/`s` with `v = 28`) pin the payload
  extremes inside the accept set.
- adversarial: `seed-adversarial-v35.bin` carries the EIP-155 `v = 35` byte
  alloy accepts but cow rejects (the strict-subset payoff), and
  `seed-adversarial-v255.bin` carries `v = 255`, which both surfaces reject on
  parity.

All seed files are intentionally tiny and platform-neutral.
