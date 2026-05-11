# Node.js viem WASM Example

This example signs a CoW order in Node.js 22 or 24 through viem's EIP-1193
wallet request path.

The repository-local project imports `cow-sdk-wasm-local` from the workspace so
the example can run before publication. In an application, replace that module
specifier with the final `<published-cow-sdk-wasm-package>` package name.

## Run

```text
pnpm install --frozen-lockfile
pnpm test
```

## What It Shows

- `createWalletClient` from viem wrapping an EIP-1193 provider.
- `signOrderWithEip1193` delegating the typed-data request to the viem wallet
  client.
- A per-call wallet timeout through `walletConfig.timeoutMs`.
