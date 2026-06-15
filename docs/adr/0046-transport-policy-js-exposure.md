# ADR 0046: Expose Transport Policy Configuration To JavaScript Clients

- Status: Superseded by [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md)
- Date: 2026-05-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, transport, retry, javascript-config

## Superseded

The typed `TransportPolicyConfig` that lets JavaScript callers override retry,
rate-limit, timeout, and user-agent policy without bypassing the shared Rust
`TransportPolicy` contract is recorded in
[ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), whose Must-Remain-True
maps the JavaScript transport-policy shape onto the shared Rust contract.
