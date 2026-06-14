#![cfg(not(target_arch = "wasm32"))]

use std::{fs, path::PathBuf};

use serde_json::Value;

#[derive(Debug, Clone)]
struct Snapshot {
    name: String,
    features: Vec<String>,
}

impl Snapshot {
    fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|candidate| candidate == feature)
    }
}

#[test]
fn generated_type_declarations_match_flavour_matrix() {
    for snapshot in snapshots() {
        let path = snapshot_path(&snapshot.name);
        assert!(path.exists(), "{} must exist", snapshot.name);
    }
}

#[test]
fn generated_type_declarations_hide_callback_registry() {
    let forbidden = [
        concat!("FetchCallback", "Handle"),
        concat!("register", "FetchCallback"),
        concat!("from", "Handle"),
        concat!("With", "Fetch"),
        concat!("HttpTo", "IpfsAdapter"),
    ];

    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        for token in forbidden {
            assert!(
                !content.contains(token),
                "{} must not expose `{token}`",
                snapshot.name
            );
        }
    }
}

#[test]
fn generated_type_declarations_use_camel_case_public_params() {
    let forbidden = [
        "app_data_hex:",
        "chain_id:",
        "custom_callback:",
        "digest_signer:",
        "ecdsa_signature:",
        "order_uid:",
        "order_uids:",
        "request_callback:",
        "signer_callback:",
        "typed_data_signer:",
    ];

    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        for token in forbidden {
            assert!(
                !content.contains(token),
                "{} must not expose `{token}`",
                snapshot.name
            );
        }
    }
}

#[test]
fn generated_type_declarations_name_callback_params() {
    let expected = [
        "digestSigner: DigestSignerCallback",
        "requestCallback: Eip1193RequestCallback",
        "typedDataSigner: TypedDataSignerCallback",
        "customCallback: CustomEip1271Callback",
    ];

    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        for token in expected {
            assert!(
                content.contains(token),
                "{} must expose `{token}`",
                snapshot.name
            );
        }
        if snapshot.has_feature("trading") {
            assert!(
                content.contains("signerCallback: TypedDataSignerCallback"),
                "{} must expose trading signer callbacks",
                snapshot.name
            );
        }
    }
}

#[test]
fn generated_type_declarations_version_errors_and_outputs() {
    let expected = [
        "export type SchemaVersion = \"v1\" | \"__unknown\";",
        "export interface WasmEnvelope<T>",
        "schemaVersion: SchemaVersion;",
        "kind: \"walletTimeout\"",
        "kind: \"forbiddenInteraction\"",
        "kind: \"__unknown\"",
    ];
    let forbidden = ["Promise<Promise<"];

    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        for token in expected {
            assert!(
                content.contains(token),
                "{} must expose `{token}`",
                snapshot.name
            );
        }
        for token in forbidden {
            assert!(
                !content.contains(token),
                "{} must not expose `{token}`",
                snapshot.name
            );
        }
    }
}

#[test]
fn generated_type_declarations_expose_abort_and_wallet_options() {
    let expected = [
        "export interface SdkClientOptions",
        "timeoutMs?: number;",
        "signal?: AbortSignal;",
        "export interface WalletConfig",
        "export interface SigningOptions extends SdkClientOptions",
        "walletConfig?: WalletConfig;",
        "options?: SigningOptions",
    ];

    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        for token in expected {
            assert!(
                content.contains(token),
                "{} must expose `{token}`",
                snapshot.name
            );
        }
        if snapshot.has_feature("orderbook")
            || snapshot.has_feature("subgraph")
            || snapshot.has_feature("ipfs")
            || snapshot.has_feature("trading")
        {
            assert!(
                content.contains("options?: SdkClientOptions"),
                "{} must expose client options",
                snapshot.name
            );
        }
    }
}

#[test]
fn generated_type_declarations_expose_transport_policy_config_for_http_flavours() {
    let expected = [
        "export interface TransportPolicyConfig",
        "retryPolicy?: RetryPolicyConfig",
        "requestRateLimiter?: RequestRateLimiterConfig",
        "jitterStrategy?: JitterStrategyConfig",
        "userAgent?: string",
        "transportPolicy?: TransportPolicyConfig",
    ];

    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        if snapshot.has_feature("transport-policy") {
            for token in expected {
                assert!(
                    content.contains(token),
                    "{} must expose `{token}`",
                    snapshot.name
                );
            }
        } else {
            assert!(
                !content.contains("export interface TransportPolicyConfig"),
                "{} must not expose transport policy config",
                snapshot.name
            );
        }
    }
}

#[test]
fn generated_type_declarations_expose_feature_scoped_workflow_helpers() {
    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);

        if snapshot.has_feature("orderbook") {
            assert_contains_all(
                &snapshot.name,
                &content,
                &[
                    "apiKey?: string | null;",
                    "export interface PaginationOptions",
                    "export interface TradesQueryInput",
                    "export interface OrderDto",
                    "export interface TradeDto",
                    "export interface NativePriceResponseDto",
                    "export interface AppDataObjectDto",
                    "getNativePrice(token: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<NativePriceResponseDto>>",
                    "getOrder(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<OrderDto>>",
                    "getOrders(owner: string, pagination?: PaginationOptions | null, options?: SdkClientOptions | null): Promise<WasmEnvelope<OrderDto[]>>",
                    "getTrades(query: TradesQueryInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<TradeDto[]>>",
                    "getAppData(appDataHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataObjectDto>>",
                    "uploadAppData(appDataHash: string, fullAppData: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<{ uploaded: true }>>",
                ],
            );
        }

        if snapshot.has_feature("cancellation") {
            assert_contains_all(
                &snapshot.name,
                &content,
                &[
                    "export interface OrderTraderParametersInput",
                    "export interface TransactionRequestDto",
                    "buildCancelOrderTx(params: OrderTraderParametersInput): WasmEnvelope<TransactionRequestDto>",
                    "buildPresignTx(params: OrderTraderParametersInput): WasmEnvelope<TransactionRequestDto>",
                ],
            );
        }

        if snapshot.has_feature("trading") {
            assert_contains_all(
                &snapshot.name,
                &content,
                &[
                    "export type ContractReadCallback",
                    "export interface AllowanceParametersInput",
                    "export interface BuiltSellNativeCurrencyTxDto",
                    "export interface ContractCallDto",
                    "export interface LimitTradeParametersInput",
                    "export interface QuoteResultsDto",
                    "buildSellNativeCurrencyTx(order: OrderInput, quoteId: number, from: string",
                    "getCowProtocolAllowance(params: AllowanceParametersInput, readContractCallback: ContractReadCallback",
                    "getQuote(params: SwapParametersInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<QuoteResultsDto>>",
                    "postLimitOrder(params: LimitTradeParametersInput, owner: string, signerCallback: TypedDataSignerCallback",
                    "postSwapOrderFromQuote(quoteResults: QuoteResultsDto, owner: string, signerCallback: TypedDataSignerCallback",
                ],
            );
        } else {
            assert!(
                !content.contains("export class TradingClient"),
                "{} must not expose TradingClient",
                snapshot.name
            );
        }
    }
}

#[test]
fn generated_type_declarations_keep_unknown_escape_hatch_scoped() {
    let expected = ["message: Value;", "params?: Value[];", "raw: Value"];
    let forbidden = [
        "input: Value",
        "request: Value",
        "params: Value",
        "signed: Value",
    ];

    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        for token in expected {
            assert!(
                content.contains(token),
                "{} must expose `{token}`",
                snapshot.name
            );
        }
        if snapshot.has_feature("subgraph") {
            assert!(
                content.contains("variables?: Value;"),
                "{} must expose subgraph variables",
                snapshot.name
            );
        }
        for token in forbidden {
            assert!(
                !content.contains(token),
                "{} must not expose primary input as `{token}`",
                snapshot.name
            );
        }
    }
}

#[test]
fn generated_type_declarations_keep_feature_scoped_client_classes() {
    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        assert_feature_class(&snapshot, &content, "ipfs", "IpfsClient");
        assert_feature_class(&snapshot, &content, "orderbook", "OrderBookClient");
        assert_feature_class(&snapshot, &content, "subgraph", "SubgraphClient");
        assert_feature_class(&snapshot, &content, "trading", "TradingClient");
    }
}

fn assert_feature_class(snapshot: &Snapshot, content: &str, feature: &str, class_name: &str) {
    let token = format!("export class {class_name}");
    if snapshot.has_feature(feature) {
        assert!(
            content.contains(&token),
            "{} must expose `{token}`",
            snapshot.name
        );
    } else {
        assert!(
            !content.contains(&token),
            "{} must not expose `{token}`",
            snapshot.name
        );
    }
}

fn assert_contains_all(snapshot: &str, content: &str, expected: &[&str]) {
    for token in expected {
        assert!(content.contains(token), "{snapshot} must expose `{token}`");
    }
}

fn snapshots() -> Vec<Snapshot> {
    let descriptor_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("npm")
        .join("flavours.json");
    let descriptor: Value = serde_json::from_str(
        &fs::read_to_string(descriptor_path).expect("flavours.json must be readable"),
    )
    .expect("flavours.json must be valid JSON");
    descriptor["flavours"]
        .as_array()
        .expect("flavours must be an array")
        .iter()
        .flat_map(|flavour| {
            let name = flavour["name"].as_str().expect("flavour name").to_owned();
            let features = flavour["features"]
                .as_array()
                .expect("flavour features")
                .iter()
                .map(|feature| feature.as_str().expect("feature name").to_owned())
                .collect::<Vec<_>>();
            flavour["targets"]
                .as_array()
                .expect("flavour targets")
                .iter()
                .map(move |target| Snapshot {
                    name: format!("{}-{}.d.ts", name, target.as_str().expect("target name")),
                    features: features.clone(),
                })
        })
        .collect()
}

fn read_snapshot(name: &str) -> String {
    fs::read_to_string(snapshot_path(name)).expect("snapshot must be readable")
}

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("snapshots")
        .join("raw")
        .join(name)
}
