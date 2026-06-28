# Bounded Response Reads Audit

Status: Current
Last reviewed: 2026-06-20
Owning surface: HTTP transport response reads across `cow-sdk-core` (including its `transport::policy` module and the browser `FetchTransport` in its `transport::fetch` module), `cow-sdk-js`, and the signature decode path in `cow-sdk-contracts`
Refresh trigger: changes to the transport read loops, the `max_response_bytes` policy field or its per-client defaults, the `ResponseTooLarge` classification, the signature hex bound, or the reqwest/web-sys decompression posture
Related docs:
- [ADR 0055](../adr/0055-bounded-response-reads.md)
- [ADR 0041](../adr/0041-transport-policy-l3-layering.md)
- `PROP-CORE-007`

## Scope

This audit covers:

- the byte bound applied to HTTP response bodies the SDK buffers, on both the
  success path and the non-2xx error path
- the per-client default bounds carried by the transport policy
- the non-retryable classification of an over-limit outcome
- the pre-decode bound on signature hex fields
- the documented residual boundaries where the SDK does not own the read loop

It does not cover request-body construction, the URL-redaction contract
(covered by the credential-redaction audit), or the on-chain log decoder
(covered by the event-log-decoding audit).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Native transport read | Response and error bodies are streamed under `max_response_bytes` and refused past the limit | Conforms |
| Decompression bomb | When the build enables reqwest gzip decompression, the bound is on decoded bytes, so an amplified body is refused on its decoded size | Conforms (with decompression enabled) |
| Browser and JS-callback reads | A post-receipt bound refuses oversized bodies the JS layer materialized | Conforms |
| Per-client defaults | Untrusted gateways carry tighter bounds than the trusted orderbook | Conforms |
| Retry posture | An over-limit outcome is classified non-retryable | Conforms |
| Signature decode | Signature hex is length-bounded before the decoder allocates | Conforms |
| Residual boundaries | RPC-stack and IPFS time bounds are stated, not implied closed | Documented |

## Current Contract

### Native transport read

`ReqwestTransport` reads the response body as a stream of chunks with a
pre-extend check, so the accumulator never exceeds the configured limit and an
over-limit body is rejected after at most one over-limit chunk. The bound applies
to both the success body and the non-2xx error body. Decoding is lenient, so a
non-UTF-8 body is handled without a new rejection path.

### Decompression bomb

When the consuming build enables reqwest's `gzip` feature, reqwest decompresses
before yielding chunks, so the bound observes the decoded size: a small
compressed body that decodes far past the limit is refused on its decoded size,
not its compressed size. The `cow-sdk` umbrella gets this via the orderbook and
subgraph clients, which enable `gzip`; a `cow-sdk-core`-only consumer of the
exported `ReqwestTransport` builds without `gzip`, so reqwest yields the raw
compressed bytes and the bound applies to the compressed size instead. This is
the only sound bound when transparent decompression is enabled.

### Browser and JS-callback reads

The browser `FetchTransport` and the runtime-neutral JS-callback transport each
apply the same limit as a post-receipt bound on a body the surrounding JS layer
has already materialized. The residual — that the JS-side allocation precedes the
SDK's view — is documented.

### Per-client defaults

The orderbook and trading clients use the generous workspace default; the
untrusted subgraph gateway uses a tighter default; the IPFS app-data read uses a
bound sized at twice the protocol app-data document limit (16 KiB against the
8 KiB `APP_DATA_MAX_BYTES`). All values are
instance-scoped policy and caller-overridable. The transport-policy builder
refines a caller-set client policy in place, so a caller-tightened
`max_response_bytes` — and a deliberately disabled timeout — survives a later
`user_agent` or `timeout` refinement instead of resetting to the default.

### Retry posture

An over-limit outcome maps to a dedicated non-retryable network kind, so the
shared retry driver never re-requests a deterministically over-limit response.

### Signature decode

Signature hex fields are length-bounded (at the orderbook request-body limit)
before the hex decoder allocates. The bound is generous enough never to reject a
valid signature, and it refuses oversized input before a large decode allocation.

### Residual boundaries

The JSON-RPC client the SDK builds disables response decompression and is
otherwise bounded by the request timeout; the alloy-managed RPC client is outside
the SDK's read loop and bounded by timeout and caller trust; the IPFS read is
byte-bounded but, by default, not time-bounded. These residuals are stated in the
security policy rather than presented as hard caps.

## Evidence

Primary implementation points:

- `crates/core/src/transport/reqwest.rs`
- `crates/core/src/config/http.rs`
- `crates/core/src/validation.rs`
- `crates/core/src/transport/policy/config.rs`
- `crates/core/src/transport/policy/classify.rs`
- `crates/core/src/transport/fetch.rs`
- `crates/js/src/exports/transport.rs`
- `crates/contracts/src/hex_field.rs`
- `crates/contracts/src/signature.rs`

Primary regression coverage:

- `crates/core/tests/transport_contract.rs::response_exceeding_cap_is_rejected_as_response_too_large`
- `crates/core/tests/transport_contract.rs::gzip_bomb_is_rejected_on_decompressed_size`
- `crates/core/tests/transport_contract.rs::response_exactly_at_cap_is_accepted_and_one_over_is_rejected`
- `crates/core/tests/transport_contract.rs::oversized_error_status_body_is_rejected_as_response_too_large`
- `crates/core/tests/transport_contract.rs::non_utf8_body_is_decoded_lossily_without_a_cap_layer_error`
- `crates/core/tests/classify_contract.rs::response_too_large_is_never_retried`
- `crates/core/tests/policy_contract.rs::default_policies_carry_per_client_response_byte_caps`
- `crates/core/tests/policy_contract.rs::builder_round_trip_preserves_every_setter`
- `crates/contracts/src/hex_field.rs::tests::decode_hex_field_bounded_rejects_payload_over_the_limit`
- `crates/app-data/tests/json_recursion_contract.rs::deeply_nested_json_is_rejected_by_the_recursion_guard`

Validation surface:

```text
cargo test -p cow-sdk-core --test transport_contract
cargo test -p cow-sdk-core --features transport-policy
cargo test -p cow-sdk-contracts
cargo test -p cow-sdk-app-data --test json_recursion_contract
```
