# Browser MetaMask WASM Example

This example signs a CoW order in a browser through a MetaMask-style
`window.ethereum` provider.

The repository-local project imports `cow-sdk-wasm-local` from the workspace so
the example can run before publication. In an application, replace that module
specifier with the final `<published-cow-sdk-wasm-package>` package name.

## Run

```text
pnpm install --frozen-lockfile
pnpm test
```

## What It Shows

- Requesting accounts from `window.ethereum`.
- Wrapping MetaMask `eth_signTypedData_v4` in
  `signOrderWithTypedDataSigner`.
- Playwright coverage with a MetaMask-compatible injected provider.
