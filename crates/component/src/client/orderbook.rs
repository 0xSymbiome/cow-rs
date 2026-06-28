use std::sync::Arc;

use cow_sdk_core::{Address, CowEnv, ErrorClass, HttpTransport, SupportedChainId};
use cow_sdk_orderbook::{
    AppDataHash, OrderCancellations, OrderCreation, OrderQuoteRequest, OrderUid, OrderbookApi,
    OrderbookError, OrdersQuery, TradesQuery, TransactionHash,
};

/// A read failure reduced to the boundary surface.
pub struct ReadError {
    pub class: ErrorClass,
    pub message: String,
    pub retryable: bool,
    pub retry_after_ms: Option<u32>,
    /// The services `errorType` wire tag for a recognised rejection
    /// (`"InsufficientBalance"`, ...), projected from the same native source the
    /// wasm-bindgen lane reads; `None` when the failure is not a recognised
    /// rejection or the tag could not be sanitized.
    pub error_type: Option<String>,
}

pub const fn invalid(message: String) -> ReadError {
    ReadError {
        class: ErrorClass::Validation,
        message,
        retryable: false,
        retry_after_ms: None,
        error_type: None,
    }
}

/// Reads the fine-grained services `errorType` tag from an [`OrderbookError`],
/// present only for the `Rejected` variant that carries a typed
/// `OrderbookRejection`.
///
/// The projection itself is single-sourced on
/// `cow_sdk_orderbook::OrderbookRejection::error_type_tag`, the same native
/// accessor the wasm-bindgen lane reads, so both wasm distribution lanes
/// surface the identical tag with no re-ported mapping (the redacted free-form
/// `description` never crosses, and an unsanitizable code becomes an absent
/// tag rather than the redaction sentinel — ADR 0053).
fn orderbook_error_type(error: &OrderbookError) -> Option<String> {
    match error {
        OrderbookError::Rejected { rejection, .. } => rejection.error_type_tag(),
        _ => None,
    }
}

fn from_orderbook(error: &OrderbookError) -> ReadError {
    ReadError {
        class: error.class(),
        retryable: error.is_retryable(),
        retry_after_ms: error
            .backoff_hint()
            .map(|delay| u32::try_from(delay.as_millis()).unwrap_or(u32::MAX)),
        error_type: orderbook_error_type(error),
        message: error.to_string(),
    }
}

/// Maps a `TradingError` to the boundary surface, preserving its class and
/// retry hint — so the signing/posting lane surfaces typed errors exactly
/// like the reads do (`TradingError::class` / `is_retryable` / `backoff_hint`).
/// When the trading failure wraps an orderbook rejection, the fine-grained
/// `errorType` tag is threaded through as well, matching the read lane and the
/// wasm-bindgen surface.
pub fn from_trading(error: &cow_sdk_trading::TradingError) -> ReadError {
    ReadError {
        class: error.class(),
        retryable: error.is_retryable(),
        retry_after_ms: error
            .backoff_hint()
            .map(|delay| u32::try_from(delay.as_millis()).unwrap_or(u32::MAX)),
        error_type: match error {
            cow_sdk_trading::TradingError::Orderbook(orderbook) => orderbook_error_type(orderbook),
            _ => None,
        },
        message: error.to_string(),
    }
}

fn parse_env(env: Option<&str>) -> Result<CowEnv, ReadError> {
    match env.unwrap_or("prod") {
        "prod" | "production" => Ok(CowEnv::Prod),
        "staging" | "barn" => Ok(CowEnv::Staging),
        other => Err(invalid(format!("unknown environment: {other}"))),
    }
}

fn api<T>(transport: T, chain_id: u64, env: Option<&str>) -> Result<OrderbookApi, ReadError>
where
    T: HttpTransport + Send + Sync + 'static,
{
    let chain = SupportedChainId::try_from(chain_id).map_err(|error| invalid(error.to_string()))?;
    OrderbookApi::builder()
        .chain(chain)
        .env(parse_env(env)?)
        .transport(Arc::new(transport))
        .build()
        .map_err(|error| invalid(error.to_string()))
}

fn uid(value: &str) -> Result<OrderUid, ReadError> {
    OrderUid::new(value).map_err(|error| invalid(error.to_string()))
}

fn address(value: &str) -> Result<Address, ReadError> {
    Address::new(value).map_err(|error| invalid(error.to_string()))
}

fn tx_hash(value: &str) -> Result<TransactionHash, ReadError> {
    TransactionHash::new(value).map_err(|error| invalid(error.to_string()))
}

fn json<T: serde::Serialize>(value: &T) -> Result<String, ReadError> {
    serde_json::to_string(value).map_err(|error| invalid(error.to_string()))
}

// === Typed orderbook response lowering ======================================
//
// The orderbook reads above return the SDK's canonical JSON `String`. The
// `get-order` / `get-orders` / `get-tx-orders` / `get-trades-*` exports lower
// that JSON into the typed `book.order` / `book.trade` WIT records. The field
// extraction is world-agnostic (these helpers over a `serde_json::Value`); the
// record construction is a `macro_rules!` so it expands with each world's
// generated record + enum types in scope — one definition, both lanes. The
// `component_wit_record_drift` test pins these field-sets to the native serde
// shape, so an upstream field addition fails CI rather than silently dropping.

use serde_json::Value;

/// Parses an orderbook JSON `String` into a `Value` for record lowering.
pub(crate) fn value(json: &str) -> Result<Value, ReadError> {
    serde_json::from_str(json).map_err(|error| invalid(error.to_string()))
}

pub(crate) fn jstr(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}
pub(crate) fn jostr(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_owned)
}
pub(crate) fn ju32(v: &Value, key: &str) -> u32 {
    u32::try_from(v.get(key).and_then(Value::as_u64).unwrap_or(0)).unwrap_or(u32::MAX)
}
pub(crate) fn ju64(v: &Value, key: &str) -> u64 {
    v.get(key).and_then(Value::as_u64).unwrap_or(0)
}
pub(crate) fn jbool(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}
pub(crate) fn jobool(v: &Value, key: &str) -> Option<bool> {
    v.get(key).and_then(Value::as_bool)
}
pub(crate) fn jos64(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(Value::as_i64)
}
/// An arbitrary-JSON field (`metadata` / `policy`) rendered back to a JSON string.
pub(crate) fn jojson(v: &Value, key: &str) -> Option<String> {
    v.get(key).filter(|x| !x.is_null()).map(Value::to_string)
}

/// Lowers an orderbook order `Value` into the world's generated `book.order`.
/// Requires `Order`, `OrderKind`, `OrderStatus`, `OrderClass`, `SigningScheme`,
/// `SellTokenSource`, `BuyTokenDestination`, `EthflowData`, `StoredOrderQuote`,
/// `OnchainOrderData`, `OrderInteractions`, `InteractionData` in scope.
macro_rules! to_wit_order {
    ($value:expr) => {{
        let v: &::serde_json::Value = $value;
        let interaction = |x: &::serde_json::Value| InteractionData {
            target: $crate::client::orderbook::jstr(x, "target"),
            value: $crate::client::orderbook::jstr(x, "value"),
            call_data: $crate::client::orderbook::jstr(x, "callData"),
        };
        let interaction_list = |i: &::serde_json::Value, key: &str| {
            i.get(key)
                .and_then(::serde_json::Value::as_array)
                .map(|a| a.iter().map(interaction).collect())
        };
        Order {
            sell_token: $crate::client::orderbook::jstr(v, "sellToken"),
            buy_token: $crate::client::orderbook::jstr(v, "buyToken"),
            receiver: $crate::client::orderbook::jostr(v, "receiver"),
            sell_amount: $crate::client::orderbook::jstr(v, "sellAmount"),
            buy_amount: $crate::client::orderbook::jstr(v, "buyAmount"),
            valid_to: $crate::client::orderbook::ju32(v, "validTo"),
            app_data: $crate::client::orderbook::jstr(v, "appData"),
            app_data_hash: $crate::client::orderbook::jostr(v, "appDataHash"),
            fee_amount: $crate::client::orderbook::jstr(v, "feeAmount"),
            full_balance_check: $crate::client::orderbook::jobool(v, "fullBalanceCheck"),
            kind: if $crate::client::orderbook::jstr(v, "kind") == "buy" {
                OrderKind::Buy
            } else {
                OrderKind::Sell
            },
            partially_fillable: $crate::client::orderbook::jbool(v, "partiallyFillable"),
            sell_token_balance: v
                .get("sellTokenBalance")
                .and_then(::serde_json::Value::as_str)
                .map(|s| match s {
                    "external" => SellTokenSource::External,
                    "internal" => SellTokenSource::Internal,
                    _ => SellTokenSource::Erc20,
                }),
            buy_token_balance: v
                .get("buyTokenBalance")
                .and_then(::serde_json::Value::as_str)
                .map(|s| match s {
                    "internal" => BuyTokenDestination::Internal,
                    _ => BuyTokenDestination::Erc20,
                }),
            signing_scheme: match $crate::client::orderbook::jstr(v, "signingScheme").as_str() {
                "ethsign" => SigningScheme::Ethsign,
                "eip1271" => SigningScheme::Eip1271,
                "presign" => SigningScheme::Presign,
                _ => SigningScheme::Eip712,
            },
            signature: $crate::client::orderbook::jstr(v, "signature"),
            from: $crate::client::orderbook::jostr(v, "from"),
            quote_id: $crate::client::orderbook::jos64(v, "quoteId"),
            class: match $crate::client::orderbook::jstr(v, "class").as_str() {
                "limit" => OrderClass::Limit,
                "liquidity" => OrderClass::Liquidity,
                _ => OrderClass::Market,
            },
            owner: $crate::client::orderbook::jstr(v, "owner"),
            uid: $crate::client::orderbook::jstr(v, "uid"),
            creation_date: $crate::client::orderbook::jstr(v, "creationDate"),
            executed_sell_amount: $crate::client::orderbook::jstr(v, "executedSellAmount"),
            executed_sell_amount_before_fees: $crate::client::orderbook::jstr(
                v,
                "executedSellAmountBeforeFees",
            ),
            executed_buy_amount: $crate::client::orderbook::jstr(v, "executedBuyAmount"),
            executed_fee: $crate::client::orderbook::jostr(v, "executedFee"),
            executed_fee_amount: $crate::client::orderbook::jostr(v, "executedFeeAmount"),
            executed_fee_token: $crate::client::orderbook::jostr(v, "executedFeeToken"),
            invalidated: $crate::client::orderbook::jobool(v, "invalidated"),
            status: match $crate::client::orderbook::jstr(v, "status").as_str() {
                "presignaturePending" => OrderStatus::PresignaturePending,
                "fulfilled" => OrderStatus::Fulfilled,
                "cancelled" => OrderStatus::Cancelled,
                "expired" => OrderStatus::Expired,
                _ => OrderStatus::Open,
            },
            is_liquidity_order: $crate::client::orderbook::jobool(v, "isLiquidityOrder"),
            onchain_user: $crate::client::orderbook::jostr(v, "onchainUser"),
            ethflow_data: v
                .get("ethflowData")
                .filter(|x| !x.is_null())
                .map(|e| EthflowData {
                    refund_tx_hash: $crate::client::orderbook::jostr(e, "refundTxHash"),
                    user_valid_to: $crate::client::orderbook::ju32(e, "userValidTo"),
                }),
            onchain_order_data: v.get("onchainOrderData").filter(|x| !x.is_null()).map(|o| {
                OnchainOrderData {
                    sender: $crate::client::orderbook::jstr(o, "sender"),
                    placement_error: $crate::client::orderbook::jostr(o, "placementError"),
                }
            }),
            full_app_data: $crate::client::orderbook::jostr(v, "fullAppData"),
            settlement_contract: $crate::client::orderbook::jstr(v, "settlementContract"),
            quote: v
                .get("quote")
                .filter(|x| !x.is_null())
                .map(|q| StoredOrderQuote {
                    gas_amount: $crate::client::orderbook::jstr(q, "gasAmount"),
                    gas_price: $crate::client::orderbook::jstr(q, "gasPrice"),
                    sell_token_price: $crate::client::orderbook::jstr(q, "sellTokenPrice"),
                    sell_amount: $crate::client::orderbook::jstr(q, "sellAmount"),
                    buy_amount: $crate::client::orderbook::jstr(q, "buyAmount"),
                    fee_amount: $crate::client::orderbook::jstr(q, "feeAmount"),
                    solver: $crate::client::orderbook::jstr(q, "solver"),
                    verified: $crate::client::orderbook::jbool(q, "verified"),
                    metadata: $crate::client::orderbook::jojson(q, "metadata"),
                }),
            interactions: v.get("interactions").filter(|x| !x.is_null()).map(|i| {
                OrderInteractions {
                    pre: interaction_list(i, "pre"),
                    post: interaction_list(i, "post"),
                }
            }),
            total_fee: $crate::client::orderbook::jostr(v, "totalFee"),
        }
    }};
}
pub(crate) use to_wit_order;

/// Lowers a trade `Value` into the world's generated `book.trade`. Requires
/// `Trade` and `ExecutedProtocolFee` in scope.
macro_rules! to_wit_trade {
    ($value:expr) => {{
        let v: &::serde_json::Value = $value;
        Trade {
            block_number: $crate::client::orderbook::ju64(v, "blockNumber"),
            log_index: $crate::client::orderbook::ju64(v, "logIndex"),
            order_uid: $crate::client::orderbook::jstr(v, "orderUid"),
            owner: $crate::client::orderbook::jstr(v, "owner"),
            sell_token: $crate::client::orderbook::jstr(v, "sellToken"),
            buy_token: $crate::client::orderbook::jstr(v, "buyToken"),
            sell_amount: $crate::client::orderbook::jstr(v, "sellAmount"),
            sell_amount_before_fees: $crate::client::orderbook::jstr(v, "sellAmountBeforeFees"),
            buy_amount: $crate::client::orderbook::jstr(v, "buyAmount"),
            executed_protocol_fees: v
                .get("executedProtocolFees")
                .and_then(::serde_json::Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|f| ExecutedProtocolFee {
                            policy: $crate::client::orderbook::jojson(f, "policy"),
                            amount: $crate::client::orderbook::jostr(f, "amount"),
                            token: $crate::client::orderbook::jostr(f, "token"),
                        })
                        .collect()
                }),
            tx_hash: $crate::client::orderbook::jstr(v, "txHash"),
        }
    }};
}
pub(crate) use to_wit_trade;

/// Materializes an `OrdersQuery` / `TradesQuery` pagination override.
const fn paginate(
    offset: Option<u32>,
    limit: Option<u32>,
    query_offset: &mut u32,
    query_limit: &mut u32,
) {
    if let Some(offset) = offset {
        *query_offset = offset;
    }
    if let Some(limit) = limit {
        *query_limit = limit;
    }
}

pub async fn order<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    order_uid: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let order = api(transport, chain_id, env)?
        .order(&uid(order_uid)?)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&order)
}

pub async fn orders<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    owner: &str,
    offset: Option<u32>,
    limit: Option<u32>,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let mut query = OrdersQuery::new(address(owner)?);
    paginate(offset, limit, &mut query.offset, &mut query.limit);
    let orders = api(transport, chain_id, env)?
        .orders(&query)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&orders)
}

pub async fn tx_orders<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    transaction: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let orders = api(transport, chain_id, env)?
        .tx_orders(&tx_hash(transaction)?)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&orders)
}

pub async fn trades_by_owner<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    owner: &str,
    offset: Option<u32>,
    limit: Option<u32>,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let mut query = TradesQuery::by_owner(address(owner)?);
    paginate(offset, limit, &mut query.offset, &mut query.limit);
    let trades = api(transport, chain_id, env)?
        .trades(&query)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&trades)
}

pub async fn trades_by_order<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    order_uid: &str,
    offset: Option<u32>,
    limit: Option<u32>,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let mut query = TradesQuery::by_order_uid(uid(order_uid)?);
    paginate(offset, limit, &mut query.offset, &mut query.limit);
    let trades = api(transport, chain_id, env)?
        .trades(&query)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&trades)
}

pub async fn quote<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    request_json: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let request: OrderQuoteRequest =
        serde_json::from_str(request_json).map_err(|error| invalid(error.to_string()))?;
    let response = api(transport, chain_id, env)?
        .quote(&request)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&response)
}

pub async fn native_price<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    token: &str,
    env: Option<&str>,
) -> Result<f64, ReadError> {
    let response = api(transport, chain_id, env)?
        .native_price(&address(token)?)
        .await
        .map_err(|error| from_orderbook(&error))?;
    Ok(response.price)
}

pub async fn total_surplus<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    owner: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let surplus = api(transport, chain_id, env)?
        .total_surplus(&address(owner)?)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&surplus)
}

pub async fn order_competition_status<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    order_uid: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let status = api(transport, chain_id, env)?
        .order_competition_status(&uid(order_uid)?)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&status)
}

pub async fn solver_competition<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    auction_id: i64,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let competition = api(transport, chain_id, env)?
        .solver_competition(auction_id)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&competition)
}

pub async fn solver_competition_by_tx<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    transaction: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let competition = api(transport, chain_id, env)?
        .solver_competition_by_tx_hash(&tx_hash(transaction)?)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&competition)
}

pub async fn app_data<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    app_data_hash: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let hash = AppDataHash::new(app_data_hash).map_err(|error| invalid(error.to_string()))?;
    let object = api(transport, chain_id, env)?
        .app_data(&hash)
        .await
        .map_err(|error| from_orderbook(&error))?;
    json(&object)
}

pub async fn version<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    env: Option<&str>,
) -> Result<String, ReadError> {
    api(transport, chain_id, env)?
        .version()
        .await
        .map_err(|error| from_orderbook(&error))
}

pub fn order_link<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    order_uid: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    api(transport, chain_id, env)?
        .order_link(&uid(order_uid)?)
        .map_err(|error| from_orderbook(&error))
}

pub async fn send_order<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    order_json: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let order: OrderCreation =
        serde_json::from_str(order_json).map_err(|error| invalid(error.to_string()))?;
    let accepted = api(transport, chain_id, env)?
        .send_order(&order)
        .await
        .map_err(|error| from_orderbook(&error))?;
    Ok(accepted.to_hex_string())
}

pub async fn send_cancellations<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    cancellations_json: &str,
    env: Option<&str>,
) -> Result<(), ReadError> {
    let cancellations: OrderCancellations =
        serde_json::from_str(cancellations_json).map_err(|error| invalid(error.to_string()))?;
    api(transport, chain_id, env)?
        .send_cancellations(&cancellations)
        .await
        .map_err(|error| from_orderbook(&error))
}

pub async fn upload_app_data<T: HttpTransport + Send + Sync + 'static>(
    transport: T,
    chain_id: u64,
    app_data_hash: &str,
    full_app_data: &str,
    env: Option<&str>,
) -> Result<(), ReadError> {
    let hash = AppDataHash::new(app_data_hash).map_err(|error| invalid(error.to_string()))?;
    api(transport, chain_id, env)?
        .upload_app_data(&hash, full_app_data)
        .await
        .map_err(|error| from_orderbook(&error))
}
