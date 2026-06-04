# `fuzz_ecdsa_v_normalization` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs`.
The target consumes a fixed 65-byte input through `arbitrary` (`r || s
|| v`) and runs it through `RecoverableSignature::parse_bytes`.
Accepted outputs must remain 65 bytes, preserve `r||s`
byte-identically, map `{0, 27} -> 27` and `{1, 28} -> 28`, and rejected
inputs must surface through `ContractsError::InvalidSignatureRecoveryByte`
with the original `v` byte preserved.

## Boundary sweep — `seed-v-00.bin` through `seed-v-ff.bin`

256 seeds named `seed-v-XX.bin` (lowercase two-digit hex) exhaustively
cover every possible recovery byte against the same zero-filled
`r||s` prefix. The fixture surface is anchored by
`parity/fixtures/ecdsa/v_normalization.json` (exercised by
`crates/contracts/tests/v_normalization_contract.rs`). Each
file is 65 bytes — 64 zero bytes for `r||s` followed by the single
recovery byte indicated by the file suffix.

The sweep spans every required class because the canonical accept
contract for `RecoverableSignature` partitions the 256-byte input space
deterministically:

- canonical (accepted by `RecoverableSignature::parse_bytes`):
  `seed-v-00.bin`, `seed-v-01.bin`, `seed-v-1b.bin` (v=27), and
  `seed-v-1c.bin` (v=28).
- boundary (input-domain extremes): `seed-v-00.bin` (minimum value),
  `seed-v-ff.bin` (maximum value), plus the contiguous sweep at
  `seed-v-02.bin..seed-v-1a.bin` and `seed-v-1d.bin..seed-v-fe.bin`
  covering every rejected v byte between the accepted endpoints.
- adversarial: `seed-v-02.bin` and `seed-v-ff.bin` are the documented
  rejected-recovery-byte examples from the canonical parity fixture,
  and the 256-byte sweep also covers every other historically-rejected
  pre-EIP-155 v byte.

## Discovered-corpus seeds

37 forty-character hex-named seeds retained from prior libFuzzer smoke
runs. Each is treated as adversarial-class coverage and kept so the
typestate parser preserves any input invariants the prior fuzz sessions
exercised. Filenames:

`03d003584fe252f2688c146ef3eb931afefff2e3`,
`0eebec66cad0684adfcf2a37f1e4ed6a52672842`,
`1aa693bdd4bfd353dc378128c3be50132dde8a72`,
`217a7db567d3b01980bdf20bb93fd8ace7c55fce`,
`2228f7f94ae041e6755d37eac9d425a14e951bb5`,
`252f2a0196c189a3f51fb2be995dd626492e6e4d`,
`261cc6e301d62d5a95366222d30f7cb052833cb9`,
`2cb8df7db268128e470e62df8cb30192224a7450`,
`2da3af26ae474ece7eb2b21f98c67f81446aa113`,
`31fcc1301789357f3118b16e7a381a3d28262a74`,
`3424c8ae12116615c72e6fe1ff84efd5e01770fc`,
`3664736c786acc0f96369d9e8da4494b5d51a32d`,
`3b2a8c91510c4724e804373aeecc96684dd1a379`,
`40adebd07cd89554df31f7d8d4244cbc66b4ac3e`,
`484ef367e856885f0fca4c00983100378cd99e9f`,
`49bc98aee2f01a83ccbb35828bb8dd2dd0e0f087`,
`4da5db2bfec46c4f426f2a36ee1aa7b7b22f0c58`,
`572e19a4396855c5bef7056b5c1c7a660a7c95e9`,
`57b61e2c2937a2c7c4047427b4ef4bcbc0521fcd`,
`5f8c2c480e8471f20b2c84c7fd0089cecb4ad0b8`,
`60f8106d58b395437ed066cf1d82de1d2ee44962`,
`6e7cca9a74163861b2bb373f2817ad9144ef75ac`,
`760e54d5238fd725d35899659515019a47305816`,
`87d04c285851c7cd2195477e087fc5e8eb9624fb`,
`90934edf54d45fa443f915c0850dd09e51f3c077`,
`9824fd0ff2d2393c99a8e4622fd6368e005b512c`,
`a0013e93aa0797613f7ae6ed7c1459d8f862bbf7`,
`a627f2f9ab79303ad4bbd637f38ab709fdc2c6ec`,
`b975ed910be4414a5b97f2a5662264341a45327a`,
`ba10ceb3ac60e01972af90da6874c554c4cb0ab1`,
`c56cb9f7a548e12f574868a7313e978cade1581a`,
`da4062a79f63bcb5c5b138a5b2995bb46ad229a7`,
`dd7349ea6a539a0f65b04946ba2be0247366b5bf`,
`f1e5c559b5c7b9e646ff700f144ffa0f84348b59`,
`f2ff4e2575f328d9b2096c35e4211aaaf2d1d3a9`,
`f865f940cfbdeb8bfed8cbaf15af294c1834c27d`,
`ffb74aedd2adc1f57292f0702f41bf7f861a4fb1`.
