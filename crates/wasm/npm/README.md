# cow-sdk-wasm package

TypeScript-callable WebAssembly bindings for the CoW Protocol Rust SDK.

The final npm package name is selected at publication time. Until then, the
examples below use `<published-cow-sdk-wasm-package>` as the placeholder module
specifier. The package exposes a TypeScript facade over deterministic Rust
protocol logic. JavaScript and TypeScript consumers get typed DTOs, explicit
wallet and HTTP callbacks, per-call cancellation, per-call timeouts, and
flavor-specific imports without depending on a specific wallet library.

## When to use this SDK

| You are building... | Choose | Why |
| --- | --- | --- |
| Browser dapp with viem, ethers, wagmi, or an EIP-1193 wallet | `<published-cow-sdk-wasm-package>` | Wallet stack stays outside the package behind typed callbacks |
| Browser dapp with a smaller orderbook bundle target | `<published-cow-sdk-wasm-package>/orderbook` | Orderbook and signing subset with a smaller raw wasm budget |
| Node.js 22 or 24 LTS backend | `<published-cow-sdk-wasm-package>` | Node target works without browser polyfills when transport is configured |
| Cloudflare Worker proxying CoW orderbook calls | `<published-cow-sdk-wasm-package>/cloudflare` | Worker-compatible web target and explicit wasm module initialization |
| Signer service or HSM proxy | `<published-cow-sdk-wasm-package>/signing` | Signing primitives without orderbook, trading, subgraph, or IPFS clients |
| Trading dashboard with quotes, orders, volumes, and app-data reads | `<published-cow-sdk-wasm-package>/full` | Full facade surface in one package flavor |
| Native Rust service, bot, solver, or treasury automation | `cow-sdk` | Avoids wasm-bindgen and npm packaging entirely |
| Rust app compiled to browser WASM | `cow-sdk-browser-wallet` plus `cow-sdk-transport-wasm` | Rust-on-wasm path; this package is for JavaScript hosts |

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
import {
  OrderBookClient,
  signOrderWithTypedDataSigner
} from "<published-cow-sdk-wasm-package>";

const client = new OrderBookClient({
  chainId: 1,
  env: "prod",
  transport: { kind: "fetch" },
  transportPolicy: {
    retryPolicy: { maxAttempts: 3, initialDelayMs: 200 },
    userAgent: "my-node-service/1.0"
  }
});

const quote = await client.getQuote({
  sellToken: "0x0000000000000000000000000000000000000000",
  buyToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  from: "0x1111111111111111111111111111111111111111",
  receiver: "0x1111111111111111111111111111111111111111",
  sellAmountBeforeFee: "1000000000000000000",
  kind: "sell"
});

const signed = await signOrderWithTypedDataSigner(
  quote.data.orderToSign,
  1,
  "0x1111111111111111111111111111111111111111",
  async (envelope) => walletClient.signTypedData(envelope),
  { walletConfig: { timeoutMs: 15_000 } }
);
```

### Browser with `window.ethereum`

```ts
import { OrderBookClient, signOrderWithEip1193 } from "<published-cow-sdk-wasm-package>";

const ethereum = window.ethereum;
const [owner] = await ethereum.request({ method: "eth_requestAccounts" });

const client = new OrderBookClient({
  chainId: 1,
  env: "prod",
  transport: { kind: "fetch" },
  timeoutMs: 10_000
});

const quote = await client.getQuote(request, {
  signal: abortController.signal,
  timeoutMs: 10_000
});

const signed = await signOrderWithEip1193(
  quote.data.orderToSign,
  1,
  owner,
  (rpc) => ethereum.request(rpc),
  { signal: abortController.signal, walletConfig: { timeoutMs: 20_000 } }
);
```

### Cloudflare Worker

```ts
import initialize, {
  OrderBookClient
} from "<published-cow-sdk-wasm-package>/cloudflare";
import wasmModule from "<published-cow-sdk-wasm-package>/cloudflare/wasm";

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
| `<published-cow-sdk-wasm-package>` | Default facade with orderbook, signing, app-data, IPFS, trading, and subgraph | General TypeScript or Node use |
| `<published-cow-sdk-wasm-package>/orderbook` | Orderbook client, cancellation helpers, and signing helpers | Browser dapps that do not need trading or subgraph clients |
| `<published-cow-sdk-wasm-package>/signing` | Signing, UID, EIP-1271, deployment, and version helpers | Signer services and HSM-facing adapters |
| `<published-cow-sdk-wasm-package>/full` | Full facade surface | Consumers that want every current client through one import |
| `<published-cow-sdk-wasm-package>/cloudflare` | Worker-compatible orderbook and trading facade | Cloudflare Workers |
| `<published-cow-sdk-wasm-package>/cloudflare/wasm` | Raw Worker wasm module asset | Pass to the Cloudflare `initialize` helper |

Do not import from `dist/raw` or generated wasm-pack target directories. Raw
wasm-bindgen output is package-internal; public imports go through the facade
subpaths above.

## Performance and bundle size

The package is built with release-size settings and a `wasm-opt -Oz` post-pass.
The current measured release artifacts are:

| Flavor | Raw wasm | Brotli | Gzip | Gate |
| --- | ---: | ---: | ---: | --- |
| default | 2.97 MiB | 790 KiB | 1129 KiB | 3.0 MiB raw / 800 KiB brotli |
| orderbook | 0.98 MiB | 321 KiB | 426 KiB | 1.5 MiB raw / 500 KiB brotli |
| signing | 0.43 MiB | 150 KiB | 183 KiB | 0.9 MiB raw / 300 KiB brotli |
| full | 2.97 MiB | 790 KiB | 1129 KiB | 3.0 MiB raw / 1000 KiB brotli |
| cloudflare | 2.88 MiB | 768 KiB | 1095 KiB | 3.0 MiB raw / 800 KiB brotli / 3.0 MiB gzip |

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
rate-limit, jitter, tracing, and user-agent behavior.

## Architecture

The TypeScript facade is the public package contract. It:

- exposes camelCase TypeScript APIs;
- hides raw wasm-bindgen resource-management members;
- maps raw wasm errors into `SdkError`;
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

## Migration from `@cowprotocol/cow-sdk`

See [Migration From @cowprotocol/cow-sdk](../../../docs/migration-from-cowprotocol-cow-sdk.md)
for side-by-side guidance on adapter setup, orderbook clients, signing
callbacks, Cloudflare Workers, error handling, and import selection.
