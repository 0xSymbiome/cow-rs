---
type: Decision Record
id: ADR-0025
title: "ADR 0025: Redact Credential-Bearing URL Fields At Storage Boundaries"
description: "Credential-bearing URL fields are stored in redacting types before they become part of public SDK state."
status: Accepted
date: 2026-04-27
authors: ["0xSymbiotic"]
tags: [security, redaction, configuration, diagnostics]
related: [ADR-0005, ADR-0013]
timestamp: 2026-04-27T00:00:00Z
---

# ADR 0025: Redact Credential-Bearing URL Fields At Storage Boundaries

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
- Runtime and support: request dispatch and IPFS fetch continue to use the
  exact raw URL bytes supplied by the caller through explicit accessors.
- Credential placement: where the protocol allows a credential to travel
  outside the URL, SDK-derived routing keeps it out of the URL rather than
  embedding it in a path or query. Subgraph production routing sends the partner
  Graph API key in the request `Authorization: Bearer` header against the
  key-free gateway URL, so the key never enters a stored URL, a request path, a
  telemetry span endpoint, or an error context, and there is nothing to redact
  on those surfaces.
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

**Proven by:**

- [Credential Redaction Audit](../audit/credential-redaction-audit.md)
