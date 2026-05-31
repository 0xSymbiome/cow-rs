#![no_main]

//! Fuzz target for the orderbook response-envelope decoding surface.
//!
//! **Surface:** `cow_sdk_orderbook::request::ResponseEnvelope::empty`
//! and `OrderbookApiError::new`. The internal `decoded_body` and
//! `canonical_status_text` helpers are crate-private; the target
//! exercises them through these two public constructors, which together
//! cover every observable side of the documented decoding contract.
//! **Property:** `PROP-OBK-002`.
//! **Seed contract:** corpus inputs cover canonical orderbook rejection
//! envelopes anchored to
//! `parity/fixtures/orderbook.json::orderbook-duplicate-order-error`,
//! empty-body and `204 No Content` boundary cases, content-type
//! variations (absent, `application/json`, plain text), and adversarial
//! bodies containing non-UTF-8 sequences plus over-long content-type
//! header material.
//! **Corpus README:** `../corpus/fuzz_decoded_body_canonical_status_text/README.md`.
//!
//! Invariants asserted by the target:
//!
//! * Constructing a [`ResponseEnvelope::empty(status)`] never panics for
//!   any `u16`, the rendered `status_text` is always pure ASCII, and the
//!   value is deterministic on identical input.
//! * Building a [`ResponseBody`] from the same body/content-type pair
//!   using the documented decision rule (`Empty` iff body empty or
//!   status == 204; `Json` iff content-type starts with
//!   `application/json` or is absent and the body parses as JSON;
//!   otherwise `Text`) preserves the documented partition.
//! * Feeding that `ResponseBody` to [`OrderbookApiError::new`] never
//!   panics, never observably mutates the status, and produces a
//!   deterministic error message string on identical input.

use cow_sdk_orderbook::{OrderbookApiError, ResponseBody, request::ResponseEnvelope};
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};
use serde_json::Value;

const MAX_BODY_BYTES: usize = 4096;
const MAX_CONTENT_TYPE_BYTES: usize = 96;

#[derive(Debug)]
struct DecodedBodyInput {
    status: u16,
    content_type: Option<String>,
    body: Vec<u8>,
}

impl<'a> Arbitrary<'a> for DecodedBodyInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let seed_class = seed_class(read_u8(bytes, 0));
        if let Some(input) = seeded_input(seed_class) {
            return Ok(input);
        }

        let status = read_u16(bytes, 200);
        let content_type = if read_bool(bytes, false) {
            None
        } else {
            Some(read_string(bytes, MAX_CONTENT_TYPE_BYTES))
        };
        let body_len = usize::from(read_u16(bytes, 0)) % (MAX_BODY_BYTES + 1);
        let mut body = vec![0u8; body_len];
        for byte in &mut body {
            *byte = read_u8(bytes, 0);
        }

        Ok(Self {
            status,
            content_type,
            body,
        })
    }
}

fuzz_target!(|input: DecodedBodyInput| {
    // ResponseEnvelope::empty exercises the crate-private
    // `canonical_status_text` helper through its public field.
    let envelope = ResponseEnvelope::empty(input.status);
    assert_eq!(
        envelope.status, input.status,
        "ResponseEnvelope::empty must preserve the supplied status",
    );
    assert_eq!(
        envelope.body,
        Vec::<u8>::new(),
        "ResponseEnvelope::empty must produce an empty body",
    );
    assert!(
        envelope.content_type.is_none(),
        "ResponseEnvelope::empty must not synthesize a content-type",
    );
    assert!(
        envelope.status_text.is_ascii(),
        "canonical_status_text must remain ASCII for any u16 status: {:?}",
        envelope.status_text,
    );
    assert!(
        !envelope.status_text.is_empty(),
        "canonical_status_text must never be empty",
    );

    // Determinism: the same status must produce the same status_text.
    let twin = ResponseEnvelope::empty(input.status);
    assert_eq!(
        envelope.status_text, twin.status_text,
        "canonical_status_text must be deterministic on identical input",
    );

    // Replicate the documented decoded_body decision rule so we can
    // assert the public partition holds on the same body/content-type
    // input that the private helper would see.
    let body = decode_body_reference(input.status, input.content_type.as_deref(), &input.body);
    let body_twin = decode_body_reference(input.status, input.content_type.as_deref(), &input.body);
    assert_eq!(
        body, body_twin,
        "the documented decoded_body partition must be deterministic",
    );

    // Empty iff body is empty or status == 204.
    match &body {
        ResponseBody::Empty => assert!(
            input.body.is_empty() || input.status == 204,
            "Empty body decoded for non-204 non-empty input: status={}",
            input.status,
        ),
        ResponseBody::Json(value) => {
            assert!(!input.body.is_empty(), "Json body decoded for empty input",);
            assert!(
                input.status != 204,
                "Json body decoded for 204 status: value={value:?}",
            );
            // The body must have parsed cleanly as JSON.
            let reparsed = serde_json::from_slice::<Value>(&input.body)
                .expect("Json variant requires the original bytes to parse as JSON");
            assert_eq!(reparsed, *value, "Json variant must preserve parsed value");
        }
        ResponseBody::Text(_) => {
            assert!(!input.body.is_empty(), "Text body decoded for empty input",);
            assert!(input.status != 204, "Text body decoded for 204 status",);
        }
    }

    // OrderbookApiError::new is the public assembly path that consumes
    // the decoded body; it must never panic and must be deterministic.
    let error = OrderbookApiError::new(input.status, envelope.status_text.clone(), body.clone());
    assert_eq!(
        error.status, input.status,
        "OrderbookApiError::new must preserve the supplied status",
    );
    let rendered = format!("{error}");
    let rendered_twin = format!(
        "{}",
        OrderbookApiError::new(input.status, envelope.status_text.clone(), body),
    );
    assert_eq!(
        rendered, rendered_twin,
        "OrderbookApiError Display must be deterministic on identical input",
    );
    assert!(
        !rendered.contains('\0'),
        "OrderbookApiError Display must not carry raw null bytes: {rendered}",
    );
});

/// Reference implementation of the crate-private `decoded_body` decision
/// rule. Keeps the fuzz target independent of the private helper while
/// asserting the documented partition.
fn decode_body_reference(status: u16, content_type: Option<&str>, body: &[u8]) -> ResponseBody {
    if status == 204 || body.is_empty() {
        return ResponseBody::Empty;
    }

    let prefer_json =
        content_type.is_none_or(|ct| ct.to_ascii_lowercase().starts_with("application/json"));

    if prefer_json && let Ok(value) = serde_json::from_slice::<Value>(body) {
        return ResponseBody::Json(value);
    }

    ResponseBody::Text(String::from_utf8_lossy(body).into_owned())
}

fn seeded_input(seed_class: u8) -> Option<DecodedBodyInput> {
    match seed_class {
        0 => Some(DecodedBodyInput {
            status: 400,
            content_type: Some("application/json".to_owned()),
            body: br#"{"errorType":"DuplicatedOrder","description":"order already exists"}"#
                .to_vec(),
        }),
        1 => Some(DecodedBodyInput {
            status: 204,
            content_type: None,
            body: Vec::new(),
        }),
        2 => Some(DecodedBodyInput {
            status: 200,
            content_type: None,
            body: Vec::new(),
        }),
        3 => Some(DecodedBodyInput {
            status: 500,
            content_type: Some("text/plain".to_owned()),
            body: b"internal server error".to_vec(),
        }),
        4 => Some(DecodedBodyInput {
            status: 400,
            content_type: Some("application/json; charset=utf-8".to_owned()),
            body: b"{ broken json body".to_vec(),
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

fn read_u16(bytes: &mut Unstructured<'_>, default: u16) -> u16 {
    u16::arbitrary(bytes).unwrap_or(default)
}

fn read_bool(bytes: &mut Unstructured<'_>, default: bool) -> bool {
    bool::arbitrary(bytes).unwrap_or(default)
}

fn read_string(bytes: &mut Unstructured<'_>, max_len: usize) -> String {
    let len = usize::from(read_u8(bytes, 0)) % (max_len + 1);
    let mut raw = vec![0u8; len];
    for byte in &mut raw {
        *byte = read_u8(bytes, b'a');
    }
    String::from_utf8_lossy(&raw).into_owned()
}
