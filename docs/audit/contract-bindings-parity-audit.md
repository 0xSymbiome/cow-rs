# Contract Bindings Parity Audit

Status: Current
Last reviewed: 2026-05-19
Owning surface: `cow-sdk-contracts` `alloy::sol!`-generated bindings for `GPv2Settlement`, `GPv2VaultRelayer`, `CoWSwapEthFlow`, EIP-1967 proxy slots, and `IERC20` / `IERC20Permit`
Refresh trigger: A new binding family landing in `cow-sdk-contracts`; a signature change in any existing binding; a drift in the committed Solidity excerpt under `crates/contracts/abi/**/*.sol`; a change to the TypeScript-SDK-derived parity fixtures that back the regression suite; a change to the EIP-712 domain-separator fixture shared with the signing crate; a change to the wasm target feature contract for the alloy/k256 dependency path
Related docs:
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0034](../adr/0034-interaction-encoder-target-policy.md)
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
| Domain separator parity | `cow-sdk-contracts` and `cow-sdk-signing` route every EIP-712 domain separator through `alloy_sol_types::Eip712Domain::separator` and pin the same fixture value | Conforms |
| Order EIP-712 hashing | The `GPv2 Order` and `OrderCancellations` typed-data structs are macro-emitted via `alloy_sol_types::sol!` and route their signing hashes through `<T as SolStruct>::eip712_signing_hash`; the eight per-chain rows in the order-digest fixture pin the wire-byte contract | Conforms |
| EIP-1271 payload encoding | The COW EIP-1271 verifier payload `abi.encode(GPv2Order.Data, bytes)` is composed from the macro-emitted `OnchainOrder` sol struct and the raw ECDSA signature via `alloy_sol_types::SolValue::abi_encode_sequence`; the inline regression contract reproduces the canonical 12-word order tuple plus dynamic-bytes tail layout byte-for-byte | Conforms |
| Boundary matrices | Compact order flags, settlement reader returns, settlement encoder stages, mixed-balance transfers, and multi-trade clearing prices have deterministic regression coverage | Conforms |
| EIP-1967 derivation | Proxy storage slots match the canonical `keccak256(label) - 1` formula as well as the golden byte payloads | Conforms |
| Vault role hash parity | Vault-relayer role helpers emit the same packed role hashes as the upstream TypeScript role-grant helpers | Conforms |
| WASM compatibility | The `alloy-primitives` `k256` path enables the browser `getrandom` backend for `wasm32-unknown-unknown` builds | Conforms |
| Scope discipline | The shipped set is the five families named above; any new family follows the same provenance and parity contract before it lands | Conforms |

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

The EIP-1967 surface (`crates/contracts/src/proxy.rs`) carries the
`ADMIN_SLOT` and `IMPLEMENTATION_SLOT` storage-slot helpers.
The regression suite verifies both the canonical hex payloads and the
formula-derived values from `keccak256("eip1967.proxy.<label>") - 1`.

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
(`OrderCancellations`). Both structs are re-exported publicly at the
crate root as `cow_sdk_contracts::GPv2Order` and
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
hashes through `<GPv2Order as SolStruct>::eip712_signing_hash` and
batch-cancellation signing hashes through
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

Only the five binding families listed above are in scope for this
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
newtypes over the canonical `alloy_primitives` byte types.
`cow_sdk_core::Address` wraps `alloy_primitives::Address`; `Hash32`,
`OrderDigest`, and `BlockHash` wrap `alloy_primitives::B256`; `HexData`
wraps `alloy_primitives::Bytes`; `OrderUid` wraps
`alloy_primitives::FixedBytes<56>`. The cached `{ inner, hex }` struct
layout from the previous parity revision has been retired for these four
newtype families, along with the `identity_ext` extension trait module
and the `cow_sdk_core::types::hex` encoder helpers that backed it.
`AppDataHash` intentionally keeps the cached layout because the
app-data wire envelope demands stable `as_str()` borrowing across the
SDK.

Construction stays through the existing `new(&str) -> Result<Self, _>`
factories; the strict newtypes parse once at construction and reject
malformed input with the same error variants the previous layout
emitted. Display, Serialize, and Deserialize impls are cow-owned on
`Address` (lowercase 0x-prefixed canonical, matching the deployed
protocol convention) and alloy-forwarding on `Hash32`, `OrderDigest`,
`BlockHash`, `HexData`, and `OrderUid` via `#[serde(transparent)]`. The
inherent stdlib-style accessor is renamed `as_str() -> &str` to
`to_hex_string() -> String` so callers receive an owned string that
honors the canonical lowercase encoding contract without depending on
internal caching. The new
`write_into(&self, f: &mut impl core::fmt::Write) -> core::fmt::Result`
accessor provides a zero-allocation path for the hot tracing and JSON
emission seams that previously borrowed the cached hex string. The
internal `pub` tuple-struct field carries a rustdoc-documented
escape-hatch caveat: it is reachable for advanced callers but is
explicitly not part of the API stability contract, and the safe
accessors (`as_alloy`, `into_alloy`, `to_hex_string`, `write_into`,
`as_slice`) cover every supported workflow.

Equality, hash, and ordering on the strict newtypes collapse onto the
underlying alloy byte comparison, which is equivalent to the previous
case-insensitive contract because every valid input parses to the same
bytes regardless of input casing. The seam helpers in
`cow_sdk_alloy_provider` and `cow_sdk_alloy` consume the packed bytes
directly through `*value.as_alloy()` and `value.into_alloy()`, replacing
the previous `cow_to_alloy_address` / `cow_to_alloy_hash` /
`alloy_address_to_cow_address` / `hex_data_from_bytes` /
`decode_0x_hex` / `parse_u256_quantity` adapter helpers, which are
removed. The cow-side hex helpers in `cow_sdk_contracts::primitives`
(`parse_hex`, `parse_hex_exact`, `parse_address_bytes`,
`parse_bytes32_hash`, `parse_hex32`, `normalize_hex_payload`) are
removed in the same change set; consumer modules
(`contracts::deploy`, `contracts::eth_flow`, `contracts::proxy`,
`contracts::signature`, `contracts::settlement::codec`,
`contracts::vault`) now route directly through the cow newtype
`into_alloy` / `as_alloy` accessors and the `alloy_primitives::hex`
decode entry point. Each remaining cow contracts helper that wraps a
byte-typed value (`parse_alloy_address`, `hash32_bytes`,
`decode_order_uid_bytes`, `decode_digest_key`, `address_to_sol`,
`encode_address_word`, `order_uid_bytes`, `role_hash`,
`alloy_to_cow_receipt`, `alloy_to_cow_block_info`, `alloy_domain_from`,
`build_eip712_domain`) is infallible by construction and returns the
wrapped value directly, with no `Result` indirection. The contract tests
at `crates/core/tests/wire_format_preservation_contract.rs` lock the
canonical wire byte sequence for every identity primitive
(`Address`, `Hash32`, `AppDataHash`, `HexData`, `OrderUid`) and pin the
`write_into` / `to_hex_string` byte-parity property against the four
strict newtypes, so the canonical lowercase hex contract stays
byte-identical across the migration.

The four byte-typed cow newtypes carry a wasm-target Tsify derive
(`#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]` with the
`into_wasm_abi`, `from_wasm_abi`, and `type = "string"` attributes) so
the canonical lowercase hex string is the wasm-bindgen ABI shape for any
future binding that exposes a cow identity newtype across the JS
boundary. The non-wasm targets pick up no extra dependency surface; the
derive is gated entirely behind `target_family = "wasm"`.

## Evidence

Primary implementation points:

- `crates/contracts/src/settlement/mod.rs`
- `crates/contracts/src/settlement/encoder.rs`
- `crates/contracts/src/settlement/codec.rs`
- `crates/contracts/src/interaction.rs`
- `crates/contracts/src/errors.rs`
- `crates/contracts/src/vault.rs`
- `crates/contracts/src/eth_flow.rs`
- `crates/contracts/src/proxy.rs`
- `crates/contracts/src/erc20.rs`
- `crates/contracts/src/primitives.rs`
- `crates/contracts/Cargo.toml`
- `crates/trading/src/onchain.rs`
- `crates/contracts/abi/settlement/`
- `crates/contracts/abi/vault-relayer/`
- `crates/contracts/abi/eth-flow/`
- `crates/contracts/abi/eip1967/`
- `crates/contracts/abi/erc20/`
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
- `crates/signing/src/domain.rs::tests::domain_separator_matches_shared_parity_fixture`
- `crates/trading/tests/onchain_contract.rs`
- `crates/trading/tests/parity_contract.rs`
- `crates/core/tests/wire_format_preservation_contract.rs`
- `crates/core/tests/property_contract.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --all-features
cargo test -p cow-sdk-contracts --test property_contract
cargo test -p cow-sdk-contracts --test interaction_contract
cargo test -p cow-sdk-contracts --test vault_contract vault_role_hashes_match_the_canonical_solidity_packed_layout
cargo test -p cow-sdk-contracts --test parity_contract parity_fixture_cases_hold
cargo test -p cow-sdk-contracts domain_separator_matches_shared_parity_fixture
cargo test -p cow-sdk-signing domain_separator_matches_shared_parity_fixture
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo clippy -p cow-sdk-contracts --all-targets --all-features -- -D warnings
cargo test -p cow-sdk-trading --all-features --tests
cargo clippy -p cow-sdk-trading --all-targets --all-features -- -D warnings
```
