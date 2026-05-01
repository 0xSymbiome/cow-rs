# `fuzz_typed_data_digest` Corpus

- canonical: seeds copied from signing parity fixture shapes.
- boundary: fixed-width byte patterns cover empty-looking domains, maximum byte values, and alternating typed-data bytes.
- adversarial: mixed ASCII and high-entropy-looking patterns exercise parser boundaries without relying on random-only discovery.
