# Workflow Security Audit

Status: Current
Last reviewed: 2026-06-20
Owning surface: every `.github/workflows/*.yml` file
Refresh trigger: any new workflow file; any unpinned action; any addition of `pull_request_target`; any third-party action new to the workspace; any permission widening or new issue-creation behavior in any workflow
Related docs:
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)

## Scope

This audit covers:

- every workflow file in `.github/workflows/`
- SHA pinning for every third-party `uses:` action reference
- explicit `permissions:` discipline at workflow or job scope
- explicit review comments for any `pull_request_target` trigger
- the third-party action review log preserved through source-ref comments

It does not cover repository branch-protection settings or GitHub-hosted
runner infrastructure outside the committed workflow definitions.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Action pinning | Every third-party action is pinned to a 40-character immutable commit SHA | Conforms |
| Pin enforcement | The shared quality gate rejects third-party `uses:` refs that are not SHA-pinned | Conforms |
| Permissions | Every workflow declares explicit least-privilege `permissions:` at workflow or job scope | Conforms |
| Trigger safety | Any workflow using `pull_request_target` must carry an explicit allow-list review comment; the current workflow set does not use the trigger | Conforms |
| Third-party review log | Each pinned third-party action keeps a nearby `# Source ref:` comment naming the reviewed tag or source ref | Conforms |
| WASM import fences | The `cow-sdk-js` import fences (`cargo check-source-fences`) run in the shared policy job with read-only permissions and SHA-pinned checkout only | Conforms |
| README inclusion check | The README docs.rs inclusion check runs at source level in the policy sweep, with no rendered-HTML scrape, new third-party action, or elevated permission | Conforms |
| Manual-dispatch retry soak | The retry-soak workflow (nightly cron paused pre-publication) uses read-only permissions, pinned actions, no privileged triggers, and a deterministic ignored test invocation | Conforms |
| Alloy canary issue creation | The report-only Alloy canary grants `issues: write` only to create or reuse a tracking issue through `gh api`, with no new third-party action | Conforms |

Workflow snapshot:

| Workflow | Permissions posture | Action pin status | `pull_request_target` |
| --- | --- | --- | --- |
| `_quality-gate.yml` | `contents: read` | SHA-pinned; includes pinning guard | Absent |
| `alloy-release-candidate.yml` | `contents: read`, `issues: write` | SHA-pinned | Absent |
| `benchmarks.yml` | `contents: read` | SHA-pinned | Absent |
| `ci.yml` | `contents: read`; aggregate job uses `{}` | SHA-pinned or same-repo reusable workflow | Absent |
| `codeql.yml` | workflow `{}`; analyze job grants `actions: read`, `contents: read`, `security-events: write` | SHA-pinned | Absent |
| `commit-format.yml` | `contents: read` | SHA-pinned | Absent |
| `crate-checks.yml` | workflow `{}`; job grants `contents: read` | SHA-pinned | Absent |
| `docs-quality.yml` | workflow `{}`; jobs grant `contents: read` | SHA-pinned | Absent |
| `fuzz.yml` | `contents: read` | SHA-pinned | Absent |
| `release-readiness.yml` | workflow `contents: read`; publication job grants `id-token: write`, `attestations: write`, `contents: read` | SHA-pinned or same-repo reusable workflow | Absent |
| `retry-soak.yml` | `contents: read` | SHA-pinned | Absent |
| `upstream-drift.yml` | `contents: read` | SHA-pinned | Absent |
| `wasm.yml` | `contents: read` | SHA-pinned | Absent |

## Current Contract

### SHA-Pinned Actions

Third-party workflow actions are pinned by immutable commit SHA, with the source
tag or branch used to choose the SHA kept in a nearby `# Source ref:` comment.
Same-repository reusable workflow calls (such as the shared quality gate) use
relative `./.github/workflows/...` references and are reviewed as committed repo
code rather than third-party actions.

### Automated Pinning Guard

The SHA-pin scan runs inside the `policy` job of
`.github/workflows/_quality-gate.yml` through `cargo check-policies`, which
invokes the xtask `check-workflow-security` policy. There is no job literally
named `workflow-security`. The policy scans `.github/workflows/*.yml` and fails
when any third-party `uses:` reference does not end in `@[0-9a-f]{40}`. It runs
through the shared quality gate used by both routine CI and release-readiness
validation.

### Permissions Discipline

Every workflow declares explicit `permissions:`. Most use `contents: read`;
narrower or elevated rights are declared at job scope. The CodeQL analyze job is
the only lane granting `security-events: write`. The release-readiness
`publication` job grants `id-token: write` and `attestations: write` to emit a
SLSA build-provenance attestation over the packaged crates. The manual Alloy
release-candidate lane grants `issues: write` only when it creates or reuses a
tracking issue.

### `pull_request_target` Review Guard

No workflow currently declares `pull_request_target`. The shared quality gate
fails any workflow that adds the trigger without an explicit
`# allow-pull-request-target:` review comment in the same file, so a future
privileged-trigger lane cannot be introduced silently.

### WASM Import Fences

The `cow-sdk-js` import fences run in the shared `policy` job (through
`cargo check-source-fences`) on every pull request, using the shared gate's
read-only permissions and SHA-pinned checkout. Enforcement is a Rust policy in
the `cargo xtask` sweep, not inline shell, so it adds no third-party action.

### Retry Soak Lane

The `retry-soak.yml` manual-dispatch lane (its nightly cron is paused
pre-publication) runs one ignored deterministic orderbook retry and timeout soak
test, using only pinned actions, `contents: read`, and no pull-request trigger.

### Alloy Canary Issue Creation

The report-only, manual-dispatch Alloy release-candidate workflow uses the
first-party GitHub CLI already on the runner to `gh api` create the `alloy-canary`
label and at most one open tracking issue when a canary step fails. This requires
`issues: write` but adds no third-party action, does not run on pull requests, and
does not mutate dependency pins.

## Evidence

Primary implementation points:

- `.github/workflows/_quality-gate.yml`
- `.github/workflows/alloy-release-candidate.yml`
- `.github/workflows/benchmarks.yml`
- `.github/workflows/ci.yml`
- `.github/workflows/codeql.yml`
- `.github/workflows/commit-format.yml`
- `.github/workflows/crate-checks.yml`
- `.github/workflows/docs-quality.yml`
- `.github/workflows/fuzz.yml`
- `.github/workflows/release-readiness.yml`
- `.github/workflows/retry-soak.yml`
- `.github/workflows/upstream-drift.yml`
- `.github/workflows/wasm.yml`

Primary regression coverage:

- `xtask/src/policy/check_workflow_security.rs`

Validation surface:

```text
cargo check-workflow-security
cargo docs-agree
```
