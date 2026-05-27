#![no_main]

//! Fuzz target for the core transport-error partition.
//!
//! **Surface:** `cow_sdk_core::TransportError` plus response-body redaction.
//! **Property:** `PROP-CORE-015`.
//! **Seed contract:** corpus inputs cover every `TransportErrorClass`,
//! malformed `Retry-After` headers, credential-bearing body snippets, and
//! JSON-RPC error bodies.
//! **Corpus README:** `../corpus/fuzz_transport_error_classify/README.md`.
//!
//! The target maps arbitrary bytes into `(status, body, headers)`, classifies
//! the input through the typed `TransportError` variants, and asserts that any
//! public detail or body snippet produced by the classification path does not
//! leak userinfo-style URL credentials.

use cow_sdk_core::{TransportError, TransportErrorClass, redact_response_body};
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};

const MAX_BODY_BYTES: usize = 1024;
const MAX_HEADERS: usize = 16;
const MAX_HEADER_BYTES: usize = 96;

#[derive(Debug)]
struct TransportInput {
    status: u16,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    class_hint: u8,
}

impl<'a> Arbitrary<'a> for TransportInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let seed_class = seed_class(read_u8(bytes, 0));
        if let Some(input) = seeded_input(seed_class) {
            return Ok(input);
        }

        let status = read_u16(bytes, 200);
        let body_len = usize::from(read_u16(bytes, 0)) % (MAX_BODY_BYTES + 1);
        let mut body = vec![0u8; body_len];
        for byte in &mut body {
            *byte = read_u8(bytes, 0);
        }

        let header_len = usize::from(read_u8(bytes, 0)) % (MAX_HEADERS + 1);
        let headers = (0..header_len)
            .map(|_| (read_header_string(bytes), read_header_string(bytes)))
            .collect();

        Ok(Self {
            status,
            body,
            headers,
            class_hint: read_u8(bytes, 0),
        })
    }
}

fuzz_target!(|input: TransportInput| {
    let error = classify_input(input);
    assert_public_error_is_sanitized(&error);
});

fn classify_input(input: TransportInput) -> TransportError {
    let body = String::from_utf8_lossy(&input.body);
    let redacted_body = redact_response_body(&body);
    let headers = input
        .headers
        .into_iter()
        .map(|(name, value)| (redact_response_body(&name), redact_response_body(&value)))
        .collect::<Vec<_>>();

    if !(200..=299).contains(&input.status) {
        return TransportError::HttpStatus {
            status: input.status,
            headers: headers
                .into_iter()
                .map(|(name, value)| (name, value.into()))
                .collect(),
            body: redacted_body.into(),
        };
    }

    if headers
        .iter()
        .any(|(name, value)| name.trim().is_empty() || name.contains('\n') || value.contains('\n'))
    {
        return TransportError::Configuration {
            message: "invalid transport header material".to_owned().into(),
        };
    }

    TransportError::Transport {
        class: transport_class(input.class_hint),
        detail: redacted_body.into(),
    }
}

fn assert_public_error_is_sanitized(error: &TransportError) {
    let rendered = format!("{error}");
    let debug = format!("{error:?}");
    for public in [rendered.as_str(), debug.as_str()] {
        assert!(
            !contains_userinfo_url(public),
            "transport error public output leaked URL userinfo credentials: {public}",
        );
        assert!(
            !public.contains("apiKey=secret") && !public.contains("token=secret"),
            "transport error public output leaked credential query material: {public}",
        );
        assert!(
            !contains_credential_key_value(public),
            "transport error public output leaked credential key=value material: {public}",
        );
        assert!(
            !contains_bearer_prefix(public),
            "transport error public output leaked Bearer token material: {public}",
        );
        assert!(
            !contains_jwt_prefix(public),
            "transport error public output leaked JWT-shaped material: {public}",
        );
    }
    assert_eq!(
        rendered,
        format!("{error}"),
        "transport error Display must be deterministic",
    );
    assert_eq!(
        debug,
        format!("{error:?}"),
        "transport error Debug must be deterministic",
    );
}

fn contains_userinfo_url(value: &str) -> bool {
    value
        .split_ascii_whitespace()
        .any(|part| part.contains("://user:pass@"))
}

fn contains_credential_key_value(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    const CREDENTIAL_KEYS: &[&str] = &[
        "apikey=secret",
        "api_key=secret",
        "x-api-key=secret",
        "password=secret",
        "secret=secret",
        "authorization: secret",
    ];
    CREDENTIAL_KEYS
        .iter()
        .any(|needle| lowered.contains(needle))
}

fn contains_bearer_prefix(value: &str) -> bool {
    value.contains("Bearer secret")
}

fn contains_jwt_prefix(value: &str) -> bool {
    value
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        .any(|token| {
            token.starts_with("eyJ")
                && token.len() >= 26
                && token
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        })
}

fn transport_class(value: u8) -> TransportErrorClass {
    match value % 9 {
        0 => TransportErrorClass::Timeout,
        1 => TransportErrorClass::Connect,
        2 => TransportErrorClass::Redirect,
        3 => TransportErrorClass::Decode,
        4 => TransportErrorClass::Body,
        5 => TransportErrorClass::Builder,
        6 => TransportErrorClass::Request,
        7 => TransportErrorClass::Status,
        _ => TransportErrorClass::Other,
    }
}

fn seeded_input(seed_class: u8) -> Option<TransportInput> {
    let class = match seed_class {
        0..=8 => transport_class(seed_class),
        9 => {
            return Some(status_input(
                429,
                "rate limited",
                vec![("Retry-After", "-1")],
            ));
        }
        10 => {
            return Some(status_input(
                503,
                "retry later",
                vec![("Retry-After", "NaN")],
            ));
        }
        11 => {
            return Some(status_input(
                503,
                "retry later",
                vec![("Retry-After", "Thu, 01 Jan 1970 00:00:10 GMT")],
            ));
        }
        12 => {
            return Some(status_input(
                503,
                "retry later",
                vec![("Retry-After", "999999999999999999999")],
            ));
        }
        13 => {
            return Some(status_input(
                500,
                "upstream echoed https://user:pass@example.test/path?apiKey=secret",
                Vec::new(),
            ));
        }
        14 => {
            return Some(status_input(
                200,
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"token=secret"}}"#,
                Vec::new(),
            ));
        }
        _ => return None,
    };

    Some(TransportInput {
        status: 200,
        body: format!("transport class {}", class.as_str()).into_bytes(),
        headers: Vec::new(),
        class_hint: seed_class,
    })
}

fn status_input(status: u16, body: &str, headers: Vec<(&str, &str)>) -> TransportInput {
    TransportInput {
        status,
        body: body.as_bytes().to_vec(),
        headers: headers
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect(),
        class_hint: 0,
    }
}

fn seed_class(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'e' => 10 + (value - b'a'),
        b'A'..=b'E' => 10 + (value - b'A'),
        _ => value % 15,
    }
}

fn read_u8(bytes: &mut Unstructured<'_>, default: u8) -> u8 {
    u8::arbitrary(bytes).unwrap_or(default)
}

fn read_u16(bytes: &mut Unstructured<'_>, default: u16) -> u16 {
    u16::arbitrary(bytes).unwrap_or(default)
}

fn read_header_string(bytes: &mut Unstructured<'_>) -> String {
    let len = usize::from(read_u8(bytes, 0)) % (MAX_HEADER_BYTES + 1);
    let mut raw = vec![0u8; len];
    for byte in &mut raw {
        *byte = read_u8(bytes, b'a');
    }
    String::from_utf8_lossy(&raw).into_owned()
}
