# ADR 0029: Trait Evolution Through Extension Traits

- Status: Rejected
- Date: 2026-04-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: traits, semver, compatibility, providers

## Rejected

This ADR proposed evolving the public traits through `*Ext` extension traits.
The pattern was rejected and never shipped — no `*Ext` trait exists anywhere in
the workspace. The SDK owns every trait it publishes, so it has no foreign or
sealed trait to extend: a genuinely new chain-RPC primitive lands directly on
the frozen core read trait (while pre-`0.1.0`) or as its own opt-in capability
supertrait in the `SigningProvider` mould — `SigningProvider` itself and
[`LogProvider`](0057-log-provider-capability-trait.md) — as recorded in
[ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md). The
`*Ext` blanket-trait idiom solves a problem the SDK does not have: adding
methods to a trait you do not own.
