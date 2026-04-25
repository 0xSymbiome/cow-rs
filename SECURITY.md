# Security Policy

## Scope

This policy covers security issues in the `cow-rs` repository, including:

- the Rust crates in this repository
- the repository-owned examples
- repository-owned verification and release workflows
- documentation mistakes that could materially mislead safe integration or
  release use

Security-relevant public surfaces worth a reviewer's attention include:

- the `Eip1271VerificationCache` trait on `verify_eip1271_signature_async`
  and its conservative caching semantics (only `Ok(())` magic-value
  matches and `Eip1271MagicValueMismatch` outcomes are cached; every
  other error class, including transport, missing-contract-code, decode,
  and provider failures, re-hits the chain)
- the `Redacted<T>` newtype applied to partner API keys, IPFS pinning
  credentials, transport base URLs, and other secret-adjacent inputs so
  debug, display, and serialized output of configuration types never
  emit the raw value
- the typed `TransportError` enum and its `TransportErrorClass`
  partition, including the URL-stripping contract on
  `ReqwestTransport` (native) and the explicit URL omission on
  `FetchTransport` (browser)

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
protocol infrastructure, or user funds beyond this repository, also follow the
CoW Protocol bug bounty program:

- [CoW Protocol bug bounty on Immunefi](https://immunefi.com/bug-bounty/cowprotocol/information/)

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

Custom `base_url` overrides bypass any host whitelist that the SDK could
enforce. A signed order sent to a non-`api.cow.fi` host can be exfiltrated by
the host operator.

Operator recommendation: use the canonical
`OrderBookApi::builder().environment(CowEnv::Prod)` default for production
bots that do not need partner-relay support. Reserve `base_url` overrides for
partner-relay integrations and in-house staging environments. For
partner-relay use, prefer treaty-bound partner identities over ad-hoc URL
overrides.

## Browser-wallet trust posture

The browser-wallet integration trusts the injected wallet provider's reported
`eth_accounts` response without re-deriving the owner from the signature. A
buggy or malicious wallet provider could report an owner address that does not
control the signing key.

Operator recommendation: wrap third-party wallet integrations with a defensive
`ecrecover` step at the consumer layer that asserts the recovered address
matches the wallet-reported address before submitting the order. The cow-sdk
`Signature::recover_owner` helper in `cow-sdk-contracts` is the canonical entry
point for the defensive recovery.
