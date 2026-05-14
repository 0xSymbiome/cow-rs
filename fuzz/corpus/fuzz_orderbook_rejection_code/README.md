# `fuzz_orderbook_rejection_code` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_orderbook_rejection_code.rs`.
The target feeds arbitrary bytes through `String::from_utf8_lossy` into
the public `OrderbookRejectionCode::new` constructor and asserts the
sanitized output is either the input verbatim (when it passes the
`[A-Z][A-Za-z0-9_]{0,47}` allowlist) or exactly the public
`cow_sdk_core::REDACTED_PLACEHOLDER` string. The canonical-class seed
is anchored to the services rejection tag pinned by
`parity/fixtures/orderbook.json::orderbook-duplicate-order-error`.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-duplicated-order.bin` | canonical | `"DuplicatedOrder"` — a representative services rejection tag from `parity/fixtures/orderbook.json::orderbook-duplicate-order-error`; exercises the verbatim-passthrough branch of the allowlist. |
| `seed-01-boundary-single-uppercase.bin` | boundary | Single ASCII uppercase letter `"A"`; exercises the minimum-length end of the allowlist (one byte, uppercase, no underscores). |
| `seed-02-boundary-max-length.bin` | boundary | 48-byte string beginning with `A` followed by 47 ASCII lowercase letters; exercises the maximum-length end of the allowlist (length 48 is the documented upper bound). |
| `seed-03-adversarial-lowercase-first.bin` | adversarial | `"duplicatedOrder"`; fails the leading-uppercase requirement and must collapse to `REDACTED_PLACEHOLDER`. |
| `seed-04-adversarial-embedded-null.bin` | adversarial | `"BadCode\x00WithControl"`; contains a null byte and characters outside `[A-Za-z0-9_]`, so the allowlist must reject the value and the wrapper must collapse to `REDACTED_PLACEHOLDER`. |
| `seed-05-adversarial-non-utf8.bin` | adversarial | Three high-bit bytes (`0xff 0xfe 0xfd`); `from_utf8_lossy` replaces them with the U+FFFD replacement character, producing a non-ASCII string that fails the allowlist's leading-uppercase requirement and collapses to `REDACTED_PLACEHOLDER`. |
