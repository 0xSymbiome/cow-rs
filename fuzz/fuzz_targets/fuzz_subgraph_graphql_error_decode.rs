#![no_main]

//! Fuzz target for the subgraph GraphQL error decoder.
//!
//! **Property:** `PROP-SBG-002`.
//! Exercises the serde-derived decoder for
//! [`SubgraphGraphQlError`] by feeding arbitrary byte sequences as
//! candidate UTF-8 JSON documents. Two shapes are covered:
//!
//! * A single [`SubgraphGraphQlError`] object — the shape a standard
//!   GraphQL error payload carries under `errors[i]`.
//! * A `Vec<SubgraphGraphQlError>` — the shape the full `errors`
//!   array carries. The wrapper shape stresses the same decoder
//!   through serde's derived sequence handling.
//!
//! The target invariants are:
//!
//! * Both decoders return `Ok` on well-formed JSON or a typed
//!   `serde_json::Error` on malformed JSON. In no case do they
//!   panic on adversarial inputs: deeply nested arrays or objects
//!   within serde_json's default recursion bound, unicode escape
//!   sequences, duplicate keys, trailing input, non-UTF-8 bytes, or
//!   size-boundary cases.
//! * Successful decode results round-trip back into serde_json
//!   without losing the structural shape — the `locations` array
//!   survives a second serialization pass entry-for-entry. The
//!   `message` and `extensions` fields are wrapped in
//!   `cow_sdk_core::Redacted<T>`, whose `Serialize` impl is
//!   designed to write the literal `[redacted]` placeholder rather
//!   than the inner value, so a full struct-equality assertion is
//!   intentionally not enforced — we only assert that the encode
//!   and decode steps remain panic-free and deterministic on the
//!   sanitized output.
//!
//! The bundled [`SubgraphError`] enum is referenced through a typed
//! construction anchor so the target fails to compile if the error
//! surface moves, even though the target itself does not construct
//! [`SubgraphError`] values from fuzz input.

use cow_sdk_subgraph::{SubgraphError, SubgraphGraphQlError};
use libfuzzer_sys::fuzz_target;

/// Maximum input width accepted by the target. GraphQL error
/// payloads are bounded in the real wire; 4 KiB is more than enough
/// to exercise every branching path in the serde-derived decoder
/// without letting individual runs balloon.
const MAX_FUZZ_INPUT: usize = 4096;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];

    // 1. Individual object decode.
    if let Ok(single) = serde_json::from_slice::<SubgraphGraphQlError>(data) {
        // Successful decode must round-trip through serde_json
        // without panicking. The Redacted<T> wrapper on `message`
        // and `extensions` deliberately rewrites those fields to the
        // sanitized placeholder during serialization, so we cannot
        // assert byte-for-byte equality on the round-tripped value.
        let re_encoded = serde_json::to_vec(&single).expect(
            "serde_json::to_vec must succeed on a successfully decoded SubgraphGraphQlError",
        );
        let decoded_again: SubgraphGraphQlError =
            serde_json::from_slice(&re_encoded).expect("re-encoded document must decode again");
        // Non-Redacted fields must survive the round-trip byte-identically.
        assert_eq!(
            single.locations.len(),
            decoded_again.locations.len(),
            "GraphQL error locations array length must survive the serde round-trip",
        );
        assert_eq!(
            single.locations, decoded_again.locations,
            "GraphQL error locations entries must survive the serde round-trip byte-identically",
        );
        // Encoder idempotency: re-encoding the decoded value yields the same bytes.
        let re_encoded_twice = serde_json::to_vec(&decoded_again)
            .expect("re-encoding the round-tripped value must succeed");
        assert_eq!(
            re_encoded, re_encoded_twice,
            "serde_json::to_vec must be deterministic for the round-tripped value",
        );
    }

    // 2. Array decode — stresses the same decoder through serde's
    //    sequence handling, which is a separate branch in the
    //    derived decoder.
    let _ = serde_json::from_slice::<Vec<SubgraphGraphQlError>>(data);

    // 3. Compile-time anchor against the typed error surface so the
    //    target fails to build if `SubgraphError::NoTotalsFound`
    //    disappears or is renamed. The anchor does not touch fuzz
    //    input; it simply pins the surface the decoder feeds into.
    let _anchor = SubgraphError::NoTotalsFound;
});
