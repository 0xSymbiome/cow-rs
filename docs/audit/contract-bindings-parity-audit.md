# Contract Bindings Parity Audit

Status: Current
Last reviewed: 2026-04-27
Owning surface: `cow-sdk-contracts` `alloy::sol!`-generated bindings for `GPv2Settlement`, `GPv2VaultRelayer`, `CoWSwapEthFlow`, EIP-1967 proxy slots, and `IERC20` / `IERC20Permit`
Refresh trigger: A new binding family landing in `cow-sdk-contracts`; a signature change in any existing binding; a drift in the committed Solidity excerpt under `crates/contracts/abi/**/*.sol`; a change to the TypeScript-SDK-derived parity fixtures that back the regression suite; a change to the EIP-712 domain-separator fixture shared with the signing crate; a change to the wasm target feature contract for the alloy/k256 dependency path
Related docs:
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [Parity Matrix](../parity-matrix.md)
- [Parity Scope](../parity-scope.md)
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the `alloy::sol!`-generated binding surfaces shipped in
  `cow-sdk-contracts`
- the committed Solidity excerpts used to author those bindings
- the byte-identity parity contract between the bindings and the
  TypeScript-SDK-derived fixtures for the encoded call-data and the
  hashed data (order digest, order UID, EIP-712 type hashes)
- the contract-side EIP-712 domain-separator fixture that must stay
  byte-identical with the signing crate's fixture
- the wasm target feature contract that keeps the `alloy-primitives`
  `k256` path buildable under `wasm32-unknown-unknown`
- the five sol! interfaces currently shipped: `IGPv2Settlement`,
  `IGPv2VaultRelayer`, `ICoWSwapEthFlow`, the EIP-1967 storage-slot
  surface, and the `IERC20` / `IERC20Permit` ERC-20 surface

It does not cover deployed-address resolution (Registry authority, a
separate audit) or the HTTP transport that delivers call-data to a
provider.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Single binding idiom | Every shipped binding is generated through `alloy::sol!`; no hand-rolled encoder remains in `cow-sdk-contracts` | Conforms |
| Committed provenance | The Solidity excerpt used to author each binding is committed under `crates/contracts/abi/<family>/` | Conforms |
| Byte-identity parity | Encoded call-data and hashed payloads match the TypeScript-SDK-derived golden fixtures on every binding | Conforms |
| Domain separator parity | `cow-sdk-contracts` and `cow-sdk-signing` pin the same EIP-712 domain-separator fixture value | Conforms |
| WASM compatibility | The `alloy-primitives` `k256` path enables the browser `getrandom` backend for `wasm32-unknown-unknown` builds | Conforms |
| Scope discipline | The shipped set is the five families named above; any new family follows the same provenance and parity contract before it lands | Conforms |

## Current Contract

### Binding Families

`GPv2Settlement` (`crates/contracts/src/settlement.rs`) carries the
`settle`, `invalidateOrder(bytes)`, `setPreSignature`, trade-struct,
and interaction-struct surface against the mainnet-deployed
`0x9008D19f58AAbD9eD0D60971565AA8510560ab41` contract.

`GPv2VaultRelayer` (`crates/contracts/src/vault.rs`) carries the
vault-relayer surface the SDK needs for authorization-role checks.

`CoWSwapEthFlow` (`crates/contracts/src/eth_flow.rs`) carries
`createOrder(EthFlowOrderData)` and `invalidateOrder(EthFlowOrderData)`
against the canonical upstream EthFlow contract. This `invalidateOrder`
variant takes the full `EthFlowOrderData` payload and is distinct from
the `GPv2Settlement::invalidateOrder(bytes)` call that takes a packed
order UID.

The EIP-1967 surface (`crates/contracts/src/proxy.rs`) carries the
`ADMIN_SLOT` and `IMPLEMENTATION_SLOT` storage-slot helpers.

The ERC-20 surface (`crates/contracts/src/erc20.rs`) carries `IERC20`
and `IERC20Permit` (EIP-2612) for the subset of methods the SDK emits
against any ERC-20 token, including the EIP-2612 `permit` domain
separator type hash.

### Provenance

Every binding is introduced by a `sol! { ... }` block that reproduces
the upstream Solidity surface verbatim. The excerpt used to author the
binding is committed under `crates/contracts/abi/<family>/*.sol` so a
reviewer can diff `HEAD` against the upstream source at any time. The
upstream repositories are named in each binding's module-level doc
comment.

### Byte-Identity Parity

Each binding has a regression contract that encodes a known input and
asserts the output matches a TypeScript-SDK-derived fixture bit for
bit. The same contract covers:

- EIP-712 domain separators (chain-id and verifying-contract swept)
- Order hash, UID, and signing-scheme payload bytes
- Settlement call-data for multi-trade batches
- Encoded trade flags (kind, partial fill, balance source, balance
  destination, signing scheme)

`crates/contracts/tests/parity_contract.rs` is the hub test harness for
the byte-identity contract; per-family tests extend it for surfaces
that need additional fixtures.

The EIP-712 domain-separator path additionally carries a compact JSON fixture
under both `crates/contracts/tests/fixtures/` and
`crates/signing/tests/fixtures/`. The contracts test and the signing test read
the same expected separator so a future change to typed-data domain encoding
cannot silently move one crate without moving the other.

### WASM Target Contract

`crates/contracts/Cargo.toml` keeps the `alloy-primitives` `k256` path
compatible with browser-target builds by enabling the `getrandom` `js`
backend only for `wasm32`. This is a build-contract detail, not a public API
dependency: callers still interact with the same contract DTOs and hashing
helpers on native and wasm targets.

### Scope Discipline

Only the five binding families listed above are in scope for this
audit. Third-party protocol bindings (Aave, bridging adapters,
condition schedulers) stay in their own capability crates and carry
their own parity contracts when they land. Hand-rolled encoder helpers
are not allowed in `cow-sdk-contracts`.

## Evidence

Primary implementation points:

- `crates/contracts/src/settlement.rs`
- `crates/contracts/src/vault.rs`
- `crates/contracts/src/eth_flow.rs`
- `crates/contracts/src/proxy.rs`
- `crates/contracts/src/erc20.rs`
- `crates/contracts/src/primitives.rs`
- `crates/contracts/Cargo.toml`
- `crates/contracts/abi/settlement/`
- `crates/contracts/abi/vault-relayer/`
- `crates/contracts/abi/eth-flow/`
- `crates/contracts/abi/eip1967/`
- `crates/contracts/abi/erc20/`
- `crates/contracts/tests/fixtures/domain_separator_parity.json`
- `crates/signing/tests/fixtures/domain_separator_parity.json`

Primary regression coverage:

- `crates/contracts/tests/parity_contract.rs`
- `crates/contracts/src/primitives.rs::tests::domain_separator_matches_shared_parity_fixture`
- `crates/signing/src/domain.rs::tests::domain_separator_matches_shared_parity_fixture`

Validation surface:

```text
cargo test -p cow-sdk-contracts --all-features
cargo test -p cow-sdk-contracts domain_separator_matches_shared_parity_fixture
cargo test -p cow-sdk-signing domain_separator_matches_shared_parity_fixture
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo clippy -p cow-sdk-contracts --all-targets --all-features -- -D warnings
```
