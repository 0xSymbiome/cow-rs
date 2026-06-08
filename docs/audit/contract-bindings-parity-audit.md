# Contract Bindings Parity Audit

Status: Current
Last reviewed: 2026-06-08
Owning surface: `cow-sdk-contracts` `alloy::sol!`-generated bindings for `GPv2Settlement`, `CoWSwapEthFlow`, `CoWSwapOnchainOrders` events, the wrapped-native token, and `IERC20` / `IERC20Permit`
Refresh trigger: A new binding family landing in `cow-sdk-contracts`; a signature change in any existing binding; a change to the upstream commit pin for any binding's source repository under `parity/source-lock.yaml`; a change to the TypeScript-SDK-derived parity fixtures that back the regression suite; a change to the EIP-712 domain-separator fixture shared with the signing crate; a change to the wasm target feature contract for the alloy/k256 dependency path
Related docs:
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [Parity Matrix](../parity.md)
- [Parity Scope](../parity.md)
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the `alloy::sol!`-generated binding surfaces shipped in
  `cow-sdk-contracts`
- the byte-identity parity contract between the bindings and the
  TypeScript-SDK-derived fixtures for the encoded call-data and the
  hashed data (order digest, order UID, EIP-712 type hashes)
- the contract-side EIP-712 domain-separator fixture that must stay
  byte-identical with the signing crate's fixture
- the wasm target feature contract that keeps the `alloy-primitives`
  `k256` path buildable under `wasm32-unknown-unknown`
- the five sol! interface families currently shipped: `IGPv2Settlement`,
  `ICoWSwapEthFlow`, the `ICoWSwapOnchainOrders` event surface, the
  `IWrappedNativeToken` (WETH9-family) surface, and the `IERC20` /
  `IERC20Permit` ERC-20 surface

It does not cover deployed-address resolution (Registry authority, a
separate audit) or the HTTP transport that delivers call-data to a
provider.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Single binding idiom | Every shipped binding is generated through `alloy::sol!`; no hand-rolled encoder remains in `cow-sdk-contracts` | Conforms |
| Pinned provenance | Every binding's `alloy::sol!` interface reproduces the upstream Solidity surface verbatim, and the upstream repository each binding mirrors is pinned by commit in `parity/source-lock.yaml` so a reviewer can diff the binding against the upstream source at the pinned commit | Conforms |
| Byte-identity parity | Encoded call-data and hashed payloads match the TypeScript-SDK-derived golden fixtures on every binding | Conforms |
| Domain separator parity | `cow-sdk-contracts` and `cow-sdk-signing` route every EIP-712 domain separator through `alloy_sol_types::Eip712Domain::separator` and pin the same fixture value | Conforms |
| Order EIP-712 hashing | The `GPv2 Order` and `OrderCancellations` typed-data structs are macro-emitted via `alloy_sol_types::sol!` and route their signing hashes through `<T as SolStruct>::eip712_signing_hash`; the eight per-chain rows in the order-digest fixture pin the wire-byte contract | Conforms |
| EIP-1271 payload encoding | The COW EIP-1271 verifier payload `abi.encode(GPv2Order.Data, bytes)` is composed from the macro-emitted `OnchainOrder` sol struct and the raw ECDSA signature via `alloy_sol_types::SolValue::abi_encode_sequence`; the inline regression contract reproduces the canonical 12-word order tuple plus dynamic-bytes tail layout byte-for-byte | Conforms |
| WASM compatibility | The `alloy-primitives` `k256` path enables the browser `getrandom` backend for `wasm32-unknown-unknown` builds | Conforms |
| Scope discipline | The shipped set is the five families named above; any new family follows the same provenance and parity contract before it lands | Conforms |

## Current Contract

### Binding Families

`GPv2Settlement` (`crates/contracts/src/settlement/mod.rs`) carries the
`settle`, `invalidateOrder(bytes)`, `setPreSignature`, trade-struct,
and interaction-struct surface against the mainnet-deployed
`0x9008D19f58AAbD9eD0D60971565AA8510560ab41` contract.

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

Every binding is introduced by an inline `alloy::sol!` interface block
that reproduces the upstream Solidity surface verbatim. The upstream
repository each binding mirrors is named in the binding's module-level
doc comment and pinned by commit under `repositories:` in
`parity/source-lock.yaml`, so a reviewer can diff the inline interface
against the upstream source at the exact pinned commit.

The shipped bindings mirror upstream surfaces from three CoW Protocol
repositories, each pinned by commit in `parity/source-lock.yaml`:

- `cowprotocol/contracts` — the `GPv2Settlement`, `GPv2Trade`,
  `GPv2Interaction`, and `IERC20` surfaces.
- `cowprotocol/ethflowcontract` — the `CoWSwapEthFlow` and
  `EthFlowOrder` surfaces.
- `cowdao-grants/cow-shed` — the COW Shed surfaces (reviewed in the
  [COW Shed Contract Bindings Audit](cow-shed-contract-bindings-audit.md)).

The provenance posture is commit-pin plus fixture proof rather than a
byte-mirrored source tree: the upstream commit pin records what each
binding mirrors, and the TypeScript-SDK-derived call-data, EIP-712, and
selector fixtures under `parity/fixtures/` together with the crate
parity tests prove the inline binding produces byte-identical wire
bytes. A binding drift therefore surfaces as a fixture regression in
`cargo test -p cow-sdk-contracts` before it can reach any consumer.

### Byte-Identity Parity

Each binding has a regression contract that encodes a known input and
asserts the output matches a TypeScript-SDK-derived fixture bit for
bit. The same contract covers:

- EIP-712 domain separators (chain-id and verifying-contract swept)
- Order hash, UID, and signing-scheme payload bytes
- Compact order flag decoding across every supported kind/source/destination
  combination
- Settlement call-data for multi-trade batches
- Settlement encoder PRE, INTRA, POST interaction ordering
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

Only the five binding families listed above are in scope for this
audit. Third-party protocol bindings (Aave, bridging adapters,
condition schedulers) stay in their own capability crates and carry
their own parity contracts when they land. Hand-rolled encoder helpers
are not allowed in `cow-sdk-contracts`.

### Interaction Normalization

`normalize_interaction` is the reviewed boundary for defaulting typed
interaction data. It remains infallible and value-neutral: a missing value
defaults to zero and missing calldata defaults to an empty payload. The
trader-facing SDK does not encode settlement batches — settlement-calldata
encoding is a solver/backend concern that upstream keeps in its `solver`
crate — so the only settlement calls the SDK builds (`setPreSignature`,
`invalidateOrder`) are encoded directly from the `IGPv2Settlement` binding.

### Wire Serde

The DTO fields that carry hex-encoded byte payloads on the JSON wire route
through `alloy_primitives::Bytes`, whose native `Serialize` / `Deserialize`
impl emits and parses the canonical `0x`-prefixed lowercase hexadecimal
string the protocol's TypeScript SDK consumes. The migrated fields are
`Interaction.call_data` and `InteractionLike.call_data` in
`crates/contracts/src/interaction.rs`. No bespoke `#[serde(with =
"...")]` adapter is interposed on the `Bytes`-typed fields; the alloy
primitive owns the canonical wire form. The `cow-sdk-contracts` parity
fixtures that exercise these fields (settlement calldata stages and the
interaction encoder stage matrices) stay green byte-identically across the
migration, so the typed value contract and the wire byte contract remain
locked together.

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
`alloy_primitives::U256`. The cached `{ inner, hex }` struct layout from
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
rejected through deserialization). The remaining byte-typed primitives (`Hash32`,
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
accessors (`as_alloy` / `as_u256`,
`into_alloy` / `into_u256`, `to_hex_string`,
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
for the wider-product oracle in the U256 overflow property test).
Address-to-32-byte-word encoding has no shipped helper: the `sol!` ABI
encoders handle word layout, and `alloy_primitives::Address::into_word`
is the canonical API for any direct need; only an independent EIP-712
parity oracle in the `cow-sdk-contracts` order-hash test module retains a
hand-shaped `[u8; 32]` form. The cow-sdk-trading slippage subsystem
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
(`Address`, `Hash32`, `AppDataHash`, `HexData`, `OrderUid`, `Amount`)
and pin the `write_into` / `to_hex_string` byte-parity
property against the four byte-typed strict newtypes, the canonical
lowercase form on uppercase `AppDataHash` input, the strict-decimal
serde boundary on `Amount` (the `0x` / `0o` / `0b` radix-prefix
rejection), so the canonical wire contract
stays byte-identical across the Stage B migration.

The six cow primitive newtypes (`Address`, `AppDataHash`, `Amount`,
`Hash32`, `HexData`, `OrderUid`) carry a wasm-target
Tsify derive (`#[cfg_attr(target_family = "wasm",
derive(tsify::Tsify))]` with the `into_wasm_abi`, `from_wasm_abi`, and
`type = "string"` attributes) so the canonical lowercase hex string (or
decimal string for `Amount`) is the wasm-bindgen ABI shape for
any future binding that exposes a cow identity newtype across the JS
boundary. The non-wasm targets pick up no extra dependency surface; the
derive is gated entirely behind `target_family = "wasm"`. The
`cow_sdk_core::prelude` re-export hub now carries `Address`, `Amount`,
`AppDataHash`, `Hash32`, `HexData`, and `OrderUid`
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
- `crates/contracts/src/eth_flow.rs`
- `crates/contracts/src/onchain_orders.rs`
- `crates/contracts/src/erc20.rs`
- `crates/contracts/src/weth.rs`
- `crates/contracts/src/primitives.rs`
- `crates/contracts/Cargo.toml`
- `crates/trading/src/onchain.rs`
- `parity/source-lock.yaml`
- `parity/fixtures/eip712/order_digests.json`
- `crates/contracts/tests/fixtures/domain_separator_parity.json`
- `crates/signing/tests/fixtures/domain_separator_parity.json`
- `parity/fixtures/contracts.json`

Primary regression coverage:

- `crates/contracts/tests/parity_contract.rs`
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
cargo test -p cow-sdk-contracts --test parity_contract parity_fixture_cases_hold
cargo test -p cow-sdk-contracts domain_separator_matches_shared_parity_fixture
cargo test -p cow-sdk-signing domain_separator_matches_shared_parity_fixture
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo clippy -p cow-sdk-contracts --all-targets --all-features -- -D warnings
cargo test -p cow-sdk-trading --all-features --tests
cargo clippy -p cow-sdk-trading --all-targets --all-features -- -D warnings
```
