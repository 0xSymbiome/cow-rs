---
type: Decision Record
id: ADR-0024
title: "ADR 0024: Split AsyncProvider Into Read-Only And Signing-Capable Traits"
description: "Provider is the read-only async chain-RPC trait."
status: Accepted
date: 2026-04-24
authors: ["0xSymbiotic"]
tags: [core, provider, signing, async, dependencies]
related: [ADR-0005, ADR-0010]
timestamp: 2026-04-24T00:00:00Z
---

# ADR 0024: Split AsyncProvider Into Read-Only And Signing-Capable Traits

## Decision

`Provider` is the read-only async chain-RPC trait. It owns the provider
error type and read methods such as chain-id lookup, bytecode lookup, contract
calls, and block reads.

Signer creation lives in `SigningProvider: Provider`. That extension
owns `type Signer: Signer<Error = Self::Error>` and
`create_signer`. Wallet-capable providers implement both traits. Read-only
adapters implement only `Provider`.

The `Provider` and `SigningProvider` method sets are frozen through `0.x.y`. A
genuinely new RPC primitive — one that cannot be expressed through the existing
methods, such as `get_logs` — lands on the core read trait (while pre-`0.1.0`)
or as its own opt-in capability supertrait in the `SigningProvider` mould (for
example `LogProvider`, [ADR 0057](0057-log-provider-capability-trait.md)).
Because the core traits use
native `async fn` and are not object-safe, there is no `dyn` vtable for an added
method to break, so the forward-compatibility basis is review discipline plus
core minimalism, with the release-time semver gate
([ADR 0030](0030-workspace-locked-versioning-tag-baseline.md)) reactivating on
the 1.0 runway.

## Why

Read-only chain adapters should not carry signer dependencies merely to expose
RPC reads. The split keeps signer creation explicit and opt-in while allowing
native provider adapters to remain dependency-light. It also preserves the
EIP-1193 path: a wallet-backed provider still implements the read trait and
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
- Trait evolution: the `Provider` / `SigningProvider` method sets stay frozen
  through `0.x.y`; a new RPC primitive lands on the core read trait or its own
  opt-in capability supertrait.

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

- [Architecture](../guides/architecture.md)
- [Providers](../providers/index.md)
- [Adapting alloy providers](../providers/adapting-alloy.md)
