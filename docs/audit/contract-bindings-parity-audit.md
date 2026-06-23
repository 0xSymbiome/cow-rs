# Contract Bindings Parity Audit

Status: Current
Last reviewed: 2026-06-20
Owning surface: `cow-sdk-contracts` `alloy::sol!`-generated bindings for `GPv2Settlement`, `CoWSwapEthFlow`, `CoWSwapOnchainOrders` events, the wrapped-native token, and `IERC20`
Refresh trigger: A new binding family landing in `cow-sdk-contracts`; a signature change in any existing binding; a change to the upstream commit pin for any binding's source repository under `parity/source-lock.yaml`; a change to the TypeScript-SDK-derived parity fixtures that back the regression suite; a change to the EIP-712 domain-separator fixture shared with the signing crate; a change to the wasm target feature contract for the alloy/k256 dependency path
Related docs:
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [Parity Matrix](../parity.md)
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the `alloy::sol!`-generated binding surfaces shipped in `cow-sdk-contracts`
- the byte-identity parity contract between the bindings and the
  TypeScript-SDK-derived fixtures for encoded call-data and hashed data
  (order digest, order UID, EIP-712 type hashes)
- the contract-side EIP-712 domain-separator fixture that must stay
  byte-identical with the signing crate's fixture
- the wasm target feature contract that keeps the `alloy-primitives` `k256`
  path buildable under `wasm32-unknown-unknown`

It does not cover deployed-address resolution (Registry authority, a separate
audit) or the HTTP transport that delivers call-data to a provider.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Single binding idiom | Every shipped binding is generated through `alloy::sol!`; no hand-rolled encoder remains in `cow-sdk-contracts` | Conforms |
| Pinned provenance | Every binding's `alloy::sol!` interface reproduces the upstream Solidity surface verbatim, and the upstream repository each binding mirrors is pinned by commit in `parity/source-lock.yaml` so a reviewer can diff the binding against the upstream source at the pinned commit | Conforms |
| Byte-identity parity | Encoded call-data and hashed payloads match the TypeScript-SDK-derived golden fixtures on every binding | Conforms |
| Domain separator parity | `cow-sdk-contracts` and `cow-sdk-signing` route every EIP-712 domain separator through `alloy_sol_types::Eip712Domain::separator` and pin the same fixture value | Conforms |
| Order EIP-712 hashing | The `GPv2 Order` and `OrderCancellations` typed-data structs are macro-emitted via `alloy_sol_types::sol!` and route their signing hashes through `<T as SolStruct>::eip712_signing_hash`; the eight representative rows in the order-digest fixture pin the wire-byte contract | Conforms |
| EIP-1271 payload encoding | The COW EIP-1271 verifier payload `abi.encode(GPv2Order.Data, bytes)` is composed from the macro-emitted `OnchainOrder` sol struct and the raw ECDSA signature via `alloy_sol_types::SolValue::abi_encode_sequence`; the inline regression contract reproduces the canonical 12-word order tuple plus dynamic-bytes tail layout byte-for-byte | Conforms |
| WASM compatibility | The `alloy-primitives` `k256` path enables the browser `getrandom` backend for `wasm32-unknown-unknown` builds | Conforms |
| Scope discipline | The shipped set is the five families named above; any new family follows the same provenance and parity contract before it lands | Conforms |

## Current Contract

### Binding Families

Five `alloy::sol!` interface families ship in `cow-sdk-contracts`:

- `GPv2Settlement` (`crates/contracts/src/settlement.rs`) against the
  mainnet-deployed `0x9008D19f58AAbD9eD0D60971565AA8510560ab41` contract. The
  SDK encodes `setPreSignature(bytes,bool)` and `invalidateOrder(bytes)`, plus
  the `freeFilledAmountStorage` / `freePreSignatureStorage` order-refund calls.
  The solver-only `settle` entry point is deliberately out of scope.
- `CoWSwapEthFlow` (`crates/contracts/src/eth_flow.rs`) carrying
  `createOrder(EthFlowOrderData)` and `invalidateOrder(EthFlowOrderData)`. This
  `invalidateOrder` variant takes the full `EthFlowOrderData` payload and is
  distinct from `GPv2Settlement::invalidateOrder(bytes)`, which takes a packed
  order UID.
- `CoWSwapOnchainOrders` (`crates/contracts/src/onchain_orders.rs`) carrying the
  `OrderPlacement` and `OrderInvalidation` event bindings used by on-chain order
  routers such as eth-flow. The topic-0 signature hashes are byte-locked against
  an independent keccak of the flattened-tuple signatures, and the fail-closed
  decoder reconstructs the broadcast `GPv2` order, resolves the owner from the
  on-chain signature, and derives the 56-byte order UID through
  `compute_order_uid`. The decoding contract is governed by
  [ADR 0054](../adr/0054-onchain-order-event-decoding-is-fail-closed.md) and the
  [Event Log Decoding Audit](event-log-decoding-audit.md).
- `IWrappedNativeToken` (`crates/contracts/src/tokens.rs`) carrying the
  WETH9-family `deposit` / `withdraw` methods, with `wrap_interaction` and
  `unwrap_interaction` helpers. The 4-byte selectors are byte-locked against an
  independent keccak of the canonical signatures.
- `IERC20` (`crates/contracts/src/tokens.rs`) for the subset of methods the SDK
  emits against any ERC-20 token: the vault-relayer `approve` flow plus the
  balance and transfer reads.

`IERC1271`, the EIP-1271 verifier interface, is co-located with the signature
codecs in `crates/contracts/src/signature.rs`.

`EthFlowOrderData::new` and `EthFlowOrderData::from_unsigned_order` reject
`Address::ZERO` for the receiver field with `ContractsError::ZeroReceiver`. This
mirrors the upstream `EthFlowOrder.toCoWSwapOrder` library function's
`ReceiverMustBeSet()` revert (selector `0xefc9ccdf`), which fires on both the
`createOrder` and `invalidateOrder` write paths through the shared library call.
The general order hash path instead treats `address(0)` as the protocol's
pay-to-owner sentinel and hashes it verbatim.

### Provenance

Every binding is introduced by an inline `alloy::sol!` interface block that
reproduces the upstream Solidity surface verbatim. The upstream repository each
binding mirrors is named in the binding's module-level doc comment and pinned by
commit under `repositories:` in `parity/source-lock.yaml`, so a reviewer can
diff the inline interface against the upstream source at the pinned commit. The
shipped bindings mirror three CoW Protocol repositories:

- `cowprotocol/contracts` — `GPv2Settlement`. `IERC20` is authored inline
  against the published EIP-20 standard, not against this repository.
- `cowprotocol/ethflowcontract` — `CoWSwapEthFlow`, `EthFlowOrder`,
  `CoWSwapOnchainOrders`, and `IWrappedNativeToken`.
- `cowdao-grants/cow-shed` — the COW Shed surfaces (reviewed in the
  [COW Shed Contract Bindings Audit](cow-shed-contract-bindings-audit.md)).

The provenance posture is commit-pin plus fixture proof: the commit pin records
what each binding mirrors, and the TypeScript-SDK-derived call-data, EIP-712,
and selector fixtures under `parity/fixtures/` prove the inline binding produces
byte-identical wire bytes. A binding drift therefore surfaces as a fixture
regression in `cargo test -p cow-sdk-contracts` before it can reach any
consumer.

### Byte-Identity Parity

Each binding has a regression contract that encodes a known input and asserts
the output matches a TypeScript-SDK-derived fixture bit for bit, covering:

- EIP-712 domain separators (chain-id and verifying-contract swept)
- Order hash, UID, and signing-scheme payload bytes
- Compact order flag decoding across every supported kind/source/destination
  combination
- Settlement `setPreSignature` / `invalidateOrder` / order-refund call-data,
  encoded through the shipped `IGPv2Settlement` binding
- Encoded trade flags (kind, partial fill, balance source, balance destination,
  signing scheme)

`crates/contracts/tests/parity_contract.rs` is the hub harness; per-family tests
extend it for surfaces that need additional fixtures.

The EIP-712 domain separator routes through
`alloy_sol_types::Eip712Domain::separator` in both `cow-sdk-contracts`
(folded into `order::hash_order`'s `eip712_signing_hash` call) and
`cow-sdk-signing` (`domain::domain_separator_for`). Both crates read one shared
JSON fixture (`parity/fixtures/eip712/settlement_domain_separator.json`,
included by each crate via `include_str!`), so a domain-encoding change cannot
move one crate without the other.

The `GPv2 Order` and `OrderCancellations` EIP-712 schemas are macro-emitted via
`alloy_sol_types::sol!`, and signing hashes route through
`<T as SolStruct>::eip712_signing_hash`. The canonical order type string keccak-
hashes to the deployed protocol constant
`0xd5a25ba2e97094ad7d83dc28a6572da797d6b3e7fc6663bd93efb789fc17e489`. The eight
representative rows in `parity/fixtures/eip712/order_digests.json` pin per-row
domain separator, struct hash, and signing hash so a future change cannot
silently move the wire bytes. The mainnet domain separator is fixed at
`0xc078f884a2676e1345748b1feace7b0abee5d00ecadb6e574dcdd109a63e8943` and sepolia
at `0xdaee378bd0eb30ddf479272accf91761e697bc00e067a268f95f1d2732ed230b`. The
canonical EIP-712 reference signature
`0x34bc8d9249f7f9399d1db57b96bfc3a2f935a25965fe265292142c305284c7241daf1b3049bc75da81012cf33aeac1de09ec5684bccf03afe7274262703780d01c`
is pinned separately as the `EXPECTED_ORDER_SIGNATURE` constant in
`crates/test-utils/src/consts.rs`, not as a row in the order-digest fixture.

The COW EIP-1271 verifier payload `abi.encode(GPv2Order.Data, bytes)` is
macro-emitted as `OnchainOrder` at `crates/signing/src/eip1271/sol_types.rs`;
`cow_sdk_signing::eip1271_signature_payload` composes the tuple and encodes it
via `alloy_sol_types::SolValue::abi_encode_sequence`. This on-chain
`GPv2Order.Data` schema stores `kind`, `sellTokenBalance`, and
`buyTokenBalance` as `bytes32` keccak labels (matching deployed storage layout),
a distinct schema from the EIP-712 typed-data `Order`. The inline regression in
`crates/signing/tests/order_signing_contract.rs` reproduces the byte layout by
hand at signature length 65.

The `cow-sdk-trading` on-chain helpers build settlement calldata by composing
the `IGPv2Settlement` sol! binding directly and routing through
`<C as alloy_sol_types::SolCall>::abi_encode`, the same canonical path the
contracts parity contract gates. EthFlow calldata is delegated to
`cow-sdk-contracts`' `encode_create_order_calldata` /
`encode_invalidate_order_calldata` helpers, which compose the `ICoWSwapEthFlow`
binding inside the contracts crate. No hand-rolled selector or offset helpers
remain in the trading crate. The fixture rows
`contracts-settlement-set-presignature-calldata` and
`contracts-settlement-invalidate-order-calldata` in
`parity/fixtures/contracts.json` lock the wire bytes for both settlement calls.

`cow-sdk-core`'s identity primitives are `#[repr(transparent)]` newtypes over
canonical `alloy_primitives` types (`Address`, `B256`-backed `Hash32` /
`OrderDigest` / `AppDataHash`, `Bytes`-backed `HexData`, `FixedBytes<56>`-backed
`OrderUid`, `U256`-backed `Amount`), exported on their module path per ADR 0052.
`crates/core/tests/wire_format_preservation_contract.rs` locks the canonical
wire byte sequence for each.

### WASM Target Contract

`crates/contracts/Cargo.toml` keeps the `alloy-primitives` `k256` path
compatible with browser-target builds by enabling the `getrandom` `wasm_js`
backend only for `wasm32`. This is a build-contract detail, not a public API change:
callers interact with the same contract DTOs and hashing helpers on native and
wasm targets.

### Scope Discipline

Only the five binding families listed above are in scope. Third-party protocol
bindings (Aave, bridging adapters, condition schedulers) stay in their own
capability crates and carry their own parity contracts when they land.
Hand-rolled encoder helpers are not allowed in `cow-sdk-contracts`.

## Evidence

Primary implementation points:

- `crates/contracts/src/settlement.rs`
- `crates/contracts/src/interaction.rs`
- `crates/contracts/src/errors.rs`
- `crates/contracts/src/eth_flow.rs`
- `crates/contracts/src/onchain_orders.rs`
- `crates/contracts/src/tokens.rs`
- `crates/contracts/src/primitives.rs`
- `crates/contracts/Cargo.toml`
- `crates/trading/src/onchain.rs`
- `parity/source-lock.yaml`
- `parity/fixtures/eip712/order_digests.json`
- `parity/fixtures/eip712/settlement_domain_separator.json`
- `parity/fixtures/contracts.json`

Primary regression coverage:

- `crates/contracts/tests/parity_contract.rs`
- `crates/contracts/src/primitives.rs::tests::domain_separator_matches_shared_parity_fixture`
- `crates/contracts/src/primitives.rs::tests::order_kind_marker_round_trips_and_rejects_unknown`
- `crates/contracts/tests/onchain_orders.rs::order_placement_topic0_matches_canonical_hash`
- `crates/contracts/tests/onchain_orders.rs::order_hash_matches_canonical_ethflow_foundry_vector`
- `crates/contracts/tests/onchain_orders.rs::eip1271_placement_decodes_owner_uid_and_trailer`
- `crates/contracts/tests/tokens_contract.rs::withdraw_selector_matches_canonical_keccak`
- `crates/contracts/src/eth_flow.rs::zero_receiver_invariant_matches_ethflow_on_chain_revert_selector`
- `crates/contracts/tests/property_contract.rs::ethflow_order_data_new_rejects_zero_receiver_iff_address_is_zero`
- `crates/signing/src/domain.rs::tests::domain_separator_matches_shared_parity_fixture`
- `crates/signing/tests/order_signing_contract.rs`
- `crates/trading/tests/onchain_contract.rs`
- `crates/trading/tests/quote_projection_parity.rs`
- `crates/core/tests/wire_format_preservation_contract.rs`
- `crates/core/tests/property_contract.rs`
- `crates/signing/tests/domain_contract.rs`
- `crates/signing/tests/cancellation_contract.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --all-features
cargo test -p cow-sdk-contracts --test property_contract
cargo test -p cow-sdk-contracts --test interaction_contract
cargo test -p cow-sdk-contracts --test onchain_orders
cargo test -p cow-sdk-contracts --test tokens_contract
cargo test -p cow-sdk-contracts --test parity_contract parity_fixture_cases_hold
cargo test -p cow-sdk-contracts domain_separator_matches_shared_parity_fixture
cargo test -p cow-sdk-signing domain_separator_matches_shared_parity_fixture
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo clippy -p cow-sdk-contracts --all-targets --all-features -- -D warnings
cargo test -p cow-sdk-trading --all-features --tests
cargo clippy -p cow-sdk-trading --all-targets --all-features -- -D warnings
```
