# Workflow Security Audit

Status: Current
Last reviewed: 2026-05-08
Owning surface: every `.github/workflows/*.yml` file
Refresh trigger: any new workflow file; any unpinned action; any addition of `pull_request_target`; any third-party action new to the workspace; any permission widening or issue-creation behavior in scheduled workflows
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
| WASM import gate | The forbidden-import workflow uses read-only permissions and SHA-pinned checkout only | Conforms |
| Inline docs smoke | The docs-quality rendered README smoke uses the existing job environment and does not introduce a new third-party action or elevated permission | Conforms |
| Scheduled retry soak | The retry-soak workflow uses read-only permissions, pinned actions, no privileged triggers, and a deterministic ignored test invocation | Conforms |
| Alloy canary issue creation | The report-only Alloy canary grants `issues: write` only to create or reuse a tracking issue through `gh api`, with no new third-party action | Conforms |

Workflow snapshot:

| Workflow | Permissions posture | Action pin status | `pull_request_target` |
| --- | --- | --- | --- |
| `_quality-gate.yml` | `contents: read` | SHA-pinned; includes pinning guard | Absent |
| `alloy-release-candidate.yml` | `contents: read`, `issues: write` | SHA-pinned | Absent |
| `benchmarks.yml` | `contents: read` | SHA-pinned | Absent |
| `browser-wallet-e2e.yml` | `contents: read` | SHA-pinned | Absent |
| `ci.yml` | `contents: read`; aggregate job uses `{}` | SHA-pinned or same-repo reusable workflow | Absent |
| `codeql.yml` | workflow `{}`; analyze job grants `actions: read`, `contents: read`, `security-events: write` | SHA-pinned | Absent |
| `commit-format.yml` | `contents: read` | SHA-pinned | Absent |
| `crate-checks.yml` | workflow `{}`; job grants `contents: read` | SHA-pinned | Absent |
| `docs-quality.yml` | workflow `{}`; jobs grant `contents: read` | SHA-pinned | Absent |
| `fuzz.yml` | `contents: read` | SHA-pinned | Absent |
| `release-readiness.yml` | `contents: read` | SHA-pinned or same-repo reusable workflow | Absent |
| `retry-soak.yml` | `contents: read` | SHA-pinned | Absent |
| `sdk-verification-e2e.yml` | `contents: read` | SHA-pinned | Absent |
| `services-drift.yml` | `contents: read`, `issues: write` | SHA-pinned | Absent |
| `test-depth.yml` | `actions: read`, `contents: read` | SHA-pinned | Absent |
| `wasm-imports-grep-gate.yml` | `contents: read` | SHA-pinned | Absent |
| `wasm-pages.yml` | `contents: read`; deploy job grants `pages: write`, `id-token: write` | SHA-pinned | Absent |
| `wasm.yml` | `contents: read` | SHA-pinned | Absent |

## Current Contract

### SHA-Pinned Actions

Third-party workflow actions are pinned by immutable commit SHA. The source
tag or source branch used to choose the SHA remains in a nearby
`# Source ref:` comment so reviewers can evaluate upgrades without relying on
mutable workflow references.

Same-repository reusable workflow calls, such as the shared quality gate, use
relative `./.github/workflows/...` references and are reviewed as committed
repository code rather than third-party actions.

### Automated Pinning Guard

The `workflow-security` job in `.github/workflows/_quality-gate.yml` scans
`.github/workflows/*.yml` and fails when any third-party `uses:` reference does
not end in `@[0-9a-f]{40}`. That check runs through the shared quality gate
used by both routine CI and release-readiness validation.

### Permissions Discipline

Every workflow declares explicit `permissions:`. Most workflows use
`contents: read`; workflows that need narrower or elevated rights declare them
at job scope. The Pages deployment job is the only workflow lane that grants
`pages: write` and `id-token: write`, the CodeQL analyze job is the only lane
that grants `security-events: write`, and scheduled drift/canary lanes grant
`issues: write` only when they create or reuse tracking issues.

### `pull_request_target` Review Guard

No workflow currently declares `pull_request_target`. The shared quality gate
fails any workflow that adds the trigger without an explicit
`# allow-pull-request-target:` review comment in the same workflow file, so a
future privileged-trigger lane cannot be introduced silently.

### Docs-Quality Inline Smoke

The docs-quality workflow now parses the rustdoc-rendered crate HTML with an
inline Python standard-library parser inside the existing docs job. The change
does not add a third-party `uses:` action, does not widen workflow
permissions, and remains covered by the same workflow-security pinning and
permissions checks as the rest of the workflow set.

### WASM Import Gate

The `wasm-imports-grep-gate.yml` workflow runs on pull requests that touch the
browser leaf crate source tree. It uses read-only repository permissions and a
SHA-pinned checkout action, and its enforcement logic runs inline in the hosted
shell without introducing a new third-party action.

### Scheduled Depth And Retry Lanes

The `test-depth.yml` mutation job now runs on the existing weekly schedule as
well as explicit manual dispatch, publishes structured mutation artifacts, and
keeps the same read-only permission posture. The `retry-soak.yml` workflow is a
separate nightly lane that runs one ignored deterministic orderbook retry and
timeout soak test. It uses only pinned third-party actions, `contents: read`,
and no pull-request trigger.

### Alloy Canary Issue Creation

The Alloy release-candidate workflow remains report-only and scheduled/manual.
When a canary step fails, the workflow uses the first-party GitHub CLI already
available on the hosted runner to call `gh api`, create the `alloy-canary`
label if needed, and create at most one open tracking issue for the failing
canary. This requires `issues: write` but does not add a third-party action,
does not run on pull requests, and does not mutate dependency pins.

## Evidence

Primary implementation points:

- `.github/workflows/_quality-gate.yml`
- `.github/workflows/alloy-release-candidate.yml`
- `.github/workflows/benchmarks.yml`
- `.github/workflows/browser-wallet-e2e.yml`
- `.github/workflows/ci.yml`
- `.github/workflows/codeql.yml`
- `.github/workflows/commit-format.yml`
- `.github/workflows/crate-checks.yml`
- `.github/workflows/docs-quality.yml`
- `.github/workflows/fuzz.yml`
- `.github/workflows/release-readiness.yml`
- `.github/workflows/retry-soak.yml`
- `.github/workflows/sdk-verification-e2e.yml`
- `.github/workflows/services-drift.yml`
- `.github/workflows/test-depth.yml`
- `.github/workflows/wasm-imports-grep-gate.yml`
- `.github/workflows/wasm-pages.yml`
- `.github/workflows/wasm.yml`

Primary regression coverage:

- `.github/workflows/_quality-gate.yml` workflow-security job

Validation surface:

```text
rg -n "^[[:space:]]*(-[[:space:]]*)?pull_request_target[[:space:]]*:|^[[:space:]]*on:[^#]*pull_request_target" .github/workflows -g "*.yml"
rg -n "^[[:space:]]*(-[[:space:]]*)?uses:[[:space:]]*[^[:space:]#]+@(?![0-9a-f]{40}\\b)" .github/workflows -g "*.yml" -P
scripts/check-release-docs-agree.sh
```
