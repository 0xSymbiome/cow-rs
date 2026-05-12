# ADR 0046: Expose Transport Policy Configuration To JavaScript Clients

- Status: Accepted
- Date: 2026-05-11
- Last reviewed: 2026-05-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, transport, retry, javascript-config
- Related: [ADR 0013](0013-http-transport-injection-and-typestate-builders.md), [ADR 0019](0019-http-transport-sole-dispatch.md), [ADR 0041](0041-transport-policy-l3-layering.md)

## Decision

The TypeScript-callable package exposes a typed `TransportPolicyConfig` for
client constructors that use HTTP transport. JavaScript callers may override
retry attempts, retry delay, jitter, rate-limit behavior, timeout, and user
agent policy without bypassing the shared Rust `TransportPolicy` contract.

## Why

The transport policy crate is the canonical retry and rate-limit authority for
orderbook and subgraph clients. JavaScript consumers need the same policy
control when they run in browsers, Node.js, Cloudflare Workers, or custom fetch
runtimes. A typed config keeps the JavaScript surface explicit while preserving
the Rust classifier and default semantics.

## Must Remain True

- Omitting `TransportPolicyConfig` preserves the Rust defaults.
- Invalid user-agent and policy values fail during constructor validation.
- All HTTP-capable wasm clients translate the same JavaScript policy shape into
  the shared Rust policy.
- Runtime abort and timeout options remain separate from retry policy.
- Adding a Rust policy field requires an intentional TypeScript config
  decision and snapshot update.

## Alternatives Rejected

- Keep transport policy Rust-only: it would make JavaScript clients less
  controllable than native clients.
- Accept untyped JavaScript policy objects: flexible, but too easy to drift
  from the Rust policy contract.
- Put retry logic in the JavaScript facade: convenient for fetch callbacks, but
  it would duplicate classifier and `Retry-After` behavior.

## Links

- [Transport](../transport.md)
- [WASM Public API Stability Audit](../audit/wasm-public-api-stability-audit.md)
- [WASM Type Generation Audit](../audit/wasm-type-generation-audit.md)

**Proven by:**

- [WASM Public API Stability Audit](../audit/wasm-public-api-stability-audit.md)
- [WASM Type Generation Audit](../audit/wasm-type-generation-audit.md)
