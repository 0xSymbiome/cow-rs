# Browser-Runtime Proof Posture

`cow-rs` ships browser-runtime support behind a two-tier proof posture that
separates deterministic contract proof from environment-sensitive
confirmation. Both tiers are required. Neither substitutes for the other.
The separation is visible in the UI, the README, and the test lanes of the
shipped WASM consoles.

## Deterministic Lane

The deterministic lane holds the reviewable contract for every browser-runtime
claim and runs on every commit.

- Host-side `cargo test` drives the Rust-native state machines inside the
  browser-wallet console and the SDK verification console. Under the
  host-side lane the consoles run against `MockEip1193Transport`, so
  discovery, selection, confirmation, connect, signing, quote, submit, and
  cancel compose deterministically without a browser.
- Browser-wallet receipt parsing is also covered in the deterministic lane:
  missing or null optional receipt fields remain tolerated, while present
  malformed `status`, `blockNumber`, `blockHash`, `gasUsed`, `from`, and `to`
  fields fail closed before reaching callers.
- In-browser `wasm-bindgen-test` runs the same surface through a real
  headless Chrome so the WebAssembly boundary and the `wasm-bindgen` interop
  idioms see continuous proof. The sdk-verification console exercises
  capability, app-data, CID, order-envelope, EIP-1271, approval, and
  trading-default outputs. The browser-wallet console exercises sample-JSON
  generators and the selection-confirmation sequence under the mock EIP-1193
  transport.
- Playwright with mocked fixtures covers end-to-end DOM behavior without a
  live wallet extension or live orderbook endpoint. The browser-wallet
  Playwright suite runs under both Chromium and Firefox projects so the
  DOM-behavior contract is validated under the two most widely deployed
  browser engines. The `e2e/browser-wallet/fixtures/injected-wallet.ts`
  fixture mocks EIP-6963 discovery, injected provider requests, and
  chain-switch events. The `e2e/sdk-verification/fixtures/cow-api.ts`
  fixture mocks the CoW orderbook and subgraph endpoints with
  deterministic payloads so the reviewer-facing panel flows reproduce
  reliably.

## Environment-Sensitive Lane

The environment-sensitive lane depends on real browser extensions and real
public endpoints. It cannot be deterministic and is never asserted as
contract proof.

- Manual QA against real EIP-1193 wallet extensions covering the supported
  browser-extension wallet families confirms that the console behaves honestly
  with real user prompts, chain availability, and vendor-specific UX. The QA
  matrix records the latest covered set.
- Optional static browser-live smoke checks that the served console page is
  reachable and still exposes the stable DOM markers before extension-backed
  actions run. Smoke results are readiness signals, not behavior proof.

## Why Both Are Required

The deterministic lane alone cannot observe real extension behavior, real
provider variance, or real public endpoint responses. The
environment-sensitive lane alone cannot give a stable pass-or-fail signal on
every commit and cannot be a release gate. Running both keeps the reviewed
contract mechanically enforced and the real-world support posture explicitly
acknowledged.

## How To Read Both Lanes

A reviewer reading the consoles should treat the mock pane and the injected
pane as different genres.

- The mock pane is the deterministic contract. If something fails here, the
  Rust SDK has a bug or the contract drifted.
- The injected pane is the environment-sensitive tier. A failure may reflect
  an SDK bug, a vendor-specific behavior, a chain availability problem, or a
  user-interaction mistake. Diagnose from the deterministic tier first.

## Staging And Production Posture

Static browser-live orderbook actions default to `staging`. Production
browser-live actions are disabled on the shipped static page. Running the
production orderbook surface from a browser requires a proxy-enabled
deployment that adds the permitted CORS headers, not the default local or
Pages-style serving path. The consoles surface this boundary directly so a
reviewer cannot accidentally submit a real order from the shipped static
build.

## TypeScript-Callable WASM Runtime Matrix

`cow-sdk-wasm` extends browser-runtime evidence beyond the Rust browser-wallet
leaf. Browser bundlers are default-http-supported through the fetch-backed
package path and Playwright coverage. Node.js 24 LTS and Cloudflare Workers are
callback-http-tested through `CowFetchCallback`; Cloudflare also verifies that
worker source does not use dynamic WebAssembly compilation or streaming
instantiation entry points. Deno is optional experimental and runs only through
the opt-in Deno package target. Bun, Vercel Edge, and Fly.io are documented as
best-effort without a CI support claim.

## Related

- [ADR 0007: Bounded Browser Wallet Support And Current Browser Runtime Contract](adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [ADR 0009: WASM Verification Consoles — Hybrid Extensibility And Two-Tier Proof](adr/0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md)
- [ADR 0039: Keep The TypeScript-Callable WASM SDK Surface As An Additive Leaf Crate](adr/0039-typescript-callable-wasm-sdk-surface.md)
- [Browser Wallet Chain Coherence Audit](audit/browser-wallet-chain-coherence-audit.md)
- [WASM Example Proof-Posture Audit](audit/wasm-example-proof-posture-audit.md)
- [WASM Surface Audit](audit/wasm-surface-audit.md)
- [Examples catalogue](examples.md)
- [Browser Wallet Console](../examples/wasm/browser-wallet-console/README.md)
- [SDK Verification Console](../examples/wasm/sdk-verification-console/README.md)
