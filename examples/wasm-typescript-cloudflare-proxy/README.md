# Cloudflare Orderbook Proxy WASM Example

This example runs the Cloudflare flavor of the TypeScript-callable WASM package
inside a Worker and forwards orderbook requests from the SDK transport callback
through Worker `fetch`.

The repository-local project imports `cow-sdk-wasm-local` from the workspace so
the example can run before publication. In an application, replace that module
specifier with the final `<published-cow-sdk-wasm-package>` package name.

## Run

```text
pnpm install --frozen-lockfile
pnpm test
```

`pnpm test` typechecks the Worker, bundles a deployable Worker entrypoint, runs
`wrangler deploy --dry-run`, and executes the Worker tests in the Cloudflare
runtime pool.

## Worker Routes

- `GET /health` initializes the WASM module and reports supported chain IDs.
- `POST /quote` reads an `OrderQuoteRequestInput`, calls `OrderBookClient.getQuote`,
  and proxies the SDK's orderbook HTTP request through Worker `fetch`.

## Configuration

- `COW_CHAIN_ID`: optional numeric chain ID, default `1`.
- `COW_ENV`: optional SDK environment, default `prod`.
- `COW_PARTNER_API_KEY`: optional partner API key forwarded through the SDK
  client configuration.
