# @symbiome-forge/cow-sdk-wasm

[CoW Protocol](https://cow.fi)'s Rust SDK, compiled to WebAssembly for JavaScript
and TypeScript. One protocol implementation runs in both runtimes, so the EIP-712
and EIP-1271 signatures a browser produces are byte-identical to the Rust
service's — checked against the upstream `cowprotocol/services` and
`cowprotocol/contracts` fixtures in CI, not asserted in prose.

```sh
npm install @symbiome-forge/cow-sdk-wasm@alpha
```

A TypeScript facade over deterministic Rust protocol logic: typed DTOs, explicit
wallet and HTTP callbacks, per-call cancellation and timeouts, and
flavor-specific imports.

## Why this package

- **One source of truth.** Quote echoing, order-UID packing, app-data hashing,
  and the EIP-712 / EIP-1271 signing path are the same Rust code a native
  `cow-sdk` service runs, compiled to wasm — so protocol drift between a Rust
  backend and a TypeScript frontend cannot happen. Every transform is proven
  byte-for-byte against pinned upstream fixtures on each CI run.
- **No private key ever enters the SDK.** Signing is a callback you supply (viem,
  ethers, an EIP-1193 wallet, or a Safe). There is no code path — not even a
  feature-gated one — that accepts a private key or holds a wallet inside wasm
  memory. The package produces typed data and transaction requests; your wallet
  signs and submits.
- **The TypeScript surface is locked.** The public `.d.ts` for every flavor is a
  committed snapshot that CI diffs on every build, so a contract change is a
  reviewed diff, never a silent drift — the wasm analog of `cargo-public-api`.
- **Honest about fit.** For a standard browser dapp where minimal bundle size
  dominates, upstream
  [`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk) is
  smaller — use it. This package is for Rust ↔ TypeScript parity, single-source
  embedding, edge runtimes, and embeddable signing.

## Pick your import

Pick the **flavor** (feature set) by its base subpath. Every flavor ships the same
runtime entries over one shared wasm binary, so pick the **runtime** by the suffix:

| Import | Surface | Use when |
| --- | --- | --- |
| `@symbiome-forge/cow-sdk-wasm/trading` | Full order lifecycle: quote, sign, post, cancel, app-data, and native wrap/unwrap | A browser dapp, a Node backend, or an edge runtime running order flow — one feature set serves all three; pick the runtime by suffix |
| `@symbiome-forge/cow-sdk-wasm/orderbook` | Orderbook reads, cancellation, and signing — no trading or app-data | A read-focused dapp that does not post orders |
| `@symbiome-forge/cow-sdk-wasm/signing` | Signing, UID, EIP-1271, deployment, and version helpers — the smallest flavor | A signer service or HSM-facing adapter |
| `@symbiome-forge/cow-sdk-wasm` | Everything above plus subgraph analytics and IPFS app-data | General use that needs subgraph or IPFS |

Each flavor exposes three runtime entries that share one wasm binary:

- the **base** import (above) auto-selects the build through standard conditional
  exports: `node` loads the Node (CommonJS) build; `browser`, `import`, and the edge
  conditions (`workerd` / `worker` / `deno` / `edge-light` / `bun`) load the
  explicit-init **web** build, which instantiates its wasm through
  `new URL(import.meta.url)` and so works across every bundler and with no bundler at
  all — call `await initialize()` once before the first call;
- **`…/edge`** is the explicit web entry for Cloudflare Workers, Deno, and Vercel
  Edge — pair it with **`…/edge/wasm`** for the precompiled module and call
  `await initialize(wasmModule)`;
- **`…/module`** is the standards-track source-phase build (`import source` / Wasm
  ESM Integration): it auto-initializes with no `initialize()` call, runs today on
  Node 24, Deno, and esbuild, and is the forward path for browser bundlers as they
  adopt source-phase.

So the focused flavors' entries are `…/trading/edge`, `…/trading/module`,
`…/orderbook/edge`, `…/signing/module`, and so on; the root (everything) flavor's are
`@symbiome-forge/cow-sdk-wasm/edge` and `@symbiome-forge/cow-sdk-wasm/module`. Public
imports go through these subpaths; do not import from `dist/raw` or generated
wasm-pack directories.

Building a **native Rust** service, or a Rust app you compile to wasm yourself?
Use [`cow-sdk`](https://crates.io/crates/cow-sdk) — this package is for
JavaScript hosts.

## Quickstart — a browser swap, end to end

Quote, then reuse that quote to sign and post in one call, so the amounts the user
confirms are the amounts that get posted — no second quote, no drift between
preview and signature. The wallet signs a typed-data envelope the SDK hands it;
no key reaches the package.

```ts
import { initialize, TradingClient } from "@symbiome-forge/cow-sdk-wasm/trading";
import { createWalletClient, custom } from "viem";
import { mainnet } from "viem/chains";

// Instantiate the wasm module once before any call. The bundled module is resolved
// from the package via `new URL(import.meta.url)`, so this works in every bundler
// and with no bundler at all — no bundler wasm plugin required.
await initialize();

const [owner] = await window.ethereum.request({ method: "eth_requestAccounts" });
const wallet = createWalletClient({ chain: mainnet, transport: custom(window.ethereum) });

const trading = new TradingClient({
  chainId: 1,
  env: "prod",
  appCode: "my-dapp",
  transport: { kind: "fetch" }
});

// 1. Quote. `getQuote` returns a fully resolved QuoteResultsDto envelope.
//    `owner` is required for a quote-only call.
const quote = await trading.getQuote({
  kind: "sell",
  owner,
  sellToken: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
  buyToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",  // USDC
  amount: "1000000000000000000"
});

// 2. Reuse the quote to sign and post. The callback receives the EIP-712
//    envelope and returns the signature — the key stays in the wallet.
const result = await trading.postSwapOrderFromQuote(
  quote.value,
  owner,
  (envelope) => {
    // viem derives EIP712Domain from `domain`; drop it from `types`.
    const types = Object.fromEntries(
      Object.entries(envelope.types).filter(([name]) => name !== "EIP712Domain")
    );
    return wallet.signTypedData({
      account: owner,
      domain: envelope.domain,
      types,
      primaryType: envelope.primaryType,
      message: envelope.message
    });
  },
  { walletConfig: { timeoutMs: 20_000 } }
);

console.log(`https://explorer.cow.fi/mainnet/orders/${result.value.orderId}`);
trading.dispose();
```

Selling the native asset is the same shape: `getQuote`, then
`buildSellNativeCurrencyTxFromQuote(quote.value, owner)`, which returns the EthFlow
transaction request for the wallet to submit.

Converting between the native asset and its wrapped form is not an order: the
trading client's `buildWrapTx(amount)` and `buildUnwrapTx(amount)` return the
WETH deposit/withdraw transaction for the wallet to submit (the chain comes from
the client), and the standalone `wrappedNativeToken(chainId)` resolves the
wrapped-native token (address, symbol, decimals) for detecting a wrap pair in a
swap UI. (Selling native currency to
trade does not need a manual wrap — eth-flow wraps on-chain.)

### Cloudflare Worker (edge)

Workers cannot compile wasm from bytes at runtime, so the edge build takes the
statically imported module through an explicit `initialize`.

```ts
import initialize, { OrderBookClient } from "@symbiome-forge/cow-sdk-wasm/trading/edge";
import wasmModule from "@symbiome-forge/cow-sdk-wasm/trading/edge/wasm";

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    await initialize(wasmModule);
    const client = new OrderBookClient({
      chainId: 1,
      env: "prod",
      apiKey: env.COW_PARTNER_API_KEY ?? null,
      transport: { kind: "fetch" }
    });
    const quote = await client.getQuote(await request.json(), { timeoutMs: 8_000 });
    client.dispose();
    return Response.json(quote);
  }
};
```

### Lower-level signing

For control over an order you build yourself, hand a typed-data method to
`signOrderWithTypedDataSigner` — with viem or ethers, the wallet's `signTypedData`.
A raw EIP-1193 provider wraps into the same callback:

```ts
const typedDataSigner: TypedDataSignerCallback = (envelope) =>
  provider.request({
    method: "eth_signTypedData_v4",
    params: [owner, JSON.stringify({
      ...envelope,
      domain: { ...envelope.domain, chainId: Number(envelope.domain.chainId) }
    })]
  });
```

The result is an envelope whose `value` is the `SignedOrderDto` you submit with
`OrderBookClient.sendOrder`.

## The callback boundary

The package names host responsibilities as typed callbacks and never reaches past
them for a key or a provider:

- `TypedDataSignerCallback` — signs an EIP-712 typed-data envelope (wrap a viem or
  ethers signer, or a raw EIP-1193 provider's `eth_signTypedData_v4`).
- `DigestSignerCallback` — signs a raw digest for explicit EthSign flows.
- `CustomEip1271Callback` — returns a smart-account's final EIP-1271 signature.
- `ContractReadCallback` — performs a read-only `eth_call` and returns the
  ABI-decoded value as a decimal string or number (e.g. viem's `readContract`
  result via `String(value)`).
- `CowFetchCallback` — dispatches HTTP for Node, Workers, Deno, and custom hosts.

A callback may return a plain value, a Promise, or a thenable. Clients expose
`dispose()` and `[Symbol.dispose]` (so `using client = new …` works) and release
the callbacks they hold on disposal.

## Cancellation and timeouts

Every call accepts an optional `signal` (an `AbortSignal`) and `timeoutMs`.
Aborting the signal rejects the pending call with a `cancelled` `CowError`;
`timeoutMs` rejects with a `timeout` error. Both resolve the *awaited call* — an
already-dispatched HTTP request may keep running in the background until it
completes or the timeout elapses, so treat cancellation as "stop waiting," not a
guarantee that the network request is halted.

## Transport

Every client takes one transport:

```ts
transport: { kind: "fetch" }                       // standards `fetch` (browser, Node, Workers)
transport: { kind: "fetch", fetch: customFetch }   // a fetch you supply
transport: { kind: "callback", callback }          // you own request dispatch
```

Use `callback` when the host must own dispatch for fixtures, proxying, custom
authentication, or observability. Each client also takes optional
`transportPolicy` settings for retry, rate-limit, jitter, and user-agent behavior.

## Errors

Every call throws a single `CowError`, a real `Error` subclass: catch it, narrow
with `isCowError(e)` (or `e instanceof CowError`), then branch on `e.kind`. The
shape is a discriminated union — `transport`, `appData`, `signing`, `orderbook`,
`subgraph`, `walletRequest`, `walletTimeout`, `invalidInput`, `unknownEnumValue`,
`unsupportedChain`, `cancelled`, `internal`, and an `__unknown` forward-compatible
sentinel that preserves the unrecognised value in `raw`. Low-cardinality fields
are visible; URLs, headers, bodies, and secret-shaped values are redacted, and the
`message` is actionable on its own.

```ts
import {
  CowError,
  isCowError,
  isRetryable,
  isUserRejection,
  retryAfterMs,
  withRetry,
} from "@symbiome-forge/cow-sdk-wasm/trading";

// Building on the quickstart's `trading` client and `quoteRequest`:
try {
  // `withRetry` retries only a transient orderbook failure, waiting the server's
  // `Retry-After` (or an exponential backoff), and rethrows everything else. The
  // optional `onRetry` hook is for telemetry — it never alters the outcome.
  const quote = await withRetry(() => trading.getQuote(quoteRequest), {
    onRetry: (attempt, error, delayMs) => console.warn(`retry ${attempt} in ${delayMs}ms`, error.kind),
  });
} catch (e) {
  if (!isCowError(e)) throw e;
  if (isUserRejection(e)) {
    // Declined signature or cancellation — a soft state, not a failure.
  } else if (e.kind === "invalidInput") {
    // `e.field` names the offending field; fix it and retry.
  } else if (e.kind === "orderbook") {
    // `e.errorType` is the exact services tag — pick the right action:
    if (e.errorType === "InsufficientAllowance") {
      // Prompt a token approval.
    } else if (e.errorType === "InsufficientBalance") {
      // Prompt the user to add funds.
    } else if (isRetryable(e)) {
      // Transient: retry later, after `retryAfterMs(e)` ms when present.
    }
  }
  throw e;
}
```

The `orderbook` variant carries `retryable` and an optional `retryAfterMs` parsed
from the response `Retry-After`, mirroring the native
`OrderbookError::is_retryable` / `backoff_hint`, so a JavaScript retry loop reaches
the same verdict as the Rust one. It also carries `errorType` — the exact services
rejection tag (`"InsufficientAllowance"` vs `"InsufficientBalance"`, and the rest)
— the fine-grained partner of the coarse `category`. `isUserRejection(e)` is `true`
for a declined wallet request (`4001`) or a cancellation, so a UI can show those as
a soft state rather than a failure. `normalizeError(value)` coerces an arbitrary
caught value to a `CowError`; `e.toJSON()` and the static `CowError.fromJSON(value)`
move an error across a `structuredClone` / worker boundary without losing its
fields.

## Bundle size

Built with release-size settings and a `wasm-opt -Oz` pass. The figures below are
representative of an alpha build (gzip is the compressed-transfer figure); the
binding contract is the per-build **Release gate** column, which CI enforces:

| Flavor | Raw wasm | Brotli | Gzip | Release gate |
| --- | ---: | ---: | ---: | --- |
| signing | 0.31 MiB | 120 KiB | 143 KiB | 0.9 MiB raw / 300 KiB brotli |
| orderbook | 1.02 MiB | 338 KiB | 445 KiB | 1.5 MiB raw / 500 KiB brotli |
| trading | 1.54 MiB | 489 KiB | 657 KiB | 3.2 MiB raw / 850 KiB brotli |
| default | 1.63 MiB | 512 KiB | 691 KiB | 3.3 MiB raw / 900 KiB brotli |

Each flavor emits one wasm binary shared across its bundler, Node, web, and
source-phase module targets — the web glue's default URL, the module glue's
`import source`, and the raw Worker module subpath all reuse the one bundler copy,
so a flavor's gzip figure above is its size on every runtime. Each flavor's gzip
size is enforced as a per-build byte budget within the current Cloudflare Workers
Paid/Bundled (~3 MB) compressed-size limit. End-to-end Workers behavior is exercised by the
`workers-vitest` CI job, which runs the package against `vitest-pool-workers`.

## Not in this package

Use the upstream TypeScript SDK for these until they ship in `cow-rs`: TWAP and
composable orders, cross-chain bridging, CoW Shed account abstraction, flash-loan
helpers, and hardware-wallet adapters. This package emits typed data or
transaction requests and lets the caller's wallet submit on-chain; it ships no
WASI, WebAssembly-component, or `no_std` guest target.

## More

- The public TypeScript surface for each flavor is the committed declaration
  snapshot under `crates/wasm/snapshots/facade/`.
- [Architecture](https://github.com/0xSymbiome/cow-rs/blob/main/docs/architecture.md),
  [Observability](https://github.com/0xSymbiome/cow-rs/blob/main/docs/observability.md),
  and the
  [WASM Surface Audit](https://github.com/0xSymbiome/cow-rs/blob/main/docs/audit/wasm-surface-audit.md).
- Runnable browser, Node, and Worker examples live in the
  [`cow-sdk-examples`](https://github.com/0xSymbiome/cow-sdk-examples) repository.

Licensed under GPL-3.0-or-later.
