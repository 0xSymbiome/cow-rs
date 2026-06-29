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

// The success envelope is version-tagged (`schemaVersion: SchemaVersion`); thrown
// errors are not, but the error discriminants stay exposed by `kind`.
#[test]
fn generated_type_declarations_version_the_envelope_and_expose_error_kinds() {
    let expected = [
        "export type SchemaVersion = \"v1\" | \"__unknown\";",
        "export interface WasmEnvelope<T>",
        "schemaVersion: SchemaVersion;",
        "kind: \"walletTimeout\"",
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
                    "export interface GetTradesRequest",
                    "export interface Order",
                    "export interface Trade",
                    "export interface NativePriceResponse",
                    "export interface AppDataObject",
                    "export interface CompetitionOrderStatus",
                    "export interface TotalSurplus",
                    "export interface SolverCompetitionResponse",
                    "export interface CompetitionAuction",
                    "export interface SolverSettlement",
                    "export interface SolverCompetitionOrder",
                    "getNativePrice(token: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<NativePriceResponse>>",
                    "getOrder(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<Order>>",
                    "getOrders(owner: string, pagination?: PaginationOptions | null, options?: SdkClientOptions | null): Promise<WasmEnvelope<Order[]>>",
                    "getTrades(query: GetTradesRequest, options?: SdkClientOptions | null): Promise<WasmEnvelope<Trade[]>>",
                    "getOrderMultiEnv(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<Order>>",
                    "getTxOrders(txHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<Order[]>>",
                    "getVersion(options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>",
                    "getOrderLink(orderUid: string): WasmEnvelope<string>",
                    "getOrderCompetitionStatus(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<CompetitionOrderStatus>>",
                    "getTotalSurplus(owner: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<TotalSurplus>>",
                    "getSolverCompetition(auctionId: number, options?: SdkClientOptions | null): Promise<WasmEnvelope<SolverCompetitionResponse>>",
                    "getSolverCompetitionByTxHash(txHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<SolverCompetitionResponse>>",
                    "getAppData(appDataHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataObject>>",
                    "uploadAppData(appDataHash: string, fullAppData: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<{ uploaded: true }>>",
                    "getQuote(request: OrderQuoteRequest, options?: SdkClientOptions | null): Promise<WasmEnvelope<OrderQuoteResponse>>",
                    "sendOrder(signed: SignedOrder, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>",
                    "sendOrderCreation(input: OrderCreation, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>",
                    "cancelOrders(signed: SignedCancellations, options?: SdkClientOptions | null): Promise<WasmEnvelope<{ cancelled: true }>>",
                ],
            );
        }

        if snapshot.has_feature("subgraph") {
            assert_contains_all(
                &snapshot.name,
                &content,
                &[
                    "export class SubgraphClient",
                    "getTotals(options?: SdkClientOptions | null): Promise<any>",
                    "getLastDaysVolume(days: number, options?: SdkClientOptions | null): Promise<any>",
                    "getLastHoursVolume(hours: number, options?: SdkClientOptions | null): Promise<any>",
                    "runQuery(query: string, variables: Value, operationName?: string | null, options?: SdkClientOptions | null): Promise<any>",
                ],
            );
        }

        if snapshot.has_feature("cancellation") {
            assert_contains_all(
                &snapshot.name,
                &content,
                &[
                    "export interface OrderTraderParams",
                    "export interface TransactionRequest",
                    "buildCancelOrderTx(params: OrderTraderParams): WasmEnvelope<TransactionRequest>",
                    "buildPresignTx(params: OrderTraderParams): WasmEnvelope<TransactionRequest>",
                ],
            );
        }

        if snapshot.has_feature("trading") {
            assert_contains_all(
                &snapshot.name,
                &content,
                &[
                    "export type ContractReadCallback",
                    "export interface AllowanceParams",
                    "export interface ApprovalParams",
                    "export interface BuiltSellNativeCurrencyTx",
                    "export interface ContractCall",
                    "export interface LimitTradeParams",
                    // The quote-result tree is generated from the native types:
                    // `QuoteResults` plus the generic
                    // amounts/costs tree and the typed-data envelope it embeds.
                    "export interface QuoteResults",
                    "export interface TradeParams",
                    "export interface QuoteAmountsAndCosts<T>",
                    "export interface Amounts<T>",
                    "export interface TypedDataEnvelope<M>",
                    "export type CowEnv = \"prod\" | \"staging\";",
                    "buildApprovalTx(params: ApprovalParams, options?: SdkClientOptions | null): Promise<WasmEnvelope<TransactionRequest>>",
                    "buildSellNativeCurrencyTx(order: OrderData, quoteId: number, from: string",
                    "buildSellNativeCurrencyTxFromQuote(quoteResults: QuoteResults, from: string",
                    "getCowProtocolAllowance(params: AllowanceParams, readContractCallback: ContractReadCallback",
                    "getQuote(params: TradeParams, options?: SdkClientOptions | null): Promise<WasmEnvelope<QuoteResults>>",
                    "postLimitOrder(params: LimitTradeParams, owner: string, signerCallback: TypedDataSignerCallback",
                    "postSwapOrderFromQuote(quoteResults: QuoteResults, owner: string, signerCallback: TypedDataSignerCallback",
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
    // The typed-data envelope is generic over its message body (`message: M`),
    // so the `Value` escape hatch reaches the message only through the concrete
    // `TypedDataEnvelope<Value>` binding; `raw` carries it on the unknown-error arm.
    let expected = ["message: M;", "TypedDataEnvelope<Value>", "raw: Value"];
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
                content.contains("variables: Value"),
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

#[test]
fn generated_type_declarations_expose_event_log_decoders() {
    // The provider-free settlement and eth-flow event-log decoders are
    // always-compiled deterministic helpers, present on every flavour.
    let expected = [
        "decodeSettlementLog(log: EventLog): WasmEnvelope<SettlementEvent>",
        "decodeEthFlowLog(log: EventLog): WasmEnvelope<EthFlowEvent>",
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
        .map(|flavour| {
            let name = flavour["name"].as_str().expect("flavour name").to_owned();
            let features = flavour["features"]
                .as_array()
                .expect("flavour features")
                .iter()
                .map(|feature| feature.as_str().expect("feature name").to_owned())
                .collect::<Vec<_>>();
            // The raw type declarations are loader-independent: wasm-bindgen emits a
            // byte-identical `.d.ts` for every wasm-pack target of a flavour, so one
            // snapshot per flavour pins the public type contract. The wasm workflow
            // asserts every target's generated declaration matches this snapshot.
            Snapshot {
                name: format!("{name}.d.ts"),
                features,
            }
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
