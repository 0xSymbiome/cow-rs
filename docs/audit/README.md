# Audits

This directory contains focused engineering audits for `cow-rs`. Each audit records the documented surface, current status, follow-up boundaries, and validation evidence.

## Available Audits

| Audit | Scope | Status |
| --- | --- | --- |
| [Duplication Audit](duplication-audit.md) | Mechanical duplication in request execution, signing payload preparation, and trading posting wrappers | Current |
| [CID Dependency Audit](cid-dependency-audit.md) | App-data CID dependency selection, compatibility boundaries, and fail-closed encoding behavior | Current |

## Update Policy

Audits should stay scoped and evidence-based:

- document the specific surface being assessed,
- distinguish addressed items from open follow-up work,
- avoid mixing unrelated quality gates into one audit,
- record only surface-relevant validation commands.
