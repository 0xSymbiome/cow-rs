# `fuzz_core_identity_validators` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_core_identity_validators.rs`.
The first byte selects which typed identity constructor the rest of the
input is routed into: `Address` (`0`), `Hash32` (`1`), `AppDataHash`
(`2`), `OrderUid` (`3`), or `HexData` (`4`) modulo five.

Seed classes:

- canonical: `seed-canonical-address-zero.bin`,
  `seed-canonical-hash32-zero.bin`,
  `seed-canonical-appdata-zero.bin`,
  `seed-canonical-orderuid-zero.bin`, and
  `seed-canonical-hexdata-empty.bin` are derived from the
  `core-evm-address-contract` fixture id in
  `parity/fixtures/core.json` as the
  canonical all-zero string form of each typed identity.
- boundary: `seed-boundary-address-uppercase.bin` covers a checksummed
  mixed-case address that the equality and round-trip contract must
  preserve, while `seed-boundary-hexdata-odd-length.bin` covers the
  odd-length normalization path documented on `HexData::new`.
- adversarial: `seed-adversarial-address-bad-prefix.bin`,
  `seed-adversarial-orderuid-truncated.bin`, and
  `seed-adversarial-hash32-nonhex.bin` carry wrong-prefix, truncated,
  and non-hex inputs that the validators must reject without panicking.

All seed files are intentionally tiny and platform-neutral.
