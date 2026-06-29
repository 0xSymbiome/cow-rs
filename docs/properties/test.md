---
type: Property
id: test
title: "Test double invariants"
description: "The published `cow-sdk-test` `MockSigner`: deterministic EIP-712 typed-data and EIP-191 message signing with a public test key."
resource: https://github.com/0xSymbiome/cow-rs/blob/main/docs/properties/test.md
families: [PROP-TST]
tags: [property, invariants]
timestamp: 2026-06-29T00:00:00Z
---

# Test double invariants

The published `cow-sdk-test` `MockSigner`: deterministic EIP-712 typed-data and EIP-191 message signing with a public test key. Part of the [Properties Registry](index.md): 1 invariant(s), 1 covered.

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-TST-001` | `cow-sdk-test` | The published `MockSigner` double signs EIP-712 typed data and EIP-191 messages with a public development key by default, emitting the canonical legacy-`v` recoverable form through `RecoverableSignature`, so its signature recovers to the address it reports and a double-driven posting flow clears the SDK's owner-recovery gate end to end. Setting a different reported address models a mismatched signer (the signature then recovers elsewhere), and the fixed-signature overrides return a caller-supplied value verbatim for error-path and wire-shape tests. The crate stays panic-free (the key address is a compile-time constant; parsing and signing defer to the `Signer` result). Governed by [ADR 0063](../adr/0063-published-consumer-test-doubles-crate.md). | Contract | Yes | `crates/test/tests/contract.rs::default_signature_recovers_to_the_reported_address`, `crates/test/tests/contract.rs::reporting_a_different_address_models_a_mismatched_signer`, `crates/test/tests/contract.rs::canned_signature_override_returns_the_fixed_value`, `crates/test/src/signer.rs`, `examples/native/scenarios/limit_order.rs` | 2026-06-12 |
