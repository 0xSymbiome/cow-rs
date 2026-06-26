# ADRs

This folder records the long-lived architectural decisions that define the
public and runtime shape of `cow-rs`.

## Index

| ADR | Status | Decision |
| --- | --- | --- |
| [0000](0000-template.md) | Template | Canonical ADR structure and writing contract. |
| [0001](0001-multi-crate-sdk-family-with-thin-facade.md) | Accepted | Keep a multi-crate workspace, an SDK-named crate family, and a thin root facade. |
| [0002](0002-dedicated-trading-orchestration-crate.md) | Accepted | Keep quote-to-order workflows in `cow-sdk-trading`. |
| [0003](0003-separate-read-only-subgraph-crate.md) | Accepted | Keep subgraph access in a separate read-only crate, re-exported behind the off-by-default `subgraph` facade feature. |
| [0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md) | Accepted | Keep runtime contracts boundary-specific and public Rust types strongly typed. |
| [0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md) | Accepted | Keep policy contracts explicit, review-visible, and instance-scoped. |
| [0010](0010-runtime-neutral-async-and-transport-posture.md) | Accepted; superseded in part by [0013](0013-http-transport-injection-and-typestate-builders.md) | Keep the async surface runtime-neutral with a `CancellationToken` contract, typed transport-error classification that strips the URL, and opt-in `tracing` instrumentation. |
| [0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md) | Accepted | Make `Amount` the single atomic newtype with on-type decimal I/O (no second amount type) and advertise `TradingBuilder` prerequisites through typestate terminals. |
| [0012](0012-alloy-sol-bindings-and-registry-authority.md) | Accepted | Generate every ABI binding through `alloy::sol!` and resolve every deployed address through a single typed `Registry` authority. |
| [0013](0013-http-transport-injection-and-typestate-builders.md) | Accepted | Route orderbook and subgraph dispatch through the typed `HttpTransport` seam in `cow-sdk-core`, and construct both public clients through typestate builders. |
| [0014](0014-eip1271-verification-cache.md) | Accepted | Thread a pluggable positive-only `Eip1271Cache` through `verify_eip1271_signature_cached`, keyed on `(verifier, digest, signature_hash)` and recording only magic-value-match outcomes; the SDK ships the trait and `NoopEip1271Cache`, and a consumer implements the trait to memoize. |
| [0015](0015-client-side-order-bounds-validator.md) | Accepted | Run the typed `OrderBoundsValidator` as the mandatory pre-transport step on every public trading submission seam and surface failures through `TradingError::ClientRejected(ClientRejection)`. |
| [0016](0016-split-sell-and-buy-token-balance-enums.md) | Accepted | Split the sell-side allowance path and the buy-side payout path into distinct `SellTokenSource` and `BuyTokenDestination` enums and reject cross-side coercion at the type system. |
| [0017](0017-typed-orderbook-rejection-parser.md) | Accepted | Classify non-2xx orderbook responses through a typed `OrderbookRejection` enum with a permanent `Unknown { code, message }` fallback and promote the typed payload onto `OrderbookError::Rejected`. |
| [0018](0018-typed-app-data-merge.md) | Accepted | Run quote-to-post app-data edits through a single typed merge pipeline and retire the opaque `serde_json::Value`-taking merge helper so the typed `signer`, `flashloan`, and `metadata.hooks` replacement semantics stay enforced end-to-end. |
| [0020](0020-ethflow-owner-threading.md) | Accepted | Thread the signer-derived owner onto `EthFlowTransaction` and read `tx.from` (not `tx.order_to_sign.receiver`) as the owner passed to the pre-HTTP validator on the native-currency submission seam. |
| [0021](0021-orderbook-total-fee-policy.md) | Accepted | Define `Order.total_fee` narrowly as the canonical executed-fee component and surface the deprecated `executedFeeAmount` wire field as a typed read-only sibling so consumers compute any legacy summation explicitly. |
| [0022](0022-ecdsa-signature-v-normalization.md) | Accepted | Canonicalize recoverable ECDSA signatures at the contracts boundary so every emitted signature carries a Solidity-compatible `27` / `28` recovery byte. |
| [0024](0024-asyncprovider-asyncsigningprovider-capability-split.md) | Accepted | Split `Provider` into a read-only chain-RPC trait and a `SigningProvider` extension that owns signer creation. |
| [0025](0025-workspace-url-redaction-convention.md) | Accepted | Store credential-bearing URL fields in redacting types before they become public SDK state. |
| [0026](0026-alloy-major-release-absorption-plan.md) | Accepted | Bound alloy major releases behind SDK-owned types and a configurable scheduled canary lane. |
| [0027](0027-post-quantum-signing-absorption-plan.md) | Accepted | Add future signing schemes through non-exhaustive signature boundaries without widening ECDSA semantics. |
| [0028](0028-account-abstraction-integration-plan.md) | Accepted | Integrate account abstraction through provider capability traits and EIP-1271-compatible signing surfaces. |
| [0030](0030-workspace-locked-versioning-tag-baseline.md) | Accepted | Keep workspace crate versions locked through `0.x` and run patch semver checks against the previous release tag. |
| [0031](0031-wire-dto-openapi-driven-with-order-auction-order-split.md) | Accepted | Drive orderbook response DTO coverage from the source-lock OpenAPI inventory; the original `Order`/`AuctionOrder` split collapsed to a single `Order` type after the auction read proved non-public. |
| [0032](0032-deployment-authority-machine-readable-provenance.md) | Accepted | Back deployment-address authority with machine-readable provenance and dual-mode live confirmation. |
| [0033](0033-minimum-viable-panic-surface.md) | Accepted | Keep production panic sites allowlisted, documented, and limited to static invariants. |
| [0035](0035-alloy-provider-adapter.md) | Accepted | Alloy adapter family: read-only provider, local signer, and composed umbrella as three native crates (consolidates the former ADR 0036 and ADR 0037). |
| [0038](0038-transaction-lifecycle-types.md) | Accepted | Split transaction broadcast acknowledgement from mined receipt observation. |
| [0039](0039-typescript-callable-wasm-sdk-surface.md) | Accepted | Keep the TypeScript-callable WASM SDK surface as an additive leaf crate. |
| [0040](0040-wallet-provider-callback-boundary-for-js-consumers.md) | Accepted | Keep wallet and provider interop behind typed JavaScript callbacks. |
| [0041](0041-transport-policy-l3-layering.md) | Accepted | Share retry, rate-limit, cooldown, and classification policy across HTTP clients. |
| [0044](0044-bundle-size-profile-and-flavor-builds.md) | Accepted | Ship feature-scoped WASM flavor builds from one package. |
| [0045](0045-async-signer-trait-narrowing.md) | Accepted | Narrow async signer capabilities by operation. |
| [0048](0048-composable-conditional-order-framework.md) | Proposed (deferred) | Plan the composable conditional-order framework as a deferred additive leaf crate (not yet rooted), bounded by the watch-tower boundary. |
| [0049](0049-cow-shed-account-abstraction-proxy.md) | Accepted | Ship COW Shed account-abstraction proxy support as a feature-gated module of `cow-sdk-contracts` behind the `cow-shed` feature. |
| [0050](0050-eip1271-signature-blob-encoding.md) | Accepted | Recognise exactly two EIP-1271 payload shapes through distinct encoder entry points selected at signer construction. |
| [0051](0051-signing-owned-eip1271-signature-provider-trait.md) | Accepted | Own `Eip1271Signer` in `cow-sdk-signing` and forbid any downstream re-export so the canonical path stays single-rooted. |
| [0052](0052-alloy-primitives-canonical-primitive-layer.md) | Accepted | Adopt `alloy_primitives` and `alloy_sol_types` as the canonical primitive and EIP-712 / ABI layer across the workspace, with the cow-named identity and numeric types resolving through cow-owned `#[repr(transparent)]` newtypes over the corresponding `alloy_primitives` type while preserving wire byte identity, parity fixture coverage, and the Solidity-compatible signature posture from ADR 0022. |
| [0053](0053-typed-signer-rejection-classification.md) | Accepted | Classify EIP-1193 user rejections through a shared `cow_sdk_core::UserRejection` trait so the signing crate emits a typed `SigningError::SignerRejection` variant across signer implementations. |
| [0054](0054-onchain-order-event-decoding-is-fail-closed.md) | Accepted | Decode `CoWSwapOnchainOrders`, `CoWSwapEthFlow`, and `GPv2Settlement` event logs through a fail-closed, provider-free decoder family that validates every field and never panics on adversarial input. |
| [0055](0055-bounded-response-reads.md) | Accepted | Bound every SDK-owned HTTP response read by a configurable per-client `max_response_bytes`, refuse an over-limit body with a typed non-retryable `TransportErrorClass::ResponseTooLarge` outcome measured on decoded bytes, and length-bound signature hex before decode. |
| [0057](0057-log-provider-capability-trait.md) | Accepted | Add an opt-in `LogProvider: Provider` capability supertrait whose single-call `get_logs` fetches event logs, mirroring the `SigningProvider` split and feeding the fail-closed decoders. |
| [0058](0058-typed-quote-request-response-surface.md) | Accepted | Mirror the orderbook `OrderParameters` quote payload in `QuoteData` with its own OpenAPI coverage target, default `priceQuality` to `optimal`, keep the quote network-cost fields read-only, lock the quote-amounts projection with a parity test, and echo-verify the request-determined response fields (failing closed with `QuoteEchoMismatch`). |
| [0059](0059-hash-concrete-orderdata-directly.md) | Accepted | Hash the concrete `cow_sdk_core::OrderData` directly and remove the contracts-layer `Order` / `NormalizedOrder` types and the `GPv2Order` re-export, collapsing the order-type topology to one concrete type. |
| [0060](0060-uniform-error-classification.md) | Accepted | Relocate the shared `ErrorClass` to `cow-sdk-core` and give every facade-family error type a `class()` accessor (facade `CowError::class()` delegates), while the native Alloy adapters keep their own per-type class enums per ADR 0053. |
| [0061](0061-wasm-abi-receiver-pay-to-owner.md) | Accepted | Treat an omitted and an explicit zero-address `receiver` identically at the WASM order boundary (both resolve to the zero-address pay-to-owner sentinel), with no receiver-to-owner reinterpretation. |
| [0062](0062-internal-shared-test-support-crate.md) | Accepted | Keep shared cross-crate test support in one unpublished `cow-sdk-test-utils` crate consumed only as a dev-dependency. |
| [0063](0063-published-consumer-test-doubles-crate.md) | Accepted | Ship consumer-facing in-memory test doubles for the public trait seams as the published `cow-sdk-test` crate, re-exported behind the facade `testing` feature. |
| [0064](0064-app-data-typed-validation.md) | Accepted | Validate app-data documents through typed Rust construction plus structural checks, not a runtime JSON-Schema validator, keeping one self-contained drift fixture per modeled metadata family. |
| [0066](0066-trading-slippage-and-suggestion-policy.md) | Accepted | Implement the established CoW SDK slippage transform, fee folding, and slippage-suggestion heuristics faithfully, byte-for-byte with `@cowprotocol/cow-sdk`, without redefining the convention. |
| [0067](0067-idiomatic-accessor-naming.md) | Accepted | Name public accessors and domain fetch methods by their bare domain noun with no `get_` prefix, retaining `get_` only on the chain-RPC `Provider` / `LogProvider` methods that mirror Ethereum JSON-RPC names. |
| [0068](0068-payload-only-typed-data-signing.md) | Accepted | Take the canonical EIP-712 typed-data payload at the signer seam — `sign_typed_data_payload(&TypedDataPayload)` is the single required typed-data method — and keep field-based signing out of the trait contract; the former wallet-compatibility carve-out is voided. |
| [0069](0069-layered-trading-operation-surface-and-signing-free-transport.md) | Accepted | Offer trading operations at layered free-function, bound-method, and fluent-builder entries (swap and limit) that thin-delegate downward, and keep the order-lifecycle builders in `cow-sdk-trading` so `cow-sdk-orderbook` and `cow-sdk-subgraph` stay signing-free typed transport clients. |
| [0070](0070-onchain-transaction-helper-boundary.md) | Accepted | Ship one single-call transaction builder per on-chain trade step (approve, wrap/unwrap, eth-flow, pre-sign, cancel) and add native-asset `wrap_transaction` / `unwrap_transaction` under that boundary. |
| [0071](0071-wasm-component-distribution-channel.md) | Proposed (deferred) | Plan a WebAssembly Component distribution channel parallel to the wasm-bindgen npm lane; the crate builds in CI, while its OCI/Release distribution and cross-runtime parity machinery are deferred. |

## Superseded And Rejected

These decisions are retired. The numbers are permanent identifiers and are never
reused, so the gaps in the index above are intentional; the full text of each
retired record lives in git history, and its current home is named below.

| ADR | Status | Disposition |
| --- | --- | --- |
| 0004 | Superseded | Feature-gated browser-wallet sidecar; the native browser-wallet crate was retired — see [0039](0039-typescript-callable-wasm-sdk-surface.md) and [0040](0040-wallet-provider-callback-boundary-for-js-consumers.md). |
| 0007 | Superseded | Bounded native browser-wallet posture; browser/wallet integration now served by [0039](0039-typescript-callable-wasm-sdk-surface.md) and [0040](0040-wallet-provider-callback-boundary-for-js-consumers.md). |
| 0008 | Superseded | Additive-capability growth rule folded into the multi-crate-family decision in [0001](0001-multi-crate-sdk-family-with-thin-facade.md). |
| 0009 | Superseded | WASM verification-console example genre retired in favour of the `cow-sdk-wasm` surface in [0039](0039-typescript-callable-wasm-sdk-surface.md). |
| 0019 | Superseded | Sole-live-dispatch invariant folded into [0013](0013-http-transport-injection-and-typestate-builders.md). |
| 0023 | Superseded | Legacy-shim removal folded into the single `OrderData` order-identity path in [0059](0059-hash-concrete-orderdata-directly.md). |
| 0029 | Rejected | Proposed evolving public traits through `*Ext` extension traits; rejected — the SDK owns its traits and grows new RPC primitives through opt-in capability supertraits ([0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [0057](0057-log-provider-capability-trait.md)). |
| 0034 | Superseded | Vault-relayer interaction-target guard; retired when the settlement encoder was removed (a solver/backend concern). |
| 0036 | Superseded | Consolidated into the Alloy adapter family ([0035](0035-alloy-provider-adapter.md)). |
| 0037 | Superseded | Consolidated into the Alloy adapter family ([0035](0035-alloy-provider-adapter.md)). |
| 0042 | Superseded | Pure-helpers crate folded into the `cow-sdk-wasm::helpers` module ([0039](0039-typescript-callable-wasm-sdk-surface.md)). |
| 0043 | Superseded | Callback-registry internalization folded into the WASM surface ([0039](0039-typescript-callable-wasm-sdk-surface.md)). |
| 0046 | Superseded | JavaScript transport-policy exposure folded into the WASM surface ([0039](0039-typescript-callable-wasm-sdk-surface.md)). |
| 0047 | Superseded | TypeScript-facade-as-public-surface folded into the WASM surface ([0039](0039-typescript-callable-wasm-sdk-surface.md)). |
| 0056 | Superseded | Settlement event decoding folded into the on-chain event-decoding ADR [0054](0054-onchain-order-event-decoding-is-fail-closed.md). |
| 0065 | Superseded | Canonical browser-wallet example; retired with the browser-wallet crate — see [0039](0039-typescript-callable-wasm-sdk-surface.md). |

## When To Write An ADR

- Use an ADR for a long-lived, cross-cutting rule that affects package
  topology, public API shape, runtime behavior, support posture, security
  boundaries, or semver expectations.
- Use an ADR when a decision changes what later implementation, verification,
  review, or release work must preserve.
- Do not use an ADR for delivery sequencing, operational history, verification
  workflow mechanics, or one-off implementation detail.

## Lifecycle Fit

- ADRs are design-history records. They explain the durable rule that later
  implementation, testing, review, and documentation must keep true.
- ADRs should justify lasting complexity, not retell the delivery timeline.
- Keep authoring and delivery detail out of the main body unless it changes the
  long-lived design itself.

## Retiring An ADR

- When a decision is replaced, mark the old record `Superseded` (or `Rejected`)
  and record it in the **Superseded And Rejected** table with the link to its
  current home. Numbers are permanent identifiers and are never reused; the gaps
  this leaves in the index are intentional.
- The retired record's full text stays in git history rather than as a standalone
  file, so the active folder lists only living decisions.

## Audit Link Contract

- Add a standing audit to an accepted ADR when that audit is the clearest
  current-state proof for one of the ADR's invariants.
- Prefer the ADR `Links` section for standing audits so the main body stays
  focused on the durable rule rather than the current review snapshot.
- Mark proof-bearing audits with a `**Proven by:**` label inside the `Links`
  section, placed after the general support links, and list each audit on its
  own line. The label makes the audit-to-decision proof relationship
  reviewable at a glance without disturbing the fixed `Links` anchor heading.
- Keep the top `Related` metadata focused on directly coupled ADRs or other
  navigation links that belong beside the decision header.
- When an accepted ADR points to a standing audit as current-state proof, the
  audit reciprocates by naming the ADR under `Related docs`. Cross-linking
  is expected to be reciprocal: every audit listed under an ADR's
  `**Proven by:**` block names that ADR in its `Related docs`, and every
  audit whose `Related docs` names an ADR is listed under that ADR's
  `**Proven by:**` block.

## Title Contract

- Titles state the chosen rule, not just the topic area.
- Prefer names that answer "what was decided?" without opening the file.
- If the title cannot be written as one concrete rule, the ADR probably holds
  more than one decision family.

## Format Contract

- Lead with the decision so a reader understands the rule before the history.
- Keep one decision family per ADR.
- Keep the rationale short and focused on why the rule exists.
- Make the durable invariants explicit for public surface, runtime or support
  posture, and validation expectations.
- Keep alternatives short and concrete.
- Put supporting material in `Links` instead of burying it in long prose.
- If a reader cannot answer "what was decided, why, and what must remain true"
  in under a minute, the ADR is too long.

## Anchor Contract

- Use the same section headings in every ADR:
  `Decision`, `Why`, `Must Remain True`, `Alternatives Rejected`, `Links`.
- Do not rename accepted ADR files casually.
- Do not rename section headings once other docs deep-link to them.
- If a structural migration is ever unavoidable, do it repository-wide in one
  controlled pass.

## Metadata Contract

- `Status`, `Date`, and `Authors` are required.
- `Authors` use Markdown links. Example:
  `[0xSymbiotic](https://github.com/0xSymbiotic)`.
- `Tags`, `Related`, `Supersedes`, and `Superseded by` are optional. Omit them
  when empty.
- Allowed statuses are `Proposed`, `Proposed (deferred)`, `Accepted`,
  `Rejected`, and `Superseded`. (Pre-1.0 the ADR set is consolidated into clean
  current-state records, so the historical `Accepted (amended)` status is
  retired — a corrected ADR is simply `Accepted`.)
- Use four-digit numbering and kebab-case filenames.

## Status Semantics

- `Proposed`: under discussion and not yet binding.
- `Proposed (deferred)`: the design is decided but the implementing crate is not
  yet rooted in the workspace; present-tense claims describe the planned shape.
- `Accepted`: active architectural record that later work must respect.
- `Rejected`: considered seriously and explicitly not chosen.
- `Superseded`: previously accepted, now replaced by a later ADR.

## History Contract

- Through the pre-1.0 cycle, an amendment-heavy ADR may be consolidated into a
  single clean current-state record; the amendment-by-amendment history stays in
  git. The aim is a record a reader can absorb in under a minute, not a changelog
  of how the decision evolved.
- Once the first functional release ships, accepted ADRs become append-only: if
  the decision changes materially, write a new ADR and link the old and new
  records rather than rewriting in place.
- Small corrections that do not change the recorded decision are always fine.

## Writing Rules

- Prefer short paragraphs and flat bullets.
- Use concrete crate names, features, and runtime surfaces.
- Avoid repository-internal process jargon in the main body.
- State what must remain true, not just what was convenient at the time.
- Keep support claims and compatibility language bounded and precise.
- Target roughly `200` to `400` words. If an ADR wants more, split the
  decision or link supporting docs.

## Author Checklist

- Does the title state the rule rather than the topic?
- Does `Decision` stand on its own without background?
- Does `Why` explain necessity rather than retell implementation history?
- Does `Must Remain True` capture the future constraints other docs must keep?
- Are links limited to durable supporting material?

## Review Checklist

- Could a new contributor understand the rule in under a minute?
- Would the ADR still make sense if issue, PR, and chat history vanished?
- Does it avoid overclaiming support, compatibility, or behavior?
- Is this truly one decision family?
- Would a later contradictory change clearly require a new ADR?
