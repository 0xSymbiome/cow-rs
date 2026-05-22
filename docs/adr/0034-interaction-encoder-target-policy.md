# ADR 0034: Guard Canonical Vault-Relayer Interaction Targets

- Status: Accepted (amended)
- Date: 2026-05-01
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, settlement, registry, interaction, error-typing
- Related: [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0019](0019-http-transport-sole-dispatch.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

`SettlementEncoder::encode_interaction` is the encoder-side target-policy
boundary. When `domain.chain_id` and `domain.verifying_contract` uniquely
identify a canonical settlement in `Registry::default()`, the encoder rejects
an interaction whose `target` equals the paired vault-relayer address for the
same chain and environment.

Unknown or custom settlement domains pass through neutrally and continue to
rely on the upstream settlement runtime as final authority. The
`cow-sdk-contracts::normalize_interaction` helper stays infallible and
value-neutral. No new constructor parameters are added; the encoder uses the
existing `domain.chain_id` and `domain.verifying_contract` fields.

The rejection is surfaced through a typed
`ContractsError::ForbiddenInteractionTarget` variant that carries the rejected
target address and stays within the workspace's non-exhaustive error-enum
policy.

## Why

`cow-sdk-contracts::SettlementEncoder::encode_interaction` accepts a
caller-supplied interaction target. The upstream settlement contract rejects the
vault relayer as an interaction target at runtime with the
`"GPv2: forbidden interaction"` revert reason. The SDK also has registry
context for canonical settlement and vault-relayer deployments, so it can guard
the common canonical misuse without becoming the authority for custom
settlement domains.

## Must Remain True

- Canonical settlement transaction builders receive a deterministic SDK-side
  rejection before submitting a vault-relayer self-target interaction to the
  on-chain runtime.
- Custom settlement domains remain possible because the SDK does not guess a
  vault-relayer address when the domain is not uniquely known.
- The settlement encoder call path becomes fallible and callers propagate the
  typed rejection through the existing contracts error channel.

## Alternatives Rejected

- Reject every vault-relayer-looking target unconditionally. Rejected because
  the SDK cannot identify the paired vault-relayer address for arbitrary custom
  settlement domains.
- Keep the SDK fully value-neutral and rely only on the settlement runtime.
  Rejected because registry-backed canonical deployments give the SDK enough
  information to prevent the common misuse earlier.

## Links

- [ADR 0019](0019-http-transport-sole-dispatch.md) records the sibling pattern
  that SDK-side guards cover what they can while upstream runtime authority
  remains final.
- [Contract Bindings Parity Audit](../audit/contract-bindings-parity-audit.md)
  records the current evidence surface for settlement and interaction encoding.
- [Deployment Registry ADR](0012-alloy-sol-bindings-and-registry-authority.md)
  records the registry authority used to identify canonical settlement domains.

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The `target` interaction-target parameter at the
`SettlementEncoder::encode_interaction` boundary, the
`verifying_contract: Address` field on the cow `TypedDataDomain`
struct, and the rejected-target payload on
`ContractsError::ForbiddenInteractionTarget { target: Address }`
resolve through the cow-owned `#[repr(transparent)]` newtype around
`alloy_primitives::Address` per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
registry-backed canonical settlement identification
(`chain_id` + `verifying_contract` -> paired vault-relayer address)
preserves the cow newtype layer end-to-end.
