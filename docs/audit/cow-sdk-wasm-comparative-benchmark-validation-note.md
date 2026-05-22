# cow-sdk-wasm Comparative Benchmark Validation Note

Status: Current
Last reviewed: 2026-05-22
Owning surface: `cow-sdk-wasm` crate and npm package
Refresh trigger:
- `cow-sdk-wasm` flavor feature change (added, removed, re-scoped)
- Upstream `@cowprotocol/cow-sdk` major version release
- wasm32 toolchain stack rotates major versions (Rust, wasm-bindgen, wasm-opt)
- Bundler-comparison esbuild version rotates a major
- Node LTS channel rotation (Node 22 LTS end-of-maintenance, Node 24 LTS phase change, future LTS arrival)
- Cloudflare Workers compressed-size or startup-time limit change
- Public TypeScript facade contract changes materially
- Any pinned JavaScript ecosystem reference package (`@noble/hashes`, `@noble/curves`, `viem`, `ethers`) rotates a major version
- Headline performance reruns on Node 22 / 24 LTS land
- A Wrangler deployment + Worker startup measurement is added to the release evidence
Related docs:
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0044](../adr/0044-bundle-size-profile-and-flavor-builds.md)
- [WASM Performance Budget Audit](wasm-performance-budget-audit.md)
- [WASM Public API Stability Audit](wasm-public-api-stability-audit.md)
- [WASM EIP-1271 Parity Audit](wasm-eip1271-parity-audit.md)

## Scope

This validation note covers:

- The comparative measurement of `cow-sdk-wasm`'s package brotli, gzip, and
  raw sizes against equivalent feature subsets of the upstream
  `@cowprotocol/cow-sdk` TypeScript SDK, bundled with esbuild in production
  mode.
- The correctness parity of `cow-sdk-wasm`'s deterministic protocol outputs
  (signing, hashing, encoding, app-data CID) against the in-tree native
  Rust implementations through the host-side cargo test suite.
- The measured per-call latency of `cow-sdk-wasm` versus the upstream
  TypeScript SDK on the signing path (single call and batched).
- The measured WASM module compile + instantiate time as a proxy for
  Cloudflare Workers cold start.
- The Cloudflare Workers compressed-size compatibility of the `cloudflare`
  flavor against Cloudflare's currently published Workers limits.
- A focused comparative measurement of the underlying cryptographic and
  encoding primitives (keccak256, secp256k1 ECDSA sign, EIP-712 typed-data
  hash) against pinned JavaScript ecosystem references.
- Direct measurement of the JavaScript / WebAssembly call boundary cost
  using varying byte payload sizes and a linear regression model.
- Per-call typed DTO marshalling overhead for the canonical Order DTO.
- Per-call cost of 256-bit unsigned-integer arithmetic through string
  serialization compared to native JavaScript `BigInt`.
- A modeled realistic-mode time decomposition of a representative
  cow-protocol order-placement workflow with stubbed wallet and network
  components.
- Per-operation hot-path identification across five representative SDK
  flows.
- A synthetic single-call batch order-UID computation hypothesis test.
- A directional lines-of-code overlap measurement comparing cow-rs's
  protocol-logic crates against the equivalent upstream TypeScript SDK
  packages.

It does not cover:

- A full Cloudflare Workers production-deployment cold-start measurement
  (the present measurement uses a Node V8 instantiate proxy; production
  telemetry is a refresh-trigger item).
- Bundler compatibility for Vite, webpack, and Rollup beyond esbuild (the
  fixture exercised only esbuild; a full bundler-matrix audit is a separate
  future refresh).
- Browser Lighthouse LCP measurements under real Chrome conditions (the
  cited LCP delta is a deterministic bandwidth-model proxy due to harness
  limitations).
- Node-LTS-channel performance characteristics. Latency and primitive
  measurements were captured on Node 25 only and are explicitly labeled as
  point-in-time diagnostic measurements; Node 22 LTS and Node 24 LTS
  reruns are refresh-trigger items.
- DX rubric scoring (covered by the WASM Public API Stability Audit and the
  WASM Type Generation Audit).

## Outcome Summary

| Area | Validation point | Result |
| --- | --- | --- |
| Bundle size, default flavor | Brotli total package size at equivalent feature subset is documented relative to upstream TS SDK | Confirmed |
| Bundle size, orderbook flavor | Brotli total package size at equivalent feature subset is documented | Confirmed |
| Bundle size, signing flavor | Brotli total package size at equivalent feature subset is documented | Confirmed |
| Bundle size, cloudflare flavor | Gzip wasm size is documented against the current Workers Free compressed-size limit; runtime support gates remain separate | Confirmed: size-compatible at time of measurement; startup / deployment validation pending |
| Correctness parity | Host-side cargo tests pass; cow-sdk-wasm and native Rust produce byte-for-byte identical protocol outputs across the measured vector set | Confirmed |
| Single sign latency (Node 25, diagnostic) | Per-call median latency ratio is documented relative to upstream TS SDK | Confirmed |
| Batch sign throughput (Node 25, diagnostic) | Throughput ratio at batch=100 is documented relative to upstream TS SDK | Confirmed |
| WASM compile + instantiate time | Node V8 instantiate proxy time is documented as a lower bound for Cloudflare Workers cold start | Confirmed (proxy) |
| Per-call IPFS overhead | Median SDK-side overhead versus a direct local fetch is documented | Confirmed |
| Cancellation roundtrip overhead | Median AbortBridge cleanup overhead versus a direct local fetch + abort is documented | Confirmed |
| Pure keccak primitive performance | WASM keccak throughput vs JavaScript ecosystem references on Node 25 is documented at three input sizes | Confirmed (WASM faster on the measured runtime) |
| Pure secp256k1 sign primitive performance | WASM ECDSA sign throughput vs JavaScript ecosystem references is documented | Confirmed (WASM faster on the measured runtime) |
| Pure EIP-712 typed-data hash primitive performance | WASM typed-data hash throughput vs viem and ethers is documented | Confirmed (competitive: slower than viem, faster than ethers) |
| JavaScript / WebAssembly boundary call cost | Per-call fixed cost and per-byte marginal cost on the measured Node version are documented | Confirmed (raw boundary cost is small) |
| Typed DTO marshalling overhead | Per-call cost beyond raw byte roundtrip for the canonical Order DTO is documented | Confirmed (Order-shape DTO marshalling adds a small per-call overhead) |
| 256-bit arithmetic per-call cost | WASM U256 per-call cost vs native JavaScript BigInt is documented for arithmetic primitives | Confirmed (per-call WASM U256 through string serialization is slower than native JavaScript BigInt) |
| Realistic workflow time decomposition | Modeled wallet and network share of user-perceived order-placement time is documented | Confirmed (wallet and network dominate in modeled realistic mode) |
| Coarse-grained batch UID hypothesis | Whether a synthetic single-call batch UID API beats the fine-grained loop on the measured workload | Confirmed (no improvement on the tested workload; result is workload-bounded) |
| Shared-logic surface overlap | Lines-of-code overlap of cow-rs protocol-logic crates vs the equivalent upstream TypeScript packages is documented | Confirmed (directional; LOC is a proxy) |

## Current Contract

### Bundle size

`cow-sdk-wasm` ships four feature-scoped flavor builds (default, orderbook,
signing, cloudflare) from a single npm package. The total package size
(wasm + JavaScript glue + compiled TypeScript facade) at brotli quality 11 is
documented below for each flavor's `web` target subpath.

For comparison, the upstream `@cowprotocol/cow-sdk` packages were built (via
tsup) and bundled (via esbuild in production mode, browser target with Node
built-ins externalized) for the equivalent feature subsets. The comparison
is documented at the time of measurement and is subject to the refresh
triggers above.

| Flavor (web target) | cow-sdk-wasm brotli (total package) | Upstream TS SDK brotli (esbuild-bundled subset) | Ratio |
| --- | --- | --- | --- |
| default | ~826 KiB | ~57 KB | ~14.5× |
| orderbook | ~353 KiB | ~52 KB | ~6.8× |
| signing | ~178 KiB | ~54 KB | ~3.3× |

Compiling the Rust SDK to wasm32 produces a binary larger than the upstream
TypeScript SDK at equivalent feature subsets. The bundle-size delta is the
core tradeoff the consumer-routing matrix in `README.md` and
`crates/wasm/README.md` accepts in exchange for deterministic Rust signing
parity, single-source-of-truth Rust + TypeScript embedding, and Cloudflare
Workers compatibility.

### Cloudflare Workers script-size tier

The `cloudflare` flavor's gzip-compressed wasm artifact at the time of
measurement is approximately **1,096,488 bytes** (about 1.05 MB).

Per Cloudflare's published Workers limits at
`https://developers.cloudflare.com/workers/platform/limits/` (verified at the
time of audit publication):

| Limit | Workers Free | Workers Paid |
| --- | ---: | ---: |
| Worker size after gzip | 3 MB | 10 MB |
| Startup time | 1 second | 1 second |
| Memory | 128 MB | 128 MB |

The measured artifact is below the current Workers Free compressed-size
limit and well below the current Workers Paid limit.

Full Workers support requires two additional gates that are not measured by
this validation note:

- **Release-bundle verification**: the package output must produce a
  Wrangler-deployable Worker bundle.
- **Worker startup measurement**: the WASM compile and instantiate work
  must complete within Cloudflare's 1-second startup limit (measurable via
  Wrangler `startup_time_ms` telemetry on deploy).

Both gates are listed in the Refresh triggers section above. A future
deployment-grade Workers measurement may extend this validation note when
those gates are exercised.

Cloudflare's published limits are subject to change; the validation note's
refresh triggers cover that case. The package release gate enforces an
explicit byte budget for the cloudflare flavor's gzip size against the
current Workers Free compressed-size limit (with safety margin); see
ADR 0044 for the gate.

### Correctness parity

The host-side cargo test suite in `crates/wasm/tests/` verifies byte-for-byte
identical protocol outputs between `cow-sdk-wasm` and the in-tree native
Rust crates across:

- Order UID derivation
- EIP-712 typed-data envelope construction
- EIP-1271 payload encoding
- Domain separator computation
- Deployment address resolution
- App-data CID computation
- App-data document validation
- Order input rejection on malformed addresses and balance enums
- Chain ID parsing
- Token balance enum mapping
- Snapshot-tested public TypeScript surface contract (camelCase APIs, named
  callback types, single constructor per client, dispose, signal +
  timeoutMs, no internal registry exposure)

The full test list is in `crates/wasm/tests/`.

### Per-call signing latency (Node 25, diagnostic)

On Node 25, the `cow-sdk-wasm` signing path's median single-call time is
measurably higher than the upstream TypeScript SDK path at the time of
measurement (cow-sdk-wasm at approximately 1.59× the TypeScript SDK median).
The boundary cost of the wasm-bindgen + tsify-derived DTO marshalling
contributes to this delta.

For one-shot signing (a single user-driven swap), the delta is acceptable in
absolute terms. For batched signing (many orders per second), the upstream
TypeScript SDK path is faster at the time of measurement (cow-sdk-wasm
throughput at approximately 0.62× the upstream TypeScript SDK throughput at
batch=100 on Node 25).

These numbers were captured on Node 25 only and are point-in-time diagnostic
measurements; see "Node.js runtime support posture" below.

### WASM compile + instantiate (Workers cold start proxy)

The `cloudflare` flavor's WASM module compile + instantiate time measured
via Node's WebAssembly API (the same V8 engine that Cloudflare Workers
uses) is approximately 7-8 ms p95 at the time of measurement. This is a
lower bound for the WASM-specific portion of Workers cold start; the full
production Workers cold start includes additional isolate and runtime setup
time that is not measured in this validation note.

### IPFS and cancellation overhead

On Node, against a local stub HTTP server:

- `cow-sdk-wasm`'s `IpfsClient.fetchAppDataFromCid` adds approximately
  100-150 µs of SDK overhead versus a direct `fetch` call.
- `cow-sdk-wasm`'s AbortBridge cancellation adds approximately zero
  measurable overhead (within measurement noise) versus a direct `fetch` +
  `AbortController.abort()`.

### Pure compute primitive performance

A focused comparative measurement of the underlying cryptographic and
encoding primitives (keccak256, secp256k1 ECDSA sign, EIP-712 typed-data
hash) was performed against the strongest JavaScript ecosystem references
at the time of measurement. The comparison uses identical input bytes and
verifies byte-for-byte equality of outputs.

The comparison was performed on Node 25, against pinned versions of each
TypeScript reference (recorded under "Pinned baseline versions" below).
Browser, Cloudflare Workers, and other Node LTS reruns are refresh-trigger
items and not part of this measurement.

**Keccak256 primitive throughput**

| Input size | cow-sdk-wasm median | Strongest JS reference median |
| --- | --- | --- |
| 32 B | ~6.4 µs | ~18 µs (across @noble/hashes / viem / ethers) |
| 1 KB | ~20.9 µs | ~87 µs (@noble/hashes) |
| 1 MB | ~20,162 µs | ~75,013 µs (ethers) |

`cow-sdk-wasm`'s keccak primitive was faster than the measured JavaScript
references at all three input sizes on Node 25. This advantage exists at
the primitive level; it does not automatically translate to end-to-end
production flow advantage because production flows also involve TypeScript-
side wallet callbacks, typed DTO marshalling, and network operations — see
the workflow-decomposition section below.

**Secp256k1 ECDSA sign throughput**

| Side | Median |
| --- | --- |
| cow-sdk-wasm | ~555-597 µs |
| @noble/curves | ~712 µs |
| ethers SigningKey | ~590-620 µs |

`cow-sdk-wasm`'s secp256k1 sign primitive was faster than the measured
JavaScript references on Node 25. As with keccak, this is a primitive-level
finding; production wallet signing flows go through a JavaScript wallet
callback (e.g., ethers, MetaMask) and the primitive-level cow-sdk-wasm
advantage does not change that.

**EIP-712 typed-data hash**

| Side | Median |
| --- | --- |
| cow-sdk-wasm | ~474-546 µs |
| viem.hashTypedData | ~359 µs |
| ethers.TypedDataEncoder.hash | ~620-700 µs |

`cow-sdk-wasm`'s typed-data hash was competitive with the measured
JavaScript references: slower than viem, faster than ethers. "Competitive"
is the appropriate framing; cow-sdk-wasm is not categorically faster on
this composed primitive.

### JavaScript / WebAssembly boundary cost

The cost of crossing the JavaScript / WebAssembly boundary was measured
directly on Node 25 with varying byte-payload sizes (32, 256, 1024, 16384
bytes). A linear regression of median per-call time against payload size
produced a stable model.

| Component | Value |
| --- | --- |
| Fixed per-call cost (regression intercept) | ~1 µs/call |
| Marginal cost per byte (regression slope) | ~0.7 µs per KB |
| Linear-fit quality (R²) | > 0.99 |

The raw byte-roundtrip boundary cost is small enough that it is not the
dominant bottleneck in measured production calls. Production call cost is
dominated by the work performed inside WebAssembly and the typed DTO
marshalling layer (documented next), not by the call boundary itself.

A separate batched empty-call measurement, which divides the total time of N
consecutive calls by N, reported a higher per-call number (tens of
microseconds). That measurement includes the JavaScript runtime's per-
iteration overhead (garbage collection, JIT settling, performance.now
resolution) and should be read as a pessimistic upper bound rather than the
per-call fixed cost. The regression-based measurement above is the cleaner
number for the per-call cost.

### Typed DTO marshalling cost

For the canonical Order DTO shape (12 fields including addresses, uint256
strings, an enum, a bool, and a bytes32), the per-call typed-DTO
marshalling cost (above raw byte roundtrip) was approximately 17-30 µs on
Node 25. Other DTO shapes (SignedOrder, AppDataDoc, quote request types)
were not measured in this round; their cost is expected to be in a similar
range but not identically quantified.

DTO marshalling is real but small relative to the work performed inside
WASM for typical production calls. In absolute terms, a per-call
marshalling cost of approximately 24 µs is well below the boundary at
which it would dominate end-to-end signing or order-placement workflows.

### 256-bit arithmetic per-call cost

For 256-bit unsigned-integer arithmetic operations (multiplication,
division, modulo) called individually through the cow-sdk-wasm WebAssembly
boundary using decimal string serialization, the per-call cost was 7-20×
higher than the equivalent operation on a native JavaScript `BigInt`.
Power-of-N modular exponentiation (which performs multiple internal
arithmetic operations per call) was roughly competitive with native BigInt
because the serialization cost was amortized over the internal loop.

This measurement informs API design rather than reframing performance:
`cow-sdk-wasm` does NOT expose per-operation 256-bit arithmetic primitives
in its production public surface. Consumers writing per-operation
arithmetic should use native JavaScript `BigInt`. Coarse-grained Rust-side
arithmetic (where many operations happen inside a single call) remains a
possible future optimization for compute-heavy workloads, but the current
public surface does not include it.

### Realistic workflow time decomposition

For a representative cow-protocol order-placement workflow (compute
app-data CID, build EIP-712 envelope, wallet sign, POST to orderbook, poll
status until settled), a modeled realistic-mode measurement was performed
using stubbed wallet (~2-second confirm delay), stubbed orderbook POST
(~200 ms), and stubbed settle polling (~5 seconds). In this modeled
realistic mode:

- Wallet signing and network operations together accounted for the
  dominant share of user-perceived workflow time.
- SDK-internal computation (compute app-data CID, build envelope,
  JavaScript / WebAssembly boundary work) accounted for less than one
  percent of user-perceived time.

The cow-sdk-wasm and upstream TypeScript SDK paths produced essentially
identical total workflow times in this modeled realistic mode, because the
wallet and network components dominate and are independent of SDK choice.

This finding supports the consumer-routing matrix's guidance: for typical
wallet-driven order placement, SDK choice does not meaningfully impact
user-perceived performance. Bundle size, developer experience, and
ecosystem fit are the relevant factors for consumer choice.

The percentages presented above are MODELED, not measured from production
telemetry. The exact stub delays were chosen as representative estimates
and a different consumer deployment with different wallet UX or different
network conditions will see different exact percentages. The qualitative
conclusion (wallet and network dominate) is robust across reasonable
assumptions; the exact percentages are not universal.

### Hot-path identification

Per-operation instrumentation across five representative flows (order
placement, order status polling, app-data preparation, quote fetching,
batch order signing) identified the following SDK-internal hot paths in
SDK-isolated stub mode:

- For order placement and app-data preparation flows, the `appDataInfo`
  operation (which canonicalizes the AppDataDoc, computes the keccak256
  hash, builds the IPFS CIDv1 multibase encoding, and returns a typed DTO)
  is the dominant SDK-internal cost.
- For batch order signing, the per-order envelope construction and the
  JavaScript wallet sign callback together dominate.
- For order status polling and quote fetching, network operations
  dominate; SDK internal work is negligible.

In modeled realistic mode (with stubbed wallet and network delays), these
SDK-internal hot paths fall below one percent of user-perceived flow time
across all five measured flows.

### Synthetic coarse-grained batch UID hypothesis

A specific hypothesis was tested: a synthetic single-call batch order-UID
computation API (taking N orders as a JSON string, returning N UIDs) was
compared against the existing fine-grained production loop (calling
`computeOrderUid` once per order). The hypothesis was that consolidating N
boundary crossings into one would recover compute-side advantage.

For the tested workload (100 orders per batch on Node 25), the synthetic
batch path was slightly slower than the fine-grained loop. The
serialization cost of passing 100 orders as a JSON string and parsing them
inside Rust exceeded the savings from amortizing the per-call boundary
cost.

This result is workload-bounded:

- It applies to UID computation specifically.
- It does not generalize to all possible batch shapes (other workload
  classes, e.g., signature verification, may yield different results).
- The synthetic batch shape (JSON-string array) is not a recommended
  production API form.

The current public surface does NOT include a coarse-grained batch UID
API, and based on this measurement no such API is recommended for the next
release.

### Shared-logic surface overlap

A directional lines-of-code overlap measurement compared cow-rs's
protocol-logic crates against the equivalent upstream TypeScript SDK
packages:

| Side | Approximate LOC (excluding tests, comments, generated files) |
| --- | --- |
| cow-rs protocol-logic Rust crates (signing, contracts, app-data, core, pure-helpers reachable from WASM exports) | ~9.5 kLOC |
| Upstream TypeScript packages with equivalent functional scope (order-signing, contracts-ts, app-data, common) | ~7.1 kLOC |

The order of magnitude is comparable. Approximately 75-80 percent of the
upstream TypeScript SDK's protocol-logic surface has a functional
equivalent in cow-rs's Rust crates.

This is a directional measurement only. Lines of code is a proxy for
surface area, not for semantic equivalence or for correctness. The shared-
logic argument for `cow-sdk-wasm` rests on the architectural property that
one Rust implementation covers both native cow-rs services and JavaScript /
TypeScript consumers via the WebAssembly bridge — not on a precise LOC
ratio.

## Evidence

Reproduction (public commands only):

```text
# Build the WASM artifacts
bash crates/wasm/npm/scripts/build.sh

# Measure WASM artifact sizes
node crates/wasm/npm/scripts/measure-wasm-size.mjs

# Run the host-side correctness tests
cargo test -p cow-sdk-wasm --tests
```

The comparative runtime measurements documented above are recorded as
point-in-time validation results from internal benchmark runs. The public
reproduction commands above cover the shipped build, size-gate, and
correctness surfaces; runtime benchmark reruns are refresh-trigger work for
maintainers (per the Refresh triggers section).

Primary implementation points:

- `crates/wasm/Cargo.toml` — flavor feature flags
- `crates/wasm/src/exports/*.rs` — wasm-bindgen exports
- `crates/wasm/npm/src/*.ts` — TypeScript facade
- `crates/wasm/npm/scripts/build.sh` — build pipeline (wasm-pack +
  wasm-opt + facade compile + verify-exports + measure-wasm-size +
  prepublish-guard)
- `crates/wasm/npm/scripts/measure-wasm-size.mjs` — size measurement and
  budget gate
- `crates/wasm/npm/flavours.json` — flavor descriptors and budget settings
  (the cloudflare flavor uses an explicit byte budget that tracks
  Cloudflare's published Workers Free compressed-size limit)

Primary regression coverage:

- `crates/wasm/tests/host_pure_helpers.rs::*`
- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::*`
- `crates/wasm/npm/tests/facade-*.test.ts`

## Limitations

This validation note documents a specific point-in-time comparison under
specific conditions:

- The Node runtime used for latency measurements is one specific Node
  version (Node 25); LTS-channel reruns (Node 22 LTS, Node 24 LTS) are a
  refresh-trigger item.
- The browser LCP delta cited in support material is a deterministic
  bandwidth-model proxy; real Lighthouse measurements are a refresh-trigger
  item.
- The bundler comparison is bounded to esbuild's production-mode behavior
  on the equivalent TypeScript subset; Vite, webpack, and Rollup are not
  exercised in this measurement set.
- The TypeScript baseline upstream commit hash and integrity is recorded
  as part of the internal point-in-time measurement; future upstream
  TypeScript SDK major releases are a refresh trigger.
- The Cloudflare Workers cold start is measured as a Node V8 instantiate
  proxy; production Workers cold start includes additional isolate and
  runtime setup time not measured here.
- All measurements documented in this note are bounded by the
  snapshot-tested public surface contract; if the contract changes
  materially, this validation note refreshes.
- The pure compute primitive measurements (keccak, secp256k1, EIP-712
  hash) were obtained on a build that did NOT have wasm-opt -Oz
  post-processing applied; future optimized-build reruns may produce
  different numbers.
- The pure compute primitive measurements compare against specific pinned
  versions of `@noble/hashes`, `@noble/curves`, `viem`, and `ethers`
  (versions recorded below); newer or older versions may produce different
  numbers.
- The 256-bit arithmetic measurement compares per-call cost using decimal-
  string serialization, which is the natural shape for an arbitrary
  JavaScript / WebAssembly integer API. Internally amortized arithmetic
  (many operations per call) is faster but is not exposed in the current
  public surface.
- The realistic workflow time-decomposition measurement uses modeled
  wallet and network delays (specific stub values representing typical
  user behavior). Real-world deployments with different wallet UX or
  different network conditions will see different exact percentages; the
  qualitative conclusion (wallet and network dominate) is robust.
- The synthetic coarse-grained batch UID hypothesis test used a single
  JSON-string-shaped batch API on Node 25. The result applies to that
  workload only and does not generalize to all batch APIs or all workload
  classes.
- A pure ABI encode / decode primitive comparison is not included in this
  note; the measurement-time WebAssembly export was semantically
  mismatched with the JavaScript reference implementations and the
  comparison is deferred until an aligned re-measurement.
- Cross-language test vectors and shared-logic LOC are directional
  supporting analysis, not definitive proof of semantic equivalence.

### Pinned baseline versions for primitive measurements

The pure compute primitive measurements compare against the following
pinned versions of the JavaScript ecosystem references:

| Package | Pinned version |
| --- | --- |
| @noble/hashes | 1.8.0 |
| @noble/curves | 1.9.7 |
| viem | 2.38.6 |
| ethers | 6.14.3 |
| Node.js | 25.2.0 |

Newer or older versions may produce different numbers. These versions are
part of the refresh-trigger criteria above.

### Node.js runtime support posture

This validation note documents measurements captured on **Node 25.2.0
only**, which is a Node Current line per Node's official release schedule
at `https://nodejs.org/en/about/previous-releases`.

The package's production support targets are **Node 22 and Node 24**
(both supported LTS lines at the time of audit publication). Node 25
latency and primitive-performance measurements documented in this note are
explicitly labeled as point-in-time diagnostic measurements and are NOT
claimed as Node LTS-channel performance evidence.

Headline performance reruns on Node 22 and Node 24 are listed as refresh-
trigger items above. Until those reruns are performed, the Node-LTS-
channel performance characteristics of `cow-sdk-wasm` are not asserted by
this validation note.

Per Node's official guidance, production applications should use Active
LTS or Maintenance LTS releases. Consumers with strict LTS requirements
should treat the Node 25 numbers in this validation note as directional
only and validate against their own LTS-version testing.
