# MSRV Policy

This workspace declares Rust `1.94.0` as its minimum supported Rust version
for the published `cow-sdk` crate family.

## Bump Cadence

MSRV bumps are minor releases. Patch releases keep the existing floor unless a
security advisory cannot be closed without raising the compiler floor.

The first public release establishes the initial MSRV floor; it is not treated
as an MSRV bump because no earlier published floor exists.

The project announces an MSRV bump at least 30 days before the release that
raises the floor. The notice names the new Rust version, the reason for the
bump, and the first release expected to require it.

The 30-day notice window is measured from the first public changelog or pull
request note that names the new Rust floor to the UTC calendar date of the
release tag that raises `workspace.package.rust-version`. A release may not
raise the floor if fewer than 30 complete calendar days have elapsed.

## Trigger Criteria

The workspace raises its MSRV only when at least one of these conditions holds:

- a workspace dependency declares a new minimum Rust version that the SDK must
  consume;
- a stable Rust feature materially improves a hot path or removes a
  meaningful maintenance burden;
- the Rust `1.94.0` floor blocks closure of a security advisory.

## Contributor Toolchain

The repository may pin a newer contributor toolchain than the public MSRV for
local checks and CI ergonomics. The public compatibility contract remains the
workspace `rust-version` floor, and compatibility checks run against that
floor before release.
