# Lens Chain Evidence Audit

Status: Accepted
Last reviewed: 2026-06-10
Owning surface: deployment registry chain taxonomy
Refresh trigger: Refresh when Lens deployment rows or chain support posture changes.

Lens appears in the deployment registry taxonomy because upstream deployment
evidence includes composable and COW Shed rows for chain id `232`. It is not
added to the runtime orderbook-supported chain list by this evidence alone.

## Evidence Planes

| Plane | Evidence |
| --- | --- |
| Registry taxonomy | `DeploymentChainId::Lens = 232` exists for deployment rows. |
| Runtime support exclusion | `SupportedChainId` does not include Lens, so orderbook clients cannot select it as a normal trading chain. |
| Upstream deployment rows | Composable-order and COW Shed rows are sourced from pinned upstream deployment artifacts. |
| Provenance lockstep | Registry addresses derive from the upstream commits pinned per source repository in `parity/source-lock.yaml`, keyed by matching chain and environment. |
| Coverage distinction | Unsupported or empty-code outcomes resolve to `None` and never surface as a deployed address. |
| Public route probes | A one-time probe of the public Lens orderbook routes recorded 404 responses on 2026-05-15, confirming Lens is not a runtime orderbook chain. |
| Source locks | The helper repositories are pinned by commit before their deployment rows are trusted. |
| Documentation | Deployment docs describe Lens as deployment evidence, not runtime chain support. |
| Tests and refresh trigger | Contracts tests cover the deployment taxonomy, and this audit refreshes when Lens deployment rows or support posture changes. |

Validation: run `cargo test -p cow-sdk-contracts --all-features` and confirm
Lens rows remain keyed by `DeploymentChainId` rather than `SupportedChainId`.
