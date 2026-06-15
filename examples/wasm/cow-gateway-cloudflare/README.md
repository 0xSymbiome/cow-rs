# Cloudflare Edge Gateway — Cloudflare-Flavor WASM Example

A Cloudflare Worker that runs CoW Protocol order flow on Cloudflare's edge
runtime, acting as a gateway in front of the CoW orderbook. It is built on the
**`cloudflare` flavor** of the TypeScript-callable WASM package, which targets the
Cloudflare Workers `web` runtime.

The repository-local project imports `cow-sdk-wasm-local` from the workspace so
the example can run before publication. In an application, replace that module
specifier with the final `<published-cow-sdk-wasm-package>` package name and
import from its `/cloudflare` subpath.

For most browser dapps and CowSwap-style UIs the upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk) is
the recommended, substantially smaller choice. Reach for the cloudflare flavor
when you are running order flow inside a Worker.

## Run

This example depends on the workspace WASM package (`cow-sdk-wasm-local`) through
a local `file:` dependency, and that package's `dist/` is a build artifact (it is
gitignored). Build it once first, from the repository root:

```text
pnpm --dir crates/wasm/npm build   # builds cow-sdk-wasm-local (requires wasm-pack + binaryen)
```

Then, from this directory:

```text
pnpm install
pnpm test
```

If you pull changes that touch the package's TypeScript facade, rebuild the
package (above) and re-run `pnpm install` here so the local copy picks up the new
types.

`pnpm test` type-checks the Worker, bundles a deployable entrypoint with the wasm
module as a Cloudflare `CompiledWasm` binding, runs `wrangler deploy --dry-run`,
and executes the tests in the Cloudflare runtime pool.

## Worker routes

- `GET /health` — initializes the wasm module once per isolate and reports the
  supported chain IDs.
- `POST /quote` — reads an `OrderQuoteRequestInput`, calls
  `OrderBookClient.getQuote`, and returns the quote. On an upstream failure it
  returns a structured error: a retryable orderbook failure (one the SDK retried
  and exhausted, such as a rate limit or a server-fault status) becomes a `503`
  with a `Retry-After` header derived from the SDK's `retryAfterMs` hint, and
  every other failure becomes a `502`.

## How it maps to the SDK

The cloudflare flavor is a `web`-target build initialized explicitly from a
bundled wasm module — no dynamic compilation:

```ts
import initialize, { OrderBookClient } from "<published-cow-sdk-wasm-package>/cloudflare";
import wasmModule from "<published-cow-sdk-wasm-package>/cloudflare/wasm";

await initialize(wasmModule); // once per isolate
const client = new OrderBookClient({ chainId: 1, env: "prod", apiKey, transport: { kind: "fetch" } });
```

### Transport: fetch first, host-owned egress when you need it

By default the gateway uses `transport: { kind: "fetch" }` — the SDK calls the
Worker's global `fetch`. The **partner API key is a first-class client field**
(`apiKey`), so a simple gateway needs no custom transport at all.

Switch to `transport: { kind: "callback" }` only to add an edge concern the SDK
does not model — observability, caching, rate-limiting, or a gateway-level auth
header. Setting `COW_TRACE=1` turns on the example's callback transport, which
logs one structured line per outbound request and **still delegates to the platform
`fetch`** (it does not re-implement HTTP).

### Relaying upstream backoff

The SDK retries transient orderbook failures internally; when it exhausts that
budget it surfaces a typed `WasmError`. The `orderbook` variant carries
`retryable` and an optional `retryAfterMs` (parsed from the orderbook's
`Retry-After` header), so the gateway can relay a retryable failure as a `503`
with a `Retry-After` header instead of hiding it behind a generic `502`:

```ts
catch (error) {
  // 503 + Retry-After when error.kind === "orderbook" && error.retryable
  return upstreamErrorResponse(error);
}
```

## Configuration

- `COW_CHAIN_ID` — optional numeric chain ID, default `1`.
- `COW_ENV` — optional SDK environment (`prod` or `staging`), default `prod`.
- `COW_PARTNER_API_KEY` — optional partner API key, forwarded by the SDK client.
- `COW_TRACE` — set to `1` to enable the host-owned egress (request tracing) path.

## Bundle size

The cloudflare flavor's gzip-compressed size is gated on every package release
build against a conservative 3,000,000-byte budget that stays under Cloudflare's
published 3 MB Free-plan compressed-size limit. `wrangler deploy --dry-run`
reports the deployable bundle size for this Worker; full Workers support also
requires Worker startup measurement against Cloudflare's 1-second startup limit.

## Quality

The example is held to the same bar as the crates:

```text
pnpm run build   # esbuild bundle (localizes the wasm as a CompiledWasm module) + wrangler deploy dry-run
pnpm test        # tsc clean + build + Cloudflare runtime-pool tests pass
```
