# Security Policy

## Scope

This policy covers security issues in the `cow-rs` repository, including:

- the Rust crates in this repository
- the repository-owned examples
- repository-owned verification and release workflows
- documentation mistakes that could materially mislead safe integration or
  release use

Security-relevant public surfaces worth a reviewer's attention include:

- the `Eip1271Cache` trait on `verify_eip1271_signature_cached`
  and its conservative positive-only caching semantics (only `Ok(())`
  magic-value matches are cached; every other outcome — including
  `Eip1271MagicValueMismatch`, transport, missing-contract-code, decode,
  and provider failures — re-hits the chain)
- the `Redacted<T>` newtype applied to partner API keys, IPFS pinning
  credentials, transport base URLs, and other secret-adjacent inputs so
  debug, display, and serialized output of configuration types never
  emit the raw value
- the typed `TransportError` enum and its `TransportErrorClass`
  partition, including the URL-stripping contract on
  `ReqwestTransport` (native) and the explicit URL omission on
  `FetchTransport` (browser)
- the `max_response_bytes` transport-policy bound that caps every
  SDK-owned HTTP response read, refusing an over-limit body with a typed
  `TransportErrorClass::ResponseTooLarge` outcome and bounding decoded
  (post-decompression) bytes (see [Bounded response reads](#bounded-response-reads))
- the `CoWSwapOnchainOrders` `OrderPlacement` / `OrderInvalidation` log
  decoder, which is fail-closed on untrusted chain logs: it validates the
  topic set, the on-chain signing scheme, the EIP-1271 owner-payload length,
  and the 56-byte UID length, returns a typed `ContractsError` rather than
  panicking on malformed input, and performs no network I/O because it
  borrows the log bytes

It does not cover:

- general feature requests
- non-security documentation typos
- local support questions about custom integrations

Use public issues or pull requests for non-sensitive bugs and improvement
requests.

## Supported Versions

| Version | Supported |
| --- | --- |
| Unreleased repository state | Yes |

Once the first tagged release is published, this table will expand to show
which release lines receive security fixes.

## Reporting A Vulnerability

Do not open a public GitHub issue for an exploitable vulnerability.

Use the private GitHub advisory flow for this repository:

- [Privately report a vulnerability](https://github.com/cowdao-grants/cow-rs/security/advisories/new)

If the issue can affect deployed CoW Protocol contracts, settlement flows,
protocol infrastructure, or user funds beyond this repository, also raise it
with the upstream protocol maintainers in the CoW Protocol Discord:

- [CoW Protocol Discord](https://discord.com/invite/cowprotocol)

The `cow-rs` SDK is not currently in scope of the CoW Protocol bug bounty
program; the upstream protocol maintainers triage protocol-affecting reports
and route them through their own disclosure channels.

Include as much of the following as you can:

- affected crate, workflow, or documentation surface
- affected version or commit range
- impact summary and threat model
- reproduction steps or proof of concept
- suggested mitigation if you already have one

## Response Timeline

Reports filed through the private channels above follow this response
posture as a best-effort service-level target:

- **Initial acknowledgement**: within 5 business days.
- **Triage and reproduction**: within 14 calendar days of acknowledgement,
  including a preliminary severity call and an indication of whether the
  report is in scope.
- **Coordinated disclosure window**: typically 30 to 90 days from triage
  to public disclosure, depending on severity, mitigation complexity, and
  any dependent upstream releases. Deep or high-severity issues may
  require an extended window; if so, maintainers communicate the new
  target with the reporter.
- **Fix delivery**: security fixes ship through the normal release flow
  with a `CHANGELOG.md` entry, and, where applicable, a private advisory
  or coordinated announcement alongside the release.

If a report has not received an acknowledgement within the window above,
re-send the advisory through the same private channel and include a note
that the initial message appears to have been missed.

## Disclosure Expectations

- Keep the report private until maintainers confirm a fix or mitigation path.
- Avoid publishing proof-of-concept details before coordinated disclosure.
- Use the normal changelog and release notes to announce fixes after the
  mitigation is ready for public consumption.

## Base-URL override risk

Custom orderbook and subgraph endpoint overrides are rejected by default unless
their hosts match the SDK's canonical service hosts. The
`ExternalHostPolicy` builder setting is the explicit opt-in for private
mirrors, open-ended routing, or local loopback fixtures. Host-policy failures
surface through sanitized `HostPolicyError` variants that do not retain raw
URL credentials, paths, queries, or fragments.

Operator recommendation: use the canonical
`OrderbookApi::builder().env(CowEnv::Prod)` default for production
bots that do not need partner-relay support. Use
`ExternalHostPolicy::Allow` only for reviewed private endpoints, and keep
`ExternalHostPolicy::Test` limited to loopback test fixtures.

## Bounded response reads

Every HTTP response body the SDK buffers is bounded by a configurable maximum
size, in decoded bytes, carried on the transport policy as
`max_response_bytes`. Where the SDK owns the read loop — the native orderbook
and subgraph transport, the browser fetch transport, and the IPFS app-data
read — an over-limit body is refused with a typed
`TransportErrorClass::ResponseTooLarge` outcome rather than buffered, and the
refusal is never retried. The bound is on decoded bytes, so a compressed body
that decompresses past the limit is refused on its decoded size. Per-client
defaults are tighter for untrusted gateways (the subgraph gateway and IPFS
gateways) than for the trusted orderbook, and every value is caller-overridable
through the transport policy. Signature hex fields are length-bounded before
decoding, with a bound equal to the orderbook request-body limit so a valid
signature is never rejected.

Residual boundaries where the SDK does not own the read loop are bounded by the
request timeout rather than a streamed byte cap, and are stated here rather
than presented as hard caps:

- The JSON-RPC client the SDK builds disables response decompression so a
  hostile or misbehaving endpoint cannot amplify a small compressed body into a
  very large buffer; it is otherwise bounded by the configured request timeout.
  An RPC client managed entirely by the underlying provider stack is bounded by
  the timeout alone.
- A consumer-supplied JavaScript `fetch` callback materializes the response
  body before the SDK sees it; the SDK refuses an oversized body but cannot
  prevent the JavaScript layer from allocating it. The callback contract is the
  place to impose a body limit before materialization.
- The browser wallet returns data the wallet has already materialized; the SDK
  imposes no response-byte cap on that boundary.
- The IPFS read is byte-bounded but, by default, not time-bounded.

Operator recommendation: route untrusted JSON-RPC and IPFS traffic through a
trusted node or a size-limiting reverse proxy, and impose a body limit inside
any custom JavaScript `fetch` callback before returning the response to the
SDK.

## Browser-wallet trust posture

The browser-wallet integration treats provider identity as an explicit trust
boundary. EIP-6963-discovered providers carry discovery metadata into the typed
provider builder. Anonymous providers require
`Eip1193ProviderBuilder::with_trusted_origin(...)` before construction
succeeds. Wallet chain-management URLs such as `rpc_urls`,
`block_explorer_urls`, and `icon_urls` are wallet payload data and are not
validated with `ExternalHostPolicy`.

Operator recommendation: wrap third-party wallet integrations with a defensive
`ecrecover` step at the consumer layer that asserts the recovered address
matches the wallet-reported address before submitting the order. The cow-sdk
`Signature::recover_ecdsa_address` helper in `cow-sdk-contracts` is the
canonical entry point for ECDSA recovery. Use `Signature::declared_address` for
non-ECDSA variants that declare an address directly, and
`cow_sdk_contracts::verify::verify_eip1271_signature_cached` for on-chain
EIP-1271 smart-account verification.
