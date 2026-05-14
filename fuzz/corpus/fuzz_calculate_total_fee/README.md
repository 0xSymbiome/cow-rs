# `fuzz_calculate_total_fee` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_calculate_total_fee.rs`. The
target feeds arbitrary bytes through `std::str::from_utf8` into
`Option<&str>` and exercises the public
`cow_sdk_orderbook::calculate_total_fee` transform, which surfaces the
normalized `totalFee` value pinned by
`parity/fixtures/orderbook.json::orderbook-total-fee-transform`.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-decimal.bin` | canonical | `"1000000000000000000"` — a representative unsigned decimal executed-fee string of the shape pinned by `parity/fixtures/orderbook.json::orderbook-total-fee-transform`. |
| `seed-01-boundary-zero.bin` | boundary | Single ASCII `0`; exercises the `total_fee = "0"` default surface documented as the missing-executed-fee fallback. |
| `seed-02-boundary-leading-zeroes.bin` | boundary | `"00042"`; exercises the documented `trim_leading_zeroes` normalization that strips the leading zeros to `"42"`. |
| `seed-03-adversarial-non-digit.bin` | adversarial | `"1.5"`; exercises the `validate_decimal` rejection path for non-ASCII-digit bytes. |
| `seed-04-adversarial-overflow.bin` | adversarial | 78 ASCII nines; passes the digit-only precondition but overflows the `Amount` `uint256` bound and exercises the `OrderbookError::Core` propagation surface. |
| `seed-05-adversarial-non-utf8.bin` | adversarial | Four high-bit bytes (`0xff 0xfe 0xfd 0xfc`); fails the `from_utf8` gate so the target invokes `calculate_total_fee(None)` and the helper falls back to the documented zero default. |
