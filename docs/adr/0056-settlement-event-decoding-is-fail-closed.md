# ADR 0056: Settlement Event Decoding Is Fail-Closed And Provider-Free

- Status: Superseded by [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md)
- Date: 2026-05-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, bindings, events, decoding, defense-in-depth

## Superseded

The `GPv2Settlement` event decoder (`decode_settlement_log` over `Trade`,
`Interaction`, `Settlement`, `OrderInvalidated`, and `PreSignature` into the
`#[non_exhaustive]` `SettlementEvent` enum, sharing the `check_topics` guard)
applies the same fail-closed, provider-free posture as the on-chain order
decoder. It is now recorded in
[ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md), which covers the
whole on-chain event-decoding family.
