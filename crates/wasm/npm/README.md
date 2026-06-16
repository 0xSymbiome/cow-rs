# @symbiome-forge/cow-sdk-wasm

TypeScript-callable WebAssembly bindings for the CoW Protocol Rust SDK.

```sh
npm install @symbiome-forge/cow-sdk-wasm@alpha
```

The package exposes a TypeScript facade over deterministic Rust protocol logic.
JavaScript and TypeScript consumers get typed DTOs, explicit wallet and HTTP
callbacks, per-call cancellation, per-call timeouts, and flavor-specific imports
without depending on a specific wallet library.

## When to use this SDK

| You are building... | Choose | Why |
| --- | --- | --- |
| Browser dapp with viem, ethers, wagmi, or an EIP-1193 wallet | `@symbiome-forge/cow-sdk-wasm` | Wallet stack stays outside the package behind typed callbacks |
| Browser dapp with a smaller orderbook bundle target | `@symbiome-forge/cow-sdk-wasm/orderbook` | Orderbook and signing subset with a smaller raw wasm budget |
| Node.js 22 or 24 LTS backend | `@symbiome-forge/cow-sdk-wasm` | Node target works without browser polyfills when transport is configured |
| Cloudflare Worker proxying CoW orderbook calls | `@symbiome-forge/cow-sdk-wasm/cloudflare` | Worker-compatible web target and explicit wasm module initialization |
| Signer service or HSM proxy | `@symbiome-forge/cow-sdk-wasm/signing` | Signing primitives without orderbook, trading, subgraph, or IPFS clients |
| Native Rust service, bot, solver, or treasury automation | `cow-sdk` | Avoids wasm-bindgen and npm packaging entirely |
| Rust app compiled to browser WASM | `cow-sdk` with `cow-sdk-core`'s browser `FetchTransport` (the `wasm32-unknown-unknown` `transport::fetch` module) | Rust-on-wasm path; this package is for JavaScript hosts |

## Not in this crate

Use the upstream TypeScript SDK packages until these capability families ship
in `cow-rs`:

- TWAP and composable orders.
- Cross-chain bridging.
- Cow Shed account abstraction.
- Flash-loan helpers.
- Weiroll command planning.
- Hardware wallet adapters.
- On-chain transaction submission; this package emits typed data or
  transaction requests and lets the caller's wallet submit.
- WASI, WebAssembly components, TinyGo, Blazor, AssemblyScript guests, and
  `no_std` embedded targets.

## Quickstart

### Node.js 22 or 24 with viem

```ts
import { TradingClient } from "@symbiome-forge/cow-sdk-wasm";

const trading = new TradingClient({
  chainId: 1,
  env: "prod",
  appCode: "my-node-service",
  transport: { kind: "fetch" },
  transportPolicy: {
    retryPolicy: { maxAttempts: 3, baseDelayMs: 200 },
    userAgent: "my-node-service/1.0"
  }
});

// `getQuote` returns a fully resolved `QuoteResultsDto` envelope.
const quote = await trading.getQuote({
  kind: "sell",
  sellToken: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
  buyToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  amount: "1000000000000000000"
});

// Reuse the quote to sign and post in one call; `quote.value` is the
// `QuoteResultsDto` and the callback receives the EIP-712 envelope.
const result = await trading.postSwapOrderFromQuote(
  quote.value,
  "0x1111111111111111111111111111111111111111",
  async (envelope) => walletClient.signTypedData(envelope),
  { walletConfig: { timeoutMs: 15_000 } }
);
// `result.value.orderId` is the posted order UID.
```

### Browser with `window.ethereum`

```ts
import { signOrderWithEip1193 } from "@symbiome-forge/cow-sdk-wasm";

const ethereum = window.ethereum;
const [owner] = await ethereum.request({ method: "eth_requestAccounts" });
const abortController = new AbortController();

// The order to sign: build it yourself or map it from a fetched quote.
const order = {
  sellToken: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
  buyToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  receiver: owner,
  sellAmount: "1000000000000000000",
  buyAmount: "3500000000",
  validTo: Math.floor(Date.now() / 1000) + 3_600,
  appData: "0x0000000000000000000000000000000000000000000000000000000000000000",
  feeAmount: "0",
  kind: "sell",
  partiallyFillable: false,
  sellTokenBalance: "erc20",
  buyTokenBalance: "erc20"
};

const signed = await signOrderWithEip1193(
  order,
  1,
  owner,
  (rpc) => ethereum.request(rpc),
  { signal: abortController.signal, walletConfig: { timeoutMs: 20_000 } }
);
// `signed.value` is the SignedOrderDto.
```

### Browser with MetaMask `eth_signTypedData_v4`

When the wallet exposes the typed-data JSON-RPC method directly, callers can
pass the envelope to `eth_signTypedData_v4` from inside the typed-data signer
callback. The helper hands the callback a typed-data envelope — plain `domain`,
`types`, `primaryType`, and `message` objects — that the callback serializes and
returns the signature string for.

```ts
import { signOrderWithTypedDataSigner } from "@symbiome-forge/cow-sdk-wasm";

const [owner] = await window.ethereum.request({ method: "eth_requestAccounts" });

const signed = await signOrderWithTypedDataSigner(order, 1, owner, async (envelope) => {
  const signature = await window.ethereum.request({
    method: "eth_signTypedData_v4",
    params: [owner, JSON.stringify(envelope)]
  });
  if (typeof signature !== "string") {
    throw new Error("wallet did not return a signature");
  }
  return signature;
});
```

### Cloudflare Worker

```ts
import initialize, {
  OrderBookClient
} from "@symbiome-forge/cow-sdk-wasm/cloudflare";
import wasmModule from "@symbiome-forge/cow-sdk-wasm/cloudflare/wasm";

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    await initialize(wasmModule);

    const client = new OrderBookClient({
      chainId: 1,
      env: "prod",
      apiKey: env.COW_PARTNER_API_KEY ?? null,
      transport: { kind: "fetch" },
      transportPolicy: { userAgent: "my-worker/1.0" }
    });

    const quote = await client.getQuote(await request.json(), {
      timeoutMs: 8_000
    });
    client.dispose();

    return Response.json(quote);
  }
};
```

## Choosing your import

| Import | Surface | Use when |
| --- | --- | --- |
| `@symbiome-forge/cow-sdk-wasm` | Default facade with orderbook, signing, app-data, IPFS, trading, and subgraph | General TypeScript or Node use |
| `@symbiome-forge/cow-sdk-wasm/orderbook` | Orderbook client, cancellation helpers, and signing helpers | Browser dapps that do not need trading or subgraph clients |
| `@symbiome-forge/cow-sdk-wasm/signing` | Signing, UID, EIP-1271, deployment, and version helpers | Signer services and HSM-facing adapters |
| `@symbiome-forge/cow-sdk-wasm/cloudflare` | Worker-compatible orderbook and trading facade | Cloudflare Workers |
| `@symbiome-forge/cow-sdk-wasm/cloudflare/wasm` | Raw Worker wasm module asset | Pass to the Cloudflare `initialize` helper |

Do not import from `dist/raw` or generated wasm-pack target directories. Raw
wasm-bindgen output is package-internal; public imports go through the facade
subpaths above.

## Performance and bundle size

The package is built with release-size settings and a `wasm-opt -Oz` post-pass.
Measured on the `0.1.0-alpha.1` build:

| Flavor | Raw wasm | Brotli | Gzip | Gate |
| --- | ---: | ---: | ---: | --- |
| default | 1.59 MiB | 501 KiB | 675 KiB | 3.3 MiB raw / 900 KiB brotli |
| orderbook | 0.99 MiB | 330 KiB | 432 KiB | 1.5 MiB raw / 500 KiB brotli |
| signing | 0.31 MiB | 119 KiB | 142 KiB | 0.9 MiB raw / 300 KiB brotli |
| cloudflare | 1.50 MiB | 478 KiB | 642 KiB | 3.2 MiB raw / 850 KiB brotli / 3,000,000 B gzip (warn at 2,700,000 B) |

The cloudflare flavor's gzip-compressed artifact is below the current
Cloudflare Workers Free compressed-size limit at the time of measurement.
Full Workers support still requires release-bundle verification and Worker
startup measurement; the release pipeline enforces the gzip byte budget on
every build, but Wrangler deployment and `startup_time_ms` telemetry are
separate operational gates.

Cloudflare Workers cold starts are runtime-sensitive. The package treats
300 ms as the warning threshold, 500 ms as the release gate, and 1 second as
the platform-limit budget that Worker consumers should stay well below.

## Transport configuration

Every client accepts one transport:

```ts
transport: { kind: "fetch" }
transport: { kind: "fetch", fetch: customFetch }
transport: { kind: "callback", callback: customHttpCallback }
```

Use `fetch` for browser, Node, and Worker runtimes that expose a standards
compatible `fetch`. Use `callback` when the host must own request dispatch,
fixtures, proxying, custom authentication, or observability.

Every client also accepts optional `transportPolicy` settings for retry,
rate-limit, jitter, and user-agent behavior.

## Cancellation and timeouts

Every call accepts an optional `signal` (an `AbortSignal`) and a per-call
`timeoutMs`. Aborting the signal rejects the pending call promptly with a
`cancelled` `CowError`; `timeoutMs` rejects with a `timeout` error. Both resolve
the *awaited call* — an already-dispatched HTTP request may keep running in the
background until it completes or the timeout elapses, so treat cancellation as
"stop waiting," not a guarantee that the network request is halted.

## Architecture

The TypeScript facade is the public package contract. It:

- exposes camelCase TypeScript APIs;
- exposes `dispose()` and `[Symbol.dispose]` (so `using client = new …` works)
  while hiding the raw wasm-bindgen `free()` handle;
- maps raw wasm errors into `CowError`;
- adapts `transport: { kind: "fetch" }` into the callback HTTP ABI;
- keeps wallet libraries outside the package behind named callback types.

## API reference

The declaration snapshots under `crates/wasm/snapshots/facade/` show the
public TypeScript surface for each flavor. Key exports include:

- clients: `OrderBookClient`, `TradingClient`, `SubgraphClient`, `IpfsClient`;
- signing helpers: `signOrderWithTypedDataSigner`, `signOrderWithEip1193`,
  `signOrderEthSignDigest`, `signOrderWithEip1271`,
  `signOrderWithCustomEip1271`;
- cancellation helpers: `signCancellationWithTypedDataSigner`,
  `signCancellationWithEip1193`, `signCancellationEthSignDigest`,
  `buildCancelOrderTx`, `buildPresignTx`;
- pure helpers: `domainSeparator`, `orderTypedData`, `computeOrderUid`,
  `deploymentAddresses`, `supportedChainIds`, `appDataInfo`,
  `validateAppDataDoc`, `appDataDoc`, `appDataHexToCid`,
  `cidToAppDataHex`, `wasmVersion`.

## When to use this package vs the upstream TypeScript SDK

For most browser dapps, web apps, and CowSwap-style UIs, the upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk)
is the recommended choice; it is substantially smaller at equivalent feature
subsets. This package is appropriate for specialized cases:

- TypeScript services that need byte-for-byte parity with the Rust SDK's
  EIP-712 + EIP-1271 signing path.
- Single-source-of-truth Rust + TypeScript embedding (one implementation
  across both runtimes).
- Cloudflare Workers (size-compatible with the current Workers Free
  compressed-size limit at the time of measurement; the `cloudflare` flavor
  is built and tested end-to-end in CI (Workers Vitest plus the Cloudflare
  gateway example), within the Workers compressed-size budget).
- Embeddable signing helpers (the `./signing` flavor is the smallest).

The "When to use this SDK" table at the top of this README routes consumers
by use case. The Quickstart sections above show the supported import shapes
for the most common runtimes.
