use std::fmt::{Debug, Display};

use cow_sdk::{
    CowError,
    app_data::{AppDataError, AppDataParams},
    contracts::ContractsError,
    core::{
        Address, Amount, AppCodeError, CoreError, CowEnv, HostPolicyError, TransportError,
        TransportErrorClass, UrlParseFailureClass, ValidationError, ValidationReason,
    },
    orderbook::{
        OrderbookApiError, OrderbookError, OrderbookRejection, ResponseBody, SigningScheme,
    },
    signing::SigningError,
    trading::{ClientRejection, OrderbookContextValue, TradingError},
};
#[cfg(feature = "subgraph")]
use cow_sdk_subgraph::{SubgraphError, SubgraphGraphQlError, SubgraphRequestErrorContext};
use serde::Serialize;
use serde_json::{Value, json};

const URL_SECRET: &str = "https://user:pass@example.com/path?key=secret";
const AUTH_SECRET: &str = "Authorization: Bearer eyJhbGc...";
const PRIVATE_KEY_SECRET: &str =
    "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PEM_SECRET: &str = "BEGIN PRIVATE KEY";

#[test]
fn core_error_surfaces_redact_secret_bearing_payloads() {
    let errors = [
        CoreError::Validation(ValidationError::EmptyField { field: "appCode" }),
        CoreError::MissingBaseUrl {
            chain_id: 1,
            env: CowEnv::Prod,
            partner_api: true,
        },
        CoreError::Serialization(secret_payload().into()),
        CoreError::TransportContract(secret_payload().into()),
        CoreError::Cancelled,
    ];

    assert_all_render("CoreError", &errors);

    let transport_errors = [
        TransportError::Transport {
            class: TransportErrorClass::Timeout,
            detail: secret_payload().into(),
        },
        TransportError::Configuration {
            message: secret_payload().into(),
        },
        TransportError::HttpStatus {
            status: 503,
            headers: vec![("authorization".to_owned(), secret_payload().into())],
            body: secret_payload().into(),
        },
    ];

    assert_all_render("TransportError", &transport_errors);
}

#[test]
fn validation_and_host_policy_errors_keep_safe_public_diagnostics() {
    let validation_errors = [
        ValidationError::EmptyField { field: "appCode" },
        ValidationError::InvalidHttpHeaderValue {
            field: "user-agent",
        },
        ValidationError::InvalidHexPrefix { field: "appData" },
        ValidationError::InvalidHexLength {
            field: "appData",
            expected: 64,
        },
        ValidationError::InvalidHexCharacters { field: "appData" },
        ValidationError::InvalidNumeric {
            field: "sellAmount",
        },
        ValidationError::NumericOverflow {
            field: "sellAmount",
        },
        ValidationError::UnsupportedChain { chain_id: 999_999 },
        ValidationError::ValidToOutOfRange {
            actual_seconds: 1,
            min: 60,
            max: 4_294_967_295,
        },
    ];
    assert_all_render("ValidationError", &validation_errors);

    let host_errors = [
        HostPolicyError::UnparsableUrl {
            class: UrlParseFailureClass::MissingHost,
        },
        HostPolicyError::HostNotAllowed {
            host: secret_payload().into(),
        },
        HostPolicyError::UnsupportedScheme { scheme: "other" },
    ];
    assert_all_render("HostPolicyError", &host_errors);
    assert_all_serialize("HostPolicyError", &host_errors);
}

/// The four fixed-width identity newtype constructors
/// ([`Address::new`], `AppDataHash::new`, `Hash32::new`, and
/// `OrderUid::new`) reject inputs whose payload contains non-hex
/// characters by emitting [`ValidationError::InvalidHexCharacters`] —
/// a variant whose `Display` and `Debug` rendering carries only the
/// `field: &'static str` tag. The classifier inside `crates/core` MUST
/// discard the offending character and the byte offset before
/// constructing the cow variant; this test pins that contract by
/// feeding [`Address::new`] a payload of 40 `'Z'` characters and
/// asserting neither `Z` nor the literal `"index"` appears in any
/// rendered surface of the returned [`CoreError`].
#[test]
fn fixed_width_identity_constructors_drop_offending_input_character_and_offset() {
    let result = Address::new("0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ");
    let error = result.expect_err("invalid hex payload must fail closed at the constructor");

    let display = error.to_string();
    let debug = format!("{error:?}");

    assert!(
        !display.contains('Z'),
        "Display rendering leaked the offending input character: {display}",
    );
    assert!(
        !display.contains("index"),
        "Display rendering leaked the input byte offset: {display}",
    );
    assert!(
        !debug.contains('Z'),
        "Debug rendering leaked the offending input character: {debug}",
    );
    assert!(
        !debug.contains("index"),
        "Debug rendering leaked the input byte offset: {debug}",
    );
}

#[test]
fn orderbook_errors_redact_api_transport_and_source_payloads() {
    let api_error =
        OrderbookApiError::new(500, secret_payload(), ResponseBody::Text(secret_payload()));
    assert_render("OrderbookApiError", &api_error);

    let rejected_api_error = OrderbookApiError::new(
        422,
        secret_payload(),
        ResponseBody::Json(json!({
            "errorType": "DuplicatedOrder",
            "description": secret_payload(),
        })),
    );
    let rejected: OrderbookError = rejected_api_error.into();

    let errors = [
        OrderbookError::Core(CoreError::Serialization(secret_payload().into())),
        OrderbookError::Api(Box::new(OrderbookApiError::new(
            500,
            secret_payload(),
            ResponseBody::Json(json!({ "detail": secret_payload() })),
        ))),
        rejected,
        OrderbookError::Transport {
            class: TransportErrorClass::Connect,
            detail: secret_payload().into(),
        },
        OrderbookError::HostPolicy(HostPolicyError::HostNotAllowed {
            host: secret_payload().into(),
        }),
        OrderbookError::from(json_error()),
        OrderbookError::InvalidTradesQuery {
            field: "owner",
            reason: ValidationReason::Missing,
        },
        OrderbookError::InvalidQuoteRequest {
            field: "sellAmount",
            reason: ValidationReason::BadShape {
                details: "not a decimal quantity",
            },
        },
        OrderbookError::IncompatibleSigningScheme {
            signing_scheme: SigningScheme::Eip712,
            onchain_order: true,
        },
        OrderbookError::InvalidTransform {
            field: "executedAmount",
            reason: ValidationReason::OutOfRange {
                details: "negative values are not supported",
            },
        },
        OrderbookError::Cancelled,
    ];

    assert_all_render("OrderbookError", &errors);
}

#[test]
fn orderbook_rejections_redact_remote_message_payloads() {
    let rejections = [
        OrderbookRejection::DuplicatedOrder,
        OrderbookRejection::OldOrderActivelyBidOn,
        OrderbookRejection::QuoteNotFound,
        OrderbookRejection::QuoteNotVerified,
        OrderbookRejection::MissingFrom,
        OrderbookRejection::WrongOwner,
        OrderbookRejection::InvalidEip1271Signature,
        OrderbookRejection::InvalidSignature,
        OrderbookRejection::IncompatibleSigningScheme,
        OrderbookRejection::InsufficientBalance,
        OrderbookRejection::InsufficientAllowance,
        OrderbookRejection::ZeroAmount,
        OrderbookRejection::NonZeroFee,
        OrderbookRejection::SellAmountOverflow,
        OrderbookRejection::TooMuchGas,
        OrderbookRejection::TooManyLimitOrders,
        OrderbookRejection::TransferSimulationFailed,
        OrderbookRejection::InsufficientValidTo,
        OrderbookRejection::ExcessiveValidTo,
        OrderbookRejection::InvalidNativeSellToken,
        OrderbookRejection::SameBuyAndSellToken,
        OrderbookRejection::UnsupportedToken,
        OrderbookRejection::UnsupportedBuyTokenDestination,
        OrderbookRejection::UnsupportedSellTokenSource,
        OrderbookRejection::UnsupportedOrderType,
        OrderbookRejection::AppDataInvalid {
            message: secret_payload().into(),
        },
        OrderbookRejection::InvalidAppData,
        OrderbookRejection::AppDataHashMismatch,
        OrderbookRejection::AppDataMismatch {
            message: secret_payload().into(),
        },
        OrderbookRejection::AppdataFromMismatch,
        OrderbookRejection::MetadataSerializationFailed,
        OrderbookRejection::NoLiquidity,
        OrderbookRejection::TradingOutsideAllowedWindow,
        OrderbookRejection::TokenTemporarilySuspended,
        OrderbookRejection::InsufficientLiquidity,
        OrderbookRejection::CustomSolverError,
        OrderbookRejection::InvalidTradeFilter,
        OrderbookRejection::InvalidLimit,
        OrderbookRejection::LimitOutOfBounds,
        OrderbookRejection::SellAmountDoesNotCoverFee {
            fee_amount: Amount::new("1").expect("amount fixture must parse"),
        },
        OrderbookRejection::AlreadyCancelled,
        OrderbookRejection::OrderFullyExecuted,
        OrderbookRejection::OrderExpired,
        OrderbookRejection::OrderNotFound,
        OrderbookRejection::NotFound {
            message: secret_payload().into(),
        },
        OrderbookRejection::OnChainOrder,
        OrderbookRejection::Forbidden,
        OrderbookRejection::InternalServerError,
        OrderbookRejection::Unknown {
            code: secret_payload().into(),
            message: secret_payload().into(),
        },
    ];

    assert_all_render("OrderbookRejection", &rejections);
    assert_all_serialize("OrderbookRejection", &rejections);
}

#[test]
fn app_data_errors_redact_public_serialized_payloads() {
    let errors = [
        AppDataError::InvalidAppDataHex,
        AppDataError::InvalidCid,
        AppDataError::InvalidSchemaVersion(secret_payload().into()),
        AppDataError::MissingSchemaVersion,
        AppDataError::from(json_error()),
        AppDataError::InvalidAppDataProvided {
            field: "document",
            reason: ValidationReason::BadShape {
                details: "typed metadata validation failed",
            },
        },
        AppDataError::InvalidPartnerFee {
            field: "bps",
            reason: ValidationReason::OutOfRange {
                details: "must be below the configured maximum",
            },
        },
        AppDataError::InvalidFlashloanHints {
            field: "token",
            reason: ValidationReason::Precondition {
                details: "address must not be zero",
            },
        },
        AppDataError::Calculation {
            source: Box::new(SafeSourceError),
        },
        AppDataError::Transport {
            class: TransportErrorClass::Request,
            detail: secret_payload().into(),
        },
        AppDataError::Cancelled,
        AppDataError::TooLarge {
            actual_bytes: 4_097,
            max_bytes: 4_096,
        },
    ];

    assert_all_render("AppDataError", &errors);
    assert_all_serialize("AppDataError", &errors);
}

/// The orderbook decode-failure variant must surface only the serde failure
/// category and structural position. A `serde_json::Error` rendering can echo
/// the decoded upstream response bytes — an unknown field name under
/// `deny_unknown_fields`, or a type-mismatched value — so the construction
/// path drops them before the `Display`/`Debug` surface ever sees them.
#[test]
fn orderbook_serialization_error_drops_decoded_response_bytes() {
    let errors = [
        OrderbookError::from(serde_unknown_field_error()),
        OrderbookError::from(serde_type_mismatch_error()),
    ];
    assert_all_render("OrderbookError::Serialization", &errors);
}

/// The app-data and contracts JSON decode-failure variants follow the same rule
/// as the orderbook one: a `serde_json::Error` rendering can echo decoded
/// document or caller bytes — an unknown field name under `deny_unknown_fields`,
/// or a type-mismatched value — so the `From<serde_json::Error>` construction
/// path drops them and keeps only the structural `{ category, line, column }`
/// triple (ADR 0025).
#[test]
fn app_data_and_contracts_serialization_errors_drop_decoded_bytes() {
    let app_data_errors = [
        AppDataError::from(serde_unknown_field_error()),
        AppDataError::from(serde_type_mismatch_error()),
    ];
    assert_all_render("AppDataError::Json", &app_data_errors);
    assert_all_serialize("AppDataError::Json", &app_data_errors);

    let contracts_errors = [
        ContractsError::from(serde_unknown_field_error()),
        ContractsError::from(serde_type_mismatch_error()),
    ];
    assert_all_render("ContractsError::Serialization", &contracts_errors);
}

/// The typed sub-metadata deserializer lifts caller-supplied `metadata.signer`,
/// `metadata.flashloan`, and `metadata.hooks` values out of an app-data
/// document. A malformed value must surface a fixed, field-tagged message and
/// never the caller's offending key or value, even though the raw
/// `serde_json::Error` rendering would echo it.
#[test]
fn app_data_metadata_parse_failures_do_not_echo_caller_input() {
    let signer_input = json!({ "metadata": { "signer": PRIVATE_KEY_SECRET } });

    let mut flashloan_object = serde_json::Map::new();
    flashloan_object.insert(AUTH_SECRET.to_owned(), json!(1));
    let flashloan_input = json!({ "metadata": { "flashloan": Value::Object(flashloan_object) } });

    for input in [signer_input, flashloan_input] {
        let serde_error = serde_json::from_value::<AppDataParams>(input)
            .expect_err("malformed metadata must fail to deserialize");
        let error = AppDataError::from(serde_error);
        assert_render("AppDataError::Json (metadata parse)", &error);
        assert_serialize("AppDataError::Json (metadata parse)", &error);
    }
}

/// `AppDataError::Calculation` boxes a typed source whose rendering could embed
/// caller-derived bytes. The variant must surface only the stable operation
/// label through `Display` and `Serialize`; the source detail stays reachable
/// solely through [`std::error::Error::source`].
#[test]
fn app_data_calculation_error_does_not_render_boxed_source() {
    let error = AppDataError::Calculation {
        source: Box::new(SecretSourceError),
    };
    assert_render("AppDataError::Calculation", &error);
    assert_serialize("AppDataError::Calculation", &error);
}

#[test]
fn contracts_and_signing_errors_redact_secret_bearing_messages() {
    let contracts_errors = [
        ContractsError::Core(CoreError::Serialization(secret_payload().into())),
        ContractsError::Cancelled,
        ContractsError::UnsupportedChain(999_999),
        ContractsError::InvalidOrderUidLength { actual: 4 },
        ContractsError::UnsupportedSigningScheme(99),
        ContractsError::InvalidEip1271SignatureData,
        ContractsError::UnsupportedEip1271Verifier {
            verifier: address("0x1111111111111111111111111111111111111111"),
        },
        ContractsError::Eip1271Provider {
            operation: "read_contract",
            message: secret_payload().into(),
        },
        ContractsError::MalformedEip1271Response {
            response: secret_payload().into(),
        },
        ContractsError::Eip1271MagicValueMismatch {
            expected: [0x16, 0x26, 0xba, 0x7e],
            actual: [0xff, 0xff, 0xff, 0xff],
        },
        ContractsError::ZeroReceiver,
        ContractsError::Provider {
            operation: "eth_call",
            message: secret_payload().into(),
        },
        ContractsError::Abi(alloy_sol_types::Error::Overrun),
        ContractsError::DecodeHex {
            field: "signature",
            source: alloy_primitives::hex::decode("zz").unwrap_err(),
        },
        ContractsError::InvalidHexPrefix { field: "signature" },
        ContractsError::InvalidDecodedLength {
            field: "signature",
            expected: 65,
            actual: 64,
        },
        ContractsError::from(json_error()),
        ContractsError::InvalidSignatureLength { actual: 64 },
        ContractsError::InvalidSignatureRecoveryByte { value: 3 },
        ContractsError::SignatureSchemeNotEcdsa,
        ContractsError::SignatureRecovery {
            message: secret_payload().into(),
        },
    ];

    assert_all_render("ContractsError", &contracts_errors);

    let signing_errors = [
        SigningError::Core(CoreError::Serialization(secret_payload().into())),
        SigningError::Contracts(ContractsError::Provider {
            operation: "eth_call",
            message: secret_payload().into(),
        }),
        SigningError::Serialization(secret_payload().into()),
        SigningError::Signer {
            operation: "sign_typed_data_payload",
            message: secret_payload().into(),
        },
        SigningError::SignerRejection {
            label: "typed-data signature",
            code: 4001,
        },
        SigningError::UnsupportedSignerGeneratedScheme {
            scheme: cow_sdk::signing::SigningScheme::Eip1271,
        },
        SigningError::Cancelled,
    ];

    assert_all_render("SigningError", &signing_errors);
}

#[test]
fn trading_errors_redact_workflow_message_and_conflict_payloads() {
    let errors = [
        TradingError::Core(CoreError::Serialization(secret_payload().into())),
        TradingError::AppData(AppDataError::Transport {
            class: TransportErrorClass::Other,
            detail: secret_payload().into(),
        }),
        TradingError::Contracts(ContractsError::Provider {
            operation: "eth_call",
            message: secret_payload().into(),
        }),
        TradingError::Orderbook(OrderbookError::Transport {
            class: TransportErrorClass::Other,
            detail: secret_payload().into(),
        }),
        TradingError::Signing(SigningError::Signer {
            operation: "sign_order",
            message: secret_payload().into(),
        }),
        TradingError::MissingQuoterParams("chainId"),
        TradingError::MissingTraderParams("appCode"),
        TradingError::QuoteValidityConflict,
        TradingError::MissingQuoteId("post_swap_order_from_quote"),
        TradingError::MissingOwner,
        TradingError::MissingSubmissionOwner,
        TradingError::InjectedOrderbookContextConflict {
            field: "baseUrl",
            requested: OrderbookContextValue::BaseUrl(secret_payload().into()),
            configured: OrderbookContextValue::BaseUrl(secret_payload().into()),
        },
        TradingError::MissingQuoteOrderbookBinding,
        TradingError::QuoteOrderbookBindingConflict {
            field: "baseUrl",
            quoted: OrderbookContextValue::BaseUrl(secret_payload().into()),
            submitted: OrderbookContextValue::BaseUrl(secret_payload().into()),
        },
        TradingError::ClientRejected(ClientRejection::MissingFrom),
        TradingError::Signer {
            operation: "sign_order",
            message: secret_payload().into(),
        },
        TradingError::Provider {
            operation: "eth_call",
            message: secret_payload().into(),
        },
        TradingError::InvalidNumeric {
            field: "sellAmount",
            value: secret_payload().into(),
        },
        TradingError::NumericOverflow {
            field: "sellAmount",
            value: secret_payload().into(),
        },
        TradingError::InvalidInput {
            field: "sellToken",
            reason: ValidationReason::BadShape {
                details: "address must be 20 bytes",
            },
        },
        TradingError::UnsupportedLocalSigningScheme {
            scheme: cow_sdk::contracts::SigningScheme::Eip1271,
        },
        TradingError::AppCode(AppCodeError::ControlCharacter),
        TradingError::Cancelled,
    ];

    assert_all_render("TradingError", &errors);
}

#[cfg(feature = "subgraph")]
#[test]
fn subgraph_errors_and_contexts_redact_serialized_request_payloads() {
    let graph_error: SubgraphGraphQlError = serde_json::from_value(json!({
        "message": secret_payload(),
        "locations": [{ "line": 1, "column": 2 }],
        "extensions": { "token": secret_payload() },
    }))
    .expect("GraphQL error fixture must deserialize through the public surface");
    assert_debug_render("SubgraphGraphQlError", &graph_error);
    assert_serialize("SubgraphGraphQlError", &graph_error);

    let context = subgraph_context();
    assert_debug_render("SubgraphRequestErrorContext", &context);
    assert_serialize("SubgraphRequestErrorContext", &context);

    let errors = [
        SubgraphError::UnsupportedNetwork { chain_id: 999_999 },
        SubgraphError::NoTotalsFound,
        SubgraphError::Transport {
            context: Box::new(subgraph_context()),
            class: TransportErrorClass::Timeout,
            details: secret_payload().into(),
        },
        SubgraphError::TransportConfiguration {
            class: TransportErrorClass::Builder,
            details: secret_payload().into(),
        },
        SubgraphError::HostPolicy(HostPolicyError::HostNotAllowed {
            host: secret_payload().into(),
        }),
        SubgraphError::HttpStatus {
            context: Box::new(subgraph_context()),
            status: 500,
            body: secret_payload().into(),
        },
        SubgraphError::Serialization {
            context: Box::new(subgraph_context()),
            body: secret_payload().into(),
            details: secret_payload().into(),
        },
        SubgraphError::GraphQl {
            context: Box::new(subgraph_context()),
            errors: vec![graph_error],
        },
        SubgraphError::MissingData {
            context: Box::new(subgraph_context()),
        },
        SubgraphError::Cancelled,
    ];

    assert_all_render("SubgraphError", &errors);
    assert_all_serialize("SubgraphError", &errors);
}

/// Every diagnostic `SubgraphError` variant must carry at least one piece of
/// plaintext structural diagnostic (chain id, error count, status code,
/// transport class, or response-body byte count) in its `Display` rendering,
/// so that the default `format!("{e}")` path remains actionable even when
/// every `Redacted<T>` field collapses to the workspace placeholder.
///
/// The check is intentionally coarse: it asserts the rendered string contains
/// at least one ASCII digit. Every accepted variant carries either a chain id,
/// a status code, an error count, or a byte count, all of which render as
/// integers; a regression that drops these into `Redacted<T>`-only territory
/// collapses the rendered output to a tautological `for [redacted]` shape and
/// fails the check.
///
/// `Cancelled` is excluded because the variant intentionally encodes no
/// request context, and `NoTotalsFound` is excluded because the typed
/// variant tag is the entire diagnostic. Both are exhaustively documented
/// rather than left to inference.
#[cfg(feature = "subgraph")]
#[test]
fn subgraph_display_carries_plaintext_structural_diagnostic() {
    let graph_error: SubgraphGraphQlError = serde_json::from_value(json!({
        "message": secret_payload(),
        "locations": [{ "line": 4, "column": 7 }],
    }))
    .expect("GraphQL error fixture must deserialize through the public surface");

    let diagnostic_variants = [
        SubgraphError::UnsupportedNetwork {
            chain_id: 11_155_111,
        },
        SubgraphError::Transport {
            context: Box::new(subgraph_context()),
            class: TransportErrorClass::Timeout,
            details: secret_payload().into(),
        },
        SubgraphError::HttpStatus {
            context: Box::new(subgraph_context()),
            status: 503,
            body: secret_payload().into(),
        },
        SubgraphError::Serialization {
            context: Box::new(subgraph_context()),
            body: secret_payload().into(),
            details: secret_payload().into(),
        },
        SubgraphError::GraphQl {
            context: Box::new(subgraph_context()),
            errors: vec![graph_error],
        },
        SubgraphError::MissingData {
            context: Box::new(subgraph_context()),
        },
    ];

    for variant in &diagnostic_variants {
        let display = variant.to_string();
        assert!(
            !display.trim().is_empty(),
            "SubgraphError Display rendering must not be empty for {variant:?}",
        );
        assert!(
            !display.contains('\n'),
            "SubgraphError Display rendering must remain single-line for {variant:?}: {display}",
        );
        assert!(
            display.bytes().any(|byte| byte.is_ascii_digit()),
            "SubgraphError Display rendering was tautological (no plaintext digit) for {variant:?}: {display}",
        );
        assert_no_secret("SubgraphError", "Display non-tautology", &display);
    }
}

#[test]
fn sdk_error_facade_redacts_nested_public_errors() {
    let errors = [
        CowError::Types(CoreError::Serialization(secret_payload().into())),
        CowError::Signing(SigningError::Signer {
            operation: "sign_order",
            message: secret_payload().into(),
        }),
        CowError::AppData(AppDataError::Transport {
            class: TransportErrorClass::Request,
            detail: secret_payload().into(),
        }),
        CowError::Contracts(ContractsError::Provider {
            operation: "eth_call",
            message: secret_payload().into(),
        }),
        CowError::Orderbook(OrderbookError::Transport {
            class: TransportErrorClass::Other,
            detail: secret_payload().into(),
        }),
        CowError::Trading(TradingError::Signer {
            operation: "sign_order",
            message: secret_payload().into(),
        }),
    ];

    assert_all_render("CowError", &errors);
}

#[cfg(feature = "alloy")]
#[test]
fn alloy_adapter_errors_redact_secret_bearing_payloads() {
    use cow_sdk::{
        alloy::AlloyClientError,
        alloy_provider::ProviderError,
        alloy_signer::SignerError,
        core::{Redacted, TransportErrorClass},
    };

    let client_errors = [
        AlloyClientError::Validation(secret_payload()),
        AlloyClientError::Transport {
            class: TransportErrorClass::Other,
            detail: Redacted::new(secret_payload()),
        },
        AlloyClientError::Signing {
            detail: Redacted::new(secret_payload()),
        },
        AlloyClientError::PendingTransaction {
            detail: Redacted::new(secret_payload()),
        },
        AlloyClientError::Internal(secret_payload()),
    ];
    assert_all_render("AlloyClientError", &client_errors);

    let provider_errors = [
        ProviderError::Validation(secret_payload()),
        ProviderError::Transport {
            class: TransportErrorClass::Other,
            detail: Redacted::new(secret_payload()),
        },
        ProviderError::Internal(secret_payload()),
    ];
    assert_all_render("ProviderError", &provider_errors);

    let signer_errors = [
        SignerError::Validation(secret_payload()),
        SignerError::Signing {
            detail: Redacted::new(secret_payload()),
        },
        SignerError::Internal(secret_payload()),
    ];
    assert_all_render("SignerError", &signer_errors);

    assert_render(
        "AlloyClientError::Remote",
        &AlloyClientError::Remote {
            code: -32_000,
            message: "execution reverted".to_owned(),
        },
    );
    assert_render(
        "ProviderError::Remote",
        &ProviderError::Remote {
            code: -32_000,
            message: "execution reverted".to_owned(),
        },
    );
    assert_render(
        "SignerError::ProviderRequired",
        &SignerError::ProviderRequired {
            method: "send_transaction",
        },
    );
}

#[cfg(feature = "browser-wallet")]
#[test]
fn browser_wallet_errors_surface_method_and_redact_message_and_data() {
    use cow_sdk::browser_wallet::{BrowserWalletError, RpcErrorPayload};

    let rpc_payload = RpcErrorPayload::new(
        4001,
        secret_payload(),
        Some(json!({ "nested": secret_payload() })),
    );
    assert_debug_render("RpcErrorPayload", &rpc_payload);
    assert_serialize("RpcErrorPayload", &rpc_payload);

    let errors = [
        BrowserWalletError::WalletUnavailable,
        BrowserWalletError::DiscoverySelectionRequired { candidates: 2 },
        BrowserWalletError::DiscoverySelectionOutOfRange {
            index: 2,
            candidates: 1,
        },
        BrowserWalletError::InvalidProviderOrigin {
            message: secret_payload().into(),
        },
        BrowserWalletError::UntrustedProviderOrigin {
            origin: secret_payload().into(),
        },
        BrowserWalletError::UserRejectedRequest {
            method: "eth_sendTransaction".to_owned(),
            code: 4001,
            message: secret_payload().into(),
        },
        BrowserWalletError::Disconnected {
            method: "eth_sendTransaction".to_owned(),
            code: 4900,
            message: secret_payload().into(),
        },
        BrowserWalletError::WrongChain {
            method: "eth_sendTransaction".to_owned(),
            code: 4901,
            message: secret_payload().into(),
        },
        BrowserWalletError::ChainNotAdded {
            chain_id: Some(8453),
            method: "eth_sendTransaction".to_owned(),
            code: 4902,
            message: secret_payload().into(),
        },
        BrowserWalletError::InvalidChainConfiguration {
            chain_id: 8453,
            message: secret_payload().into(),
        },
        BrowserWalletError::SessionChainMismatch {
            expected_chain_id: 1,
            session_chain_id: 8453,
        },
        BrowserWalletError::TypedDataChainMismatch {
            expected_chain_id: 1,
            typed_data_chain_id: 8453,
        },
        BrowserWalletError::UnsupportedRpcMethod {
            method: "eth_sendTransaction".to_owned(),
            message: secret_payload().into(),
        },
        BrowserWalletError::MalformedResponse {
            method: "eth_sendTransaction".to_owned(),
            message: secret_payload().into(),
        },
        BrowserWalletError::Rpc {
            method: "eth_sendTransaction".to_owned(),
            code: -32_000,
            message: secret_payload().into(),
            data: Some(json!({ "payload": secret_payload() }).into()),
        },
        BrowserWalletError::JsInterop {
            message: secret_payload().into(),
        },
        BrowserWalletError::Serialization {
            message: secret_payload().into(),
        },
        BrowserWalletError::Core(CoreError::Serialization(secret_payload().into())),
        BrowserWalletError::Cancelled,
    ];

    assert_all_render("BrowserWalletError", &errors);

    // The RPC method name is a closed-set protocol identifier supplied by the
    // SDK, not a credential, so it is surfaced on the public message while the
    // wallet's free-form message stays redacted. The `4001` user-rejection
    // renders as cleanly as the signing-layer typed-data rejection.
    let rejection = BrowserWalletError::UserRejectedRequest {
        method: "eth_sendTransaction".to_owned(),
        code: 4001,
        message: secret_payload().into(),
    };
    let rendered = rejection.to_string();
    assert!(
        rendered.contains("eth_sendTransaction") && rendered.contains("4001"),
        "the rejected method and EIP-1193 code must be visible: {rendered}"
    );
    assert_no_secret("UserRejectedRequest", "Display", &rendered);
}

fn assert_all_render<E>(label: &str, errors: &[E])
where
    E: Debug + Display,
{
    for error in errors {
        assert_render(label, error);
    }
}

fn assert_all_serialize<T>(label: &str, values: &[T])
where
    T: Serialize,
{
    for value in values {
        assert_serialize(label, value);
    }
}

fn assert_render<E>(label: &str, error: &E)
where
    E: Debug + Display,
{
    let display = error.to_string();
    assert_non_empty(label, "Display", &display);
    assert_no_secret(label, "Display", &display);

    let debug = format!("{error:?}");
    assert_non_empty(label, "Debug", &debug);
    assert_no_secret(label, "Debug", &debug);
}

#[cfg(any(feature = "subgraph", feature = "browser-wallet"))]
fn assert_debug_render<T>(label: &str, value: &T)
where
    T: Debug,
{
    let debug = format!("{value:?}");
    assert_non_empty(label, "Debug", &debug);
    assert_no_secret(label, "Debug", &debug);
}

fn assert_serialize<T>(label: &str, value: &T)
where
    T: Serialize,
{
    let json = serde_json::to_string(value).expect("public error JSON serialization must succeed");
    assert_non_empty(label, "Serialize", &json);
    assert_no_secret(label, "Serialize", &json);
}

fn assert_non_empty(label: &str, channel: &str, rendered: &str) {
    assert!(
        !rendered.trim().is_empty(),
        "{label} {channel} rendering must retain a diagnostic"
    );
}

fn assert_no_secret(label: &str, channel: &str, rendered: &str) {
    // The fixtures below are deterministic redaction sentinels chosen to
    // exercise URL, bearer-token, private-key, and PEM redaction paths.
    // The assertion message references each fixture by name rather than
    // by value so the failure log carries enough context to localise the
    // regression without re-emitting the fixture text.
    let fixtures: &[(&str, &str)] = &[
        ("URL_SECRET", URL_SECRET),
        ("AUTH_SECRET", AUTH_SECRET),
        ("PRIVATE_KEY_SECRET", PRIVATE_KEY_SECRET),
        ("PEM_SECRET", PEM_SECRET),
    ];
    for (fixture_name, fixture) in fixtures {
        assert!(
            !rendered.contains(fixture),
            "{label} {channel} leaked the {fixture_name} fixture in rendering"
        );
    }
}

fn secret_payload() -> String {
    format!("{URL_SECRET}\n{AUTH_SECRET}\n{PRIVATE_KEY_SECRET}\n{PEM_SECRET}")
}

fn json_error() -> serde_json::Error {
    serde_json::from_str::<Value>("{ malformed").unwrap_err()
}

/// A `serde_json::Error` whose rendering echoes a secret-bearing field *name*,
/// produced by feeding a `deny_unknown_fields` struct an unknown field keyed on
/// a redaction sentinel.
fn serde_unknown_field_error() -> serde_json::Error {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Strict {
        #[serde(default)]
        #[allow(
            dead_code,
            reason = "field exercises serde(deny_unknown_fields) without being read"
        )]
        known: u8,
    }

    let mut object = serde_json::Map::new();
    object.insert(AUTH_SECRET.to_owned(), json!(1));
    serde_json::from_value::<Strict>(Value::Object(object))
        .err()
        .expect("an unknown field must fail the deny_unknown_fields struct")
}

/// A `serde_json::Error` whose rendering echoes a secret-bearing *value*,
/// produced by a type mismatch feeding a numeric target a secret string.
fn serde_type_mismatch_error() -> serde_json::Error {
    serde_json::from_value::<u64>(json!(PRIVATE_KEY_SECRET))
        .expect_err("a string payload must fail u64 deserialization")
}

fn address(value: &str) -> Address {
    Address::new(value).expect("address fixture must parse")
}

#[cfg(feature = "subgraph")]
fn subgraph_context() -> SubgraphRequestErrorContext {
    SubgraphRequestErrorContext::new(
        1,
        secret_payload(),
        format!("query Totals {{ totals(first: 1) {{ id }} }}\n{URL_SECRET}"),
        Some(secret_payload()),
        Some(json!({ "authorization": secret_payload() })),
    )
}

#[derive(Debug)]
struct SafeSourceError;

impl Display for SafeSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("safe typed source failure")
    }
}

impl std::error::Error for SafeSourceError {}

#[derive(Debug)]
struct SecretSourceError;

impl Display for SecretSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&secret_payload())
    }
}

impl std::error::Error for SecretSourceError {}
