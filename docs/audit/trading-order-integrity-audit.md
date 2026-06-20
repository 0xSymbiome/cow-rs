# Trading Order Integrity Audit

Status: Current
Last reviewed: 2026-06-20
Owning surface: `cow-sdk-trading` order assembly, the `OrderBoundsValidator` client-rejection gate, the quote-to-post app-data merge path, and the EthFlow owner-identity threading.
Refresh trigger: Changes to quote-derived or direct order construction, balance semantics, the same-token predicate, `Trading` injected-orderbook builder terminals, the post-sign owner-recovery gate (`assert_owner_matches_signer` over `RecoverableSignature::recover`); the `OrderBoundsValidator::validate` signature, the `ClientRejection` variant set, the `ValidToInPast` check, `services_default_for_chain`, or the `is_eth_flow` skip rule; the `merge_and_seal_app_data` / `params_from_doc` signatures, the `metadata.hooks` replacement rule, or the submission-seam `app_data_signer` derivation; the `EthFlowTransaction.from` threading, `eth_flow_transaction` owner resolution, or the `LimitTradeParamsFromQuote` newtype invariant and its EthFlow entry binding.
Related docs:
- [ADR 0015](../adr/0015-client-side-order-bounds-validator.md)
- [ADR 0018](../adr/0018-typed-app-data-merge.md)
- [ADR 0020](../adr/0020-ethflow-owner-threading.md)

## Scope

This audit covers:

- order construction and submission helpers in `cow-sdk-trading`: quote-derived and direct order assembly, signing payload generation, receiver fallback, balance semantics, and the `Trading` injected-orderbook builder terminals
- the typed `OrderBoundsValidator` and its public `validate` entry point, the `ClientRejection` enum, the `TradingError::ClientRejected` lifting variant, and the validator wiring on every public submission seam routed through `post_cow_protocol_trade`
- the post-sign owner-recovery gate and the pre-sign self-report fast-fail
- the quote-to-post app-data edit path: `merge_and_seal_app_data`, `params_from_doc`, the private typed merge with its hooks-replacement rule, and the submission-seam `app_data_signer` derivation
- the EthFlow submission seam: the `EthFlowTransaction` bundle, `eth_flow_transaction` owner resolution, the `is_eth_flow: true` skip rule, and the `LimitTradeParamsFromQuote` entry binding

It does not cover host wallet session management, approval flows, leaf-crate transport policy, the direct-build app-data path, the IPFS preflight, the EthFlow on-chain encoding, the off-chain cancellation pipeline, or the orderbook authoritative server-side validation.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Balance semantics | Preserve reviewed `sellTokenBalance` and `buyTokenBalance` values end to end | Conforms |
| Receiver fallback | Signing payload falls back to the effective `from` address when `receiver` is unset or zero-address | Conforms |
| Same-token builder policy | `TradeParams::validate` and `LimitTradeParams::validate` reject buy-side same-token orders and accept sell-side before construction | Conforms |
| Same-token posting policy | Direct posting rejects buy-side same-token orders before upload or signing and submits sell-side | Conforms |
| Injected-orderbook terminals | Typestate and total-input builder terminals enforce one fail-fast authority contract | Conforms |
| Validator signature | `validate(order, from, app_data_signer, now, is_eth_flow) -> Result<(), ClientRejection>` is the canonical entry point | Conforms |
| Variant coverage | Every stable invariant has a typed `ClientRejection` variant; the enum is `#[non_exhaustive]` | Conforms |
| Validity invariant | Rejects `valid_to <= now` (`ValidToInPast`); leaves the operator-tunable window to services | Conforms |
| Submission-seam policy | Every seam builds the validator via `services_default_for_chain` and runs `validate` between construction and upload | Conforms |
| Owner-recovery gate | After signing, `post_cow_protocol_trade` recovers the signer via `RecoverableSignature::recover` and rejects `OwnerMismatch`; a pre-sign self-report fast-fail rejects explicit owner ≠ `signer.address()`; EIP-1271 and pre-sign skip the gate | Conforms |
| EthFlow skip rule | `is_eth_flow: true` skips the native-sentinel sell-token check and runs every other invariant | Conforms |
| WETH-paired guard | A WETH-bound validator rejects buy-side `sell=WETH`/`buy=native sentinel` as `SameBuyAndSellToken`; accepts the sell-side pair | Conforms |
| Purity | The validator reads no clock or environment, performs no I/O, and is idempotent | Conforms |
| Gas overhead | EthFlow and pre-sign helpers apply the documented 20% gas overhead with floor rounding | Conforms |
| Cancellation gas fallback | On-chain cancellation falls back to `DEFAULT_GAS_LIMIT` when estimation is unavailable | Conforms |
| Merge pipeline | `merge_and_seal_app_data` is the only canonical quote-to-post merge helper; returns `(TradingAppDataInfo, AppDataParams)` | Conforms |
| Typed field survival | Override `signer` and `flashloan` survive into `metadata.signer` and `metadata.flashloan` on the wire | Conforms |
| Hooks replacement | An override `metadata.hooks` fully replaces the base set before recursive merge | Conforms |
| Signer derivation | The seam reads `app_data_signer` from `merged_params.signer`; no override-only read remains | Conforms |
| Round-trip idempotency | A value built through the typed setters satisfies `params_from_doc(generate_app_data_doc(p)) == p`; both seal helpers fail closed rather than emit a document that cannot round-trip back through `params_from_doc` | Conforms |
| EthFlow bundle shape | `EthFlowTransaction` carries a public typed `from: Address` populated at construction | Conforms |
| EthFlow owner resolution | `eth_flow_transaction` resolves the owner via `Signer::address` once and stores it on the bundle | Conforms |
| EthFlow submission validation | The seam passes `tx.from` directly to `validate`; no intermediate `OrderCreation`, no receiver-as-owner fallback | Conforms |
| EthFlow entry binding | The native-currency entry and transaction helper accept only `LimitTradeParamsFromQuote`, lifting the quote-id requirement to the type system while preserving the `MissingQuoteId` diagnostic | Conforms |

## Current Contract

### Order construction

`cow-sdk-trading` preserves reviewed `sellTokenBalance` and `buyTokenBalance` semantics across quote overrides, quote-derived assembly, direct construction, signing payload generation, and final submission; non-default selections stay part of the signed contract. `order_to_sign` treats both an absent receiver and the zero address as unset and emits the effective `from` address as the receiver, matching the reviewed upstream helper. Both direct and quote-derived posting consume `OrderCreation` at the submission seam, preserving the services `OrderCreationAppData` untagged-enum wire shape (`Full`, `Hash`, `Both`).

`TradeParams::validate` and `LimitTradeParams::validate` reject buy-side exact same-token orders with `ClientRejection::SameBuyAndSellToken` and accept sell-side; chain-specific WETH/native-sentinel pairing remains on `OrderBoundsValidator`. Direct posting keeps the same split at the submission boundary. Typestate and total-input builder terminals for `Trading` share one injected-orderbook validation boundary: conflicting explicit trader/quoter defaults fail SDK construction before the surface is exposed. Posting flows for recoverable signature schemes reject explicit owner or signer mismatch before upload, signing, or submission; `PreSign` and `Eip1271` remain separate non-recoverable contracts.

### OrderBoundsValidator

`OrderBoundsValidator::validate` (`crates/trading/src/validation.rs`) accepts the signing order (`cow_sdk_core::OrderData`), the submission owner (`from: Address`, threaded separately because the order carries no owner field), the typed `Option<Address>` app-data signer, the caller-supplied UNIX-seconds `now`, and the `is_eth_flow` flag, returning `Result<(), ClientRejection>`. It is documented as a defence-in-depth guard: a successful local validation does not guarantee services acceptance, and deny-list, transferability, gas budget, banned-users, and market-class classification are explicitly left to the authoritative services surface.

The only stable validity invariant checked is `valid_to <= now` rejecting as `ValidToInPast { valid_to, now }`; minimum/maximum lifetimes are operator configuration left to services. `ClientRejection` is `#[non_exhaustive]` with a typed variant per invariant: `ValidToInPast`, `MissingFrom`, `AppdataFromMismatch`, `SameBuyAndSellToken`, `InvalidNativeSellToken`, `ZeroAmount { side: AmountSide }`, `OwnerMismatch { expected, recovered }`, and `InvalidPartnerFee { field, reason }`. `TradingError::ClientRejected(ClientRejection)` lifts every variant onto the public surface.

Every public submission entry point constructs the chain-aware default via `OrderBoundsValidator::services_default_for_chain(chain_id)`, runs `validate` between construction and HTTP upload, and surfaces failures through `ClientRejected`. The central `post_cow_protocol_trade` sink is the shared submission helper; no caller-side policy configuration is exposed.

The validator mirrors the reviewed services `AllowSell` same-token policy: exact same-token and WETH-paired-with-native-sentinel orders accept on sell-side and reject on buy-side with `SameBuyAndSellToken { token }`. When configured with the chain's wrapped-native address via `with_weth_address`, the paired sell-WETH / buy-native-sentinel case rejects for buy-side as `SameBuyAndSellToken { token: weth_address }`; without it, the exact-match guard still applies. The validator never reads `SystemTime::now`, opens no network connection, and inspects no environment.

The post-sign owner-recovery gate runs inside `post_cow_protocol_trade`: after signing and before submission it recovers the signer from the produced ECDSA signature (`Eip712`/`EthSign`) via `RecoverableSignature::recover` and rejects `OwnerMismatch { expected, recovered }` when the recovered address is not the declared owner — the client-side mirror of the services `WrongOwner` check. A pre-sign self-report fast-fail rejects an explicit owner ≠ `signer.address()` earlier; EIP-1271 and pre-sign carry no recoverable ECDSA signature and skip the gate. EthFlow and pre-sign transaction helpers apply `gas + (gas * 20) / 100` with floor rounding; on-chain cancellation falls back to `DEFAULT_GAS_LIMIT` when signer gas estimation fails.

### App-data merge

`merge_and_seal_app_data` parses the base wire document into `AppDataParams` (lifting `metadata.signer`/`metadata.flashloan` into typed sub-fields), calls `merge_app_data_params(&base_params, override_params)`, re-emits the wire document via `generate_app_data_doc`, and derives the digest via `app_data_info`. It returns `(TradingAppDataInfo, AppDataParams)` so the submission seam reads the signer from the same typed value that produced the upload. The opaque `serde_json::Value`-taking merge helper is deleted; no public merge helper reaches into the wire shape directly.

`AppDataParams` carries typed `signer: Option<Address>` and `flashloan: Option<FlashloanHints>`; `metadata_wire_value` is the only translation point lifting them onto the wire, so both survive every override merge. `merge_app_data_params` removes the base's `metadata.hooks` before recursive merge when the override carries `"hooks"`, so the override hook set fully replaces the base; the rule is narrow to `metadata.hooks`. Array-valued siblings (including `metadata.userConsents`) fall through to the override value via `deep_merge_values`, replacing rather than concatenating. Partner-fee metadata supplied through `advanced_settings.app_data` flows through the same typed path. The submission seam reads `app_data_signer = merged_params.signer`; when the override is absent it re-parses the base via `params_from_doc` so the derivation still reflects the uploaded document. A value built through the typed setters satisfies `params_from_doc(generate_app_data_doc(p)) == p`. Both seal helpers (`build_app_data` and `merge_and_seal_app_data`) re-parse their freshly generated document through `params_from_doc` and fail closed with a typed `AppData` error when an override shadows a reserved metadata key (`signer`, `hooks`, or `flashloan`) — for example through the open `metadata` map — with a value the typed extractor rejects but the lighter `validate_app_data_doc` pass does not re-check, so the SDK never emits a sealed document it cannot itself re-parse. `cow_sdk_app_data::validate_app_data_doc` validates the same `AppDataDoc` shape used by browser bindings, keeping validation ownership in `cow-sdk-app-data`.

### EthFlow owner identity

`cow_sdk_trading::EthFlowTransaction` is a `#[non_exhaustive]` struct with `order_id: OrderUid`, `transaction: PreparedTransaction`, `order_to_sign: OrderData`, and a typed `from: cow_sdk_core::Address` carrying the signer-derived owner captured at construction. `EthFlowTransaction::new` accepts the owner as a required parameter. `eth_flow_transaction` resolves the owner through a single `signer.address().await` near the top of the helper, threads it into `OrderToSignParams` and onto the returned bundle's `from` field; no second signer round-trip happens on the seam.

`post_sell_native_currency_order` passes `tx.from` directly to `OrderBoundsValidator::validate(&tx.order_to_sign, tx.from, …)`. No intermediate `OrderCreation` is constructed for validation and no receiver-as-owner fallback remains. Because the seam passes the owner, `AppdataFromMismatch { appdata_signer, from }` reports the owner identity in `from`, not the payout receiver. The validator is invoked with `is_eth_flow: true`: the native-sentinel sell-token check is skipped while zero amount, same-token, owner mismatch, and lifetime bounds still fire. `tx.order_to_sign.receiver` continues to carry the payout recipient and may legitimately differ from owner without a false rejection; the native-currency entry binds to `LimitTradeParamsFromQuote`, lifting the quote-id requirement to the type system while preserving the `MissingQuoteId` diagnostic.

## Evidence

Primary implementation points:

- `crates/trading/src/error.rs`
- `crates/trading/src/order.rs`
- `crates/trading/src/params.rs`
- `crates/trading/src/validation.rs`
- `crates/trading/src/onchain.rs`
- `crates/trading/src/app_data.rs`
- `crates/trading/src/quote.rs`
- `crates/trading/src/post.rs`
- `crates/trading/src/client/helpers.rs`
- `crates/trading/src/types/params.rs`, `crates/trading/src/types/result.rs`, `crates/trading/src/types/seams.rs`
- `crates/trading/src/slippage.rs`
- `crates/trading/src/lib.rs`
- `crates/core/src/types/amount.rs` (`Amount::is_zero`)
- `crates/core/src/types/identity.rs` (`Address`)
- `crates/app-data/src/types/partner_fee.rs`, `crates/app-data/src/schema.rs`
- `crates/sdk/src/lib.rs`
- `fuzz/fuzz_targets/fuzz_order_bounds_validator.rs`

Primary regression coverage:

- `crates/trading/tests/order_contract.rs`
- `crates/trading/tests/order_contract.rs::order_to_sign_receiver_falls_back_to_from_when_zero_or_unset`
- `crates/trading/tests/validation_contract.rs`
- `crates/trading/tests/validation_contract.rs::validate_same_token_matches_services_allow_sell_policy`
- `crates/trading/tests/parameters_contract.rs::tradeparameters_validate_mirrors_services_allow_sell`
- `crates/trading/tests/parameters_contract.rs::limittradeparameters_validate_mirrors_services_allow_sell`
- `crates/trading/tests/property_contract.rs::validator_classification_is_stable_while_order_stays_in_the_future`
- `crates/trading/tests/property_contract.rs::validator_handles_u32_max_validto_without_overflow`
- `crates/trading/tests/onchain_contract.rs::eth_flow_gas_estimate_applies_documented_floor_overhead`
- `crates/trading/tests/onchain_contract.rs::pre_sign_gas_estimate_applies_documented_floor_overhead`
- `crates/trading/tests/cancel_contract.rs::cancellation_gas_estimation_fallback_uses_documented_constant`
- `crates/trading/tests/post_contract.rs`
- `crates/trading/tests/post_contract.rs::post_swap_order_appdata_from_mismatch_does_not_upload_or_sign`
- `crates/trading/tests/post_contract.rs::post_swap_order_same_buy_sell_token_does_not_upload_or_sign`
- `crates/trading/tests/post_contract.rs::post_swap_order_sell_side_same_buy_sell_token_uploads_signs_and_submits`
- `crates/trading/tests/post_contract.rs::post_swap_order_zero_amount_does_not_upload_or_sign`
- `crates/trading/tests/app_data_merge_contract.rs`
- `crates/trading/tests/app_data_merge_contract.rs::override_with_only_signer_survives_into_wire_doc`
- `crates/trading/tests/app_data_merge_contract.rs::override_with_only_flashloan_survives_into_wire_doc`
- `crates/trading/tests/app_data_merge_contract.rs::merge_replaces_hooks_per_adr_0018`
- `crates/trading/tests/app_data_merge_contract.rs::merge_fails_closed_when_override_shadows_a_reserved_key_with_an_invalid_value`
- `crates/trading/tests/app_data_merge_contract.rs::merge_with_a_valid_reserved_key_override_still_round_trips`
- `crates/trading/tests/app_data_merge_contract.rs::partner_fee_in_advanced_settings_appdata_merges_through_to_post`
- `crates/trading/tests/quote_contract.rs`
- `crates/trading/tests/quote_contract.rs::order_id_collision_retries_with_new_salt_until_success_or_cap`
- `crates/trading/tests/quote_projection_parity.rs`
- `crates/trading/tests/sdk_contract.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-trading --test validation_contract
cargo test -p cow-sdk-trading --test parameters_contract
cargo test -p cow-sdk-trading --test property_contract
cargo test -p cow-sdk-trading --test onchain_contract
cargo test -p cow-sdk-trading --test app_data_merge_contract
cargo test -p cow-sdk-trading --test post_contract
cargo test -p cow-sdk-trading --test quote_projection_parity
cargo test -p cow-sdk-trading --all-features
cargo test --workspace --all-features
cargo check --workspace --all-features --target wasm32-unknown-unknown
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_order_bounds_validator
cargo +nightly fuzz run fuzz_order_bounds_validator --fuzz-dir fuzz -- -runs=1024
cargo run -p xtask -- policy check-property-citations
```
