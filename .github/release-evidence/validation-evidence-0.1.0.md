# Validation Evidence - cow-rs 0.1.0

Generated: 2026-05-02T14:29:31Z
Workflow: release-readiness
Workflow file: .github/workflows/release-readiness.yml
Workflow run: pending final run
Candidate commit: pending-final-commit
Release classification: first_functional (semver-checks: skip)

## Lane Status

| Lane | Status | Step | Notes |
| --- | --- | --- | --- |
| adr-coverage | pass | quality-gate/adr-coverage | policy-maintainer check-adr-coverage --mode blocking |
| alloy-provider-invariant | pass | quality-gate/alloy-provider-invariant | policy-maintainer check-alloy-provider-invariant |
| audit | pass | quality-gate/audit | cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2024-0436 |
| cargo-semver-checks | pass | quality-gate/cargo-semver-checks | first functional release classified with semver-checks skip |
| chain-patch-eligibility | pass | quality-gate/chain-patch-eligibility | policy-maintainer check-chain-patch-eligibility |
| clippy | pass | quality-gate/clippy | cargo clippy --workspace --all-targets --all-features -- -D warnings |
| compatibility-floor | pass | release-readiness/compatibility-floor | cargo +1.94.0 check --workspace --all-features; cargo +1.94.0 test --workspace |
| deny | pass | quality-gate/deny | cargo deny check --config .github/config/deny.toml |
| deny-unknown-fields-coverage | pass | quality-gate/deny-unknown-fields-coverage | policy-maintainer check-deny-unknown-fields |
| docs | pass | quality-gate/docs | cargo doc --workspace --all-features --no-deps |
| doctest | pass | release-readiness/doctest | cargo test --workspace --doc |
| enum-policy | pass | quality-gate/enum-policy | policy-maintainer check-enum-policy |
| feature-matrix | pass | quality-gate/feature-matrix | cargo hack check --workspace --feature-powerset --depth 1 |
| fmt | pass | quality-gate/fmt | cargo fmt --all -- --check |
| msrv-notice-window | pass | release-readiness/msrv-notice-window | policy-maintainer check-msrv-notice --initial-release |
| native-examples-locked | pass | release-readiness/native-examples-locked | cargo check and cargo test with examples/native lockfile |
| nextest | pass | quality-gate/nextest | cargo nextest run --workspace --all-features |
| openapi-coverage | pass | quality-gate/openapi-coverage | parity-maintainer openapi-coverage --validate |
| panic-allowlist | pass | quality-gate/panic-allowlist | policy-maintainer check-panic-allowlist |
| parity-maintainer-local | pass | release-readiness/parity-maintainer-local | parity-maintainer validate --source-lock parity/source-lock.yaml |
| parity-maintainer-provenance | pass | release-readiness/parity-maintainer-provenance | parity-maintainer validate against fresh source-lock-pinned checkouts |
| public-api-lints | pass | quality-gate/public-api-lints | RUSTFLAGS missing-docs, missing-debug-implementations, unreachable-pub, unnameable-types as deny |
| publication | pass | release-readiness/publication | cargo build --frozen; cargo package and cargo publish --dry-run package family |
| registry-confirm | pass | release-readiness/registry-confirm | validation-smoke registry-confirm --mode release --check for all supported release chains |
| sbom | pass | release-readiness/sbom | cargo cyclonedx --format json --all --override-filename cow-rs-sbom |
| source-lock-freshness | pass | release-readiness/source-lock-freshness | parity-maintainer check-freshness report-only lane |
| test | pass | quality-gate/test | cargo test --workspace --all-features |
| typos | pass | quality-gate/typos | typos --config .github/config/typos.toml |
| validation-evidence | pass | release-readiness/validation-evidence | policy-maintainer generate-validation-evidence --release-version 0.1.0 --check |
| wasm-pack-pinned | pass | release-readiness/wasm-pack-pinned | wasm-pack lanes use validation-smoke wasm-runner-setup |
| wasm-runner-freshness | pass | release-readiness/wasm-runner-freshness | policy-maintainer check-wasm-runner-freshness |
| windows-stable | pass | release-readiness/windows-stable | cargo +stable check --workspace --all-features; cargo +stable test --workspace --lib --tests |
| workspace-version-alignment | pass | quality-gate/workspace-version-alignment | policy-maintainer check-workspace-versions |

## Source-Lock

Generated at: 2026-04-29T00:00:00Z

| Repository | Remote | Pinned commit | Role |
| --- | --- | --- | --- |
| contracts | https://github.com/cowprotocol/contracts.git | c94c595a791681cf8ba7495117dcde397b932885 | primary |
| cow-sdk | https://github.com/cowprotocol/cow-sdk.git | 00c3dbd41c086ff9a51d5e5a30648615d4c66d0d | primary |
| services | https://github.com/cowprotocol/services.git | 0720b9bc15138ecc362078f505d0e3ba1c7b9883 | reference-only |

## OpenAPI Vendoring

| Source | Path | Pinned commit | Generated at |
| --- | --- | --- | --- |
| cowprotocol/services | crates/orderbook/openapi.yml | 0720b9bc15138ecc362078f505d0e3ba1c7b9883 | 2026-05-02T14:24:41Z |

## WASM Runner

| Field | Value |
| --- | --- |
| Channel | Stable |
| Chrome version | 148.0.7778.56 |
| ChromeDriver version | 148.0.7778.56 |
| Revision | 1610480 |
| Released at | 2026-04-28T20:36:36.653Z |

## Deployment Provenance

Generated at: 2026-04-29T00:00:00Z

| Chain ID | Environment | Contract | Address | Code hash | Confirmed at |
| --- | --- | --- | --- | --- | --- |
| 1 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0x875c543737c0bc49033a5c35b0c84ce8f8d40e636e54753eab74093fe1802f68 | 2026-04-29T17:26:25Z |
| 1 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0x7c11bcb168b80a1aecdcf02f59c62833680953da66983e3976ed029fc94a5514 | 2026-04-29T17:26:25Z |
| 1 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0x744d58584e38d214eb190629f131d5cf8b8703bd68e04452f9692177c37c4bc9 | 2026-04-29T17:26:25Z |
| 1 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0xb9da0b79eac25fa06600d4f5cdd99ecea6c56c40fec47756323e607a72d9d7bd | 2026-04-29T17:26:25Z |
| 1 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 1 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 56 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0x2eb86ed732e0891f77facc91c96b7a2e51ee4f640498ecb3e36cf84ef37b7923 | 2026-04-29T17:26:25Z |
| 56 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0x59a1af5881bb00519f8d758cbe64d291017aff4f244c18de665856a95e0ec5db | 2026-04-29T17:26:25Z |
| 56 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0x681b39b3355153c3f8ff25d44d73abeca42b51e499d9c54d7354121405a004c4 | 2026-04-29T17:26:25Z |
| 56 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0x4be734a4feee4d8bf57b59e540101d18e1e9449fc8cafe57fb5288aee29079e6 | 2026-04-29T17:26:25Z |
| 56 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 56 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 100 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0x900da705b73ba6d9563531dff170bc2ca21f6105114014f54e0fdb88a8e0baae | 2026-04-29T17:26:25Z |
| 100 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0xe10b269faaef98fabe6d490cfea008fc65cb6e981514770671061bd26b3289d4 | 2026-04-29T17:26:25Z |
| 100 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0x68963e5b27aadd4ee70ecd933fac9312fe5f527390b88ae6092c68937b80f5e2 | 2026-04-29T17:26:25Z |
| 100 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0xe49c7157f6cc80593dfc7c57ac5cbac68cd89f2365f2841b4e368316d07dcce7 | 2026-04-29T17:26:25Z |
| 100 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 100 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 137 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0xbed50afd8aed8367eca302bc10348ed26ab860c4aab6b8776b34b4fea283804e | 2026-04-29T17:26:25Z |
| 137 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0xe21e06906be5e488a73440efdb79bb0a6e6abb1c41d986b83283c69c8c7aad2c | 2026-04-29T17:26:25Z |
| 137 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0xe264ecc678de6464b9365ff73e07858e2c2b07adcd5c8209cb04ebe0e9ef2c14 | 2026-04-29T17:26:25Z |
| 137 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0x74b2d81d7c3fc31f19781b41803f716f9c752a96b9ed835a68747953a0541324 | 2026-04-29T17:26:25Z |
| 137 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 137 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 8453 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0xcf69f4ef2e68552be7168f0772f26f9edad8d680f83d6053cec64874cab1a538 | 2026-04-29T17:26:25Z |
| 8453 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0x943ab77836221a02a7a94927b7d2ae29d8af605e6f05440f416105e2d5fc190b | 2026-04-29T17:26:25Z |
| 8453 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0x851476c2307a7c011d2435d5e5aaae3a41c517f52461b5abefe2a4e42114cbf6 | 2026-04-29T17:26:25Z |
| 8453 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0x3f480b4e35da45d472e0d931df9dc492fc5644f1346e02a7c9cc4cf3072820a0 | 2026-04-29T17:26:25Z |
| 8453 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 8453 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 9745 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0xd6a42f593076cc49b4bd61e11d4a1053ffaff28df4ac57a1c18b99b8d172c583 | 2026-04-29T17:26:25Z |
| 9745 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0xccceb467fc2f1bc1f84cacdace6d2372c329ae860e825e8a3e32b0179a615a2d | 2026-04-29T17:26:25Z |
| 9745 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0x0f867891bddf9798e580617b4e4b04a7cc8010aca4b5240c07eb7823c8b7e16f | 2026-04-29T17:26:25Z |
| 9745 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0xc6faa5ddc6413bff877c947fc8c7897a1b793e650fa47773b221e5e855defd99 | 2026-04-29T17:26:25Z |
| 9745 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 9745 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 42161 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0x0f2dff362284a851efe1e467f5456ce11e8eb23e74364a17d73c6e4c7b5f32b8 | 2026-04-29T17:26:25Z |
| 42161 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0xdfaacd6ea51ac9f3b3d99acea4ed1af536892ed4345db1ccfa1d7513c14cc303 | 2026-04-29T17:26:25Z |
| 42161 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0xc5d94a317d3c8f717d4238b1a4bee2bc9cec18697c82e1d63865833ebdbd523c | 2026-04-29T17:26:25Z |
| 42161 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0xc7b79b602e144a539497c7f27662bd1c365d0a454bc9ca3dc9ba4f50b29540d4 | 2026-04-29T17:26:25Z |
| 42161 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 42161 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 43114 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0x72f239a7a05b8b4d9a875eb26fb1d7400a6ad0778861d79a81c91a72e14edbf0 | 2026-04-29T17:26:25Z |
| 43114 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0x82262ad5aaa42776aa470273ce09bbddb05296fa9c44b129828742b8d0660286 | 2026-04-29T17:26:25Z |
| 43114 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0xc5b8516d7e501ef4c79c135ba4a55b674211f5b2add786f00c89bfd2ad250f5b | 2026-04-29T17:26:25Z |
| 43114 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0x864cc1450450e7ecb079c24aa168015279895908ca7cd02834d02d5d9d87c2b9 | 2026-04-29T17:26:25Z |
| 43114 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 43114 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 57073 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0x9bc4c4f9de7b6c566a67b32d96d7b36fa3bc6a65bf1c0838530cb78a25fec787 | 2026-04-29T17:26:25Z |
| 57073 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0xfd715c6df0966a922331daf40fc57e5c4f4d052a4722cae2a1bea79205be9397 | 2026-04-29T17:26:25Z |
| 57073 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0xf00e931b6bf28069ba1c67e93a561e5ffefed30a13ea6b48ca98f39a7803d2a3 | 2026-04-29T17:26:25Z |
| 57073 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0xd3fa8e1f9e2fe03e6a85bfd634b6c2c9b80a57b62c04666a86f1a07545f9ab0a | 2026-04-29T17:26:25Z |
| 57073 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 57073 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 59144 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0xbe858a5a5d014ab3b0c1dfc209bc6e9be31c79d210c6882d7ef186a7457f8765 | 2026-04-29T17:26:25Z |
| 59144 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0xc1d4b86edeb28de980e8114924eeb4d041f39fd3a07be2e074fc2b65b653ecf8 | 2026-04-29T17:26:25Z |
| 59144 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0x7462bb78e289d0aeaad6346fd60740a631496d033d2d606ce0d489c340dad3f3 | 2026-04-29T17:26:25Z |
| 59144 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0x00824780e6b65d5779cfb7ae062b7cf2287088347e9f71bd0d43f6bed2570208 | 2026-04-29T17:26:25Z |
| 59144 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 59144 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
| 11155111 | prod | EthFlow | 0xba3cb449bd2b4adddbc894d8697f5170800eadec | 0x775e133373822a18a1ff4530c3391399d8cbc067fa31a7abd67991cfca61afbd | 2026-04-29T17:26:25Z |
| 11155111 | staging | EthFlow | 0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC | 0x80cc0f6a7ebf4385b1ecdd101b10dd0f8b745650276c3ab139c7e9ab85f0319d | 2026-04-29T17:26:25Z |
| 11155111 | prod | Settlement | 0x9008D19f58AAbD9eD0D60971565AA8510560ab41 | 0x9fbace363dc778e25fecb202c12981d916faea80c9aab8167aeeedcaed84df53 | 2026-04-29T17:26:25Z |
| 11155111 | staging | Settlement | 0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13 | 0x2bd5287a0e8ee6859ac371fac032caf3e193c8785a476913bc017325f83ac2aa | 2026-04-29T17:26:25Z |
| 11155111 | prod | VaultRelayer | 0xC92E8bdf79f0507f65a392b0ab4667716BFE0110 | 0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e | 2026-04-29T17:26:25Z |
| 11155111 | staging | VaultRelayer | 0xC7242d167563352E2BCA4d71C043fbe542DB8FB2 | 0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55 | 2026-04-29T17:26:25Z |
