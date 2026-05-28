# Subgraph Error Display Audit

Status: Current
Last reviewed: 2026-05-28
Owning surface: `cow-sdk-subgraph`
Refresh trigger: any new variant on `cow_sdk_subgraph::SubgraphError`, any change to the `#[error(...)]` template on an existing variant, any change to `SubgraphRequestErrorContext`'s `Redacted<T>` field set, or any change to the workspace `Redacted<T>` `Display` impl that would alter the placeholder rendering
Related docs:
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [ADR 0017](../adr/0017-typed-orderbook-rejection-parser.md)

## Scope

This audit covers:

- The `Display` rendering of every diagnostic variant on
  `cow_sdk_subgraph::SubgraphError` (`Transport`, `HttpStatus`,
  `Serialization`, `GraphQl`, `MissingData`, `UnsupportedNetwork`)
- The pairing rule between redacted route identity (carried under
  `Redacted<String>` in `SubgraphRequestErrorContext.api`) and the
  plaintext structural diagnostic each variant interpolates alongside it
- The `first_graphql_location_suffix` helper that lifts the first
  GraphQL error's first source location into the `GraphQl` variant's
  `Display` template

It does not cover the `Debug` or `Serialize` renderings (governed by the
workspace credential redaction surface audit), the typed parsing of the
GraphQL response envelope (governed by the wire-DTO coverage audit), or
the `Cancelled` and `NoTotalsFound` variants, whose tag-only Display
strings are exhaustively descriptive on their own.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| `Transport` Display | Pairs the typed `class` label with plaintext chain id; redacted `api` and `details` stay behind the workspace placeholder | Conforms |
| `HttpStatus` Display | Pairs the numeric `status` code with plaintext chain id; redacted `api` and `body` stay behind the workspace placeholder | Conforms |
| `Serialization` Display | Pairs plaintext chain id with response-body byte count taken from the inner string length; redacted `api`, `body`, and `details` stay behind the workspace placeholder | Conforms |
| `GraphQl` Display | Pairs plaintext chain id with the error count and the first error's first source location when present; redacted `api` and per-error `message` stay behind the workspace placeholder | Conforms |
| `MissingData` Display | Pairs the variant tag with plaintext chain id; redacted `api` stays behind the workspace placeholder | Conforms |
| `UnsupportedNetwork` Display | Carries plaintext chain id only; no redacted fields | Conforms |
| `format!("{e}")` actionability | Every diagnostic variant carries at least one ASCII-digit token in its Display rendering, so the default formatting path remains useful when every `Redacted<T>` field collapses to the placeholder | Conforms |
| `Redacted<T>` posture | No `Display` template interpolates `.as_inner()` on any redacted field, including the free-form `errors[].message` payload on the `GraphQl` variant | Conforms |

## Current Contract

### Variant Display pairing rule

Every diagnostic `SubgraphError` variant whose typed shape includes a
`SubgraphRequestErrorContext` interpolates **both** the redacted
`context.api` and the plaintext `context.chain_id` into its
`#[error(...)]` template. The redacted route identity stays behind the
workspace `Redacted<T>` placeholder; the plaintext chain id stands
alongside it as structural diagnostic.

`Transport` additionally interpolates the typed
`TransportErrorClass` label (`builder`, `connect`, `timeout`, ...).
`HttpStatus` additionally interpolates the numeric status code.
`Serialization` additionally interpolates the byte count of the
response body taken from `body.as_inner().len()`. `GraphQl`
additionally interpolates `errors.len()` and, when present, the first
GraphQL error's first source location formatted as `at line:column`.

### `GraphQl` location helper

`first_graphql_location_suffix(errors)` returns ` at line:column` for
`errors[0].locations[0]` when both index slots are populated, and the
empty string otherwise. The fields rendered (`line: u32`, `column: u32`)
originate from the GraphQL specification as positions within the
SDK-submitted document and cannot carry credential-bearing content, so
they are safe to interpolate into the public `Display` template.

### What the Display path never interpolates

The free-form `errors[].message: Redacted<String>` payload on every
`SubgraphGraphQlError` is reachable only through the explicit
`.as_inner()` accessor at the call site. No `Display` template invokes
that accessor. The redaction contract sweep at
`crates/sdk/tests/error_redaction_contract.rs` exercises this with the
workspace `URL_SECRET`, `AUTH_SECRET`, `PRIVATE_KEY_SECRET`, and
`PEM_SECRET` fixtures plumbed through `errors[0].message` and asserts
they never appear in the rendered Display string.

### Non-tautology contract

The `subgraph_display_carries_plaintext_structural_diagnostic` test in
the redaction contract sweep asserts that every diagnostic variant's
rendered Display string contains at least one ASCII digit. The
acceptance set covers `UnsupportedNetwork` (chain id), `Transport`
(chain id), `HttpStatus` (chain id + status), `Serialization`
(chain id + byte count), `GraphQl` (chain id + count + optional
location), and `MissingData` (chain id). The check forbids a regression
that drops every plaintext field into `Redacted<T>`-only territory and
collapses the rendered output to a tautological
`for [redacted]` shape.

### Caller access pattern for upstream-authored content

Consumers that need the upstream-authored GraphQL error text — the
indexer's `errors[i].message` payload — reach it through explicit typed
access on the carried `errors` vector. The `.as_inner()` call on the
workspace `Redacted<T>` wrapper is the deliberate boundary-crossing
marker; the SDK never routes that payload through `Display`. The
`GraphQl` variant's rustdoc carries the canonical caller-side shape as
a doctest so the pattern is discoverable through standard reference
tooling. Callers that integrate the message into structured logging
should route it through a named log field rather than through a
free-form format string, so credential-carrying upstream content does
not flow into downstream sinks unintentionally.

## Evidence

Primary implementation points:

- `crates/subgraph/src/error.rs` (`SubgraphError` enum, the
  per-variant `#[error(...)]` templates, and the
  `first_graphql_location_suffix` helper)
- `crates/core/src/redaction/wrappers.rs` (the `Redacted<T>` `Display`
  impl emitting the workspace placeholder)

Primary regression coverage:

- `crates/subgraph/tests/error_contract.rs::graphql_display_includes_error_count_singular`
- `crates/subgraph/tests/error_contract.rs::graphql_display_includes_error_count_plural`
- `crates/subgraph/tests/error_contract.rs::graphql_display_includes_chain_id`
- `crates/subgraph/tests/error_contract.rs::graphql_display_includes_first_location_when_present`
- `crates/subgraph/tests/error_contract.rs::graphql_display_omits_location_when_absent`
- `crates/subgraph/tests/error_contract.rs::graphql_display_is_single_line`
- `crates/subgraph/tests/error_contract.rs::graphql_display_does_not_leak_message_content`
- `crates/subgraph/tests/error_contract.rs::serialization_display_includes_body_byte_count`
- `crates/subgraph/tests/error_contract.rs::missingdata_display_includes_chain_id`
- `crates/subgraph/tests/error_contract.rs::httpstatus_display_includes_chain_id_and_status_code`
- `crates/subgraph/tests/error_contract.rs::transport_variant_carries_typed_class_and_sanitized_detail`
- `crates/sdk/tests/error_redaction_contract.rs::subgraph_errors_and_contexts_redact_serialized_request_payloads`
- `crates/sdk/tests/error_redaction_contract.rs::subgraph_display_carries_plaintext_structural_diagnostic`

Validation surface:

```text
cargo nextest run -p cow-sdk-subgraph --test error_contract
cargo nextest run -p cow-sdk --test error_redaction_contract
```
