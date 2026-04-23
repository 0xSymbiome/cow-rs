# Release Checklist

```text
cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2026-0097 --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2024-0436 --ignore RUSTSEC-2099-9999
cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk-browser-wallet -p cow-sdk
bun run --cwd e2e/browser-wallet playwright install --with-deps chromium firefox
```
