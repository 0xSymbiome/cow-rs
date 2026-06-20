#![cfg(target_arch = "wasm32")]

use cow_sdk_app_data::AppDataError;
use cow_sdk_core::{Redacted, TransportError, TransportErrorClass};
use cow_sdk_wasm::exports::WasmError;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const SECRET_URL: &str = "https://token:secret@example.test/orders";
const SECRET_BODY: &str = "{\"apiKey\":\"secret\"}";

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn error_value(error: WasmError) -> Value {
    json(error.into_js())
}

#[wasm_bindgen_test]
fn transport_connect_error_uses_redacted_message() {
    let error = WasmError::from(TransportError::Transport {
        class: TransportErrorClass::Connect,
        detail: Redacted::new(SECRET_URL.to_owned()),
    });
    let value = error_value(error);

    assert_eq!(value["kind"], "transport");
    let message = value["message"].as_str().unwrap();
    assert!(message.contains("[redacted]"));
    assert!(!message.contains(SECRET_URL));
    assert!(!value.to_string().contains("secret"));
}

#[wasm_bindgen_test]
fn transport_configuration_error_uses_redacted_message() {
    let error = WasmError::from(TransportError::Configuration {
        message: Redacted::new(SECRET_URL.to_owned()),
    });
    let value = error_value(error);

    assert_eq!(value["class"], "builder");
    let message = value["message"].as_str().unwrap();
    assert!(message.contains("[redacted]"));
    assert!(!message.contains(SECRET_URL));
}

#[wasm_bindgen_test]
fn http_status_error_redacts_headers_and_body() {
    let error = WasmError::from(TransportError::HttpStatus {
        status: 401,
        headers: vec![(
            "authorization".to_owned(),
            Redacted::new("Bearer secret".to_owned()),
        )],
        body: Redacted::new(SECRET_BODY.to_owned()),
    });
    let value = error_value(error);

    assert_eq!(value["status"], 401);
    assert_eq!(value["headers"][0][1], "[redacted]");
    assert_eq!(value["body"], "{\"apiKey\":\"[redacted]\"}");
    assert!(!value.to_string().contains("secret"));
}

#[wasm_bindgen_test]
fn app_data_transport_error_redacts_detail() {
    let error = WasmError::from(AppDataError::Transport {
        class: TransportErrorClass::Other,
        detail: Redacted::new(SECRET_URL.to_owned()),
    });
    let value = error_value(error);

    assert_eq!(value["kind"], "appData");
    let message = value["message"].as_str().unwrap();
    assert!(message.contains("[redacted]"));
    assert!(!message.contains(SECRET_URL));
}

#[wasm_bindgen_test]
fn debug_format_of_redacted_transport_error_does_not_expose_secret() {
    let error = TransportError::Configuration {
        message: Redacted::new(SECRET_URL.to_owned()),
    };
    let debug = format!("{error:?}");

    assert!(!debug.contains("secret"));
    assert!(debug.contains("[redacted]"));
}

#[wasm_bindgen_test]
fn display_format_of_redacted_transport_error_does_not_expose_secret() {
    let error = TransportError::Transport {
        class: TransportErrorClass::Connect,
        detail: Redacted::new(SECRET_URL.to_owned()),
    };
    let display = error.to_string();

    assert!(!display.contains("secret"));
    assert!(display.contains("[redacted]"));
}

#[wasm_bindgen_test]
fn wasm_error_debug_does_not_expose_redacted_transport_secret() {
    let error = WasmError::from(TransportError::Configuration {
        message: Redacted::new(SECRET_URL.to_owned()),
    });
    let debug = format!("{error:?}");

    assert!(!debug.contains("secret"));
    assert!(debug.contains("[redacted]"));
}

#[wasm_bindgen_test]
fn errors_module_does_not_unwrap_redacted_values() {
    let source = include_str!("../src/exports/errors.rs");

    assert!(!source.contains("Redacted::into_inner"));
    assert!(!source.contains(".into_inner()"));
}
