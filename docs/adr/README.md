# ADRs

This folder records the long-lived architectural decisions that define the
public and runtime shape of `cow-rs`.

## Index

| ADR | Status | Decision |
| --- | --- | --- |
| [0000](0000-template.md) | Template | Canonical ADR structure and writing contract. |
| [0001](0001-multi-crate-sdk-family-with-thin-facade.md) | Accepted | Keep a multi-crate workspace, an SDK-named crate family, and a thin root facade. |
| [0002](0002-dedicated-trading-orchestration-crate.md) | Accepted | Keep quote-to-order workflows in `cow-sdk-trading`. |
| [0003](0003-separate-read-only-subgraph-crate.md) | Accepted | Keep subgraph access in a separate read-only crate. |
| [0004](0004-feature-gated-browser-wallet-sidecar.md) | Accepted | Keep browser wallet support in a feature-gated sidecar crate. |
| [0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md) | Accepted | Keep runtime contracts boundary-specific and public Rust types strongly typed. |
| [0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md) | Accepted | Keep policy contracts explicit, review-visible, and instance-scoped. |
| [0007](0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md) | Accepted | Keep browser wallet support explicit, bounded, and aligned to the current browser-runtime seam. |
| [0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md) | Accepted | Grow new capability surfaces through additive leaf crates and owned sidecars. |
| [0009](0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md) | Accepted | Keep WASM examples as named verification consoles with one naming shape, one ship checklist, a two-tier proof posture, and a hybrid extensibility rule. |
| [0010](0010-runtime-neutral-async-and-transport-posture.md) | Accepted (superseded in part by [0013](0013-http-transport-injection-and-typestate-builders.md)) | Keep the async surface runtime-neutral with a `CancellationToken` contract, typed transport-error classification that strips the URL, and opt-in `tracing` instrumentation. |
| [0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md) | Accepted | Distinguish atomic and decimal-scaled amounts through dedicated newtypes and advertise `TradingSdkBuilder` prerequisites through typestate terminals. |
| [0012](0012-alloy-sol-bindings-and-registry-authority.md) | Accepted | Generate every ABI binding through `alloy::sol!` and resolve every deployed address through a single typed `Registry` authority. |
| [0013](0013-http-transport-injection-and-typestate-builders.md) | Accepted | Route orderbook and subgraph dispatch through the typed `HttpTransport` seam in `cow-sdk-core`, and construct both public clients through typestate builders. |
| [0014](0014-eip1271-verification-cache.md) | Accepted | Thread a pluggable `Eip1271VerificationCache` through `verify_eip1271_signature_async` and cache only magic-value-match and explicit mismatch outcomes. |
| [0015](0015-client-side-order-bounds-validator.md) | Accepted | Run the typed `OrderBoundsValidator` as the mandatory pre-transport step on every public trading submission seam and surface failures through `TradingError::ClientRejected(ClientRejection)`. |
| [0016](0016-split-sell-and-buy-token-balance-enums.md) | Accepted | Split the sell-side allowance path and the buy-side payout path into distinct `SellTokenSource` and `BuyTokenDestination` enums and reject cross-side coercion at the type system. |
| [0017](0017-typed-orderbook-rejection-parser.md) | Accepted | Classify non-2xx orderbook responses through a typed `OrderbookRejection` enum with a permanent `Unknown { code, message }` fallback and promote the typed payload onto `OrderbookError::Rejected`. |
| [0018](0018-typed-app-data-merge.md) | Accepted | Run quote-to-post app-data edits through a single typed merge pipeline and retire the opaque `serde_json::Value`-taking merge helper so the typed `signer`, `flashloan`, and `metadata.hooks` replacement semantics stay enforced end-to-end. |
| [0019](0019-http-transport-sole-dispatch.md) | Accepted | Make `HttpTransport` in `cow-sdk-core` the sole live-dispatch surface on `OrderBookApi` and `SubgraphApi` and carry non-2xx responses through the typed `TransportError::HttpStatus` channel. |
| [0020](0020-ethflow-owner-threading.md) | Accepted | Thread the signer-derived owner onto `EthFlowTransaction` and read `tx.from` (not `tx.order_to_sign.receiver`) when building the pre-HTTP validator preview on the native-currency submission seam. |
| [0021](0021-orderbook-total-fee-policy.md) | Accepted | Define `Order.total_fee` narrowly as the canonical executed-fee component and surface the deprecated `executedFeeAmount` wire field as a typed read-only sibling so consumers compute any legacy summation explicitly. |

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
- Allowed statuses are `Proposed`, `Accepted`, `Rejected`, and `Superseded`.
- Use four-digit numbering and kebab-case filenames.

## Status Semantics

- `Proposed`: under discussion and not yet binding.
- `Accepted`: active architectural record that later work must respect.
- `Rejected`: considered seriously and explicitly not chosen.
- `Superseded`: previously accepted, now replaced by a later ADR.

## History Contract

- Accepted ADRs are append-only records.
- If the decision changes materially, write a new ADR and link the old and new
  records.
- Do not rewrite old ADRs to make history look cleaner.
- Small corrections are fine when they do not change the recorded decision.

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
