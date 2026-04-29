# Verification Matrix

This matrix maps current evidence by crate, example surface, and workflow lane.
It is a navigation aid, not a claim that tests prove the absence of bugs.

Use it with:

- [Validation Scope](validation-scope.md)
- [Release Checklist](release-checklist.md)
- [Properties Registry](../PROPERTIES.md)
- [Verification Guide](verification-guide.md)
- [Parity Matrix](parity-matrix.md)

## Core SDK Crates

| Crate | Boundary | Deterministic evidence | Primary command |
| --- | --- | --- | --- |
| `cow-sdk-core` | Shared chain config, validated partner-route selection, domain types, runtime traits, the `HttpTransport` seam with `ReqwestTransport` native default, and redacted API-context diagnostics | `config_contract.rs`, `types_contract.rs`, `traits_contract.rs`, `transport_contract.rs`, `docs/audit/partner-api-routing-audit.md`, `docs/audit/http-transport-contract-audit.md` | `cargo test -p cow-sdk-core` |
| `cow-sdk-contracts` | `alloy::sol!`-generated typed bindings for Settlement, VaultRelayer, EthFlow, the EIP-1967 proxy, and ERC-20 / ERC-20 Permit; the typed `Registry` deployment authority; and the `Eip1271VerificationCache` trait co-located with `verify_eip1271_signature_async` | `parity_contract.rs`, `order_contract.rs`, `signature_contract.rs`, `deployment_contract.rs`, `settlement_contract.rs`, `vault_contract.rs`, `proxy_contract.rs`, `reader_contract.rs`, `swap_contract.rs`, `interaction_contract.rs`, `registry.rs`, `build_rs_compile_fail.rs`, `docs/audit/contract-bindings-parity-audit.md`, `docs/audit/deployment-registry-audit.md` | `cargo test -p cow-sdk-contracts` |
| `cow-sdk-signing` | EIP-712 order signing, typed-data payload construction, generated ids, EIP-1271 payloads, cancellation signing, domain separation, and the `NoopEip1271VerificationCache` / `InMemoryEip1271VerificationCache` default implementations | `property_contract.rs`, `order_signing_contract.rs`, `eip1271_contract.rs`, `eip1271_cache_contract.rs`, `cancellation_contract.rs`, `domain_contract.rs`, `docs/audit/eip1271-verification-cache-audit.md` | `cargo test -p cow-sdk-signing` |
| `cow-sdk-app-data` | Canonical JSON rendering, app-data schema handling, typed partner-fee metadata, CID conversion, fail-closed CID/app-data encoding, fetch, pinning seams, and redacted pinning-config diagnostics | `property_contract.rs`, `app_data_info_contract.rs`, `cid_contract.rs`, `schema_contract.rs`, `fetch_contract.rs`, `pinning_contract.rs`, `docs/audit/cid-dependency-audit.md`, `docs/audit/dependency-gate-audit.md`, `docs/audit/credential-surface-contract-hygiene-audit.md` | `cargo test -p cow-sdk-app-data` |
| `cow-sdk-orderbook` | Typed orderbook transport over the `HttpTransport` seam, typestate builder construction, retry/status behavior, DTO conversion, quote-request `appData` and pagination fidelity, malformed-payload failure boundaries, source-schema evidence, redacted context-override diagnostics, and validated partner header assembly | `property_contract.rs`, `api_contract.rs`, `builder_contract.rs`, `request_contract.rs`, `transform_contract.rs`, `types_contract.rs`, `schema_source_contract.rs`, `docs/audit/dependency-gate-audit.md`, `docs/audit/partner-api-routing-audit.md`, `docs/audit/typestate-builder-contract-audit.md` | `cargo test -p cow-sdk-orderbook` |
| `cow-sdk-trading` | Quote, post, allowance, approval, cancellation, slippage monotonicity, calldata boundary preservation, quote-request override precedence, quote-to-order orchestration, typed partner-fee public inputs, balance-semantics preservation, ready-versus-helper-only `TradingSdk` construction, helper-specific prerequisite resolution, and recoverable-signature owner or signer validation | `property_contract.rs`, `quote_contract.rs`, `post_contract.rs`, `order_contract.rs`, `allowance_contract.rs`, `cancel_contract.rs`, `onchain_contract.rs`, `slippage_contract.rs`, `sdk_contract.rs`, `parity_contract.rs`, `docs/audit/trading-order-construction-integrity-audit.md`, `docs/audit/trading-sdk-runtime-prerequisites-audit.md`, `docs/audit/credential-surface-contract-hygiene-audit.md` | `cargo test -p cow-sdk-trading` |
| `cow-sdk-subgraph` | Read-only GraphQL query construction over the `HttpTransport` seam, typestate builder construction, redacted production route identity, sanitized typed request-failure context, nested request-variable fidelity, typed responses, equivalent string-or-number scalar decoding, malformed-scalar failure boundaries, and source-schema evidence | `property_contract.rs`, `api_contract.rs`, `builder_contract.rs`, `query_contract.rs`, `types_contract.rs`, `schema_source_contract.rs`, `tests/ui/builder_wasm32_missing_transport.rs`, `docs/audit/dependency-gate-audit.md`, `docs/audit/credential-surface-contract-hygiene-audit.md`, `docs/audit/typestate-builder-contract-audit.md` | `cargo test -p cow-sdk-subgraph` |
| `cow-sdk-transport-wasm` | Browser-target `HttpTransport` implementation (`FetchTransport`) plus cross-adapter parity against the native `ReqwestTransport` default | `parity_contract.rs`, `docs/audit/http-transport-contract-audit.md` | `cargo check -p cow-sdk-transport-wasm --target wasm32-unknown-unknown` |
| `cow-sdk-browser-wallet` | EIP-1193 browser wallet provider/signer boundaries, direct browser-bridge proof, deterministic mock proof, explicit session-state transitions, typed chain-management postconditions, and typed-data transport | `state_machine_contract.rs`, `provider_contract.rs`, `wallet_contract.rs`, `wasm_bridge_contract.rs`, `wasm-pack test --headless --chrome` | `cargo test -p cow-sdk-browser-wallet` and `cd crates/browser-wallet && wasm-pack test --headless --chrome` |
| `cow-sdk` | Thin facade exports and public crate surface | `public_api.rs` | `cargo test -p cow-sdk` |

## Registered Property Evidence

| Property | Boundary | Regression evidence | Primary command |
| --- | --- | --- | --- |
| `PROP-CORE-014` | Redacted URL maps, API-context base URLs, and sanitized response-body snippets | `crates/core/tests/redaction_contract.rs::redacted_url_map_public_representations_redact_values_and_preserve_keys`, `crates/core/tests/redaction_contract.rs::redacted_optional_url_map_public_representations_redact_some_values_and_keep_none`, `crates/core/tests/redaction_contract.rs::api_context_redacts_base_urls_in_debug_and_serialize_but_resolves_raw_url`, `crates/core/tests/redaction_contract.rs::response_body_redaction_strips_credential_shapes_without_regex_dependency`, `crates/app-data/tests/error_redaction_contract.rs::pinning_error_body_is_redacted_at_storage_and_public_representations` | `cargo test -p cow-sdk-core -p cow-sdk-app-data` |
| `PROP-ORD-007` | GET-side orderbook rejection tags for trade filters and pagination limits | `crates/orderbook/tests/rejection_contract.rs::every_known_services_tag_parses_to_its_typed_variant` covers `InvalidTradeFilter`, `InvalidLimit`, and `LIMIT_OUT_OF_BOUNDS` | `cargo test -p cow-sdk-orderbook --test rejection_contract` |
| `PROP-SIG-006` | Shared EIP-712 domain separator parity across contracts and signing | `crates/contracts/src/primitives.rs::tests::domain_separator_matches_shared_parity_fixture`, `crates/signing/src/domain.rs::tests::domain_separator_matches_shared_parity_fixture` | `cargo test -p cow-sdk-contracts -p cow-sdk-signing` |
| `PROP-CON-008` | EIP-1271 verification tracing and cache telemetry | `crates/contracts/tests/verify_telemetry_contract.rs::verifier_emits_canonical_span_and_safe_miss_store_events`, `crates/contracts/tests/verify_telemetry_contract.rs::verifier_emits_hit_event_without_reaching_provider`, `crates/contracts/tests/verify_telemetry_contract.rs::verifier_emits_skip_event_for_non_cacheable_errors` | `cargo test -p cow-sdk-contracts --test verify_telemetry_contract` |
| `PROP-SEC-002` | Canonical-host guard rails for orderbook and subgraph base-URL overrides | `crates/orderbook/tests/host_policy_contract.rs::orderbook_builder_blocks_custom_hosts_by_default`, `crates/orderbook/tests/host_policy_contract.rs::orderbook_builder_accepts_explicit_allow_and_loopback_policy`, `crates/subgraph/tests/host_policy_contract.rs::subgraph_builder_blocks_custom_hosts_by_default`, `crates/subgraph/tests/host_policy_contract.rs::subgraph_builder_accepts_explicit_allow_and_loopback_policy` | `cargo test -p cow-sdk-orderbook -p cow-sdk-subgraph` |
| `PROP-BWL-003` | Trust-aware EIP-1193 provider construction | `crates/browser-wallet/tests/provider_contract.rs::anonymous_provider_builder_requires_trusted_origin`, `crates/browser-wallet/tests/provider_contract.rs::provider_builder_accepts_explicit_trusted_origin`, `crates/browser-wallet/tests/wasm_bridge_contract.rs` | `cargo test -p cow-sdk-browser-wallet` |
| `PROP-CON-009` | Scheme-aware ECDSA recovery and declared-owner reporting | `crates/contracts/tests/signature_contract.rs::signature_helpers_preserve_public_contract_surface`, `crates/contracts/tests/signature_contract.rs::recover_ecdsa_address_recovers_eip712_prehash_signer`, `crates/contracts/tests/signature_contract.rs::recover_ecdsa_address_recovers_eth_sign_digest_signer`, `crates/contracts/tests/signature_contract.rs::recover_ecdsa_address_rejects_non_ecdsa_variants` | `cargo test -p cow-sdk-contracts --test signature_contract` |

## Examples And Runtime Surfaces

| Surface | Boundary | Deterministic evidence | Environment-sensitive or manual boundary | Primary command |
| --- | --- | --- | --- | --- |
| Native examples | Deterministic consumer scenarios for app-data, signing, orderbook, quote-only, limit-order, native-sell / EthFlow, pre-sign, off-chain cancellation, on-chain cancellation, and subgraph behavior | `examples/native/tests/scenario_contract.rs` plus runnable scenario binaries including `ethflow_transaction_simulation.rs` and `onchain_order_actions_simulation.rs` | `subgraph_live_query` remains opt-in because it depends on external configuration. | `cargo test --manifest-path examples/native/Cargo.toml` |
| Native scenario binaries | Readable command output for the complete native trading workflow surface without live order placement | `examples/native/scenarios/*.rs` | None beyond the explicit opt-in live subgraph scenario. | `cargo check --manifest-path examples/native/Cargo.toml --examples` |
| SDK WASM verification console | WASM-compatible SDK verification surface with deterministic exports | `examples/wasm/sdk-verification-console/tests/deterministic_exports.rs`, `wasm-pack test --headless --chrome`, `sdk-verification-e2e.yml` | Quote, orderbook, and subgraph actions remain manual when pointed at live endpoints. | `wasm-pack test --headless --chrome` |
| Browser wallet WASM console | Browser wallet verification shell that separates deterministic mock mode from injected-provider execution | The direct crate bridge proof in `crates/browser-wallet/tests/wasm_bridge_contract.rs`, `cargo test -p cow-sdk-browser-wallet`, mock-wallet console mode, the browser-wallet console WASM build, and `browser-wallet-e2e.yml` with local EIP-6963 fixtures plus route-mocked orderbook requests | Live extension-backed connect, sign, quote, submit, and cancel remain environment-sensitive because they depend on the installed wallet, authorization state, and vendor-specific behavior. | `bun run --cwd e2e/browser-wallet test` |

## Workspace Gates

| Gate | Purpose |
| --- | --- |
| `cargo fmt --all --check` | Formatting gate for consistent public diffs |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Lint gate across crates and test targets |
| `cargo deny check --config .github/config/deny.toml` | Blocking advisory, license, source, and duplicate-version policy gate |
| `cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2026-0097 --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2024-0436` | Blocking RustSec vulnerability, unsound, and unmaintained advisory gate; `scripts/check-release-docs-agree.sh` keeps the ignore-token list aligned with the release checklist and `.github/config/deny.toml`. |
| `cargo test --workspace` | Main workspace test gate |
| `cargo test --workspace --doc` | Explicit doctest gate for rustdoc examples |
| Published crate README doctests | Every published crate README is wired into crate rustdoc with a `cfg_attr(doctest, ...)` shim, so `cargo test --workspace --doc` compiles every fenced README example on CI. |
| `cargo test --all-features --workspace --doc` | All-feature doctest gate for the public docs contract |
| Windows stable lane (`windows-latest`) | Light native host compatibility gate with `cargo check --workspace --all-features` and `cargo test --workspace --lib --tests` |
| `cargo doc --workspace --all-features --no-deps` | Public rustdoc build gate |
| `docs-quality.yml` | Nightly docs.rs-style rustdoc lane with `DOCS_RS=1`, `--cfg docsrs`, and nightly rustdoc presentation flags |
| `RUSTFLAGS="-Dmissing-docs -Dmissing-debug-implementations -Dunreachable-pub -Dunnameable-types" cargo check --workspace --all-features` | Blocking public API rustc lint gate for the published crate family |
| `codeql.yml` | Dedicated semantic security-analysis gate for Rust and GitHub Actions |
| `cargo parity-validate --source-lock parity/source-lock.yaml` | Repo-local parity fixture and source-lock gate for committed publication evidence |
| `ci-success` | Aggregate routine CI status for branch protection across the required native validation and publication jobs |
| Alloy release-candidate canary | Scheduled and manual forward-compat drift workflow in `.github/workflows/alloy-release-candidate.yml` checks configurable `ALLOY_CANARY_REF` with a pinned SHA fallback and has no pull-request trigger. |
| `cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk-browser-wallet -p cow-sdk` returns empty | Blocking stability-invariant gate asserting no shipped leaf crate transitively depends on `alloy-provider`; `scripts/check-release-docs-agree.sh` keeps the command copy aligned across the release checklist, `_quality-gate.yml`, `CONTRIBUTING.md`, and `PROPERTIES.md`. |
| Release reproducibility posture | Reproducible-build posture documented across the release checklist with explicit source-and-lockfile guarantees and a documented future extension for WebAssembly artifact byte-reproducibility. |

## Publication Gates

- `ci.yml` runs the repo-local publication contract: `parity/source-lock.yaml` validation, locked dependency fetch, `cargo build --frozen`, and the full published package-family package and publish dry-runs from the current workspace.
- `release-readiness.yml` reruns that repo-local contract and then provisions pinned independent upstream clones from `parity/source-lock.yaml` before explicit-root provenance validation.
- Same-checkout copies are not treated as provenance evidence for upstream parity sources.

## Validation Boundaries

- Required tests and examples avoid private keys, seed phrases, live wallet authorization, and live order submission.
- Doctests stay deterministic and are limited to local examples that do not require live-network or host-specific behavior.
- The nightly docs-quality lane stays documentation-only. It exercises docs.rs-style rustdoc flags and all-feature doctests without widening validation into browser-extension, live-network, or host-sensitive behavior.
- The Windows stable lane stays intentionally narrow and does not absorb browser-target, WASM, or publication-only validation.
- CodeQL complements dependency policy by scanning Rust and GitHub Actions semantics; it does not replace `cargo-deny` or `cargo-audit`.
- Dependency policy is intentionally split: `cargo-deny` owns bans, licenses, sources, and yanked advisory policy, while `cargo-audit` blocks vulnerabilities plus unsound and unmaintained advisories. Yanked published-upstream cases require explicit audit evidence instead of widened ignore lists or hidden unreleased overrides.
- Routine native validation workflows and the dedicated WASM workflows disable checkout credential persistence and use explicit timeout budgets per job. `wasm-pages.yml` scopes elevated Pages permissions to the deployment job.
- Mocked transports should assert request shape and failure behavior where those paths are part of the validated surface.
- WASM/browser evidence is separated from native examples so browser runtime assumptions stay visible.
- Direct `wasm-bindgen-test` coverage in `cow-sdk-browser-wallet` proves the owned browser bridge separately from the broader browser-wallet console automation lane.
- `cow-sdk-browser-wallet` chain-switch proofs verify refreshed session state after successful switch acknowledgements instead of trusting wallet RPC success alone.
- Live quote, orderbook, subgraph, and wallet checks stay manual unless explicitly promoted into a deterministic routed or injected test.
- Schema-derived evidence stays test-only and outside the public SDK API.
- Higher-iteration search-profile tests remain limited to narrow deterministic helper families whose inputs are large enough to justify the extra exploration and whose failures stay readable in ordinary crate test output.
- `cow-sdk-browser-wallet` tests, mock console mode, and the committed browser-wallet console automation provide deterministic proof without a live extension, public RPC endpoint, or external website.
- Extension-backed injected-provider execution remains environment-sensitive because authorization, chain inventory, wallet UX, and vendor-specific behavior are controlled by the installed extension.
- The public rustc lint gate applies to `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-trading`, `cow-sdk-browser-wallet`, and the `cow-sdk` facade.
