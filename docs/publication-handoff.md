# Publication Handoff

## Purpose

This document describes how publication ownership for the `cow-rs`
crates on crates.io is managed: how invitations are issued, how
maintainers are rotated on and off the owner list, and how a broken
release is retracted. It complements
[release-checklist.md](release-checklist.md) by focusing on the
crates.io owner-list state rather than on the publish commands
themselves.

## Current Publisher Identity

The `cow-sdk*` crates on crates.io — published at `0.1.0-alpha.1`,
alongside the earlier `0.0.1-reserved.0` name reservations — are owned
by the `0xSymbiotic` crates.io account. Any ownership change is
initiated by that account.

## Trusted Publisher List

Long-lived maintenance of a published crate depends on keeping an
accurate owner list on crates.io. The owner list enforces who can
publish new versions, yank existing versions, or invite further
maintainers. An aligned list prevents accidental single-key custody
and preserves recovery paths when a maintainer identity changes.

Every crate in the `cow-sdk*` family maintains the same owner list
so the family moves as one unit under a single set of trusted
maintainers.

## `cargo owner --add` Sequence

To invite a new identity onto the owner list for every crate in the
published family, run one `cargo owner --add` invocation per crate
per new identity:

```text
cargo owner --add <new-owner> cow-sdk-core
cargo owner --add <new-owner> cow-sdk-contracts
cargo owner --add <new-owner> cow-sdk-app-data
cargo owner --add <new-owner> cow-sdk-orderbook
cargo owner --add <new-owner> cow-sdk-signing
cargo owner --add <new-owner> cow-sdk-subgraph
cargo owner --add <new-owner> cow-sdk-trading
cargo owner --add <new-owner> cow-sdk-alloy-provider
cargo owner --add <new-owner> cow-sdk-alloy-signer
cargo owner --add <new-owner> cow-sdk-alloy
cargo owner --add <new-owner> cow-sdk
cargo owner --add <new-owner> cow-sdk-test
```

`cargo owner --add` issues an invitation that crates.io surfaces to
the `<new-owner>` identity. The owner change takes effect only after
the invited account accepts the invitation.

## Rotating Maintainer Procedure

Rotating a maintainer is always `--add` first, `--remove` second:

1. Invite the incoming owner with
   `cargo owner --add <incoming> <crate>` for every crate above.
2. Confirm the incoming owner has accepted the invitation on
   crates.io and can publish successfully against each crate.
3. Remove the outgoing owner with
   `cargo owner --remove <outgoing> <crate>` for the same set.

The staged order keeps the owner list non-empty at every step, so
the published name never drops to zero owners during a rotation.

Record every `cargo owner --add` and `cargo owner --remove` in
[`CHANGELOG.md`](../CHANGELOG.md) under the release heading, or in a
release announcement, so the maintenance posture stays auditable
from public history.

## Yank-Rollback Procedure

If a published version must not be resolved by new builds, yank
each affected crate at that version:

```text
cargo yank --version <version> cow-sdk-<crate>
```

`cargo yank` marks the version unsafe to select by default for new
dependency resolution. It does not remove the crate from crates.io;
projects that have already locked the yanked version continue to
build.

Rules:

- Yank every crate in the release that carries the broken change.
- Post a release-retraction notice in `CHANGELOG.md` naming the
  yanked version, the reason, and the recommended replacement, and
  notify downstream consumers through the same announcement channel
  used for routine releases.
- When the yank responds to a security issue, follow the private
  disclosure path in [SECURITY.md](../SECURITY.md) before publishing
  the retraction notice.
- Open a rollback pull request that either reverts the breaking
  change or advances to a corrected patch version. The next
  functional release is always a new version; a yanked version
  number is not re-used.
- If the flaw does not materialize and the yank is reversed, run
  `cargo yank --undo --version <version>` against every previously
  yanked crate and update the `CHANGELOG.md` retraction notice
  accordingly.

## Related Documents

- [Release Checklist](release-checklist.md) — the end-to-end release
  procedure, including the Manual Publish Sequence the owner runs to
  ship a new version.
- [SECURITY.md](../SECURITY.md) — the private disclosure path used
  when a yank responds to a security issue.
- [CHANGELOG.md](../CHANGELOG.md) — the public change history where
  ownership changes, yanks, and retractions are recorded.
