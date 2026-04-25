# Workflow Security Audit

Status: Current
Last reviewed: 2026-04-23
Owning surface: every `.github/workflows/*.yml` file
Refresh trigger: any new workflow file; any unpinned action; any addition of `pull_request_target`; any third-party action new to the workspace

## Scope

This audit covers:

- every workflow file in `.github/workflows/`
- SHA pinning for every third-party `uses:` action reference
- explicit `permissions:` discipline at workflow or job scope
- absence of `pull_request_target` triggers
- the third-party action review log preserved through source-ref comments

It does not cover repository branch-protection settings or GitHub-hosted
runner infrastructure outside the committed workflow definitions.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Action pinning | Every third-party action is pinned to a 40-character immutable commit SHA | Conforms |
| Pin enforcement | The shared quality gate rejects third-party `uses:` refs that are not SHA-pinned | Conforms |
| Permissions | Every workflow declares explicit least-privilege `permissions:` at workflow or job scope | Conforms |
| Trigger safety | No workflow uses `pull_request_target` | Conforms |
| Third-party review log | Each pinned third-party action keeps a nearby `# Source ref:` comment naming the reviewed tag or source ref | Conforms |

Workflow snapshot:

| Workflow | Permissions posture | Action pin status | `pull_request_target` |
| --- | --- | --- | --- |
| `_quality-gate.yml` | `contents: read` | SHA-pinned; includes pinning guard | Absent |
| `benchmarks.yml` | `contents: read` | SHA-pinned | Absent |
| `browser-wallet-e2e.yml` | `contents: read` | SHA-pinned | Absent |
| `ci.yml` | `contents: read`; aggregate job uses `{}` | SHA-pinned or same-repo reusable workflow | Absent |
| `codeql.yml` | workflow `{}`; analyze job grants `actions: read`, `contents: read`, `security-events: write` | SHA-pinned | Absent |
| `commit-format.yml` | `contents: read` | SHA-pinned | Absent |
| `crate-checks.yml` | workflow `{}`; job grants `contents: read` | SHA-pinned | Absent |
| `docs-quality.yml` | workflow `{}`; jobs grant `contents: read` | SHA-pinned | Absent |
| `fuzz.yml` | `contents: read` | SHA-pinned | Absent |
| `release-readiness.yml` | `contents: read` | SHA-pinned or same-repo reusable workflow | Absent |
| `sdk-verification-e2e.yml` | `contents: read` | SHA-pinned | Absent |
| `test-depth.yml` | `actions: read`, `contents: read` | SHA-pinned | Absent |
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
`pages: write` and `id-token: write`, and the CodeQL analyze job is the only
lane that grants `security-events: write`.

### `pull_request_target` Zero Tolerance

No workflow declares `pull_request_target`. Any future addition of that trigger
invalidates this audit and must be rejected or accompanied by a refreshed,
surface-specific security review before it ships.

## Evidence

Primary implementation points:

- `.github/workflows/_quality-gate.yml`
- `.github/workflows/benchmarks.yml`
- `.github/workflows/browser-wallet-e2e.yml`
- `.github/workflows/ci.yml`
- `.github/workflows/codeql.yml`
- `.github/workflows/commit-format.yml`
- `.github/workflows/crate-checks.yml`
- `.github/workflows/docs-quality.yml`
- `.github/workflows/fuzz.yml`
- `.github/workflows/release-readiness.yml`
- `.github/workflows/sdk-verification-e2e.yml`
- `.github/workflows/test-depth.yml`
- `.github/workflows/wasm-pages.yml`
- `.github/workflows/wasm.yml`

Primary regression coverage:

- `.github/workflows/_quality-gate.yml` workflow-security job

Validation surface:

```text
rg -n "pull_request_target" .github/workflows -g "*.yml"
rg -n "^[[:space:]]*(-[[:space:]]*)?uses:[[:space:]]*[^[:space:]#]+@(?![0-9a-f]{40}\\b)" .github/workflows -g "*.yml" -P
```
