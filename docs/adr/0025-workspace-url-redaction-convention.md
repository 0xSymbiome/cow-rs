# ADR 0025: Redact Credential-Bearing URL Fields At Storage Boundaries

- Status: Accepted
- Date: 2026-04-27
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: security, redaction, configuration, diagnostics
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)

## Decision

Credential-bearing URL fields are stored in redacting types before they become
part of public SDK state. Single URL fields use `Redacted<String>`. Required
URL maps use `RedactedUrlMap<K>`. Optional URL maps use
`RedactedOptionalUrlMap<K>` so unsupported-chain `None` entries remain visible
without exposing configured endpoint bytes.

## Why

Endpoint URLs commonly carry credentials in userinfo, paths, or query strings.
Relying on each caller to avoid `Debug`, `Display`, or generic serialization is
fragile. The SDK instead makes accidental public formatting safe by default and
requires raw URL access to happen through explicit accessors at dispatch seams.

## Must Remain True

- Public surface: public `Debug`, `Display`, and `Serialize` output for
  credential-bearing URL fields emits `[redacted]` for configured URL values
  while preserving map keys and unsupported-chain `None` markers.
- Runtime and support: request dispatch, IPFS fetch, and
  `wallet_addEthereumChain` payload construction continue to use the exact raw
  URL bytes supplied by the caller through explicit accessors.
- Validation and review: regression tests cover compact debug, pretty debug,
  JSON serialization, and byte-identical raw dispatch access for every
  credential-bearing URL surface.
- Cost: adding a new URL-shaped public field requires choosing the matching
  redacting storage type before deriving `Debug` or `Serialize`.

## Alternatives Rejected

- Per-struct custom formatting: too easy for future fields to bypass, and it
  duplicates the redaction contract across crates.
- Plain `BTreeMap<K, Redacted<String>>`: works for required URL maps but does
  not model optional subgraph support markers cleanly.
- Post-format string filtering: can miss non-standard credential placement and
  treats leaked bytes as acceptable intermediate state.

## Links

- [Core redaction wrappers](../../crates/core/src/redaction/wrappers.rs)
- [Browser wallet chain parameters](../../crates/browser-wallet/src/wallet/chain.rs)

**Proven by:**

- [URL Credential Redaction Audit](../audit/url-credential-redaction-audit.md)
- [Credential Surface Audit](../audit/credential-surface-audit.md)
- [Credential Surface Contract Hygiene Audit](../audit/credential-surface-contract-hygiene-audit.md)
