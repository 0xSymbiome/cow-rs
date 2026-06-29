---
type: Runbook
title: "Alloy Major-Release Absorption Runbook"
description: "This runbook supplements ADR 0026 with the operational procedure for absorbing a major-version Alloy release into the cow-rs workspace."
timestamp: 2026-06-20T00:00:00Z
---

# Alloy Major-Release Absorption Runbook

This runbook supplements ADR 0026 with the operational procedure for absorbing
a major-version Alloy release into the cow-rs workspace.

## Pre-release Rehearsal

1. Open a release-candidate branch from the current mainline.
2. Trigger the Alloy release-candidate workflow manually and confirm it uses
   the intended Alloy and Alloy Core refs.
3. Update the Alloy runtime family with exact versions for
   `alloy-consensus`, `alloy-json-rpc`, `alloy-network`, `alloy-provider`,
   `alloy-rpc-types-eth`, `alloy-signer`, `alloy-signer-local`,
   `alloy-transport`, and `alloy-transport-http`.
4. Update the Alloy Core ABI family with exact versions for `alloy-dyn-abi`,
   `alloy-json-abi`, `alloy-primitives`, `alloy-sol-macro`, and
   `alloy-sol-types`.
5. Refresh workspace dependency declarations so manifests and `Cargo.lock`
   resolve to the same reviewed set.
6. Run formatting, clippy, workspace tests, alloy family lockfile invariant
   tests, provider invariant checks, and signer invariant checks.
7. Refresh dependency-audit evidence from raw `cargo audit --json` output into
   reviewer-facing Markdown, keeping raw JSON out of committed Markdown.
8. Append the new ADR 0026 compatibility-matrix row and refresh standing
   audits that name the prior pinned versions.

The cow-owned `#[repr(transparent)]` newtype layer per
[ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md) absorbs
`alloy_primitives` major bumps at the cow boundary; the workspace-test
sweep in step 6 pins the cow constructor and wire-serialization invariants
byte-identically across alloy major-version transitions.

## Release-Day Execution

1. Rebase the rehearsed branch on the intended release commit.
2. Re-run the full rehearsal validation sweep without changing dependency
   versions.
3. Confirm the compatibility matrix, dependency-audit summary, source-lock
   provenance, and standing audits all name the same Alloy and Alloy Core
   versions.
4. Merge only after the release-candidate workflow and invariant checks are
   green.
5. Leave the manually-dispatched canary (weekly schedule currently paused per
   ADR 0026) informational unless maintainers explicitly promote it to a
   blocking gate.

## Rollback

1. Revert the Alloy runtime family to the previously absorbed exact versions.
2. Revert the Alloy Core ABI family to the previously absorbed exact versions.
3. Restore workspace dependency literals so the manifests match the reverted
   lockfile resolution.
4. Keep the current compatibility-matrix row unchanged until the upstream fix
   lands or the in-place absorption is feasible.
5. Document the blocked version, failure mode, and upstream issue link in the
   dependency-audit summary or a linked release note.

## Escalation

Escalate to maintainers when the canary or rehearsal shows a public API
breakage, binding-generation drift, signer or provider invariant failure,
security advisory, or unresolved upstream release blocker. The escalation note
should include the candidate versions, failing command, relevant log excerpt,
affected crates, and proposed rollback or hold decision.
