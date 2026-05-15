# COW Shed Contract Bindings Audit

Status: Current
Last reviewed: 2026-05-15
Owning surface: COW Shed Solidity excerpts, proxy creation-code artifacts, version-call evidence, and deployment registry rows
Refresh trigger: Refresh when COW Shed deployments, proxy creation code, factory ABIs, hook type strings, or the deployed `VERSION()` return value change upstream.
Related docs:
- [ADR 0049](../adr/0049-cow-shed-account-abstraction-proxy.md)
- [ADR 0050](../adr/0050-eip1271-signature-blob-encoding.md)
- [ADR 0051](../adr/0051-signing-owned-eip1271-signature-provider-trait.md)
- [COW Shed App-Data Integration Audit](cow-shed-app-data-integration-audit.md)

## Scope

This audit covers:

- the vendored COW Shed Solidity excerpts that anchor the SDK's typed
  bindings;
- the per-version proxy creation-code artifacts and SHA-256 digest
  neighbors;
- the per-chain `VERSION()` call evidence captured in
  `crates/contracts/abi/cow-shed/version-call-results.json`;
- the schema v2 deployment registry rows for the COW Shed factory and
  implementation contracts;
- the Gnosis-only `COWShedForComposableCoW` forwarder gate that enforces
  chain id 100 for the bridge variant;
- the EIP-712 type strings used by the hook structure, including the
  whitespace-free declaration order and the EOA signature byte order
  `r || s || v`.

It does not cover the COW Shed hook metadata schema integration with the
app-data crate; that boundary is governed by the
[COW Shed App-Data Integration Audit](cow-shed-app-data-integration-audit.md).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Solidity excerpts | The vendored COW Shed Solidity excerpts compile under `alloy::sol!` and emit type strings byte-identical to the upstream sources, including no whitespace between commas | Conforms |
| Proxy creation-code | `v1.0.0.bin` and `v1.0.1.bin` artifacts ship with adjacent `.sha256` digest neighbors validated by `crates/contracts/build.rs` | Conforms |
| Version-call evidence | Every per-chain row in `version-call-results.json` records `decoded_version == "1.0.1"` and `expected_sdk_version == "CowShedVersion::V1_0_1"` | Conforms |
| Deployment registry | COW Shed factory and implementation rows are present in `registry.toml` for every supported chain id; `COWShedForComposableCoW` is present only for chain id 100 | Conforms |
| Gnosis forwarder gate | The Gnosis-only forwarder is reachable only when the caller selects chain id 100; all other chains return the typed `CowShedError::COWShedForComposableCoWGnosisOnly { chain }` variant | Conforms (contract; helper body lands in a later capability landing) |
| Hook type strings | Canonical type strings carry no whitespace between commas in declaration order; the EOA signature byte order is `r || s || v` | Conforms |

## Current Contract

### Solidity excerpts

The vendored COW Shed Solidity excerpts live under
`crates/contracts/abi/cow-shed/`. The set covers `COWShed.sol`,
`COWShedFactory.sol`, `COWShedForComposableCoW.sol`, `COWShedProxy.sol`,
`COWShedStorage.sol`, `ERC1271Forwarder.sol`, the two interface modules
under `interfaces/`, `LibAuthenticatedHooks.sol`, `LibCowOrder.sol`, and
`PreSignStateStorage.sol`. The EIP-712 type strings inside these excerpts
carry no whitespace between commas in declaration order; any future
amendment that adds whitespace is a regression of the byte-identity
contract.

### Proxy creation-code

Per-version proxy creation-code artifacts ship at
`crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.0.bin` and
`v1.0.1.bin` with adjacent `.sha256` digest neighbors. The build script
`crates/contracts/build.rs` reads each `.bin` file, computes SHA-256, and
compares to the digest neighbor; a mismatch fails the build. The init-code
hash used at CREATE2 derivation time is computed per call as
`keccak256(abi.encodePacked(PROXY_CREATION_CODE, abi.encode(implementation, who)))`;
the `.bin` files store the deployer bytecode prefix and never the full
init code, so derivation works correctly for any user address.

### Version-call evidence

The per-chain `VERSION()` call evidence at
`crates/contracts/abi/cow-shed/version-call-results.json` records the
deployed implementation address, the factory address, and the decoded
version string per chain id. Every row records
`decoded_version == "1.0.1"` and `expected_sdk_version ==
"CowShedVersion::V1_0_1"`, anchoring the SDK's default version to deployed
reality.

### Gnosis forwarder gate

The `COWShedForComposableCoW` contract is deployed only on Gnosis Chain
(chain id 100). The forwarder gate is anchored by the typed
`CowShedError::COWShedForComposableCoWGnosisOnly { chain }` variant; any
constructor or interaction helper that targets the forwarder on a
non-Gnosis chain id must return this variant. The ENS-related helpers gate
behind the `cow-shed-ens` Cargo feature (default off) so non-Gnosis builds
do not pull in the ENS resolver surface.

### Hook type strings

The canonical EIP-712 type strings are
`Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`
and
`ExecuteHooks(Call[] calls,bytes32 nonce,uint256 deadline)Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`.
The EOA signature byte order is `r || s || v` (not the standard
`v || r || s`); the signature field is a fixed-length 65-byte array in
that order, enforced by a compile-fail fixture in a later capability
landing. The `isDelegateCall = true` setting is opt-in only via an
explicit builder method that requires a `// SAFETY:` comment in the
preceding three lines of the call site.

## Evidence

Primary implementation points:

- `crates/contracts/abi/cow-shed/`
- `crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.0.bin`
- `crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.0.bin.sha256`
- `crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.1.bin`
- `crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.1.bin.sha256`
- `crates/contracts/abi/cow-shed/version-call-results.json`
- `crates/contracts/registry.toml`
- `crates/contracts/build.rs` (`validate_cow_shed_proxy_artifacts`)
- `parity/cow-shed-invariants.md`
- `parity/fixtures/cow_shed/`

Primary regression coverage:

- `crates/contracts/tests/schema_v2_success.rs`
- `crates/contracts/tests/schema_v2_rejection.rs`
- `crates/contracts/tests/trybuild_schema_v2.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --all-features
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate-fixture-catalog --root .
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml
```
