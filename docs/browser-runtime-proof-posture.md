# Browser-Runtime Proof Posture

`cow-rs` ships browser-runtime support behind a two-tier proof posture that
separates deterministic contract proof from environment-sensitive
confirmation. Both tiers are required. Neither substitutes for the other. The
proof lives in the `cow-sdk-browser-wallet` crate test lane and the
browser-transport tests under `crates/wasm`, not in an example surface.

## Deterministic Lane

The deterministic lane holds the reviewable contract for every browser-runtime
claim and runs on every commit.

- Host-side `cargo test` drives the Rust-native state machines inside
  `cow-sdk-browser-wallet`, including the mock EIP-1193 transport
  (`MockEip1193Transport`), so discovery, selection, confirmation, connect,
  signing, and chain-management compose deterministically without a browser.
- Browser-wallet receipt parsing is also covered in the deterministic lane:
  missing or null optional receipt fields remain tolerated, while present
  malformed `status`, `blockNumber`, `blockHash`, `gasUsed`, `from`, and `to`
  fields fail closed before reaching callers.
- In-browser `wasm-bindgen-test` runs the owned EIP-1193 bridge through a real
  headless browser so the WebAssembly boundary and the `wasm-bindgen` interop
  idioms see continuous proof. These cases include the deterministic mock-wallet
  state machine and EIP-6963 discovery-event serialization round trips.
- The fetch-backed browser transport — `cow-sdk-core`'s `FetchTransport`,
  exercised by the tests under `crates/wasm` — runs directly through a headless
  browser, covering its browser dispatch shape, redacted endpoint telemetry, and
  native-versus-browser error-class parity.

## Environment-Sensitive Lane

The environment-sensitive lane depends on real browser extensions and real
public endpoints. It cannot be deterministic and is never asserted as
contract proof.

- Manual QA against real EIP-1193 wallet extensions covering the supported
  browser-extension wallet families confirms honest behavior with real user
  prompts, chain availability, and vendor-specific UX. This acceptance window
  and its operator steps are exercised manually and are environment-sensitive.
- The canonical browser-wallet example (`examples/wasm/cow-trader-dioxus/`) is
  the runnable demonstration of the end-to-end flow against the live orderbook.
  It is a consumer demonstration, not a deterministic proof surface.

## Why Both Are Required

The deterministic lane alone cannot observe real extension behavior, real
provider variance, or real public endpoint responses. The
environment-sensitive lane alone cannot give a stable pass-or-fail signal on
every commit and cannot be a release gate. Running both keeps the reviewed
contract mechanically enforced and the real-world support posture explicitly
acknowledged.

## How To Read Both Lanes

Diagnose from the deterministic tier first.

- The crate test lanes are the deterministic contract. If something fails
  there, the Rust SDK has a bug or the contract drifted.
- An extension-backed failure may reflect an SDK bug, a vendor-specific
  behavior, a chain availability problem, or a user-interaction mistake.

## Production Posture

The example talks to the production CoW API (`api.cow.fi`), where CoW's Sepolia
liquidity is served. A production browser deployment should set a
Content-Security-Policy `connect-src` scoped to the host it calls
(`connect-src 'self' https://api.cow.fi;`).

## TypeScript-Callable WASM Runtime Matrix

`cow-sdk-wasm` extends browser-runtime evidence beyond the Rust browser-wallet
leaf. Browser bundlers are default-http-supported through the fetch-backed
package path and Playwright coverage. Node.js (22 and 24) and Cloudflare Workers are
callback-http-tested through `CowFetchCallback`; Cloudflare also verifies that
worker source does not use dynamic WebAssembly compilation or streaming
instantiation entry points. Deno is optional experimental: the runtime-neutral
callback transport supports a self-built Deno target on a best-effort basis, with
no shipped build or CI support claim. Bun, Vercel Edge, and Fly.io are documented
as best-effort without a CI support claim.

The runtime evidence boundary that this proof posture covers is intentionally
narrow:

- Browser bundler evidence is bounded to the bundlers exercised in the
  release pipeline; bundler-matrix completion is documented as a future
  refresh.
- Node.js LTS support targets the lines listed in the package's
  `engines.node` range; production applications should use Active LTS or
  Maintenance LTS releases per Node's official guidance. Performance
  characteristics on a specific Node version are recorded as point-in-time
  diagnostic measurements rather than LTS-channel performance evidence.
- Cloudflare Workers support is split into two gates that this proof
  posture treats independently. The compressed-size gate is enforced on
  every release build (the cloudflare flavor's gzip artifact is verified
  against an explicit byte budget that tracks Cloudflare's published
  Workers Free compressed-size limit). Worker startup time and bundle
  deployment behavior are separate operational gates that production
  consumers verify with `wrangler deploy --dry-run` and the
  `startup_time_ms` telemetry Wrangler reports against Cloudflare's
  1-second startup limit.
- Browser LCP figures cited in support material are derived from a
  deterministic bandwidth-model proxy; real Lighthouse measurements on
  Linux are tracked as a future refresh.

Runtime claims beyond these boundaries are not made by the shipped
artifacts.

## Related

- [ADR 0007: Bounded Browser Wallet Support And Current Browser Runtime Contract](adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [ADR 0065: Single Canonical Browser-Wallet Example](adr/0065-canonical-browser-wallet-example.md)
- [ADR 0039: Keep The TypeScript-Callable WASM SDK Surface As An Additive Leaf Crate](adr/0039-typescript-callable-wasm-sdk-surface.md)
- [Browser Wallet Chain Coherence Audit](audit/browser-wallet-chain-coherence-audit.md)
- [Browser Wallet Trust Posture Audit](audit/browser-wallet-trust-posture-audit.md)
- [WASM Surface Audit](audit/wasm-surface-audit.md)
- [Examples catalogue](examples.md)
- [Browser-wallet trade example](../examples/wasm/cow-trader-dioxus/README.md)
