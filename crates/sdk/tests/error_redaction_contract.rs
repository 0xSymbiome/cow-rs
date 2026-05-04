use std::fmt::{Debug, Display};

use cow_sdk::{
    SdkError,
    app_data::{AppDataError, SchemaVersion, ValidationResult},
    contracts::ContractsError,
    core::{
        Address, Amount, CoreError, CowEnv, HostPolicyError, TransportError, TransportErrorClass,
        UrlParseFailureClass, ValidationError, ValidationReason,
    },
    orderbook::{
        OrderBookApiError, OrderbookError, OrderbookRejection, ResponseBody, SigningScheme,
    },
    signing::SigningError,
    trading::{AppCodeError, ClientRejection, OrderbookContextValue, TradingError},
};
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

#[test]
fn orderbook_errors_redact_api_transport_and_source_payloads() {
    let api_error =
        OrderBookApiError::new(500, secret_payload(), ResponseBody::Text(secret_payload()));
    assert_render("OrderBookApiError", &api_error);

    let rejected_api_error = OrderBookApiError::new(
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
        OrderbookError::Api(Box::new(OrderBookApiError::new(
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
        OrderbookError::Serialization(json_error()),
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
        AppDataError::UnknownSchemaVersion(
            SchemaVersion::new("99.99.99").expect("fixture is a valid semver"),
        ),
        AppDataError::MissingSchemaVersion,
        AppDataError::Json(json_error()),
        AppDataError::Schema {
            message: secret_payload().into(),
            source: Box::new(schema_error_source()),
        },
        AppDataError::InvalidAppDataProvided {
            field: "document",
            reason: ValidationReason::BadShape {
                details: "schema validation failed",
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
        AppDataError::MissingIpfsCredentials,
        AppDataError::Pinning {
            status: Some(401),
            message: secret_payload().into(),
        },
        AppDataError::TooLarge {
            actual_bytes: 4_097,
            max_bytes: 4_096,
        },
    ];

    assert_all_render("AppDataError", &errors);
    assert_all_serialize("AppDataError", &errors);

    let validation_result = ValidationResult::new(false, Some(secret_payload()));
    assert_debug_render("ValidationResult", &validation_result);
    assert_serialize("ValidationResult", &validation_result);
}

#[test]
fn contracts_and_signing_errors_redact_secret_bearing_messages() {
    let contracts_errors = [
        ContractsError::Core(CoreError::Serialization(secret_payload().into())),
        ContractsError::Cancelled,
        ContractsError::UnsupportedChain(999_999),
        ContractsError::InvalidOrderUidLength { actual: 4 },
        ContractsError::InvalidNumeric {
            field: "sellAmount",
            value: secret_payload().into(),
        },
        ContractsError::NumericOverflow {
            field: "sellAmount",
            value: secret_payload().into(),
        },
        ContractsError::InvalidFlags(0b1000_0000),
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
        ContractsError::MissingClearingPrice {
            token: address("0x2222222222222222222222222222222222222222"),
        },
        ContractsError::MissingExecutedAmount,
        ContractsError::MissingTrade,
        ContractsError::ZeroReceiver,
        ContractsError::InvalidTokenIndex {
            index: 4,
            registered: 2,
        },
        ContractsError::ForbiddenInteractionTarget {
            target: address("0x3333333333333333333333333333333333333333"),
        },
        ContractsError::Provider {
            operation: "eth_call",
            message: secret_payload().into(),
        },
        ContractsError::Abi(alloy_sol_types::Error::Overrun),
        ContractsError::DecodeHex {
            field: "signature",
            source: hex::decode("zz").unwrap_err(),
        },
        ContractsError::InvalidHexPrefix { field: "signature" },
        ContractsError::InvalidDecodedLength {
            field: "signature",
            expected: 65,
            actual: 64,
        },
        ContractsError::Serialization(json_error()),
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
            operation: "sign_typed_data",
            message: secret_payload().into(),
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
        TradingError::MissingQuoterParameters("chainId"),
        TradingError::MissingTraderParameters("appCode"),
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
        TradingError::MissingInjectedOrderbookClient,
    ];

    assert_all_render("TradingError", &errors);
}

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

#[test]
fn sdk_error_facade_redacts_nested_public_errors() {
    let errors = [
        SdkError::Types(CoreError::Serialization(secret_payload().into())),
        SdkError::Signing(SigningError::Signer {
            operation: "sign_order",
            message: secret_payload().into(),
        }),
        SdkError::AppData(AppDataError::Pinning {
            status: Some(401),
            message: secret_payload().into(),
        }),
        SdkError::Contracts(ContractsError::Provider {
            operation: "eth_call",
            message: secret_payload().into(),
        }),
        SdkError::Orderbook(OrderbookError::Transport {
            class: TransportErrorClass::Other,
            detail: secret_payload().into(),
        }),
        SdkError::Trading(TradingError::Signer {
            operation: "sign_order",
            message: secret_payload().into(),
        }),
    ];

    assert_all_render("SdkError", &errors);
}

#[cfg(feature = "browser-wallet")]
#[test]
fn browser_wallet_errors_and_rpc_payloads_redact_method_message_and_data() {
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
            method: secret_payload().into(),
            code: 4001,
            message: secret_payload().into(),
        },
        BrowserWalletError::Disconnected {
            method: secret_payload().into(),
            code: 4900,
            message: secret_payload().into(),
        },
        BrowserWalletError::WrongChain {
            method: secret_payload().into(),
            code: 4901,
            message: secret_payload().into(),
        },
        BrowserWalletError::ChainNotAdded {
            chain_id: 8453,
            method: secret_payload().into(),
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
            method: secret_payload().into(),
            message: secret_payload().into(),
        },
        BrowserWalletError::MalformedResponse {
            method: secret_payload().into(),
            message: secret_payload().into(),
        },
        BrowserWalletError::Rpc {
            method: secret_payload().into(),
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
    for secret in [URL_SECRET, AUTH_SECRET, PRIVATE_KEY_SECRET, PEM_SECRET] {
        assert!(
            !rendered.contains(secret),
            "{label} {channel} leaked secret substring {secret:?} in {rendered:?}"
        );
    }
}

fn secret_payload() -> String {
    format!("{URL_SECRET}\n{AUTH_SECRET}\n{PRIVATE_KEY_SECRET}\n{PEM_SECRET}")
}

fn json_error() -> serde_json::Error {
    serde_json::from_str::<Value>("{ malformed").unwrap_err()
}

fn schema_error_source() -> jsonschema::ValidationError<'static> {
    let schema = json!({"type": "object", "required": ["safe"]});
    let candidate = json!({});
    let validator = jsonschema::validator_for(&schema).expect("schema fixture must compile");
    validator
        .iter_errors(&candidate)
        .next()
        .expect("missing required property must surface a validation error")
        .to_owned()
}

fn address(value: &str) -> Address {
    Address::new(value).expect("address fixture must parse")
}

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
