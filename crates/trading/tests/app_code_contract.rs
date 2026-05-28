use cow_sdk_core::{AppCode, AppCodeError, SupportedChainId};
use cow_sdk_trading::{TradingError, TradingSdkBuilder};

#[test]
fn app_code_accepts_source_backed_examples_without_extra_shape_rules() {
    for value in [
        "CoW Swap",
        "cow-rs/wasm-console",
        "COW_BRIDGING_REACT_EXAMPLE",
        "YOUR_APP_CODE",
    ] {
        let app_code = AppCode::new(value).expect("source-backed appCode should validate");
        assert_eq!(app_code.as_str(), value);
    }
}

#[test]
fn app_code_rejects_only_empty_nul_and_ascii_control_characters() {
    assert!(matches!(AppCode::new(""), Err(AppCodeError::Empty)));
    assert!(matches!(
        AppCode::new("cow-rs\0console"),
        Err(AppCodeError::NulByte)
    ));
    assert!(matches!(
        AppCode::new("cow-rs\nconsole"),
        Err(AppCodeError::ControlCharacter)
    ));
    assert!(matches!(
        AppCode::new("cow-rs\u{7f}console"),
        Err(AppCodeError::ControlCharacter)
    ));
}

#[test]
fn builder_surfaces_app_code_validation_as_typed_trading_error() {
    let error = TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_app_code("")
        .build_ready()
        .expect_err("empty appCode must fail at the ready terminal");

    assert!(matches!(error, TradingError::AppCode(AppCodeError::Empty)));
}
