# WASM Surface Audit

Status: Current
Last reviewed: 2026-06-19
Owning surface: the `cow-sdk-wasm` TypeScript-callable crate, its npm package layout/exports, the JavaScript callback runtime boundary, DTO/type generation, schema-versioned envelopes, the size-budget gate, unsupported-target diagnostics, and the deterministic browser test runner.
Refresh trigger: Changes to `crates/wasm/src/**`, exported DTOs or `tsify` usage, wasm-pack targets, declaration/facade snapshots, package export maps, callback shapes or registry ownership, the `JsCallbackHttpTransport` contract, transport-policy or error-envelope schema, release-profile size settings or measured budgets, native Alloy adapter `wasm32` guards, or the wasm-pack browser lanes.
Related docs:
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0040](../adr/0040-wallet-provider-callback-boundary-for-js-consumers.md)
- [ADR 0044](../adr/0044-bundle-size-profile-and-flavor-builds.md)

## Scope

This audit covers:

- the four-layer `cow-sdk-wasm` public surface (deterministic helpers, wallet
  callbacks, service clients, trading) and its mapping to the native `cow-rs`
  crates, including capabilities intentionally not surfaced
- the JavaScript callback boundary: typed wallet/signer/cancellation/EIP-1271
  callbacks, the fetch-callback registry, and the `JsCallbackHttpTransport`
- DTO/type generation through `tsify`, raw and facade declaration snapshots,
  and schema-versioned success/error envelopes
- the TypeScript facade as the public contract and its raw-export denylist
- npm export maps for browser bundlers, Node.js, and Cloudflare Workers
- flavor builds and the size-budget release gate
- the unsupported-target diagnostics for native Alloy adapters on `wasm32`
- the headless Firefox browser test runner

It does not cover npm publication or package-name ownership, live wallet-vendor
or on-chain verification behavior, service API schema evolution, or third-party
bundler behavior.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Surface layering | The four layers — deterministic helpers (host-safe in `cow-sdk-wasm::helpers`), wallet callbacks, service clients, trading — are present and contract-tested; wasm-bindgen exports own JS interop | Conforms |
| Workflow + capability coverage | The ADR 0039 / `docs/parity.md` workflow set is exposed; every non-surfaced native capability is classified with a rationale | Conforms / Documented |
| Runtime-model boundary | The wasm32 tree excludes native Alloy adapters, reqwest, and hyper; no Rust signer broadcasts and no provider polls | Conforms |
| Shape correspondence | Native types/signatures map to the WASM DTO + TS surface through a fixed transform set; divergences beyond it are enumerated | Documented |
| Wallet/signer callbacks | Typed-data, EIP-1193, digest, and custom EIP-1271 callbacks are named, explicit, capability-scoped, and fail closed | Conforms |
| HTTP callback transport | `JsCallbackHttpTransport` owns timeout, abort signal, internal callback retention, and typed error mapping | Conforms |
| Event decoding | `decodeSettlementLog` / `decodeEthFlowLog` produce typed events with no network access and fail closed on malformed input | Conforms |
| Type generation + snapshots | Cross-ABI DTOs are `tsify`-generated; one raw snapshot per flavor catches drift and asserts per-target agreement; map fields declare `Record<...>` to match the runtime shape | Conforms |
| Facade + API stability | Public imports resolve through compiled facade modules; raw wasm-bindgen output is package-internal and denied as a public import target | Conforms |
| Schema versioning | Success envelopes carry `schemaVersion`; unknown variants round-trip behind a scoped `__unknown` sentinel; facade normalizes raw failures to `CowError` | Conforms |
| Error posture | `WasmError` (aliased `CowError`) preserves typed redaction; input-DTO deserialization failures map to `invalidInput`, not `internal`; the `orderbook` variant carries `retryable` + optional `retryAfterMs` | Conforms |
| Performance budget | Flavor builds expose feature-scoped subpaths; release artifacts run the size profile + wasm opt pass; raw/brotli/gzip budgets are recorded and gated, with a dedicated Cloudflare gzip budget | Conforms |
| Unsupported targets | Native Alloy adapter crates and the `alloy`/`alloy-provider`/`alloy-signer` facade features fail closed on `wasm32` with a compile-time diagnostic, CI-asserted | Conforms |
| Browser runner determinism | Browser lanes provision headless Firefox via pinned setup actions (pinned geckodriver, `latest-esr` Firefox); tests use in-test state + serde round trips | Conforms |

## Current Contract

### Surface and package exports

`cow-sdk-wasm` exposes four layers, sourced from the native crates rather than
reimplemented:

1. **Deterministic helpers** — domain separator, order typed-data, order-UID,
   app-data document/info/validation, CID and hash conversion, supported-chain
   and deployment-address lookup, EIP-1271 payload encoding, and the
   provider-free, fail-closed `decodeSettlementLog` / `decodeEthFlowLog`
   event-log decoders (they reconstruct borrowed log bytes and dispatch to the
   `cow-sdk-contracts` decoders without network access).
2. **Wallet-callback signing** — typed-data, EIP-1193, digest, EIP-1271, and
   custom EIP-1271 order signing; cancellation signing; and the pre-sign and
   cancellation transaction builders.
3. **Service clients** — `OrderBookClient`, `SubgraphClient`, and `IpfsClient`
   over default or callback HTTP.
4. **Trading** — `TradingClient` quote and post flows, including the
   EIP-1271-backed swap path, the native-currency-sell transaction builder, and
   the vault-relayer approval transaction builder.

The package keeps one installable npm package while exposing flavor-specific
public subpaths (`default`, `orderbook`, `signing`, `trading`). Public
imports resolve through compiled facade subpaths, never generated `dist/raw`
paths; the export-map verifier walks string and conditional exports, asserts
every target exists, and rejects nested wasm-pack metadata in `dist`. Every
flavor is built for the `bundler`, `nodejs`, `web`, and source-phase `module`
targets. Each flavor's `node` condition resolves the nodejs CommonJS build; its
`browser`, `import`, `default`, and edge conditions (`workerd`, `worker`, `deno`,
`edge-light`, `bun`) and the explicit `…/edge` subpath all resolve the web build,
whose `new URL(import.meta.url)` loader is portable where the bundler target's
`import * as wasm` ESM integration is not. Browser callers run `initialize()`
once; Cloudflare Workers, which cannot compile WebAssembly from bytes at runtime,
pass the precompiled module from `…/edge/wasm` to `initialize` and use no
dynamic-compilation or streaming-instantiation APIs. Each flavor's `…/module`
subpath is the standards-track source-phase build (`import source` / Wasm ESM
Integration), auto-initializing and opt-in (Node 24, Deno, esbuild today), driven
through `wasm-bindgen --target module` because wasm-pack does not emit it. No
flavor is browser-portable while another is bundler-only — `default`, `orderbook`,
`signing`, and `trading` ship the same target coverage.

### Capability coverage

Native operations map to WASM exports under uniform transforms; the canonical
inventory is pinned at the declaration level by `wasm_snapshot_surface_contract.rs`
and exercised behaviorally by `wasm_surface_contract.rs` and
`wasm_workflow_coverage_contract.rs`. Orderbook reads/writes (`quote`,
`send_order`, `send_cancellations`, `order(s)`, `trades`, `native_price`,
`app_data`, `version`, `order_link`, lookup, status/surplus, and the v2
solver-competition routes) are surfaced. Trading surfaces quote/post/limit/swap
plus builder-form transactions (`buildPresignTx`, `buildCancelOrderTx`,
`buildSellNativeCurrencyTx`, `buildSellNativeCurrencyTxFromQuote`,
`buildApprovalTx`) and the allowance read, completing the
read-allowance-then-approve path. `buildSellNativeCurrencyTxFromQuote` is the
native-sell sibling of `postSwapOrderFromQuote`: it derives the EthFlow
transaction from a `getQuote` result, failing closed when the quote was not a
native-currency sell. Signing surfaces typed-data,
EIP-1193, digest, EIP-1271, and cancellation signing plus the deterministic
helpers. App-data, subgraph (totals, daily/hourly volume, arbitrary GraphQL),
and the consumer-relevant contracts surface (decoders, deployment lookup,
builder calldata) are surfaced.

Intentionally **not surfaced**, each by stated rationale: (1) managed
broadcast/receipt flows (`approve_cow_protocol`, `poll_for_receipt`,
`submit_and_wait_for_receipt`) — `cow-sdk-wasm` is a callback leaf, the JS host
owns the wallet, event loop, and provider, and the native Alloy adapters are
native-only; the upstream TS SDK draws the same line. (2) On-chain EIP-1271
verification and its caches — outside the defined workflow scope, no upstream
core-surface analogue. (3) The low-level `contracts` encoding/verification
surface — internal building-block code on every target. (4) The composable
conditional-order framework — a deferred capability (ADR 0048) on every target,
not WASM-specific.

### Type generation and schema versioning

Types crossing the ABI live in the `exports` module tree and derive their TS
shape there via `tsify`; host-safe helpers in `cow-sdk-wasm::helpers` compile
natively without wasm-bindgen, JsValue, or tsify-derived public types. The
cross-ABI serializer is `serde_wasm_bindgen::Serializer::json_compatible`, which
emits plain objects for Rust `BTreeMap`/`HashMap` fields, so those fields carry
an explicit `#[tsify(type = "Record<...>")]` override so the declared shape
matches the runtime shape. Decoded event DTOs are internally tagged unions
(serde `tag = "kind"`).

One committed raw declaration per flavor under `crates/wasm/snapshots/raw/`
represents the public TypeScript contract. wasm-bindgen emits a byte-identical
`.d.ts` for every wasm-pack target of a flavor (the type surface is
loader-independent; only the JS loader glue and `.wasm` packaging differ), so
the workflow diffs every target's generated declaration against the single
per-flavor snapshot — detecting export drift and asserting the targets agree,
failing closed on any future per-target divergence. Declarations using
`[Symbol.dispose]` must include the `esnext.disposable` reference. Facade
snapshots under `crates/wasm/snapshots/facade/` are checked separately so
generated implementation classes do not become the published contract.

Success envelopes serialize through `WasmEnvelope<T> = { schemaVersion: "v1" |
"__unknown"; value: T }`, identifying the JS-visible shape (not the service
schema). Three deterministic helpers (`domainSeparator`, `supportedChainIds`,
`wasmVersion`) return bare values. Unknown enum variants round-trip behind a
scoped `__unknown` sentinel that keeps the raw payload while preventing
misclassification.

### Callback boundary

The package exposes named, capability-scoped callbacks rather than raw provider
objects: `TypedDataSignerCallback`, `Eip1193RequestCallback`,
`DigestSignerCallback`, `CustomEip1271Callback`, and `CowFetchCallback`; each
receives a typed payload/request DTO and may return a value, Promise, or
thenable. Each signing/cancellation function requests only the callback it needs.
When a cow identity newtype (`Address`, `Hash32`, `AppDataHash`, `HexData`,
`OrderUid`) or the `Amount` newtype crosses the boundary, the ABI shape is the
canonical lowercase `0x`-hex string (identity) or strict-decimal string
(`Amount`), via a `Tsify` derive gated to `target_family = "wasm"` (ADR 0052).

Callback registry state is implementation-owned: public TS declarations expose
no registry classes, ids, or handle constructors. Facade clients retain
callbacks for the owning client's lifetime, scoped to one wasm module instance,
and release them on disposal. Per-call options carry `signal` and `timeoutMs`;
signing options also carry `walletConfig.timeoutMs`. HTTP callback requests
receive a live `AbortSignal`; abort and timeout paths clean up listeners and
timer handles. Callback throws, rejects, malformed outputs, timeout overflow,
and aborts all map to typed errors.

### Facade architecture and API stability

The facade modules under `crates/wasm/npm/src/**` adapt raw wasm-bindgen output
into stable TypeScript classes, helpers, and config objects, and are the public
package contract. Raw binding imports remain behind package-internal adapter
modules; verification scripts reject public raw export entries and the facade
denylist, and facade snapshots assert raw wasm-bindgen classes do not leak.
Facade clients own callback retention and expose explicit `dispose`. Errors
crossing the facade normalize into schema-versioned `CowError` envelopes with
redacted, low-cardinality fields; input-DTO deserialization failures at the wasm
boundary (unknown enum variant, missing required field, wrong field type)
normalize to `invalidInput`, leaving `internal` for genuine SDK-side faults.
HTTP-capable constructors accept a single typed config object including
`TransportPolicyConfig`, translated into the shared Rust policy and rejecting
invalid values.

### Performance budget

Default, orderbook, signing, and `trading` flavors each have their own facade
declarations and a raw wasm snapshot. Every flavor is built for the `bundler`,
`nodejs`, and `web` targets plus the source-phase `module` build; the `bundler`
and `nodejs` raw builds back the facade ESM and CommonJS entries, while the `web`
and `module` targets' declarations add only wasm-bindgen's standard target
scaffolding on top of the bundler surface and so are not snapshotted separately
(the facade snapshot pins the public `initialize` contract). Each flavor's
`bundler`, `web`, and `module` targets emit a byte-identical wasm binary, so the
package ships one binary per flavor; the `…/edge/wasm` Worker module, the web
glue's default loader URL, and the module glue's `import source` specifier all
reuse the bundler copy. Release artifacts run through the size-oriented
release profile and a wasm optimization pass during package generation. The npm
README records current raw, brotli, gzip, and gate values per flavor. Each
flavor's gzip budget is an explicit byte budget below Cloudflare's published
Workers Free compressed-size limit (safety margin avoids MB/MiB ambiguity). End-to-end Cloudflare support additionally depends on
`wrangler deploy --dry-run` release-bundle verification and a Worker
startup-time gate against the 1-second startup limit, tracked separately.

### Unsupported-target diagnostics

Each native Alloy adapter crate (`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`,
`cow-sdk-alloy`) fails closed on `wasm32` with a compile-time diagnostic, and
enabling any of the `cow-sdk` facade features `alloy`, `alloy-provider`, or
`alloy-signer` on `wasm32-unknown-unknown` fails with the documented native-only
message. CI asserts all three facade features fail on wasm and treats a
successful wasm build as a failure. The documented browser path for wallet
signing is the `cow-sdk-wasm` EIP-1193 callback surface plus consumer-supplied
EIP-1193 provider reads. Residual risk: future upstream Alloy releases may add
browser-compatible provider components; until a separate browser-provider design
is accepted and tested, these adapters stay unsupported on wasm.

### Deterministic browser runner

Browser-targeted WASM tests run under headless Firefox. The compatibility lane
(`.github/workflows/wasm.yml`) installs Firefox via `browser-actions/setup-firefox`
on the `latest-esr` channel and geckodriver at a pinned version via
`browser-actions/setup-geckodriver`, then runs `wasm-pack test --headless
--firefox`. Provisioning through these setup actions keeps the lane off the
ambient runner image's drifting browser install and pins the WebDriver and
provisioning path (the browser channel itself tracks `latest-esr`). Firefox is
used because Chrome 148 with wasm-bindgen-test 0.3.x SIGKILLs ChromeDriver
mid-handshake on hosted runners, while the same release-profile binary runs
cleanly under Firefox and geckodriver. The tests run as `wasm_bindgen_test`
cases exercising the callback boundary against in-test state and serde round
trips, so determinism does not depend on the browser version, a live wallet, or
a live chain.

### Shape correspondence (systematic transforms)

A surfaced capability does not carry the native Rust shape unchanged. The public
consumer surface is the committed facade snapshot, which re-exports the
`tsify`-generated DTO types from the raw snapshot (raw wasm-bindgen output is
package-internal per ADR 0039). The fixed transforms: typestate builders →
single typed config object; trait-generic capability injection → JS callbacks
and `HttpTransportConfig`; typed input structs → camelCase input DTOs; typed
outputs → `WasmEnvelope<T>`; `Amount` → decimal `string`; address/UID/hash
newtypes → lowercase `0x` `string`; `serde_json::Value` → `unknown`; chain id /
quote id → `number` (quote id validated to the JS safe-integer range); Rust enums
→ string-literal unions; per-chain maps → `Record<string, string>`;
cancellation/timeout → `options?: { signal?; timeoutMs? }`; typed `Result` errors
→ rejected `Promise<WasmError>` (a `kind`-tagged discriminated union with
redacted, lower-cardinality fields); `async fn` → `Promise`-returning method;
Rust ownership release → explicit `free()` / `dispose()`.

Divergences beyond the uniform transforms: subgraph response payloads are
untyped (`Promise<WasmEnvelope<unknown>>`); `getOrders` decomposes the native
`OrdersQuery` into `(owner, pagination?)` and `getTrades` enforces its
exactly-one-of `owner`/`orderUid` constraint at runtime; signing/managed-post
take `owner: string` positionally (no Rust `Signer` resolves it); error
cardinality is reduced; `feeAmount` is structurally present for EIP-712
struct-hash compatibility but services accepts only `"0"`; client instances
require explicit release; and the native fluent `Trading::swap()` typestate
builder has no TS counterpart (its `Set`/`Unset` typestate cannot cross the ABI
and its safety is already provided by the named-field `SwapParametersInput` DTO —
covered through `postSwapOrder`, `postSwapOrderFromQuote`, and `getQuote`).

### Runtime support and open questions

Browser bundlers (`default-http-supported`), Node.js 22/24 LTS and Cloudflare
Workers (`callback-http-tested`) are claimed and CI-evidenced. Deno is
supported through the shipped web build — every flavor ships one (the same web
target as Cloudflare Workers), exercised in CI via `./trading/edge` — with the
runtime-neutral `CowFetchCallback`. Bun, Vercel Edge, and Fly.io remain
`best-effort` until dedicated fixtures and CI evidence exist; they share the same
web build.

## Evidence

Primary implementation points:

- `crates/wasm/src/helpers/`
- `crates/wasm/src/exports/`
- `crates/wasm/src/exports/dto/`
- `crates/wasm/src/exports/callbacks.rs`
- `crates/wasm/src/exports/registry.rs`
- `crates/wasm/src/exports/transport.rs`
- `crates/wasm/src/exports/signing.rs`
- `crates/wasm/src/exports/cancel.rs`
- `crates/wasm/src/exports/envelope.rs`
- `crates/wasm/src/exports/errors.rs`
- `crates/wasm/snapshots/raw/{default,orderbook,signing,trading}.d.ts`
- `crates/wasm/snapshots/facade/`
- `crates/wasm/npm/src/` (`index.ts`, `default.ts`, `orderbook.ts`, `signing.ts`, `trading.ts`, `callbacks.ts`, `internal.ts`, `options.ts`, `envelope.ts`, `errors.ts`, `raw/`)
- `crates/wasm/npm/package.template.json`
- `crates/wasm/npm/README.md`
- `crates/wasm/npm/scripts/` (`build.sh`, `compile-facade.sh`, `render-package-json.mjs`, `measure-wasm-size.mjs`, `dedupe-target-wasm.mjs`, `verify-exports.mjs`, `verify-no-raw-exports.mjs`, `verify-facade-denylist.mjs`, `verify-package-resolution.sh`)
- `crates/orderbook/src/api.rs`, `crates/trading/src/`, `crates/signing/src/`
- `crates/alloy-provider/src/lib.rs`, `crates/alloy-signer/src/lib.rs`, `crates/alloy/src/lib.rs`, `crates/sdk/src/lib.rs`
- `Cargo.toml`, `crates/wasm/Cargo.toml`
- `.github/workflows/wasm.yml`, `.github/workflows/ci.yml`
- `docs/providers/adapting-alloy.md`, `docs/transport.md`

Primary regression coverage:

- `crates/wasm/tests/host_pure_helpers.rs` (incl. `typed_data_payload_matches_signing_module_output`, `wasm_version_matches_package_version`)
- `crates/wasm/tests/wasm_surface_contract.rs` (incl. `order_typed_data_serializes_to_expected_js_shape`, `wasm_version_matches_crate_version`)
- `crates/wasm/tests/wasm_workflow_coverage_contract.rs`
- `crates/wasm/tests/wasm_snapshot_surface_contract.rs` (incl. `generated_type_declarations_version_errors_and_outputs`, `generated_type_declarations_hide_callback_registry`, `generated_type_declarations_name_callback_params`, `generated_type_declarations_expose_abort_and_wallet_options`, `generated_type_declarations_expose_transport_policy_config_for_http_flavours`, `generated_type_declarations_match_flavour_matrix`)
- `crates/wasm/tests/wasm_facade_snapshot_contract.rs` (`facade_declarations_match_flavour_matrix`, `facade_declarations_hide_raw_wasm_bindgen_surface`, `facade_declarations_expose_dispose_and_named_callback_types`)
- `crates/wasm/tests/wasm_envelope_contract.rs` (`envelope_serializes_schema_version_and_payload`, `envelope_preserves_unknown_schema_sentinel`)
- `crates/wasm/tests/wasm_error_abi_contract.rs` (`invalid_input_variant_round_trips`, `unknown_enum_variant_round_trips`, `unknown_sentinel_round_trips_raw_payload`)
- `crates/wasm/tests/wasm_callback_contract.rs` (`wallet_config_timeout_rejects_pending_signer_callback`, `typed_cancellation_signer_returns_order_uids`, `eip1193_cancellation_callback_shape_is_stable`)
- `crates/wasm/tests/wasm_callback_lifetime_contract.rs::client_owned_callback_survives_until_request_resolves`
- `crates/wasm/tests/wasm_callback_transport_contract.rs`
- `crates/wasm/tests/wasm_cancellation_contract.rs` (`abort_bridge_removes_listener_after_{success,callback_throw,callback_reject,parse_error,timeout_overflow}`)
- `crates/wasm/tests/wasm_transport_policy_contract.rs` (`all_client_constructors_accept_transport_policy`, `invalid_transport_policy_user_agent_is_rejected`)
- `crates/wasm/tests/wasm_fail_closed_contract.rs::flavour_descriptor_exposes_web_and_module_subpaths`
- `crates/wasm/tests/wasm_redaction_contract.rs`
- `crates/wasm/tests/transport_fetch_smoke.rs`
- `tests/wasm_dependency_invariant.rs`
- `crates/wasm/npm/tests/` (`facade-default.test.ts`, `facade-orderbook.test.ts`, `facade-signing.test.ts`, `facade-cancellation.test.ts`, `facade-resource-cleanup.test.ts`, `facade-error-normalization.test.ts`)
- `e2e/wasm-typescript/tests/browser/browser.spec.ts`, `e2e/wasm-typescript/tests/signing.spec.ts`
- `e2e/wasm-typescript-cf/tests/forbidden-instantiation.spec.ts`

Validation surface:

```text
cargo test -p cow-sdk-wasm --test host_pure_helpers
cargo test -p cow-sdk-wasm --test wasm_surface_contract
cargo test -p cow-sdk-wasm --test wasm_snapshot_surface_contract
cargo test -p cow-sdk-wasm --test wasm_facade_snapshot_contract
cargo test -p cow-sdk-wasm --test wasm_envelope_contract
cargo test -p cow-sdk-wasm --test wasm_error_abi_contract
cargo test -p cow-rs-workspace-tests --test wasm_dependency_invariant
cargo check -p cow-sdk --target wasm32-unknown-unknown --features alloy
cargo check -p cow-sdk --target wasm32-unknown-unknown --features alloy-provider
cargo check -p cow-sdk --target wasm32-unknown-unknown --features alloy-signer
wasm-pack test crates/wasm --headless --firefox
bash crates/wasm/npm/scripts/build.sh
node crates/wasm/npm/scripts/verify-exports.mjs
node crates/wasm/npm/scripts/verify-no-raw-exports.mjs
node crates/wasm/npm/scripts/verify-facade-denylist.mjs
node crates/wasm/npm/scripts/measure-wasm-size.mjs
bash crates/wasm/npm/scripts/verify-package-resolution.sh
pnpm --dir crates/wasm/npm test
pnpm --dir e2e/wasm-typescript test
pnpm --dir e2e/wasm-typescript-cf test
```
