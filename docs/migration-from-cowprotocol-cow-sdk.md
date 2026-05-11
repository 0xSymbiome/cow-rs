# Migration From @cowprotocol/cow-sdk

This guide helps TypeScript consumers move selected CoW Protocol workflows from
the upstream `@cowprotocol/cow-sdk` packages to the cow-rs TypeScript-callable
WASM package.

The final npm package name is selected at publication time. Until then, examples
use `<published-cow-sdk-wasm-package>` as the module specifier.

## Should You Migrate?

Migrate the workflows that benefit from deterministic Rust protocol logic,
explicit wallet callbacks, Cloudflare Worker support, and a smaller dependency
surface.

Stay on upstream packages for capability families that are not part of this
package yet:

- TWAP and composable orders.
- Cross-chain bridging.
- Cow Shed account abstraction.
- Flash-loan helpers.
- Weiroll command planning.
- Hardware wallet adapters.
- Direct on-chain transaction submission helpers.

You can use both SDK families side by side: use the WASM package for supported
signing, orderbook, trading, app-data, subgraph, and IPFS flows, and keep
upstream packages for workflows that still live there.

## Install

```text
npm install <published-cow-sdk-wasm-package>
```

For local repository examples before publication, the examples use a workspace
alias named `cow-sdk-wasm-local`. Application code should use the final package
name once it is selected.

## Replace Global Adapter Setup

Upstream adapter setup usually centralizes wallet or transport behavior before
calling SDK methods.

```ts
import { setGlobalAdapter, ViemAdapter } from "@cowprotocol/cow-sdk";

setGlobalAdapter(new ViemAdapter({ wallet: walletClient }));
```

The WASM package uses explicit callbacks at the call site.

```ts
import { signOrderWithTypedDataSigner } from "<published-cow-sdk-wasm-package>";

const signed = await signOrderWithTypedDataSigner(
  order,
  1,
  owner,
  (typedData) => walletClient.signTypedData(typedData),
  { walletConfig: { timeoutMs: 20_000 } }
);
```

The wallet library remains application-owned. The SDK only requires a typed
callback that returns the signature string.

## Replace Orderbook Client Setup

```ts
import { OrderBookApi } from "@cowprotocol/cow-sdk";

const api = new OrderBookApi({ chainId: 1, env: "prod" });
```

```ts
import { OrderBookClient } from "<published-cow-sdk-wasm-package>";

const client = new OrderBookClient({
  chainId: 1,
  env: "prod",
  transport: { kind: "fetch" },
  transportPolicy: {
    userAgent: "my-app/1.0"
  }
});
```

Use `transport: { kind: "fetch" }` when the runtime exposes standards-compatible
`fetch`. Use `transport: { kind: "callback", callback }` when your application
owns proxying, custom headers, tracing, or test fixtures.

## Replace EIP-1193 Signing

```ts
import { signOrderWithEip1193 } from "<published-cow-sdk-wasm-package>";

const signed = await signOrderWithEip1193(
  order,
  1,
  owner,
  (request) => ethereum.request(request),
  { signal: abortController.signal, walletConfig: { timeoutMs: 20_000 } }
);
```

Use this path for viem, ethers, wagmi, browser wallets, and any wallet provider
that can handle EIP-1193 requests.

## Replace MetaMask Typed-Data Signing

```ts
import { signOrderWithTypedDataSigner } from "<published-cow-sdk-wasm-package>";

const [owner] = await window.ethereum.request({ method: "eth_requestAccounts" });

const signed = await signOrderWithTypedDataSigner(order, 1, owner, async (envelope) => {
  const types = envelope.types instanceof Map ? Object.fromEntries(envelope.types) : envelope.types;
  const signature = await window.ethereum.request({
    method: "eth_signTypedData_v4",
    params: [owner, JSON.stringify({ ...envelope, types })]
  });
  if (typeof signature !== "string") {
    throw new Error("wallet did not return a signature");
  }
  return signature;
});
```

The helper builds the CoW order typed data. Your callback only decides how the
wallet signs it.

## Replace Cloudflare Worker Proxying

```ts
import initialize, {
  OrderBookClient
} from "<published-cow-sdk-wasm-package>/cloudflare";
import wasmModule from "<published-cow-sdk-wasm-package>/cloudflare/wasm";

await initialize(wasmModule);

const client = new OrderBookClient({
  chainId: 1,
  env: "prod",
  apiKey: env.COW_PARTNER_API_KEY ?? null,
  transport: { kind: "fetch" },
  transportPolicy: { userAgent: "my-worker/1.0" }
});
```

Cloudflare Workers use the `./cloudflare` facade and the `./cloudflare/wasm`
module asset. Initialize the module once per isolate and reuse the initialized
package for subsequent requests.

## Error Handling

The WASM package throws typed `SdkError` objects instead of unstructured
JavaScript exceptions whenever the error crosses the SDK boundary.

```ts
try {
  await client.getQuote(request);
} catch (error) {
  const sdkError = error as { kind?: string; message?: string };
  if (sdkError.kind === "orderbook") {
    console.error(sdkError.message);
  }
}
```

Every known error variant carries `schemaVersion`, `kind`, and actionable
`message` text. Future variants normalize to `kind: "__unknown"` so applications
can log and surface them without breaking exhaustive switches.

## Resource Management

Client classes expose `dispose()`.

```ts
const client = new OrderBookClient(config);
try {
  return await client.getOrder(orderUid);
} finally {
  client.dispose();
}
```

Use `dispose()` when a client is short-lived, especially inside serverless
handlers. Long-lived services can keep clients for the process or isolate
lifetime.

## Import Selection

| Runtime or need | Import |
| --- | --- |
| General TypeScript, Node.js, or browser use | `<published-cow-sdk-wasm-package>` |
| Smaller browser orderbook bundle | `<published-cow-sdk-wasm-package>/orderbook` |
| Signing-only service | `<published-cow-sdk-wasm-package>/signing` |
| Full facade surface | `<published-cow-sdk-wasm-package>/full` |
| Cloudflare Worker facade | `<published-cow-sdk-wasm-package>/cloudflare` |
| Cloudflare Worker wasm module asset | `<published-cow-sdk-wasm-package>/cloudflare/wasm` |

Do not import generated wasm-bindgen files or package-internal `dist/raw` files.
Public imports go through the facade paths above.

## Example Projects

- [Node.js viem example](../examples/wasm-typescript-node-viem/README.md)
- [Browser MetaMask example](../examples/wasm-typescript-browser-mm/README.md)
- [Cloudflare orderbook proxy example](../examples/wasm-typescript-cloudflare-proxy/README.md)
