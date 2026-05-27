#![no_main]

//! Fuzz target for the orderbook query-string assembly surface.
//!
//! **Surface:** `cow_sdk_orderbook::request::FetchParams`. The internal
//! `append_query_string` helper that joins a base URL with query pairs
//! is crate-private and only reachable through the async transport
//! dispatch path. The target therefore exercises the closest public
//! sync surface — the [`FetchParams`] descriptor that captures the same
//! `(path, method, query, body)` material the private helper consumes —
//! and asserts the public assembly path is panic-free and deterministic
//! for any arbitrary `(base, pairs)` shape.
//! **Property:** `PROP-ORD-003`.
//! **Seed contract:** corpus inputs cover canonical orderbook GET
//! endpoint URLs anchored to
//! `parity/fixtures/orderbook.json::orderbook-get-orders-pagination`,
//! empty-pair and maximum-pair boundaries, IPv6-shaped and percent-encoded
//! adversarial URLs, and non-ASCII / control-byte adversarial query keys.
//! **Corpus README:** `../corpus/fuzz_append_query_string/README.md`.
//!
//! Invariants asserted by the target:
//!
//! * [`FetchParams::new`] and the fluent [`FetchParams::with_query`] and
//!   [`FetchParams::with_body`] setters never panic for any
//!   caller-controlled `(path, method, key, value)` material.
//! * The descriptor preserves the supplied query pairs verbatim and is
//!   deterministic on identical input.

use cow_sdk_orderbook::request::{FetchParams, HttpMethod};
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};

const MAX_PAIRS: usize = 16;
const MAX_KEY_BYTES: usize = 96;
const MAX_VALUE_BYTES: usize = 96;
const MAX_BASE_BYTES: usize = 256;

#[derive(Debug)]
struct QueryStringInput {
    base: String,
    method_hint: u8,
    pairs: Vec<(String, String)>,
}

impl<'a> Arbitrary<'a> for QueryStringInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let seed_class = seed_class(read_u8(bytes, 0));
        if let Some(input) = seeded_input(seed_class) {
            return Ok(input);
        }

        let base = read_string(bytes, MAX_BASE_BYTES);
        let method_hint = read_u8(bytes, 0);
        let pair_count = usize::from(read_u8(bytes, 0)) % (MAX_PAIRS + 1);
        let mut pairs = Vec::with_capacity(pair_count);
        for _ in 0..pair_count {
            pairs.push((
                read_string(bytes, MAX_KEY_BYTES),
                read_string(bytes, MAX_VALUE_BYTES),
            ));
        }

        Ok(Self {
            base,
            method_hint,
            pairs,
        })
    }
}

fuzz_target!(|input: QueryStringInput| {
    let method = http_method(input.method_hint);

    // Build the descriptor twice to confirm determinism.
    let descriptor_one = build_descriptor(&input.base, method, &input.pairs);
    let descriptor_two = build_descriptor(&input.base, method, &input.pairs);

    assert_eq!(
        descriptor_one, descriptor_two,
        "FetchParams assembly must be deterministic on identical input",
    );
    assert_eq!(
        descriptor_one.path, input.base,
        "FetchParams::new must preserve the supplied path verbatim",
    );
    assert_eq!(
        descriptor_one.method, method,
        "FetchParams::new must preserve the supplied method",
    );
    assert_eq!(
        descriptor_one.query.len(),
        input.pairs.len(),
        "FetchParams::with_query must accumulate every supplied pair",
    );
    for (built, supplied) in descriptor_one.query.iter().zip(input.pairs.iter()) {
        assert_eq!(built, supplied, "FetchParams must preserve pair order");
    }
    assert!(
        descriptor_one.body.is_none(),
        "FetchParams::new must not synthesize a body",
    );
});

fn build_descriptor(base: &str, method: HttpMethod, pairs: &[(String, String)]) -> FetchParams {
    let mut params = FetchParams::new(base.to_owned(), method);
    for (key, value) in pairs {
        params = params.with_query(key.clone(), value.clone());
    }
    params
}

fn http_method(value: u8) -> HttpMethod {
    match value % 4 {
        0 => HttpMethod::Get,
        1 => HttpMethod::Post,
        2 => HttpMethod::Delete,
        _ => HttpMethod::Put,
    }
}

fn seeded_input(seed_class: u8) -> Option<QueryStringInput> {
    match seed_class {
        0 => Some(QueryStringInput {
            base: "https://api.cow.fi/mainnet/api/v1/orders".to_owned(),
            method_hint: 0,
            pairs: vec![
                (
                    "owner".to_owned(),
                    "0x0000000000000000000000000000000000000001".to_owned(),
                ),
                ("offset".to_owned(), "0".to_owned()),
                ("limit".to_owned(), "50".to_owned()),
            ],
        }),
        1 => Some(QueryStringInput {
            base: "https://api.cow.fi/mainnet/api/v1/version".to_owned(),
            method_hint: 0,
            pairs: Vec::new(),
        }),
        2 => Some(QueryStringInput {
            base: String::new(),
            method_hint: 1,
            pairs: vec![("key".to_owned(), "value".to_owned())],
        }),
        3 => Some(QueryStringInput {
            base: "https://[2001:db8::1]/api/v1/orders".to_owned(),
            method_hint: 0,
            pairs: vec![("q".to_owned(), "%00\x01\x7f".to_owned())],
        }),
        4 => Some(QueryStringInput {
            base: "not a url".to_owned(),
            method_hint: 0,
            pairs: vec![("\n".to_owned(), "\r".to_owned())],
        }),
        _ => None,
    }
}

fn seed_class(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        _ => value % 6,
    }
}

fn read_u8(bytes: &mut Unstructured<'_>, default: u8) -> u8 {
    u8::arbitrary(bytes).unwrap_or(default)
}

fn read_string(bytes: &mut Unstructured<'_>, max_len: usize) -> String {
    let len = usize::from(read_u8(bytes, 0)) % (max_len + 1);
    let mut raw = vec![0u8; len];
    for byte in &mut raw {
        *byte = read_u8(bytes, b'a');
    }
    String::from_utf8_lossy(&raw).into_owned()
}
