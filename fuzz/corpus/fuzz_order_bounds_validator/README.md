# `fuzz_order_bounds_validator` Corpus

Each seed is a short class byte consumed by
`fuzz/fuzz_targets/fuzz_order_bounds_validator.rs`. The target maps that byte
to a deterministic `OrderCreation`, `SigningScheme`, optional signer, `now`,
and EthFlow flag before running the tuple through
`OrderBoundsValidator::services_default().validate(...)`.

## Seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-happy.bin` | Happy path | Baseline `validation_contract.rs` shape: non-zero amounts, non-zero owner, distinct ERC-20 tokens, and a valid lifetime window. |
| `seed-01-missing-from.bin` | `MissingFrom` | Baseline order with `from = 0x0000000000000000000000000000000000000000`. |
| `seed-02-valid-to-insufficient.bin` | `ValidToInsufficient` | Baseline order with `valid_to = now + 59`, below the services-default 60 second minimum. |
| `seed-03-valid-to-excessive.bin` | `ValidToExcessive` | Baseline limit order at `now = 0` with `valid_to = u32::MAX`, beyond the one-year limit ceiling. |
| `seed-04-invalid-native-sell.bin` | `InvalidNativeSellToken` | Non-EthFlow path with the native-currency sentinel as the sell token. |
| `seed-05-same-token.bin` | `SameBuyAndSellToken` | Baseline order with buy token overwritten to match sell token. |
| `seed-06-zero-sell.bin` | `ZeroAmount(Sell)` | Baseline order with zero sell amount. |
| `seed-07-zero-buy.bin` | `ZeroAmount(Buy)` | Baseline order with zero buy amount. |
| `seed-08-appdata-mismatch.bin` | `AppdataFromMismatch` | Baseline order with an explicit app-data signer different from `from`. |
| `seed-09-owner-mismatch.bin` | `OwnerMismatch` | Baseline order with a recovered signer sentinel different from `from`, routed through the adjacent owner/signer assertion after the validator call. |
| `seed-10-invalid-partner-fee.bin` | `InvalidPartnerFee` | Enum-surface sentinel proving the shared typed result matcher covers the parameter-level rejection variant not emitted by `validate`. |
| `seed-11-valid-to-u32-max.bin` | `valid_to == u32::MAX` | Baseline order with `valid_to = u32::MAX` and `now = u32::MAX - 1`. |
| `seed-12-now-u64-max.bin` | `now == u64::MAX` | Baseline order with `valid_to = u32::MAX` and `now = u64::MAX`. |
| `seed-13-weth-native.bin` | WETH/native paired edge | WETH sell token paired with the native buy sentinel on a WETH-configured validator. |
| `seed-14-ethflow-native.bin` | EthFlow native sentinel | Native sell token on the EthFlow path, proving the native-sell skip still leaves the tuple well-defined. |

All seed files are intentionally tiny and platform-neutral. The target treats
any additional bytes after the class marker as perturbation data for fuzzing.
