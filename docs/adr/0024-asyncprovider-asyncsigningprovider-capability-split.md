# ADR 0024: Split AsyncProvider Into Read-Only And Signing-Capable Traits

- Status: Accepted
- Date: 2026-04-24
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: core, provider, signing, async, dependencies
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)

## Decision

`Provider` is the read-only async chain-RPC trait. It owns the provider
error type and read methods such as chain-id lookup, bytecode lookup, contract
calls, and block reads.

Signer creation lives in `SigningProvider: Provider`. That extension
owns `type Signer: Signer<Error = Self::Error>` and
`create_signer`. Wallet-capable providers implement both traits. Read-only
adapters implement only `Provider`.

## Why

Read-only chain adapters should not carry signer dependencies merely to expose
RPC reads. The split keeps signer creation explicit and opt-in while allowing
native provider adapters to remain dependency-light. It also preserves the
browser-wallet path: an EIP-1193 provider still implements the read trait and
the signing extension, so wallet flows keep signer creation available through a
separate capability bound.

## Must Remain True

- Public surface: `Provider` carries no `Signer` associated type and no
  `create_signer` method. `SigningProvider: Provider` carries both.
- Runtime and support: wallet-capable providers implement both traits; read-only
  adapters implement only `Provider`.
- Dependency posture: read-only adapter crates do not need signer crates or
  wallet-runtime bindings to satisfy the provider trait.
- Validation: contract tests cover read-only dispatch and signing-capable
  dispatch on the same provider type.

## Alternatives Rejected

- Keep one async provider trait with a signer slot: forces every read-only
  adapter to name a signer type even when no signing capability exists.
- Use a default associated `NoSigner` placeholder: associated type defaults are
  not available on stable Rust, and a non-generic placeholder cannot match every
  provider error family.
- Require every provider error type to absorb a no-signer variant: widens
  unrelated error surfaces and turns a compile-time capability distinction into
  a runtime failure path.

## Links

- [Architecture](../architecture.md)
- [Providers](../providers/README.md)
- [Adapting alloy providers](../providers/adapting-alloy.md)

**Proven by:**

- [Browser Wallet Trust Posture Audit](../audit/browser-wallet-trust-posture-audit.md)
