use cow_sdk_alloy_provider::{HttpTransport, TransportUnset};
use cow_sdk_core::Redacted;

fn main() {
    let _unset = TransportUnset { _private: () };
    let _http = HttpTransport {
        url: Redacted::new(reqwest::Url::parse("https://example.invalid").unwrap()),
    };
}
