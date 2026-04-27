# Verification Matrix

| Gate | Purpose |
| --- | --- |
| `cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2026-0097 --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2024-0436` | RustSec gate |
| `cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk-browser-wallet -p cow-sdk` returns empty | Stability gate |
