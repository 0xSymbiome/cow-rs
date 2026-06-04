# Verification

Use this guide to understand how `cow-rs` justifies its public behavior and
where the current executable evidence lives. It is a navigation aid, not a claim
that tests prove the absence of bugs.

## Verification Model

`cow-rs` uses a layered public evidence model:

- [Properties Registry](../PROPERTIES.md): the canonical index of invariants and
  state contracts
- crate contract, property, and state-machine tests: the primary executable
  proof for crate behavior
- examples: consumer-facing scenario proof
- workflow lanes: repository-wide quality, compatibility, documentation, and
  publication gates
- parity fixtures and source locks: provenance and upstream traceability
- audits and ADRs: current-state review records and durable design history

## Where To Start

| Surface | Start with | Then inspect |
| --- | --- | --- |
| Crate boundaries and crate ownership | [Architecture](architecture.md) | [ADRs](adr/README.md) |
| Proof classes and support posture | [Proof Classes](#proof-classes) | [Crate Evidence Matrix](#crate-evidence-matrix) |
| Invariant ownership | [Properties Registry](../PROPERTIES.md) | crate-local contract and property tests |
| Release, publication, and provenance | [Release Checklist](release-checklist.md) | [Parity And Provenance](parity.md) |
| Focused engineering review | [Audits](audit/README.md) | surface-local tests and source files |
| Example behavior | [Examples](examples.md) | example README files and scenario code |

When a change materially moves a named audited surface, the corresponding audit
should remain `Current` in the same change set.

## Proof Classes

| Class | Meaning | Typical examples |
| --- | --- | --- |
| Deterministic proof | Repository-owned tests, fixtures, builds, and workflow lanes that do not require floating external state. | Crate contract tests, doctests, package dry runs, source-lock validation, mock-wallet flows. |
| Environment-sensitive proof | Checks that depend on host OS, browser runtime, injected wallet, or external endpoint configuration. | Windows compatibility, browser-hosted WASM execution, injected-provider wallet flows. |
| Manual confirmation | Optional live checks that are useful before release but are not part of the routine blocking contract. | GitHub Pages inspection, live orderbook or subgraph smoke checks, extension-backed wallet checks. |

## Crate Evidence Matrix

Each crate maps to its boundary, the deterministic evidence that proves it, the
primary command, and any environment-sensitive or manual boundary that stays
outside the routine blocking contract.

| Crate | Boundary | Deterministic evidence | Primary command | Environment-sensitive or manual boundary |
| --- | --- | --- | --- | --- |
| `cow-sdk-core` | Shared chain config, validated partner-route selection, domain types, runtime traits, the `HttpTransport` seam with `ReqwestTransport` native default, and redacted API-context diagnostics | `config_contract.rs`, `types_contract.rs`, `traits_contract.rs`, `transport_contract.rs`, `docs/audit/partner-api-routing-audit.md`, `docs/audit/http-transport-contract-audit.md` | `cargo test -p cow-sdk-core` | Foundational seam; no live dependency |
| `cow-sdk-transport-policy` | Default policy stability, retryable-status completeness, jitter bounds, per-host limiter keying, and optional reqwest classifier coverage | `cargo test -p cow-sdk-transport-policy` over default-policy, retryable-status, jitter-bound, limiter-keying, and classifier coverage | `cargo test -p cow-sdk-transport-policy` | Live endpoint timing remains environment-sensitive |
| `cow-sdk-contracts` | `alloy::sol!`-generated typed bindings for Settlement, VaultRelayer, EthFlow, CoWSwapOnchainOrders event decoding, the wrapped-native token, the EIP-1967 proxy, and ERC-20 / ERC-20 Permit; the typed `Registry` deployment authority; and the `Eip1271VerificationCache` trait co-located with `verify_eip1271_signature_cached` | `parity_contract.rs`, `order_contract.rs`, `onchain_orders.rs`, `weth.rs`, `signature_contract.rs`, `deployment_contract.rs`, `settlement_contract.rs`, `vault_contract.rs`, `proxy_contract.rs`, `reader_contract.rs`, `swap_contract.rs`, `interaction_contract.rs`, `registry.rs`, `schema_v2_rejection.rs`, `docs/audit/contract-bindings-parity-audit.md`, `docs/audit/onchain-order-log-decoding-audit.md`, `docs/audit/deployment-registry-audit.md` | `cargo test -p cow-sdk-contracts` | Live chain-backed spot checks are optional |
| `cow-sdk-signing` | EIP-712 order signing, typed-data payload construction, generated ids, EIP-1271 payloads, cancellation signing, domain separation, the always-available `NoopEip1271VerificationCache`, and the feature-gated `InMemoryEip1271VerificationCache` implementation | `property_contract.rs`, `order_signing_contract.rs`, `eip1271_contract.rs`, `eip1271_cache_contract.rs`, `cancellation_contract.rs`, `domain_contract.rs`, `docs/audit/eip1271-verification-cache-audit.md` | `cargo test -p cow-sdk-signing --features in-memory-cache` | Live chain-backed spot checks are optional |
| `cow-sdk-app-data` | Canonical JSON rendering, app-data schema handling, typed partner-fee metadata, CID conversion, fail-closed CID/app-data encoding, the IPFS read seam, and redacted IPFS-config diagnostics | `property_contract.rs`, `app_data_info_contract.rs`, `cid_contract.rs`, `schema_contract.rs`, `fetch_contract.rs`, `ipfs_config_redaction_contract.rs`, `docs/audit/cid-dependency-audit.md`, `docs/audit/dependency-gate-audit.md`, `docs/audit/credential-surface-contract-hygiene-audit.md` | `cargo test -p cow-sdk-app-data` | Live IPFS read access remains an optional integration check |
| `cow-sdk-orderbook` | Typed orderbook transport over the `HttpTransport` seam, typestate builder construction, retry/status behavior, DTO conversion, typed quote-request `oneOf`s, quote-request `appData` and pagination fidelity, quote-response `OrderParameters` coverage, malformed-payload failure boundaries, source-schema evidence, redacted context-override diagnostics, and validated partner header assembly | `property_contract.rs`, `api_contract.rs`, `builder_contract.rs`, `request_contract.rs`, `transform_contract.rs`, `types_contract.rs`, `openapi_dto_coverage.rs`, `docs/audit/dependency-gate-audit.md`, `docs/audit/partner-api-routing-audit.md`, `docs/audit/typestate-builder-contract-audit.md`, `docs/audit/quote-response-surface-audit.md` | `cargo test -p cow-sdk-orderbook` | Live orderbook behavior depends on remote endpoints |
| `cow-sdk-trading` | Quote, post, allowance, approval, cancellation, slippage monotonicity and boundary clamping, calldata boundary preservation, quote-request override precedence, quote-amounts projection parity, quote-to-order orchestration, order-id collision retry, receiver fallback, typed partner-fee public inputs and app-data merge-through, balance-semantics preservation, `Trading` construction and `AppCode` validation, helper-specific prerequisite resolution, and recoverable-signature owner or signer validation | `property_contract.rs`, `quote_contract.rs`, `post_contract.rs`, `order_contract.rs`, `allowance_contract.rs`, `cancel_contract.rs`, `onchain_contract.rs`, `slippage_contract.rs`, `sdk_contract.rs`, `app_code_contract.rs`, `parity_contract.rs`, `quote_projection_parity.rs`, `ui.rs`, `docs/audit/trading-order-construction-integrity-audit.md`, `docs/audit/trading-sdk-runtime-prerequisites-audit.md`, `docs/audit/trading-app-data-merge-audit.md`, `docs/audit/quote-response-surface-audit.md`, `docs/audit/credential-surface-contract-hygiene-audit.md` | `cargo test -p cow-sdk-trading` | Optional live API calls remain outside the routine blocking contract |
| `cow-sdk-subgraph` | Read-only GraphQL query construction over the `HttpTransport` seam, typestate builder construction, redacted production route identity, sanitized typed request-failure context, nested request-variable fidelity, typed responses, equivalent string-or-number scalar decoding, malformed-scalar failure boundaries, and source-schema evidence | `property_contract.rs`, `api_contract.rs`, `builder_contract.rs`, `query_contract.rs`, `types_contract.rs`, `docs/audit/dependency-gate-audit.md`, `docs/audit/credential-surface-contract-hygiene-audit.md`, `docs/audit/typestate-builder-contract-audit.md` | `cargo test -p cow-sdk-subgraph` | Live subgraph access depends on external endpoint configuration |
| `cow-sdk-transport-wasm` | Browser-target `HttpTransport` implementation (`FetchTransport`) plus cross-adapter parity against the native `ReqwestTransport` default, request/response-only scope, and cache-control header forwarding | `parity_contract.rs`, `fetch_contract.rs`, `docs/transport.md`, `docs/audit/http-transport-contract-audit.md` | `cargo check -p cow-sdk-transport-wasm --target wasm32-unknown-unknown` and `wasm-pack test --headless --firefox crates/transport-wasm --all-features` | Live browser fetch behavior depends on vendor-specific network stacks |
| `cow-sdk-browser-wallet` | EIP-1193 browser wallet provider/signer boundaries, direct browser-bridge proof, deterministic mock proof, explicit session-state transitions, typed chain-management postconditions, and typed-data transport | `state_machine_contract.rs`, `provider_contract.rs`, `wallet_contract.rs`, `wasm_bridge_contract.rs`, `wasm-pack test --headless --chrome` | `cargo test -p cow-sdk-browser-wallet` and `cd crates/browser-wallet && wasm-pack test --headless --chrome` | Live extension-backed authorization, prompts, and vendor behavior remain environment-sensitive |
| `cow-sdk-alloy-provider` | Native Alloy read-only RPC adapter, redacted builder and errors, contract-read ABI bridge, dependency boundary, and rich transaction receipt conversion | `provider_contract.rs`, `dependency_boundary_contract.rs`, `read_contract_parity.rs`, `read_contract_no_panic.rs`, `redaction_contract.rs`, `cancellation_contract.rs`, `src/conversion.rs` unit tests, `docs/audit/alloy-provider-adapter-audit.md` | `cargo test -p cow-sdk-alloy-provider --all-features` | Live RPC behavior depends on caller-supplied endpoints |
| `cow-sdk-alloy-signer` | Native Alloy local-key signer adapter, typed-data primary-type preservation, ECDSA normalization, provider-required transaction boundary, redacted errors, and cancellation propagation | `signer_contract.rs`, `eip191_reference_vectors.rs`, `eip712_reference_vectors.rs`, `proptests.rs`, `redaction_contract.rs`, `cancellation_contract.rs`, `compile_fail.rs`, `docs/audit/alloy-signer-adapter-audit.md` | `cargo test -p cow-sdk-alloy-signer --all-features` | Live RPC behavior depends on caller-supplied endpoints |
| `cow-sdk-alloy` | Composed native Alloy provider plus signer, owned signer handles, no implicit receipt polling during submission, rich receipt delegation, and Trading helper compatibility | `provider_contract.rs`, `signing_provider_contract.rs`, `send_transaction_does_not_wait_for_confirmation.rs`, `tests/transaction_lifecycle_cross_adapter_invariant.rs`, `tests/alloy_umbrella_composition.rs`, `docs/audit/alloy-umbrella-adapter-audit.md` | `cargo test -p cow-sdk-alloy --all-features` | Live RPC behavior depends on caller-supplied endpoints |
| `cow-sdk` | Thin facade exports, curated prelude, feature-scoped public crate surface, named-module re-export reachability, and sorted SDK parity fixtures | `public_api.rs`, `public_api_default_features_only.rs`, `public_api_with_all_features.rs`, `parity_fixture_sort.rs`, `tests/fixtures/public_api_default_features_only.snap`, `tests/fixtures/public_api_with_all_features.snap`, `tests/ui/orderbook_client_reachable_through_trading_re_export.rs` | `cargo test -p cow-sdk` | Optional live API calls remain outside the routine blocking contract |

Two whole-workspace surfaces sit above any single crate: the **stability
invariant** (native Alloy dependencies stay inside explicit allow-lists —
`alloy-provider` only in `cow-sdk-alloy-provider` and `cow-sdk-alloy`, and
`alloy-signer-local` only in `cow-sdk-alloy-signer` and `cow-sdk-alloy`, with the
policy-maintainer wrappers validating the full published crate list), and
**quality and publishability** (formatting, linting, tests, doctests, docs,
source-lock validation, and package dry runs). Both are exercised by the gates
below.

## Registered Property Evidence

The [Properties Registry](../PROPERTIES.md) is the canonical index of all
registered invariants and state contracts. The rows below highlight
cross-cutting properties whose evidence spans more than one crate.

| Property | Boundary | Regression evidence | Primary command |
| --- | --- | --- | --- |
| `PROP-CORE-014` | Redacted URL maps, API-context base URLs, and sanitized response-body snippets | `crates/core/tests/redaction_contract.rs::redacted_url_map_public_representations_redact_values_and_preserve_keys`, `crates/core/tests/redaction_contract.rs::redacted_optional_url_map_public_representations_redact_some_values_and_keep_none`, `crates/core/tests/redaction_contract.rs::api_context_redacts_base_urls_in_debug_and_serialize_but_resolves_raw_url`, `crates/core/tests/redaction_contract.rs::response_body_redaction_strips_credential_shapes_without_regex_dependency`, `crates/app-data/tests/ipfs_config_redaction_contract.rs::ipfs_config_public_debug_and_serialize_redact_configured_uris` | `cargo test -p cow-sdk-core -p cow-sdk-app-data` |
| `PROP-ORD-007` | GET-side orderbook rejection tags for trade filters and pagination limits | `crates/orderbook/tests/rejection_contract.rs::every_known_services_tag_parses_to_its_typed_variant` covers `InvalidTradeFilter`, `InvalidLimit`, and `LIMIT_OUT_OF_BOUNDS` | `cargo test -p cow-sdk-orderbook --test rejection_contract` |
| `PROP-ORD-011` | Content-addressed-write verification for `OrderbookApi::upload_app_data` (client-side keccak precheck plus server-echo hash verification, plus the bare hex-string response decode against the services PUT schema) | `crates/orderbook/tests/api_contract.rs::upload_app_data_rejects_client_precheck_mismatch_without_network`, `crates/orderbook/tests/api_contract.rs::upload_app_data_rejects_server_echo_mismatch`, `crates/orderbook/tests/api_contract.rs::upload_app_data_accepts_status_200_for_already_existing_documents`, `crates/orderbook/tests/parity_contract.rs::app_data_upload_response_fixture_decodes_as_app_data_hash`, `crates/orderbook/tests/error_variant_shape.rs::app_data_hash_mismatch_carries_typed_hashes_and_stage_discriminator`, `crates/core/tests/types_contract.rs::app_data_hash_from_full_app_data_matches_keccak256_of_bytes` | `cargo test -p cow-sdk-orderbook -p cow-sdk-core` |
| `PROP-OBK-001` | Typed quote-request `oneOf`s — validity, side, and signing scheme are exactly-one by construction, with conflicting or malformed wire input rejected at deserialization by the `SellAmount` and `QuoteSigningScheme` `try_from` guards | `crates/orderbook/tests/types_contract.rs::quote_signing_scheme_rejects_incompatible_onchain_ecdsa_wire_pairs`, `crates/orderbook/tests/types_contract.rs::quote_signing_scheme_rejects_verification_gas_limit_without_eip1271`, `crates/orderbook/tests/types_contract.rs::quote_request_app_data_routes_to_server_valid_wire_shapes`, `crates/orderbook/tests/types_contract.rs::quote_request_validate_accepts_services_signing_scheme_pairs` | `cargo test -p cow-sdk-orderbook --test types_contract` |
| `PROP-SIG-006` | Shared EIP-712 domain separator parity across contracts and signing | `crates/contracts/src/primitives.rs::tests::domain_separator_matches_shared_parity_fixture`, `crates/signing/src/domain.rs::tests::domain_separator_matches_shared_parity_fixture` | `cargo test -p cow-sdk-contracts -p cow-sdk-signing` |
| `PROP-CON-008` | EIP-1271 verification tracing and cache telemetry | `crates/contracts/tests/verify_telemetry_contract.rs::verifier_emits_canonical_span_and_safe_miss_store_events`, `crates/contracts/tests/verify_telemetry_contract.rs::verifier_emits_hit_event_without_reaching_provider`, `crates/contracts/tests/verify_telemetry_contract.rs::verifier_emits_skip_event_for_non_cacheable_errors` | `cargo test -p cow-sdk-contracts --test verify_telemetry_contract` |
| `PROP-SEC-002` | Canonical-host guard rails for orderbook and subgraph base-URL overrides | `crates/orderbook/tests/host_policy_contract.rs::orderbook_builder_blocks_custom_hosts_by_default`, `crates/orderbook/tests/host_policy_contract.rs::orderbook_builder_accepts_explicit_allow_and_loopback_policy`, `crates/subgraph/tests/host_policy_contract.rs::subgraph_builder_blocks_custom_hosts_by_default`, `crates/subgraph/tests/host_policy_contract.rs::subgraph_builder_accepts_explicit_allow_and_loopback_policy` | `cargo test -p cow-sdk-orderbook -p cow-sdk-subgraph` |
| `PROP-BWL-003` | Trust-aware EIP-1193 provider construction | `crates/browser-wallet/tests/provider_contract.rs::anonymous_provider_builder_requires_trusted_origin`, `crates/browser-wallet/tests/provider_contract.rs::provider_builder_accepts_explicit_trusted_origin`, `crates/browser-wallet/tests/wasm_bridge_contract.rs` | `cargo test -p cow-sdk-browser-wallet` |
| `PROP-CON-009` | Scheme-aware ECDSA recovery and declared-owner reporting | `crates/contracts/tests/signature_contract.rs::signature_helpers_preserve_public_contract_surface`, `crates/contracts/tests/signature_contract.rs::recover_ecdsa_address_recovers_eip712_prehash_signer`, `crates/contracts/tests/signature_contract.rs::recover_ecdsa_address_recovers_eth_sign_digest_signer`, `crates/contracts/tests/signature_contract.rs::recover_ecdsa_address_rejects_non_ecdsa_variants` | `cargo test -p cow-sdk-contracts --test signature_contract` |
| `PROP-AP-009` | Alloy provider rich receipt conversion | `crates/alloy-provider/tests/provider_contract.rs::get_transaction_receipt_populates_status_block_gas_from_to`, `crates/alloy-provider/src/conversion.rs` unit tests | `cargo test -p cow-sdk-alloy-provider --test provider_contract` and `cargo test -p cow-sdk-alloy-provider --lib` |
| `PROP-AU-004` | Alloy umbrella broadcast timing and receipt delegation | `crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs::send_transaction_does_not_dispatch_get_transaction_receipt`, `crates/alloy/tests/provider_contract.rs::get_transaction_receipt_populates_rich_fields_from_alloy_receipt` | `cargo test -p cow-sdk-alloy --test send_transaction_does_not_wait_for_confirmation` and `cargo test -p cow-sdk-alloy --test provider_contract` |
| `PROP-WS-RX-001` | Cross-adapter transaction receipt shape | `crates/browser-wallet/tests/transaction_receipt_parsing.rs`, `tests/transaction_lifecycle_cross_adapter_invariant.rs::alloy_get_transaction_receipt_populates_status_and_block`, `tests/transaction_lifecycle_cross_adapter_invariant.rs::browser_wallet_get_transaction_receipt_populates_status_and_block` | `cargo test -p cow-sdk-browser-wallet --test transaction_receipt_parsing` and `cargo test -p cow-rs-workspace-tests --test transaction_lifecycle_cross_adapter_invariant` |
| `PROP-WS-TX-001` | Cross-adapter no receipt polling during submission | `crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs`, `tests/transaction_lifecycle_cross_adapter_invariant.rs::alloy_send_transaction_does_not_poll_for_receipt`, `tests/transaction_lifecycle_cross_adapter_invariant.rs::browser_wallet_send_transaction_does_not_poll_for_receipt` | `cargo test -p cow-sdk-alloy --test send_transaction_does_not_wait_for_confirmation` and `cargo test -p cow-rs-workspace-tests --test transaction_lifecycle_cross_adapter_invariant` |

## Examples And Runtime Surfaces

| Surface | Boundary | Deterministic evidence | Environment-sensitive or manual boundary | Primary command |
| --- | --- | --- | --- | --- |
| Native examples | Deterministic consumer scenarios for app-data, signing, orderbook, quote-only, limit-order, native-sell / EthFlow, pre-sign, off-chain cancellation, on-chain cancellation, and subgraph behavior | `examples/native/tests/scenario_contract.rs` plus runnable scenario binaries including `ethflow_transaction_simulation.rs` and `onchain_order_actions_simulation.rs` | `subgraph_live_query` and `orderbook_live_probe` remain opt-in because they depend on external services or configuration. | `cargo test --manifest-path examples/native/Cargo.toml` |
| Native and per-crate deterministic example binaries | Readable command output for the complete native trading workflow surface without live order placement, plus crate-local examples for trading, orderbook, subgraph, and facade smoke coverage | `examples/native/scenarios/*.rs`, `crates/trading/examples/*.rs`, `crates/orderbook/examples/*.rs`, `crates/subgraph/examples/*.rs`, `crates/sdk/examples/wasm_smoke.rs` | Live service examples are intentionally excluded from the deterministic runner. | `cargo run-deterministic-examples --locked` |
| Browser-wallet WASM proof | Deterministic browser-runtime proof for the `cow-sdk-browser-wallet` EIP-1193 bridge | The direct crate bridge proof in `crates/browser-wallet/tests/wasm_bridge_contract.rs`, `cargo test -p cow-sdk-browser-wallet`, and the headless bridge run in `browser-wallet-wasm.yml` | Live extension-backed connect, sign, quote, submit, and cancel remain environment-sensitive because they depend on the installed wallet, authorization state, and vendor-specific behavior. | `cd crates/browser-wallet && wasm-pack test --headless --firefox` |
| Browser-wallet WASM example | Runnable end-to-end browser-wallet trade demonstration built on `cow-sdk` public types | `examples/wasm/cow-trader-dioxus` builds for `wasm32-unknown-unknown` and stays clippy/rustfmt-clean under `browser-wallet-wasm.yml` | The example talks to the live orderbook, so it is a demonstration and compile gate rather than deterministic proof. | `cargo check --target wasm32-unknown-unknown --manifest-path examples/wasm/cow-trader-dioxus/Cargo.toml` |

## Workspace Gates

| Gate | Purpose |
| --- | --- |
| `cargo fmt --all --check` | Formatting gate for consistent public diffs |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Lint gate across crates and test targets |
| `cargo deny check --config .github/config/deny.toml` | Blocking advisory, license, source, and duplicate-version policy gate |
| `cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2024-0436` | Blocking RustSec vulnerability, unsound, and unmaintained advisory gate; `scripts/check-release-docs-agree.sh` keeps the ignore-token list aligned with the release checklist and `.github/config/deny.toml`, and requires a dependency-gate audit rationale for every ignored RustSec token. |
| `cargo test --workspace` | Main workspace test gate |
| `cargo test -p cow-rs-workspace-tests` | Workspace policy tests for MSRV alignment, root dependency default-feature review, nested Alloy pin lockstep, and native Alloy adapter composition coverage |
| `cargo test -p cow-sdk-alloy --test send_transaction_does_not_wait_for_confirmation` | Native Alloy submission timing gate that rejects hidden receipt polling during broadcast |
| `cargo test -p cow-sdk-browser-wallet --test transaction_receipt_parsing` | Browser-wallet receipt parser gate for absent-tolerant and present-malformed-strict fields |
| `cargo test -p cow-rs-workspace-tests --test transaction_lifecycle_cross_adapter_invariant` | Cross-adapter transaction broadcast and receipt-shape invariant |
| `cargo test --workspace --doc` | Explicit doctest gate for rustdoc examples |
| Published crate README doctests | Every published crate README is wired into crate rustdoc with a `cfg_attr(doctest, ...)` shim, so `cargo test --workspace --doc` compiles every fenced README example on CI. |
| `cargo test --all-features --workspace --doc` | All-feature doctest gate for the public docs contract |
| Native host matrix | Shared quality-gate host coverage on Ubuntu, macOS, and Windows |
| `wasm-imports-grep-gate.yml` | Forbidden-import gate for `cow-sdk-wasm` source files; rejects native-only Alloy crates, `reqwest`, Tokio runtime entrypoints, Tokio macros, and the `cow-sdk-core` reqwest re-exports before they can enter the browser leaf crate. |
| Shared nextest OS matrix | `_quality-gate.yml` runs `cargo nextest run --workspace --all-features --config-file .github/config/nextest.toml` on Ubuntu, macOS, and Windows with `fail-fast: false`. |
| IpfsFetch await static gate | `_quality-gate.yml` rejects `fetch_doc_from_*` calls without `.await` and rejects any `IpfsFetchTransport` implementation that defines a synchronous `fn get`. |
| `cargo doc --workspace --all-features --no-deps` | Public rustdoc build gate |
| `docs-quality.yml` | Nightly docs.rs-style rustdoc lane with `DOCS_RS=1`, `--cfg docsrs`, nightly rustdoc presentation flags, and rendered README heading smoke coverage |
| `fuzz.yml` | Weekly report-only fuzz canary across every active `cargo +nightly fuzz list --fuzz-dir fuzz` target with crash corpus artifact upload |
| `retry-soak.yml` | Nightly deterministic orderbook retry and timeout soak through an explicitly ignored long-run test |
| `RUSTFLAGS="-Dmissing-docs -Dmissing-debug-implementations -Dunreachable-pub -Dunnameable-types" cargo check --workspace --all-features` | Blocking public API rustc lint gate for the published crate family |
| `codeql.yml` | Dedicated semantic security-analysis gate for Rust and GitHub Actions |
| `cargo parity-validate --source-lock parity/source-lock.yaml` | Repo-local parity fixture and source-lock gate for committed publication evidence |
| `cargo parity-verify-sol-provenance --abi-root crates/contracts/abi --source-lock parity/source-lock.yaml --mode strict` | Blocking byte-identity gate for every `.sol` file under `crates/contracts/abi/` against the `vendored:` SHA-256 rows in `parity/source-lock.yaml`; `--upstream-root <path>` extends the comparison to local-checkout `git show <commit>:<path>` bytes at the pinned commit, and `--upstream-github` extends it to `https://raw.githubusercontent.com/<owner>/<repo>/<commit>/<path>` so CI verifies the manifest against GitHub canonical content on every run with no local upstream clone required |
| `cargo check-source-lock-roots --contracts-root <path> --services-root <path>` | Report-only warning command for manually supplied upstream roots that do not match the source-lock remotes or commits |
| `ci-success` | Aggregate routine CI status for branch protection across the required native validation and publication jobs |
| Alloy release-candidate canary | Scheduled and manual forward-compat drift workflow in `.github/workflows/alloy-release-candidate.yml` checks configurable `ALLOY_CANARY_REF` with a pinned SHA fallback and has no pull-request trigger. |
| `cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk-browser-wallet -p cow-sdk-transport-wasm -p cow-sdk-alloy-provider -p cow-sdk-alloy-signer -p cow-sdk-alloy -p cow-sdk` | Blocking allow-list gate asserting `alloy-provider` remains limited to `cow-sdk-alloy-provider` and `cow-sdk-alloy`; `cargo check-alloy-provider-invariant` normalises the raw Cargo tree output, and `scripts/check-release-docs-agree.sh` keeps the raw package list aligned across the release checklist, `_quality-gate.yml`, `CONTRIBUTING.md`, and `PROPERTIES.md`. |
| `cargo check-alloy-signer-invariant` | Blocking allow-list gate asserting `alloy-signer-local` remains limited to `cow-sdk-alloy-signer` and `cow-sdk-alloy`. |
| Release reproducibility posture | Reproducible-build posture documented across the release checklist with explicit source-and-lockfile guarantees and a documented future extension for WebAssembly artifact byte-reproducibility. |

## Publication Gates

- `ci.yml` runs the repo-local publication contract: `parity/source-lock.yaml` validation, locked dependency fetch, `cargo build --frozen`, and the full published package-family package and publish dry-runs from the current workspace.
- `release-readiness.yml` reruns that repo-local contract and then provisions pinned independent upstream clones from `parity/source-lock.yaml` before explicit-root provenance validation.
- Same-checkout copies are not treated as provenance evidence for upstream parity sources.

## Boundary Checks

### Runtime And Typed-Data Contracts

`cow-sdk-core` owns the shared runtime seams. Signer and provider
contracts are async by construction, and typed-data payloads remain structured
rather than being reconstructed from field-name heuristics. Review configuration changes at
the owning crate boundary as well: default diagnostics and serialized forms for
credential-bearing config must keep secrets redacted while leaving explicit
inputs and override seams intact. EIP-1271 verification routes through
`verify_eip1271_signature_cached` with a mandatory
`Eip1271VerificationCache` argument; the cache is a positive-only set
keyed on the full `(verifier, digest, signature_hash)` probe identity,
records only `Ok(())` outcomes, and re-hits the chain for a mismatch and
every other error class.

### Transport Ownership

HTTP dispatch for the orderbook and subgraph surfaces flows through the
`HttpTransport` trait in `cow-sdk-core`. The native default is
`ReqwestTransport`; the browser default is `FetchTransport` from
`cow-sdk-transport-wasm`. Every adapter strips the URL through
`reqwest::Error::without_url` (native) or explicit omission (browser)
before wrapping, so credential-bearing query strings never surface
through the typed `TransportError` enum. Retry behavior, rate limits,
GraphQL request shape, and API-key handling sit above the transport and
remain owned by the orderbook and subgraph crates. For
`cow-sdk-subgraph`, that includes keeping stable route identity and typed
request failures free of raw Graph API credentials.

### Stability Invariant

Native Alloy dependencies are intentionally narrow. `alloy-provider` is
allowed only in `cow-sdk-alloy-provider` and `cow-sdk-alloy`, while
`alloy-signer-local` is allowed only in `cow-sdk-alloy-signer` and
`cow-sdk-alloy`. Review every dependency change against these allow-lists. CI
normalises the raw Cargo tree output via
`cargo check-alloy-provider-invariant` and
`cargo check-alloy-signer-invariant`; contributors should use the wrappers
rather than reading raw Cargo output directly.

### Workflow Ownership

`cow-sdk-trading` owns quote-to-order orchestration. Review trading changes at
the workflow layer first, then inspect the lower-level crates it composes.
That surface is responsible for preserving reviewed balance semantics across
quote-derived and direct order construction, locking the quote-amounts projection that derives the signable order from a `/quote` response with a parity regression test, retrying order-id collisions
without reusing salts, falling back from an unset or zero receiver to the
effective owner address, enforcing one injected-orderbook validation contract
across all `Trading` builder terminals, and rejecting recoverable-signature
owner or signer mismatch before submission. User-facing partner-fee policy also remains typed
here until the explicit app-data metadata translation boundary.

### Browser-Runtime Support

Browser wallet support is explicit, bounded, and feature-gated. Deterministic
proof comes from crate tests, direct browser-bridge coverage, mock-wallet
flows, and fixture-backed browser automation. The deterministic Playwright lane
excludes installed-wallet live-extension specs; those checks remain a manual,
environment-sensitive canary. When a
browser workflow already owns a chain authority,
`BrowserWallet::signer_for_chain` keeps address,
signature, gas, and transaction operations bound to that chain. Typed
chain-management helpers such as `switch_chain` and `switch_or_add_chain`
return success only after the refreshed wallet session confirms the requested
chain. Live extension behavior remains environment-sensitive, and the shipped
static browser consoles keep production live orderbook calls explicitly gated
behind a proxy-enabled deployment requirement.

### Published Crate Policy

MSRV, docs.rs posture, public rustc lints, dependency policy, publication dry
runs, and provenance-sensitive parity checks are part of the published
crate-family contract. Review publication-policy changes through the release
docs rather than as local implementation details. Workspace policy tests keep
the root MSRV aligned with CI, review root dependency default-feature posture,
and check that the native Alloy provider and signer allow-list invariants
enumerate every published crate. Dependency policy is split
deliberately: `cargo deny` owns bans, licenses, source policy, and yanked
advisory policy, while
`cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2024-0436`
blocks RustSec vulnerabilities plus unsound and unmaintained advisories.
The ignored advisories are derived from `.github/config/deny.toml` in CI and
cover reviewed upstream postures for which no direct upgrade path exists; each
entry is tracked in
`docs/audit/dependency-gate-audit.md` and, where the reachability
flows through a crate family boundary, in the corresponding crate
dependency audit.
Yanked crates are denied by the cargo-deny advisory gate unless the current
published upstream state is covered by an explicit public audit exception.
Release artifacts ship reproducible at the source and lockfile level today;
the release checklist records the two-tier reproducibility posture and the path
to binary reproducibility for the WebAssembly artifacts.

The `cargo tree --invert alloy-provider` package list, the `cargo audit --deny ... --ignore RUSTSEC-...` ignore-token list, each ignored RustSec rationale entry, and the browser-wallet Playwright install browser set are guarded against their source-of-truth files by `scripts/check-release-docs-agree.sh`.

### Deployment And Capability Evidence

Contract deployment verification is split into addressable registry evidence
and non-addressable coverage evidence. Registry rows carry one of four
verification statuses:

- `CodeHashVerified`: the deployed bytecode is code-hash-verified at the pinned
  upstream manifest (upstream deployments are explorer/Sourcify-verified); cow-rs
  does not commit a local code-hash digest
- `ExternalVerified`: a third-party verifier or explorer attested the bytecode
- `ReadmeTableUnverified`: the row is sourced from an upstream README table and
  has not yet been independently probed
- `CanonicalUnverified`: the row is canonical source evidence, but no committed
  hash or external attestation is available

Coverage rows carry not-deployed, not-supported, or out-of-scope status and do
not resolve through `Registry::address`. The review procedure is:

1. Confirm `registry.toml` and `deployment-provenance.yaml` have identical
   `(contract, chain, environment, address, verification)` rows.
2. For code-hash rows, confirm the upstream manifest at the pinned `source_commit`
   lists the address, and that the live presence probe (`registry-confirm`)
   reports non-empty `eth_getCode` bytecode on the expected chain.
3. For external rows, inspect the named explorer or attestation source and
   confirm the address, chain, and contract family match.
4. For canonical-unverified rows, confirm the address comes from the pinned
   source-lock commit; these carry no upstream-manifest entry or external
   attestation.
5. For not-deployed coverage, confirm the probe returned empty bytecode.
6. For unsupported coverage, confirm the chain is outside the Rust runtime
   support set and is not present in the registry.

COW Shed adds one extra bytecode check: proxy creation-code files under the
contracts ABI directory carry neighboring SHA-256 files, and `build.rs`
validates those bytes before fixture-based CREATE2 address derivation is
trusted.

### CI Architecture Gates

The workflow layer carries three static architecture gates in addition to the
ordinary Rust build and test jobs. The `wasm-imports-grep-gate.yml` workflow
rejects native-only Alloy, `reqwest`, Tokio runtime, Tokio macro, and
`cow-sdk-core` reqwest re-export references in `cow-sdk-wasm` sources. The
shared quality gate runs the standard nextest suite on Ubuntu, macOS, and
Windows with `fail-fast: false`, replacing duplicate single-host jobs with one
matrix-owned host-coverage lane. The same shared quality gate also checks
that every `fetch_doc_from_*` caller awaits the returned future and every
`IpfsFetchTransport` implementation keeps `get` async.

## Validation Boundaries

- Repo-local source-lock validation proves the committed parity contract from this repository checkout; provenance-sensitive parity proof is separate and requires independent upstream checkouts at the pinned commits.
- Report-only source-lock root warnings catch suspicious manually supplied upstream roots before provenance-sensitive validation relies on them.
- Required tests and examples avoid private keys, seed phrases, live wallet authorization, and live order submission.
- Doctests stay deterministic and are limited to local examples that do not require live-network or host-specific behavior.
- The nightly docs-quality lane stays documentation-only. It exercises docs.rs-style rustdoc flags and all-feature doctests without widening validation into browser-extension, live-network, or host-sensitive behavior.
- The native host matrix stays intentionally narrow and does not absorb browser-target, WASM, or publication-only validation.
- CodeQL complements dependency policy by scanning Rust and GitHub Actions semantics; it does not replace `cargo-deny` or `cargo-audit`.
- Dependency policy is intentionally split: `cargo-deny` owns bans, licenses, sources, and yanked advisory policy, while `cargo-audit` blocks vulnerabilities plus unsound and unmaintained advisories. Yanked published-upstream cases require explicit audit evidence instead of widened ignore lists or hidden unreleased overrides.
- Routine native validation workflows and the dedicated WASM workflows disable checkout credential persistence and use explicit timeout budgets per job.
- Mocked transports should assert request shape and failure behavior where those paths are part of the validated surface.
- WASM/browser evidence is separated from native examples so browser runtime assumptions stay visible, and the direct `wasm-bindgen-test` coverage in `cow-sdk-browser-wallet` proves the owned browser bridge independent of any example surface.
- `cow-sdk-browser-wallet` chain-switch proofs verify refreshed session state after successful switch acknowledgements instead of trusting wallet RPC success alone.
- Live quote, orderbook, subgraph, and wallet checks stay manual unless explicitly promoted into a deterministic routed or injected test.
- Schema-derived evidence stays test-only and outside the public SDK API.
- Higher-iteration search-profile tests remain limited to narrow deterministic helper families whose inputs are large enough to justify the extra exploration and whose failures stay readable in ordinary crate test output.
- `cow-sdk-browser-wallet` tests, mock console mode, and the committed browser-wallet console automation provide deterministic proof without a live extension, public RPC endpoint, or external website; extension-backed injected-provider execution remains environment-sensitive because authorization, chain inventory, wallet UX, and vendor-specific behavior are controlled by the installed extension.
- The public rustc lint gate applies to `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-trading`, `cow-sdk-browser-wallet`, the native Alloy adapter crates, and the `cow-sdk` facade.

## Going Deeper

Use deeper evidence only when the change warrants it:

- search-profile tests for larger deterministic helper families
- targeted mutation scopes for deterministic transport or helper seams
- provenance-sensitive parity validation when fixture provenance changes
- report-only source-lock root warnings before relying on manually supplied
  upstream checkouts
- saved query documents and test-only schema evidence when a schema-backed
  subgraph boundary changes
- optional smoke checks when browser pages or live services must be confirmed

The canonical command set lives in [Release Checklist](release-checklist.md).
Every shipped `README.md` is wired into crate rustdoc with a `cfg_attr(doctest, doc = include_str!("../README.md"))` shim, so `cargo test --workspace --doc` covers every fenced example.
`retry-soak.yml` runs the deterministic long-run retry and timeout soak nightly.

## Review Rules

- start from the owning crate, not from the facade
- use the properties registry to identify what must remain true
- use the matrix above to identify the current executable evidence
- keep deterministic proof separate from environment-sensitive confirmation
- treat browser-runtime support, live services, and upstream provenance as
  explicit boundaries rather than hidden assumptions

## See Also

- [Parity And Provenance](parity.md) — upstream authorities, source-lock, and scope
- [Release Checklist](release-checklist.md) — the canonical command set
- [Properties Registry](../PROPERTIES.md) — the full invariant registry
- [Architecture](architecture.md) and [Audits](audit/README.md) — crate ownership and focused review records
