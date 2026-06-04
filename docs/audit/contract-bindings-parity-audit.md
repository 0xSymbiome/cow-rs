# Contract Bindings Parity Audit

Status: Current
Last reviewed: 2026-05-30
Owning surface: `cow-sdk-contracts` `alloy::sol!`-generated bindings for `GPv2Settlement`, `GPv2VaultRelayer`, `CoWSwapEthFlow`, `CoWSwapOnchainOrders` events, the wrapped-native token, EIP-1967 proxy slots, and `IERC20` / `IERC20Permit`
Refresh trigger: A new binding family landing in `cow-sdk-contracts`; a signature change in any existing binding; a drift in the byte-identical Solidity mirror under `crates/contracts/abi/**/*.sol`; a change to a `vendored:` SHA-256 row under any repository in `parity/source-lock.yaml`; a change to the TypeScript-SDK-derived parity fixtures that back the regression suite; a change to the EIP-712 domain-separator fixture shared with the signing crate; a change to the wasm target feature contract for the alloy/k256 dependency path
Related docs:
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0034](../adr/0034-interaction-encoder-target-policy.md)
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [Parity Matrix](../parity.md)
- [Parity Scope](../parity.md)
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the `alloy::sol!`-generated binding surfaces shipped in
  `cow-sdk-contracts`
- the byte-identical Solidity mirrors used to author those bindings
- the byte-identity parity contract between the bindings and the
  TypeScript-SDK-derived fixtures for the encoded call-data and the
  hashed data (order digest, order UID, EIP-712 type hashes)
- the contract-side EIP-712 domain-separator fixture that must stay
  byte-identical with the signing crate's fixture
- the wasm target feature contract that keeps the `alloy-primitives`
  `k256` path buildable under `wasm32-unknown-unknown`
- the seven sol! interface families currently shipped: `IGPv2Settlement`,
  `IGPv2VaultRelayer`, `ICoWSwapEthFlow`, the `ICoWSwapOnchainOrders` event
  surface, the `IWrappedNativeToken` (WETH9-family) surface, the EIP-1967
  storage-slot surface, and the `IERC20` / `IERC20Permit` ERC-20 surface

It does not cover deployed-address resolution (Registry authority, a
separate audit) or the HTTP transport that delivers call-data to a
provider.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Single binding idiom | Every shipped binding is generated through `alloy::sol!`; no hand-rolled encoder remains in `cow-sdk-contracts` | Conforms |
| Committed provenance | Every Solidity file under `crates/contracts/abi/` is a byte-identical mirror of a single upstream source pinned in `parity/source-lock.yaml`; `cargo parity-verify-sol-provenance` enforces SHA-256 equality against the manifest `vendored:` row and (when run with `--upstream-root`) against the live upstream bytes at the pinned commit. All 40 shipped `.sol` files follow this posture; no excerpt-style or documentation-only file ships in the workspace | Conforms |
| Byte-identity parity | Encoded call-data and hashed payloads match the TypeScript-SDK-derived golden fixtures on every binding | Conforms |
| Domain separator parity | `cow-sdk-contracts` and `cow-sdk-signing` route every EIP-712 domain separator through `alloy_sol_types::Eip712Domain::separator` and pin the same fixture value | Conforms |
| Order EIP-712 hashing | The `GPv2 Order` and `OrderCancellations` typed-data structs are macro-emitted via `alloy_sol_types::sol!` and route their signing hashes through `<T as SolStruct>::eip712_signing_hash`; the eight per-chain rows in the order-digest fixture pin the wire-byte contract | Conforms |
| EIP-1271 payload encoding | The COW EIP-1271 verifier payload `abi.encode(GPv2Order.Data, bytes)` is composed from the macro-emitted `OnchainOrder` sol struct and the raw ECDSA signature via `alloy_sol_types::SolValue::abi_encode_sequence`; the inline regression contract reproduces the canonical 12-word order tuple plus dynamic-bytes tail layout byte-for-byte | Conforms |
| Boundary matrices | Compact order flags, settlement reader returns, settlement encoder stages, mixed-balance transfers, and multi-trade clearing prices have deterministic regression coverage | Conforms |
| EIP-1967 derivation | Proxy storage slots match the canonical `keccak256(label) - 1` formula as well as the golden byte payloads | Conforms |
| Vault role hash parity | Vault-relayer role helpers emit the same packed role hashes as the upstream TypeScript role-grant helpers | Conforms |
| WASM compatibility | The `alloy-primitives` `k256` path enables the browser `getrandom` backend for `wasm32-unknown-unknown` builds | Conforms |
| Scope discipline | The shipped set is the seven families named above; any new family follows the same provenance and parity contract before it lands | Conforms |

## Current Contract

### Binding Families

`GPv2Settlement` (`crates/contracts/src/settlement/mod.rs`) carries the
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

`EthFlowOrderData::new` and `EthFlowOrderData::from_unsigned_order` return
`Result<Self, ContractsError>`, rejecting `Address::ZERO` for the
receiver field with `ContractsError::ZeroReceiver`. The rejection mirrors
the upstream `EthFlowOrder.toCoWSwapOrder` library function's
`ReceiverMustBeSet()` revert (selector `0xefc9ccdf`), which fires on both
the `createOrder` and `invalidateOrder` write paths through the shared
library call. The rule lives in the private `reject_zero_receiver`
helper invoked by the `EthFlowOrderData` construction paths; the general
order hash path treats `address(0)` as the protocol's pay-to-owner
sentinel and hashes it verbatim rather than rejecting it. The unit test
`zero_receiver_invariant_matches_ethflow_on_chain_revert_selector` in
`crates/contracts/src/eth_flow.rs` re-derives the selector via
`alloy_primitives::keccak256("ReceiverMustBeSet()")[..4]` and pins it
against any future upstream rename, and the proptest
`ethflow_order_data_new_rejects_zero_receiver_iff_address_is_zero` in
`crates/contracts/tests/property_contract.rs` covers the bidirectional
invariant under the full 2^160 address space.

The EIP-1967 surface (`crates/contracts/src/proxy.rs`) carries the
`ADMIN_SLOT` and `IMPLEMENTATION_SLOT` storage-slot helpers.
The regression suite verifies both the canonical hex payloads and the
formula-derived values from `keccak256("eip1967.proxy.<label>") - 1`.

The ERC-20 surface (`crates/contracts/src/erc20.rs`) carries `IERC20`
and `IERC20Permit` (EIP-2612) for the subset of methods the SDK emits
against any ERC-20 token, including the EIP-2612 `permit` domain
separator type hash.

`CoWSwapOnchainOrders` (`crates/contracts/src/onchain_orders.rs`) carries the
`OrderPlacement` and `OrderInvalidation` event bindings used by on-chain order
routers such as eth-flow. The topic-0 signature hashes are byte-locked against
an independent keccak of the flattened-tuple signatures, and the fail-closed
decoder reconstructs the broadcast `GPv2` order, resolves the owner from the
on-chain signature, and derives the 56-byte order UID through
`compute_order_uid`. The decoding contract is governed by
[ADR 0054](../adr/0054-onchain-order-event-decoding-is-fail-closed.md) and the
[On-Chain Order Log Decoding Audit](onchain-order-log-decoding-audit.md).

The `IWrappedNativeToken` surface (`crates/contracts/src/weth.rs`) carries the
WETH9-family `deposit` / `withdraw` methods, with `wrap_interaction` and
`unwrap_interaction` helpers that emit the canonical settlement interaction for
converting between the native asset and its wrapped form. The 4-byte selectors
are byte-locked against an independent keccak of the canonical signatures.

### Provenance

Every binding is introduced by a `sol! { ... }` block that reproduces
the upstream Solidity surface verbatim. The byte-identical Solidity
mirror used to author the binding is committed under
`crates/contracts/abi/<family>/*.sol` so a reviewer can diff `HEAD`
against the upstream source at any time. The upstream repositories are
named in each binding's module-level doc comment and pinned in
`parity/source-lock.yaml`.

Every `.sol` file under `crates/contracts/abi/` is a byte-identical
mirror of a single upstream source pinned in `parity/source-lock.yaml`,
and `cargo parity-verify-sol-provenance` enforces the discipline before
any workspace build is considered green. The local path, the upstream
path under the repository root, and the SHA-256 of the upstream bytes
at the pinned commit live as a `vendored:` row under the matching
repository. The verifier rejects any drift between the on-disk SHA and
the manifest SHA, and (when run with `--upstream-root <path>`) any
drift between the manifest SHA and the live upstream bytes at the
pinned commit. All 40 shipped `.sol` files follow this posture; no
excerpt-style or documentation-only file ships in the workspace.

The 40 byte-identical mirrors are sourced from four upstream repositories,
each pinned in `parity/source-lock.yaml`:

- `cowprotocol/contracts` — `settlement/GPv2Settlement.sol`,
  `settlement/GPv2Trade.sol`, `settlement/GPv2Interaction.sol`,
  `vault-relayer/GPv2VaultRelayer.sol`, `eip1967/GPv2EIP1967.sol`,
  `erc20/IERC20.sol`.
- `cowprotocol/ethflowcontract` — `eth-flow/CoWSwapEthFlow.sol`,
  `eth-flow/EthFlowOrder.sol`.
- `cowprotocol/composable-cow` — every file under
  `crates/contracts/abi/composable-cow/` including the Safe Global
  `extensible/ExtensibleFallbackHandler.sol` mirror, which is reached
  transitively through composable-cow's `lib/safe` submodule SHA
  captured by composable-cow's pinned commit.
- `cowdao-grants/cow-shed` — every file under
  `crates/contracts/abi/cow-shed/`.

Each `.sol` is LF-normalised on every host through `.gitattributes`
so the SHA gate stays byte-stable across Windows, macOS, and Linux
checkouts. The verifier ships as a subcommand of the
`parity-maintainer` binary and is wired into the CI quality gate so
the workspace cannot ship with an unverified `.sol` file under
`crates/contracts/abi/`. The verifier's source code retains a fallback
excerpt code path as an escape hatch for future contracts whose
canonical upstream might not be a single vendorable file, but no
currently-shipped file uses that path; a reviewer's audit of the abi
tree is `sha256sum` on every file against the matching `vendored:`
row in `parity/source-lock.yaml`.

#### How a `vendored:` SHA is generated and verified

Every `sha256` field in `parity/source-lock.yaml` is the SHA-256 of the
bytes git stores for the upstream file at the pinned commit, not the
SHA of the working-tree file after checkout. The canonical generation
command for any row is:

```
git -C <upstream-checkout> show <pinned-commit>:<upstream-path> | sha256sum
```

For example, the `settlement/GPv2Settlement.sol` row is verified by:

```
git -C <contracts-checkout> show \
    c6b61ce75841ce4c25ab126def9cc981c568e6c6:src/contracts/GPv2Settlement.sol \
    | sha256sum
```

This anchors the SHA to git's content-addressable storage at the
pinned commit and eliminates three working-tree hazards that affect
plain `sha256sum`:

1. CRLF line-ending normalisation at checkout time (e.g. Windows
   without `text eol=lf` would convert LF→CRLF and produce a different
   SHA than the canonical git-tree bytes).
2. Local working-tree edits that do not appear in `git status` (e.g.
   filter drivers or merge conflict residue).
3. Checkout at a different commit than the pinned one (the working
   tree could be at `HEAD` while the pin references an older or newer
   commit; the on-disk bytes would not match the pinned commit's
   content).

`cargo parity-verify-sol-provenance --upstream-root <path>` performs
the same `git show <commit>:<path>` read against each upstream
checkout under `<path>/<repo-id>/`, computes SHA-256 of the resulting
bytes, and compares against the `sha256` field. If the pinned commit
is not present in the local checkout, the verifier emits an error
naming the exact `git fetch origin <commit>` invocation required.

A third mode, `cargo parity-verify-sol-provenance --upstream-github`,
fetches each `vendored:` row from
`https://raw.githubusercontent.com/<owner>/<repo>/<commit>/<upstream-path>`
(parsed from the row's `remote:` field) and compares the bytes against
the manifest SHA-256. This mode is the strongest available trust
posture for the gate: it verifies that the manifest is byte-identical
to GitHub's canonical content at the pinned commit on every gate run,
without requiring any local upstream checkout. The CI quality-gate
workflow runs this mode on every push so the manifest cannot silently
drift from upstream GitHub content. The two upstream modes are
additive: passing both `--upstream-root <path>` and `--upstream-github`
runs all three checks (on-disk vs manifest, local `git show` vs
manifest, GitHub raw vs manifest) and requires every check to agree.

The Safe Global `ExtensibleFallbackHandler` is reached transitively
through composable-cow's `lib/safe` git submodule. The submodule's
pinned commit (`11273c1f08eda18ed8ff49ec1d4abec5e451ff21`) is captured
under its own `composable-cow/lib/safe` repository row in
`parity/source-lock.yaml`, with `remote:` pointing at
`https://github.com/cowdao-grants/extensible-fallback-handler.git`.
Verifying that row requires running `git submodule update --init lib/safe`
inside the composable-cow checkout so the submodule's own `.git`
directory carries the pinned commit; the verifier then performs the
same `git show <pinned-commit>:<upstream-path>` read against the
submodule's git directory.

### Byte-Identity Parity

Each binding has a regression contract that encodes a known input and
asserts the output matches a TypeScript-SDK-derived fixture bit for
bit. The same contract covers:

- EIP-712 domain separators (chain-id and verifying-contract swept)
- Order hash, UID, and signing-scheme payload bytes
- Compact order flag decoding across every supported kind/source/destination
  combination
- Settlement call-data for multi-trade batches
- Settlement reader `filledAmountsForOrders` typed return decoding
- Settlement encoder PRE, INTRA, POST interaction ordering
- Vault relayer mixed ERC-20, external, and internal balance transfer batches
- Multi-trade settlement clearing-price ordering
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

The `cow_sdk_contracts::primitives::domain_separator` and
`cow_sdk_contracts::primitives::typed_data_digest` helpers delegate to
`alloy_sol_types::Eip712Domain::separator` for the domain preimage and
to `alloy_primitives::keccak256` for the canonical `0x19 0x01 ||
separator || struct_hash` envelope. The shared parity fixture locks the
byte contract; an inline regression test in `primitives.rs` reproduces
the EIP-712 encoding from first principles and asserts the helper
output matches at the byte level, so the alloy delegation can never
silently drift from the protocol-specified formula.

`cow_sdk_signing::domain::domain_separator_for` and the chain-aware
`cow_sdk_signing::domain::domain_separator` wrapper route through the
same `alloy_sol_types::Eip712Domain::separator` primitive. The signing
helper owns the chain-id and protocol-options resolution (settlement
contract lookup through `cow_sdk_contracts::Registry`) and formats the
32-byte separator as the lowercase 0x-prefixed hex string that the
signer-facing API exposes; the EIP-712 algorithm itself is the alloy
canonical, so the contracts-side and signing-side fixture cases gate
the same byte contract from both crate boundaries.

The `GPv2` order and batch-cancellation EIP-712 schemas are
macro-emitted via `alloy_sol_types::sol!` at
`crates/contracts/src/order/sol_types.rs` (`Order`) and
`crates/contracts/src/order/sol_cancellations.rs`
(`OrderCancellations`). The order struct is crate-internal codec
machinery — order hashing flows through `hash_order` and the canonical
type hash is exposed as `cow_sdk_contracts::order_eip712_type_hash()` —
while the cancellation struct is re-exported at the crate root as
`cow_sdk_contracts::GPv2OrderCancellations`. The macro emits the
canonical EIP-712 type strings at expansion time:
`Order(address sellToken,address buyToken,address receiver,uint256
sellAmount,uint256 buyAmount,uint32 validTo,bytes32 appData,uint256
feeAmount,string kind,bool partiallyFillable,string sellTokenBalance,
string buyTokenBalance)` keccak-hashes to the deployed protocol
constant
`0xd5a25ba2e97094ad7d83dc28a6572da797d6b3e7fc6663bd93efb789fc17e489`
and `OrderCancellations(bytes[] orderUids)` keccak-hashes to the
canonical batch-cancellation type hash. Callers route order signing
hashes through `<Order as SolStruct>::eip712_signing_hash` on the
crate-internal codec struct and batch-cancellation signing hashes through
`<GPv2OrderCancellations as SolStruct>::eip712_signing_hash`; the
public functions `cow_sdk_contracts::hash_order`,
`cow_sdk_contracts::hash_order_cancellation`, and
`cow_sdk_contracts::hash_order_cancellations` are thin wrappers over
that alloy path. The eight representative rows in
`parity/fixtures/eip712/order_digests.json` (vanilla mainnet sell and
buy, gnosis chain native-in, sepolia partial fill, arbitrum one
eth-flow, base partner-fee, mainnet zero-app-data edge, and mainnet
max-amount U256 edge) pin per-row domain separator, struct hash, and
signing hash so a future change to the order typed-data encoding
cannot silently move the wire bytes.

The COW EIP-1271 verifier expects `abi.encode(GPv2Order.Data, bytes)`
as the signature payload. The on-chain `GPv2Order.Data` representation
stores `kind`, `sellTokenBalance`, and `buyTokenBalance` as `bytes32`
holding the keccak256 of the canonical label string (matching the
deployed settlement contract's storage layout), so it is a different
schema from the EIP-712 typed-data `Order` even though both describe
the same protocol order. The on-chain schema is macro-emitted via
`alloy_sol_types::sol!` at
`crates/signing/src/eip1271/sol_types.rs` as `OnchainOrder`; the
verifier payload is the Rust tuple alias
`cow_sdk_signing::OrderAndSignature = (OnchainOrder, Bytes)`.
`cow_sdk_signing::eip1271_signature_payload` composes the payload
field-by-field, hashes the on-chain label fields with
`alloy_primitives::keccak256`, and encodes the tuple via
`alloy_sol_types::SolValue::abi_encode_sequence` to produce the
canonical head-and-dynamic-tail wire layout (twelve 32-byte order
words, then the offset, length, and padded signature bytes). The
inline regression contract in
`crates/signing/tests/order_signing_contract.rs` reproduces the
expected byte layout by hand and pins both the full payload and the
per-word offsets at `signature` length 65, so any drift in the wire
layout fails the contract.

Deterministic CREATE2 addresses for the deployer-derived contracts in
`cow_sdk_contracts::deploy` route through
`alloy_primitives::Address::create2_from_code`, which assembles the
canonical EIP-1014 preimage (`0xff || deployer || salt ||
keccak256(init_code)`) and hashes it internally. The inline regression
tests in `deploy.rs` reconstruct the EIP-1014 formula by hand and
assert byte-identity against the alloy delegation, so any silent
divergence between the maintained primitive and the
shipped CREATE2 salt + deployer constants is caught at test time.

The `cow-sdk-trading` on-chain transaction helpers build the
`setPreSignature(bytes,bool)` and `invalidateOrder(bytes)` settlement
calldata by composing `IGPv2Settlement::setPreSignatureCall` and
`IGPv2Settlement::invalidateOrderCall` and routing the encoding through
`<C as alloy_sol_types::SolCall>::abi_encode`, the same canonical path
the `cow-sdk-contracts` parity contract gates. No hand-rolled selector,
dynamic-bytes offset, or word-padding helpers remain in the trading
crate for these two calls; the trading layer consumes the
`IGPv2Settlement` sol! bindings cross-crate and inherits the
byte-identity contract automatically. The pinned fixture rows
`contracts-settlement-set-presignature-calldata` and
`contracts-settlement-invalidate-order-calldata` in
`parity/fixtures/contracts.json` lock the wire bytes for both calls, so
any drift in the upstream sol! emitter surfaces in the contracts-side
regression before it can reach the trading-side transaction builder.
The `EthFlowTransaction` create and invalidate helpers continue to
route through `cow_sdk_contracts::eth_flow::encode_create_order_calldata`
and `encode_invalidate_order_calldata`, which themselves call
`ICoWSwapEthFlow::createOrderCall.abi_encode` and
`ICoWSwapEthFlow::invalidateOrderCall.abi_encode` inside the contracts
crate, so every settlement-bound and EthFlow-bound calldata the trading
public surface emits is now produced by an `alloy::sol!`-generated
encoder.

### WASM Target Contract

`crates/contracts/Cargo.toml` keeps the `alloy-primitives` `k256` path
compatible with browser-target builds by enabling the `getrandom` `js`
backend only for `wasm32`. This is a build-contract detail, not a public API
dependency: callers still interact with the same contract DTOs and hashing
helpers on native and wasm targets.

### Scope Discipline

Only the seven binding families listed above are in scope for this
audit. Third-party protocol bindings (Aave, bridging adapters,
condition schedulers) stay in their own capability crates and carry
their own parity contracts when they land. Hand-rolled encoder helpers
are not allowed in `cow-sdk-contracts`.

### Interaction Encoder

Settlement interaction encoding is the reviewed boundary for translating
typed interaction data into contract calldata. `normalize_interaction` remains
infallible and value-neutral: missing value defaults to zero and missing
calldata defaults to an empty payload.

`SettlementEncoder::encode_interaction` is fallible. When the encoder's
typed-data domain resolves through `Registry::default()` to exactly one
canonical settlement for the domain chain id and verifying contract, the
encoder rejects an interaction whose target is the paired vault relayer for the
same chain and environment with
`ContractsError::ForbiddenInteractionTarget`. Unknown or custom settlement
domains pass through neutrally and leave final target authority to the
settlement contract runtime. `PROP-CON-011` records the invariant.

### Vault Relayer Role Hash Parity

Vault-relayer role hash helpers are part of the reviewed binding parity
surface because callers use the emitted role identifiers in Balancer
Authorizer grant calls. The helpers derive each role with the same packed
formula as the upstream TypeScript role-grant helpers:
`solidityKeccak256(["uint256","bytes4"], [vaultAddress, selector])`.

The Rust helper pads the 20-byte Vault address to the `uint256` width,
appends the 4-byte method selector, and hashes the resulting 36-byte payload.
`PROP-CON-010` records the invariant, and fixture
`contracts-vault-role-hashes-match-upstream-typescript` pins the canonical
Mainnet Vault role hashes for `manageUserBalance` and `batchSwap`.

### Wire Serde

The DTO fields that carry hex-encoded byte payloads on the JSON wire route
through `alloy_primitives::Bytes`, whose native `Serialize` / `Deserialize`
impl emits and parses the canonical `0x`-prefixed lowercase hexadecimal
string the protocol's TypeScript SDK consumes. The migrated fields are
`Interaction.call_data` and `InteractionLike.call_data` in
`crates/contracts/src/interaction.rs`, and `BatchSwapStep.user_data` and
`Swap.user_data` in `crates/contracts/src/swap.rs`. No bespoke `#[serde(with =
"...")]` adapter is interposed on the `Bytes`-typed fields; the alloy
primitive owns the canonical wire form. The `cow-sdk-contracts` parity
fixtures that exercise these fields (settlement calldata stages, batch-swap
user data, and the interaction encoder stage matrices) stay green
byte-identically across the migration, so the typed value contract and the
wire byte contract remain locked together.

Two related cross-workspace wire-serde surfaces follow the same
alloy-canonical pattern and are referenced here because their byte
contracts share the protocol's TypeScript-SDK-derived fixture authority.
`cow_sdk_app_data::metadata::Hook.gas_limit` carries the protocol's
decimal-string `gasLimit` envelope through `#[serde(with =
"alloy_serde::displayfromstr")]`, which serializes any `Display + FromStr`
type into the same JSON-string-of-decimal-digits the hooks fixture
`parity/fixtures/app_data/hooks_v1.14.0.json` pins. The
`cow-sdk-browser-wallet` provider helpers
`provider::async_provider::hex_quantity` and `parse_chain_id_value` parse
the EIP-1474 hex-quantity wire form through
`alloy_primitives::U256::from_str_radix` and format the canonical
`0x`-prefixed lowercase hex via the U256 `LowerHex` impl, replacing the
previous hand-rolled `BigUint` parser path with the canonical alloy
primitive.

### Identity Primitive Newtypes

The cow identity primitives collapse onto strict `#[repr(transparent)]`
newtypes over the canonical `alloy_primitives` byte and integer types.
`cow_sdk_core::Address` wraps `alloy_primitives::Address`; `Hash32`,
`OrderDigest`, `BlockHash`, and `AppDataHash` wrap
`alloy_primitives::B256`; `HexData` wraps `alloy_primitives::Bytes`;
`OrderUid` wraps `alloy_primitives::FixedBytes<56>`; `Amount` wraps
`alloy_primitives::U256`; `SignedAmount` wraps
`alloy_primitives::I256`. The cached `{ inner, hex }` struct layout from
the historical parity revision is retired across every primitive in the
family, along with the `identity_ext` extension trait module, the
`cow_sdk_core::types::hex` encoder helpers, and the previous
`AppDataHash::{ inner: B256, hex: String }` half-state.

Construction stays through the existing `new(&str) -> Result<Self, _>`
factories; the strict newtypes parse once at construction and reject
malformed input with the same `cow_sdk_core::ValidationError` /
`CoreError` variants the previous layout emitted. Display, Serialize,
and Deserialize impls are cow-owned on `Address` (lowercase 0x-prefixed
canonical, matching the deployed protocol convention), `Amount`
(canonical base-10 decimal string, strict-decimal-fail-closed at the
serde boundary so radix-prefixed `0x`, `0o`, or `0b` inputs the alloy
`ruint::Uint::FromStr` impl would otherwise silently accept are
rejected through deserialization), and `SignedAmount` (canonical
signed-decimal string with optional leading minus, same strict-decimal
serde boundary). The remaining byte-typed primitives (`Hash32`,
`OrderDigest`, `BlockHash`, `AppDataHash`, `HexData`, `OrderUid`)
forward Serialize / Deserialize to the inner alloy primitive via
`#[serde(transparent)]` because the alloy lowercase 0x-prefixed default
already matches the cow wire form. The inherent stdlib-style accessor
is renamed `as_str() -> &str` to `to_hex_string() -> String` so callers
receive an owned string that honors the canonical lowercase encoding
contract without depending on internal caching. The new
`write_into(&self, f: &mut impl core::fmt::Write) -> core::fmt::Result`
accessor provides a zero-allocation path for the hot tracing and JSON
emission seams that previously borrowed the cached hex string. The
internal `pub` tuple-struct field carries a rustdoc-documented
escape-hatch caveat: it is reachable for advanced callers but is
explicitly not part of the API stability contract, and the safe
accessors (`as_alloy` / `as_u256` / `as_i256`,
`into_alloy` / `into_u256` / `into_i256`, `to_hex_string`,
`write_into`, `as_slice`) cover every supported workflow.

Equality, hash, and ordering on the strict newtypes collapse onto the
underlying alloy byte comparison, which is equivalent to the previous
case-insensitive contract because every valid input parses to the same
bytes regardless of input casing. The seam helpers in
`cow_sdk_alloy_provider`, `cow_sdk_alloy`, and `cow_sdk_browser_wallet`
consume the packed bytes directly through `*value.as_alloy()` and
`value.into_alloy()`, replacing the previous `cow_to_alloy_address` /
`cow_to_alloy_hash` / `alloy_address_to_cow_address` /
`hex_data_from_bytes` / `decode_0x_hex` /
`parse_u256_quantity` adapter helpers, which are removed. The
`parse_u256` JSON-Value adapters that historically lived in each of
`cow-sdk-alloy`, `cow-sdk-alloy-provider`, and `cow-sdk-browser-wallet`
now delegate to `alloy_primitives::U256::from_str`, which natively
recognises both the canonical decimal and `0x`-prefixed hex forms used
by the JSON-RPC `eth_call` response shape and enforces the `uint256`
ceiling at parse time, so the historical hand-rolled radix sniffer and
the BigUint fallback path in the browser-wallet copy are retired and
the `num-bigint` direct dependency is dropped from
`cow-sdk-core` `[dependencies]` (it persists only as a `[dev-dependency]`
for the wider-product oracle in the U256 overflow property test). The
production `encode_address_word(&Address) -> [u8; 32]` helper that
right-aligns an EVM address into a 32-byte ABI word is now a single
`cow_sdk_contracts::encode_address_word` re-export; the duplicate
`fn encode_address_word(address: &Address)` body that lived in
`cow-sdk-trading::allowance` is removed and the trading crate consumes
the shared helper cross-crate. The cow-sdk-trading slippage subsystem
(`order.rs`, `slippage/amounts.rs`, `slippage/breakdown.rs`,
`slippage/policy.rs`) drops its `num_bigint::BigInt` direct dependency
and routes the percentage and partner-fee arithmetic through
`alloy_primitives::aliases::I512`; the 512-bit signed primitive carries
a 256-bit headroom over the worst-case intermediate
(`U256::MAX * percent_scaled` ≈ `2^283`) so the cow uint256 ceiling and
the negative-intermediate behaviour the slippage math depends on stay
exact. The cow `cargo tree --invert num-bigint` lane now shows no cow-rs
first-party crate as a direct consumer; the surviving paths are the
third-party `jsonschema -> fraction -> num -> num-bigint` chain reached
via the `cow-sdk-app-data` JSON-Schema validator dependency plus the
`cow-sdk-core` `[dev-dependencies]` entry that the U256 overflow
property test pins as the arbitrary-width oracle. Each remaining cow
contracts helper that
wraps a byte-typed value
(`parse_alloy_address`, `hash32_bytes`, `decode_order_uid_bytes`,
`decode_digest_key`, `address_to_sol`, `order_uid_bytes`, `role_hash`,
`alloy_to_cow_receipt`, `alloy_to_cow_block_info`, `alloy_domain_from`,
`build_eip712_domain`) is infallible by construction and returns the
wrapped value directly, with no `Result` indirection. The
`amount_to_u256(&Amount)` / `biguint_to_u256(&'static str, &BigUint)`
overflow-guard helpers in
`cow-sdk-contracts::settlement::codec`,
`cow-sdk-contracts::order::hash`, `cow-sdk-contracts::eth_flow`, and
`cow-sdk-signing::order_signing` are retired in favour of a direct
`*amount.as_u256()` deref on the cow newtype, because the `uint256`
ceiling is enforced by the type system at construction and the runtime
overflow guards collapse to constant-true invariants. The contract
tests at `crates/core/tests/wire_format_preservation_contract.rs` lock
the canonical wire byte sequence for every identity primitive
(`Address`, `Hash32`, `AppDataHash`, `HexData`, `OrderUid`, `Amount`,
`SignedAmount`) and pin the `write_into` / `to_hex_string` byte-parity
property against the four byte-typed strict newtypes, the canonical
lowercase form on uppercase `AppDataHash` input, the strict-decimal
serde boundary on `Amount` (the `0x` / `0o` / `0b` radix-prefix
rejection), and the strict-decimal serde boundary on `SignedAmount`
(the `0x` and leading-plus rejection), so the canonical wire contract
stays byte-identical across the Stage B migration.

The seven cow primitive newtypes (`Address`, `AppDataHash`, `Amount`,
`Hash32`, `HexData`, `OrderUid`, `SignedAmount`) carry a wasm-target
Tsify derive (`#[cfg_attr(target_family = "wasm",
derive(tsify::Tsify))]` with the `into_wasm_abi`, `from_wasm_abi`, and
`type = "string"` attributes) so the canonical lowercase hex string (or
decimal string for the numeric pair) is the wasm-bindgen ABI shape for
any future binding that exposes a cow identity newtype across the JS
boundary. The non-wasm targets pick up no extra dependency surface; the
derive is gated entirely behind `target_family = "wasm"`. The
`cow_sdk_core::prelude` re-export hub now carries `Address`, `Amount`,
`AppDataHash`, `Hash32`, `HexData`, `OrderUid`, and `SignedAmount`
together, so a single `use cow_sdk_core::prelude::*;` brings every
strict newtype into scope per ADR 0052.

### EIP-712 Domain Shape

`cow_sdk_core::TypedDataDomain` is a cow-owned `#[non_exhaustive]`
struct with four required fields (`name: String`, `version: String`,
`chain_id: ChainId`, `verifying_contract: Address`) and no `salt`,
matching the GPv2 Solidity domain shape that every shipped
GPv2Settlement instance has burnt into immutable bytecode since 2021.
Cow callers construct the domain through the cow-owned
`TypedDataDomain::new(name, version, chain_id, verifying_contract)`
constructor or via direct struct-literal initialisation. The cow
struct's derived `Serialize`/`Deserialize` impls emit and parse the
canonical EIP-1193 `eth_signTypedData_v4` wire shape directly: numeric
`chainId` (cow `ChainId` newtype serialises through its u64 inner),
lowercase-hex `verifyingContract`, and no `salt` field on the wire.

The `crates/alloy-signer/src/conversion.rs` module provides the
one-way cow → alloy adapter the EIP-712 hashing seam needs. Two
caller helpers (`cow_flat_to_alloy_typed_data` and
`cow_typed_data_payload_to_alloy`) lift the cow envelope into the
alloy `Eip712Domain` shape so `alloy_sol_types::SolStruct::eip712_signing_hash`
can compute the canonical separator and signing hash. The cow type
remains the public API surface; the alloy type is the transient
hashing-step helper.

The `signer_contract.rs::validate_typed_data_chain_rejects_payload_with_wrong_domain_chain_id`
contract test exercises the cow `ChainId` field's strict equality
against the signer's bound chain id, and the `domain_contract.rs`
+ `parity_contract.rs` suites in the `cow-sdk-signing` crate pin the
canonical wire shape and the byte-identity invariants. The byte-
identity gates fix the mainnet domain separator
`0xc078f884a2676e1345748b1feace7b0abee5d00ecadb6e574dcdd109a63e8943`,
the sepolia separator `0xdaee378bd0eb30ddf479272accf91761e697bc00e067a268f95f1d2732ed230b`,
the GPv2 Order type hash `0xd5a25ba2e97094ad7d83dc28a6572da797d6b3e7fc6663bd93efb789fc17e489`,
and the canonical EIP-712 reference signature
`0x34bc8d9249f7f9399d1db57b96bfc3a2f935a25965fe265292142c305284c7241daf1b3049bc75da81012cf33aeac1de09ec5684bccf03afe7274262703780d01c`.

## Evidence

Primary implementation points:

- `crates/contracts/src/settlement/mod.rs`
- `crates/contracts/src/settlement/encoder.rs`
- `crates/contracts/src/settlement/codec.rs`
- `crates/contracts/src/interaction.rs`
- `crates/contracts/src/errors.rs`
- `crates/contracts/src/vault.rs`
- `crates/contracts/src/eth_flow.rs`
- `crates/contracts/src/onchain_orders.rs`
- `crates/contracts/src/proxy.rs`
- `crates/contracts/src/erc20.rs`
- `crates/contracts/src/weth.rs`
- `crates/contracts/src/primitives.rs`
- `crates/contracts/Cargo.toml`
- `crates/trading/src/onchain.rs`
- `crates/contracts/abi/settlement/`
- `crates/contracts/abi/vault-relayer/`
- `crates/contracts/abi/eth-flow/`
- `crates/contracts/abi/eip1967/`
- `crates/contracts/abi/erc20/`
- `crates/contracts/abi/weth/`
- `crates/contracts/tests/fixtures/domain_separator_parity.json`
- `crates/signing/tests/fixtures/domain_separator_parity.json`
- `parity/fixtures/contracts.json`

Primary regression coverage:

- `crates/contracts/tests/parity_contract.rs`
- `crates/contracts/tests/order_contract.rs::order_flag_matrix_enumerates_all_twelve_combinations`
- `crates/contracts/tests/reader_contract.rs::settlement_reader_filled_amounts_decodes_known_payload`
- `crates/contracts/tests/settlement_contract.rs::settlement_encoder_stage_order_pre_intra_post`
- `crates/contracts/tests/proxy_contract.rs::eip1967_slot_constants_match_canonical_keccak_minus_one`
- `crates/contracts/tests/property_contract.rs::decode_trade_flags_accepts_0b00_and_0b01_as_erc20`
- `crates/contracts/tests/property_contract.rs::decode_order_rejects_out_of_bounds_token_indices`
- `crates/contracts/tests/interaction_contract.rs::interaction_encoder_rejects_vault_relayer_target_for_canonical_settlement_domain`
- `crates/contracts/tests/interaction_contract.rs::interaction_encoder_accepts_non_vault_target_for_canonical_settlement_domain`
- `crates/contracts/tests/interaction_contract.rs::interaction_encoder_does_not_cross_match_chain_or_env`
- `crates/contracts/tests/interaction_contract.rs::interaction_encoder_neutral_for_unknown_custom_settlement_domain`
- `crates/contracts/tests/vault_contract.rs::vault_role_hashes_match_the_canonical_solidity_packed_layout`
- `crates/contracts/src/primitives.rs::tests::domain_separator_matches_shared_parity_fixture`
- `crates/contracts/src/primitives.rs::tests::order_kind_marker_round_trips_and_rejects_unknown`
- `crates/contracts/tests/onchain_orders.rs::order_placement_topic0_matches_canonical_hash`
- `crates/contracts/tests/onchain_orders.rs::order_hash_matches_canonical_ethflow_foundry_vector`
- `crates/contracts/tests/onchain_orders.rs::eip1271_placement_decodes_owner_uid_and_trailer`
- `crates/contracts/tests/weth.rs::withdraw_selector_matches_canonical_keccak`
- `crates/signing/src/domain.rs::tests::domain_separator_matches_shared_parity_fixture`
- `crates/trading/tests/onchain_contract.rs`
- `crates/trading/tests/parity_contract.rs`
- `crates/core/tests/wire_format_preservation_contract.rs`
- `crates/core/tests/property_contract.rs`
- `crates/browser-wallet/tests/signer_contract.rs::validate_typed_data_chain_rejects_payload_with_wrong_domain_chain_id`
- `crates/browser-wallet/tests/signer_contract.rs::typed_data_payload_emits_canonical_eip1193_wire_shape_against_fixture`
- `crates/signing/tests/domain_contract.rs`
- `crates/signing/tests/parity_contract.rs`
- `parity/fixtures/signing/eth_sign_typed_data_request.json`

Validation surface:

```text
cargo test -p cow-sdk-contracts --all-features
cargo test -p cow-sdk-contracts --test property_contract
cargo test -p cow-sdk-contracts --test interaction_contract
cargo test -p cow-sdk-contracts --test onchain_orders
cargo test -p cow-sdk-contracts --test weth
cargo parity-verify-sol-provenance
cargo test -p cow-sdk-contracts --test vault_contract vault_role_hashes_match_the_canonical_solidity_packed_layout
cargo test -p cow-sdk-contracts --test parity_contract parity_fixture_cases_hold
cargo test -p cow-sdk-contracts domain_separator_matches_shared_parity_fixture
cargo test -p cow-sdk-signing domain_separator_matches_shared_parity_fixture
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo clippy -p cow-sdk-contracts --all-targets --all-features -- -D warnings
cargo test -p cow-sdk-trading --all-features --tests
cargo clippy -p cow-sdk-trading --all-targets --all-features -- -D warnings
```
