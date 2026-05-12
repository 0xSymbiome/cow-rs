# Deployment Registry Audit

Status: Current
Last reviewed: 2026-05-12
Re-review by: 2026-08-02
Owning surface: `cow-sdk-contracts` deployment registry and provenance manifest
Refresh trigger: Changes to `crates/contracts/registry.toml`, `crates/contracts/deployment-provenance.yaml`, the compile-time validator in `build.rs`, the `registry-confirm` live-confirmation contract, deployed addresses, or supported chains
Related docs:
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0032](../adr/0032-deployment-authority-machine-readable-provenance.md)
- [Deployments](../deployments.md)
- [Architecture](../architecture.md)
- [Parity Matrix](../parity-matrix.md)

## Scope

This audit covers:

- the embedded `registry.toml` address manifest and typed `Registry` lookup surface
- the `deployment-provenance.yaml` source-provenance manifest
- compile-time agreement checks between registry rows and provenance rows
- live bytecode confirmation through `validation-smoke registry-confirm`

It does not cover binding generation, partner API routing, arbitrary consumer RPC configuration, or future contract upgrades after the recorded confirmation time.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Registry completeness | Every `(ContractId, SupportedChainId, CowEnv)` registry row has one provenance row | Conforms |
| Chain provenance | Every `SupportedChainId` variant has a source-cited services, TypeScript SDK, deployment-provenance, and wrapped-native-token row | Conforms |
| Runtime lookup matrix | Every supported `(ContractId, SupportedChainId, CowEnv)` tuple is either a typed deployed address or an explicit unsupported lookup without silent fallback | Conforms |
| Source authority | Each provenance row records primary or secondary upstream authority at a pinned source commit | Conforms |
| Compile-time validation | `build.rs` rejects missing, duplicate, extra, malformed, or address-mismatched provenance rows | Conforms |
| Live confirmation | Every release-facing row records `kind: code_hash` with a non-zero `keccak256(eth_getCode)` value | Conforms |
| Release check | `registry-confirm --mode release --check` recomputes bytecode evidence without mutating the repository | Conforms |

## Current Contract

### Registry And Provenance

`crates/contracts/registry.toml` remains the runtime address source of truth. `crates/contracts/deployment-provenance.yaml` is the auditable evidence layer keyed by the same `(contract_id, chain_id, env)` tuple. The provenance file carries one row for every registry row: three contract families, eleven supported chains, and both production and staging environments.

The provenance row records:

- `address`
- `authority`
- `source_repo`
- `source_commit`
- `source_path`
- `source_symbol`
- `live_confirmation`

The runtime lookup regression enumerates every shipped contract id across each
supported chain and environment. Tuples present in the embedded manifest must
resolve to the same non-zero address as their manifest row; unsupported tuples
must stay typed misses rather than falling back to another chain, environment,
or contract family.

### Source Authority

| Authority | Rows | Source contract |
| --- | ---: | --- |
| `primary` | 16 | Production `Settlement` and `VaultRelayer` rows sourced from `cowprotocol/contracts` `networks.json` at commit `c94c595a791681cf8ba7495117dcde397b932885` |
| `secondary` | 50 | Staging rows, Plasma/Linea/Ink `Settlement` and `VaultRelayer` rows, and all `EthFlow` rows sourced from `cowprotocol/cow-sdk` `packages/config/src/chains/const/contracts.ts` at commit `00c3dbd41c086ff9a51d5e5a30648615d4c66d0d` |

## Per-chain Provenance

The table below is the canonical supported-chain provenance view for the
release-facing registry. It intentionally lives in this deployment-registry
audit so chain support, deployed contract provenance, services-generated
metadata, TypeScript SDK support, and wrapped-native-token evidence have one
reviewed authority.

| Chain | `SupportedChainId` variant | Numeric chain id | Deployment provenance | Services metadata | TypeScript SDK source | Wrapped native token | Last reviewed |
| --- | --- | ---: | --- | --- | --- | --- | --- |
| Ethereum Mainnet | `Mainnet` | 1 | `crates/contracts/deployment-provenance.yaml:5` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5075` | `packages/config/src/chains/const/chainIds.ts:21`; `README.md:19` | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` (`crates/core/src/config/chains.rs:11`) | 2026-05-04 |
| BNB Smart Chain | `Bnb` | 56 | `crates/contracts/deployment-provenance.yaml:40` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5083` | `packages/config/src/chains/const/chainIds.ts:27`; `README.md:20` | `0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c` (`crates/core/src/config/chains.rs:25`) | 2026-05-04 |
| Gnosis Chain | `GnosisChain` | 100 | `crates/contracts/deployment-provenance.yaml:75` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5087` | `packages/config/src/chains/const/chainIds.ts:22`; `README.md:21` | `0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d` (`crates/core/src/config/chains.rs:13`) | 2026-05-04 |
| Polygon PoS | `Polygon` | 137 | `crates/contracts/deployment-provenance.yaml:110` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5091` | `packages/config/src/chains/const/chainIds.ts:26`; `README.md:22` | `0x0d500b1d8e8ef31e21c99d1db9a6444d3adf1270` (`crates/core/src/config/chains.rs:21`) | 2026-05-04 |
| Base | `Base` | 8453 | `crates/contracts/deployment-provenance.yaml:145` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5095` | `packages/config/src/chains/const/chainIds.ts:24`; `README.md:23` | `0x4200000000000000000000000000000000000006` (`crates/core/src/config/chains.rs:17`) | 2026-05-04 |
| Plasma | `Plasma` | 9745 | `crates/contracts/deployment-provenance.yaml:180` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5099` | `packages/config/src/chains/const/chainIds.ts:28`; `README.md:24` | `0x6100e367285b01f48d07953803a2d8dca5d19873` (`crates/core/src/config/chains.rs:27`) | 2026-05-04 |
| Arbitrum One | `ArbitrumOne` | 42161 | `crates/contracts/deployment-provenance.yaml:214` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5103` | `packages/config/src/chains/const/chainIds.ts:23`; `README.md:25` | `0x82aF49447D8a07e3bd95BD0d56f35241523fBab1` (`crates/core/src/config/chains.rs:15`) | 2026-05-04 |
| Avalanche C-Chain | `Avalanche` | 43114 | `crates/contracts/deployment-provenance.yaml:249` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5107` | `packages/config/src/chains/const/chainIds.ts:25`; `README.md:26` | `0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7` (`crates/core/src/config/chains.rs:23`) | 2026-05-04 |
| Ink | `Ink` | 57073 | `crates/contracts/deployment-provenance.yaml:284` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5111` | `packages/config/src/chains/const/chainIds.ts:30`; `README.md:27` | `0x4200000000000000000000000000000000000006` (`crates/core/src/config/chains.rs:17`) | 2026-05-04 |
| Linea | `Linea` | 59144 | `crates/contracts/deployment-provenance.yaml:318` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5115` | `packages/config/src/chains/const/chainIds.ts:29`; `README.md:28` | `0xe5d7c2a44ffddf6b295a15c148167daaaf5cf34f` (`crates/core/src/config/chains.rs:29`) | 2026-05-04 |
| Sepolia (Ethereum testnet) | `Sepolia` | 11155111 | `crates/contracts/deployment-provenance.yaml:352` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5119` | `packages/config/src/chains/const/chainIds.ts:31`; `README.md:29` | `0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14` (`crates/core/src/config/chains.rs:19`) | 2026-05-04 |

### Live Confirmation

`registry-confirm --mode release --write` refreshed every row on 2026-04-29. The committed file contains no `skipped` confirmations and no all-zero code-hash sentinels. `registry-confirm --mode release --check` re-ran the same chain set and reported 66 confirmed rows, zero skipped rows, zero failures, and zero diffs.

Selector probes are currently disabled for all rows; the release evidence is the code hash of deployed bytecode returned by `eth_getCode`.

| Selector check state | Rows | Result |
| --- | ---: | --- |
| Disabled | 66 | No selector probe configured; bytecode identity is confirmed by code hash |

## Code-Hash Evidence

| Chain | Env | Settlement code hash | VaultRelayer code hash | EthFlow code hash | Confirmed at |
| --- | --- | --- | --- | --- | --- |
| Mainnet (1) | prod | `0x744d58584e38d214eb190629f131d5cf8b8703bd68e04452f9692177c37c4bc9` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0x875c543737c0bc49033a5c35b0c84ce8f8d40e636e54753eab74093fe1802f68` | 2026-04-29T17:26:25Z |
| Mainnet (1) | staging | `0xb9da0b79eac25fa06600d4f5cdd99ecea6c56c40fec47756323e607a72d9d7bd` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0x7c11bcb168b80a1aecdcf02f59c62833680953da66983e3976ed029fc94a5514` | 2026-04-29T17:26:25Z |
| Gnosis (100) | prod | `0x68963e5b27aadd4ee70ecd933fac9312fe5f527390b88ae6092c68937b80f5e2` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0x900da705b73ba6d9563531dff170bc2ca21f6105114014f54e0fdb88a8e0baae` | 2026-04-29T17:26:25Z |
| Gnosis (100) | staging | `0xe49c7157f6cc80593dfc7c57ac5cbac68cd89f2365f2841b4e368316d07dcce7` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0xe10b269faaef98fabe6d490cfea008fc65cb6e981514770671061bd26b3289d4` | 2026-04-29T17:26:25Z |
| Arbitrum One (42161) | prod | `0xc5d94a317d3c8f717d4238b1a4bee2bc9cec18697c82e1d63865833ebdbd523c` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0x0f2dff362284a851efe1e467f5456ce11e8eb23e74364a17d73c6e4c7b5f32b8` | 2026-04-29T17:26:25Z |
| Arbitrum One (42161) | staging | `0xc7b79b602e144a539497c7f27662bd1c365d0a454bc9ca3dc9ba4f50b29540d4` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0xdfaacd6ea51ac9f3b3d99acea4ed1af536892ed4345db1ccfa1d7513c14cc303` | 2026-04-29T17:26:25Z |
| Base (8453) | prod | `0x851476c2307a7c011d2435d5e5aaae3a41c517f52461b5abefe2a4e42114cbf6` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0xcf69f4ef2e68552be7168f0772f26f9edad8d680f83d6053cec64874cab1a538` | 2026-04-29T17:26:25Z |
| Base (8453) | staging | `0x3f480b4e35da45d472e0d931df9dc492fc5644f1346e02a7c9cc4cf3072820a0` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0x943ab77836221a02a7a94927b7d2ae29d8af605e6f05440f416105e2d5fc190b` | 2026-04-29T17:26:25Z |
| Sepolia (11155111) | prod | `0x9fbace363dc778e25fecb202c12981d916faea80c9aab8167aeeedcaed84df53` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0x775e133373822a18a1ff4530c3391399d8cbc067fa31a7abd67991cfca61afbd` | 2026-04-29T17:26:25Z |
| Sepolia (11155111) | staging | `0x2bd5287a0e8ee6859ac371fac032caf3e193c8785a476913bc017325f83ac2aa` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0x80cc0f6a7ebf4385b1ecdd101b10dd0f8b745650276c3ab139c7e9ab85f0319d` | 2026-04-29T17:26:25Z |
| Polygon (137) | prod | `0xe264ecc678de6464b9365ff73e07858e2c2b07adcd5c8209cb04ebe0e9ef2c14` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0xbed50afd8aed8367eca302bc10348ed26ab860c4aab6b8776b34b4fea283804e` | 2026-04-29T17:26:25Z |
| Polygon (137) | staging | `0x74b2d81d7c3fc31f19781b41803f716f9c752a96b9ed835a68747953a0541324` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0xe21e06906be5e488a73440efdb79bb0a6e6abb1c41d986b83283c69c8c7aad2c` | 2026-04-29T17:26:25Z |
| Avalanche (43114) | prod | `0xc5b8516d7e501ef4c79c135ba4a55b674211f5b2add786f00c89bfd2ad250f5b` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0x72f239a7a05b8b4d9a875eb26fb1d7400a6ad0778861d79a81c91a72e14edbf0` | 2026-04-29T17:26:25Z |
| Avalanche (43114) | staging | `0x864cc1450450e7ecb079c24aa168015279895908ca7cd02834d02d5d9d87c2b9` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0x82262ad5aaa42776aa470273ce09bbddb05296fa9c44b129828742b8d0660286` | 2026-04-29T17:26:25Z |
| BNB (56) | prod | `0x681b39b3355153c3f8ff25d44d73abeca42b51e499d9c54d7354121405a004c4` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0x2eb86ed732e0891f77facc91c96b7a2e51ee4f640498ecb3e36cf84ef37b7923` | 2026-04-29T17:26:25Z |
| BNB (56) | staging | `0x4be734a4feee4d8bf57b59e540101d18e1e9449fc8cafe57fb5288aee29079e6` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0x59a1af5881bb00519f8d758cbe64d291017aff4f244c18de665856a95e0ec5db` | 2026-04-29T17:26:25Z |
| Plasma (9745) | prod | `0x0f867891bddf9798e580617b4e4b04a7cc8010aca4b5240c07eb7823c8b7e16f` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0xd6a42f593076cc49b4bd61e11d4a1053ffaff28df4ac57a1c18b99b8d172c583` | 2026-04-29T17:26:25Z |
| Plasma (9745) | staging | `0xc6faa5ddc6413bff877c947fc8c7897a1b793e650fa47773b221e5e855defd99` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0xccceb467fc2f1bc1f84cacdace6d2372c329ae860e825e8a3e32b0179a615a2d` | 2026-04-29T17:26:25Z |
| Linea (59144) | prod | `0x7462bb78e289d0aeaad6346fd60740a631496d033d2d606ce0d489c340dad3f3` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0xbe858a5a5d014ab3b0c1dfc209bc6e9be31c79d210c6882d7ef186a7457f8765` | 2026-04-29T17:26:25Z |
| Linea (59144) | staging | `0x00824780e6b65d5779cfb7ae062b7cf2287088347e9f71bd0d43f6bed2570208` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0xc1d4b86edeb28de980e8114924eeb4d041f39fd3a07be2e074fc2b65b653ecf8` | 2026-04-29T17:26:25Z |
| Ink (57073) | prod | `0xf00e931b6bf28069ba1c67e93a561e5ffefed30a13ea6b48ca98f39a7803d2a3` | `0x500097799c1379a3728ed70b17de4132de2c07f6937b041c361deaade22b6a5e` | `0x9bc4c4f9de7b6c566a67b32d96d7b36fa3bc6a65bf1c0838530cb78a25fec787` | 2026-04-29T17:26:25Z |
| Ink (57073) | staging | `0xd3fa8e1f9e2fe03e6a85bfd634b6c2c9b80a57b62c04666a86f1a07545f9ab0a` | `0xc310eb15f864d09fc8854b390574d0d9433da110c019d91dd44d1096d83a1f55` | `0xfd715c6df0966a922331daf40fc57e5c4f4d052a4722cae2a1bea79205be9397` | 2026-04-29T17:26:25Z |

## Evidence

Primary implementation points:

- `crates/contracts/src/deployments/registry.rs`
- `crates/core/src/config/chains.rs`
- `crates/contracts/registry.toml`
- `crates/contracts/deployment-provenance.yaml`
- `crates/contracts/build.rs`
- `scripts/validation-smoke/src/registry_confirm.rs`

Primary regression coverage:

- `crates/contracts/tests/registry.rs`
- `crates/contracts/tests/registry.rs::registry_address_lookup_matrix_is_exhaustive`
- `crates/contracts/tests/build_rs_compile_fail.rs`
- `crates/contracts/tests/deployment_provenance_contract.rs`
- `tests/supported_chains_doc_table.rs::supported_networks_doc_table_matches_enum`
- `scripts/validation-smoke/tests/registry_confirm.rs`

Validation surface:

```text
cargo build -p cow-sdk-contracts
cargo test -p cow-sdk-contracts --test deployment_provenance_contract
cargo test -p cow-rs-workspace-tests --test supported_chains_doc_table
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- registry-confirm --mode release --check --chain-ids 1,100,42161,8453,11155111,137,43114,56,9745,59144,57073
bash scripts/check-release-docs-agree.sh
```
