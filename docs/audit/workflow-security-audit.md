---
type: Audit
id: workflow-security
title: "Workflow Security Audit"
description: "Every GitHub Actions workflow pins third-party actions by commit SHA, declares least-privilege permissions, and guards privileged triggers, enforced by a policy gate."
status: Current
owning_surface: "GitHub Actions workflow security posture"
related: [ADR-0026]
timestamp: 2026-06-20
---

# Workflow Security Audit

## Scope

Reviews every `.github/workflows/*.yml` file: SHA-pinning of third-party
actions, least-privilege `permissions:` discipline, the `pull_request_target`
guard, and the reviewed-action source-ref log. It does not cover
branch-protection settings or GitHub-hosted runner infrastructure outside the
committed workflow definitions.

## Findings

- Every third-party action is pinned to a 40-character immutable commit SHA, and
  the shared quality gate fails any third-party `uses:` ref that is not
  SHA-pinned, so the audit does not re-enumerate the workflow set.
- Each pinned third-party action keeps a nearby `# Source ref:` comment naming
  the reviewed tag, preserving the review log next to the pin.
- Every workflow declares explicit `permissions:`; most are `contents: read`,
  and the few elevated grants (CodeQL `security-events: write`, the release
  attestation `id-token`/`attestations: write`, the canary `issues: write`) are
  declared at job scope.
- No workflow uses `pull_request_target`; the gate fails any workflow that adds
  the trigger without an explicit allow-list review comment, so a privileged
  lane cannot be introduced silently.
- The pinning and trigger checks run inside the shared `policy` job with
  read-only permissions and a SHA-pinned checkout, adding no third-party action.

## Evidence

- Decision: [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md).
- Registered as evidence for `PROP-AUD-001` ([documentation governance](../properties/docs.md)).
- Governing gate: `cargo check-workflow-security` (`xtask/src/policy/check_workflow_security.rs`).
- Code: `.github/workflows/*.yml`, `.github/workflows/_quality-gate.yml`.
