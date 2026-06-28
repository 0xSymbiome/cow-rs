wit_bindgen::generate!({ world: "client-sync" });

use cow::protocol::book::{
    EthflowData, ExecutedProtocolFee, InteractionData, OnchainOrderData, Order, OrderInteractions,
    StoredOrderQuote, Trade,
};
use cow::protocol::order::{
    BuyTokenDestination, CowEnv, OrderClass, OrderKind, OrderStatus, SellTokenSource, SigningScheme,
};

use super::orderbook::{to_wit_order, to_wit_trade};
use exports::cow::protocol::orderbook_read::{ErrorClass, Pagination, QueryError};
use std::time::Duration;

use cow_sdk_core::{
    HttpTransport, Redacted, TransportError, TransportErrorClass, TransportResponse, async_trait,
};
use wstd::http::body::Bytes;
use wstd::http::{Body, BodyExt, Client, Method, Request};

/// A wstd-backed implementation of the SDK `HttpTransport` seam. The
/// orderbook client resolves the full URL and passes it here, so this
/// adapter is a pure executor.
#[derive(Debug)]
struct WstdTransport;

impl WstdTransport {
    async fn send(
        &self,
        method: Method,
        url: &str,
        headers: &[(String, String)],
        body: Option<&str>,
    ) -> Result<TransportResponse, TransportError> {
        let request_body = body.map_or_else(Body::empty, |text| {
            Body::from(Bytes::from(text.as_bytes().to_vec()))
        });
        let mut builder = Request::builder().uri(url).method(method);
        for (name, value) in headers {
            builder = builder.header(name, value);
        }
        let request = builder
            .body(request_body)
            .map_err(|error| TransportError::Transport {
                class: TransportErrorClass::Builder,
                detail: Redacted::new(error.to_string()),
            })?;
        let response =
            Client::new()
                .send(request)
                .await
                .map_err(|error| TransportError::Transport {
                    class: TransportErrorClass::Request,
                    detail: Redacted::new(error.to_string()),
                })?;
        let status = response.status().as_u16();
        let bytes = response
            .into_body()
            .into_boxed_body()
            .collect()
            .await
            .map_err(|error| TransportError::Transport {
                class: TransportErrorClass::Body,
                detail: Redacted::new(error.to_string()),
            })?;
        let text = String::from_utf8_lossy(bytes.to_bytes().as_ref()).to_string();
        if (200..300).contains(&status) {
            Ok(TransportResponse::new(status, Vec::new(), text))
        } else {
            Err(TransportError::HttpStatus {
                status,
                headers: Vec::new(),
                body: Redacted::new(text),
            })
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl HttpTransport for WstdTransport {
    async fn get(
        &self,
        path: &str,
        headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.send(Method::GET, path, headers, None).await
    }
    async fn post(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.send(Method::POST, path, headers, Some(body)).await
    }
    async fn put(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.send(Method::PUT, path, headers, Some(body)).await
    }
    async fn delete(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.send(Method::DELETE, path, headers, Some(body)).await
    }
}

struct Component;

/// Maps a WIT `order-kind` to the engine's string form.
const fn order_kind(kind: OrderKind) -> &'static str {
    match kind {
        OrderKind::Buy => "buy",
        OrderKind::Sell => "sell",
    }
}

/// Maps the WIT `cow-env` enum to the SDK environment label.
const fn cow_env(env: CowEnv) -> &'static str {
    match env {
        CowEnv::Prod => "prod",
        CowEnv::Staging => "staging",
    }
}

/// Maps the WIT `sell-token-source` enum to its wire form.
const fn sell_token_source(source: SellTokenSource) -> &'static str {
    match source {
        SellTokenSource::Erc20 => "erc20",
        SellTokenSource::External => "external",
        SellTokenSource::Internal => "internal",
    }
}

/// Maps the WIT `buy-token-destination` enum to its wire form.
const fn buy_token_destination(destination: BuyTokenDestination) -> &'static str {
    match destination {
        BuyTokenDestination::Erc20 => "erc20",
        BuyTokenDestination::Internal => "internal",
    }
}

/// Lowers a generated `partner-fee` record into the lane-agnostic policy list
/// the shared `core` builders accept, borrowing each recipient string.
fn partner_fee_policies(
    partner_fee: &exports::cow::protocol::trading::PartnerFee,
) -> Vec<super::core::PartnerFeePolicyParams<'_>> {
    partner_fee
        .policies
        .iter()
        .map(|policy| super::core::PartnerFeePolicyParams {
            volume_bps: policy.volume_bps,
            surplus_bps: policy.surplus_bps,
            price_improvement_bps: policy.price_improvement_bps,
            max_volume_bps: policy.max_volume_bps,
            recipient: &policy.recipient,
        })
        .collect()
}

/// Lowers the shared optional `swap-request` / `limit-request` fields into the
/// lane-agnostic `CommonTradeParams`, borrowing strings and slices from the
/// generated record so no field is dropped between the WIT record and the
/// native `with_*` builders.
fn common_swap_params(
    request: &exports::cow::protocol::trading::SwapRequest,
) -> super::core::CommonTradeParams<'_> {
    super::core::CommonTradeParams {
        receiver: request.receiver.as_deref(),
        valid_to: request.valid_to,
        valid_for: request.valid_for,
        partially_fillable: request.partially_fillable,
        sell_token_balance: request.sell_token_balance.map(sell_token_source),
        buy_token_balance: request.buy_token_balance.map(buy_token_destination),
        settlement_contract_override: request.settlement_contract_override.as_deref(),
        eth_flow_contract_override: request.eth_flow_contract_override.as_deref(),
        partner_fee: request.partner_fee.as_ref().map(partner_fee_policies),
    }
}

/// Lowers the shared `limit-request` fields into the lane-agnostic
/// `CommonTradeParams`.
fn common_limit_params(
    request: &exports::cow::protocol::trading::LimitRequest,
) -> super::core::CommonTradeParams<'_> {
    super::core::CommonTradeParams {
        receiver: request.receiver.as_deref(),
        valid_to: request.valid_to,
        valid_for: request.valid_for,
        partially_fillable: request.partially_fillable,
        sell_token_balance: request.sell_token_balance.map(sell_token_source),
        buy_token_balance: request.buy_token_balance.map(buy_token_destination),
        settlement_contract_override: request.settlement_contract_override.as_deref(),
        eth_flow_contract_override: request.eth_flow_contract_override.as_deref(),
        partner_fee: request.partner_fee.as_ref().map(partner_fee_policies),
    }
}

/// Borrows a `swap-request` into the lane-agnostic `SwapParams`.
fn swap_params<'a>(
    request: &'a exports::cow::protocol::trading::SwapRequest,
    kind: Option<&'a str>,
) -> super::core::SwapParams<'a> {
    super::core::SwapParams {
        chain_id: request.chain_id,
        owner: &request.owner,
        sell_token: &request.sell_token,
        buy_token: &request.buy_token,
        amount: &request.amount,
        app_code: &request.app_code,
        kind,
        slippage_bps: request.slippage_bps,
        env: request.env.map(cow_env),
        common: common_swap_params(request),
    }
}

impl exports::cow::protocol::orderbook_read::Guest for Component {
    fn get_order(chain_id: u64, uid: String, env: Option<String>) -> Result<Order, QueryError> {
        let json = block(super::orderbook::order(
            WstdTransport,
            chain_id,
            &uid,
            env.as_deref(),
        ))?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(to_wit_order!(&value))
    }

    fn get_orders(
        chain_id: u64,
        owner: String,
        page: Option<Pagination>,
        env: Option<String>,
    ) -> Result<Vec<Order>, QueryError> {
        let (offset, limit) = pagination(page);
        let json = block(super::orderbook::orders(
            WstdTransport,
            chain_id,
            &owner,
            offset,
            limit,
            env.as_deref(),
        ))?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(value
            .as_array()
            .map(|orders| orders.iter().map(|order| to_wit_order!(order)).collect())
            .unwrap_or_default())
    }

    fn get_tx_orders(
        chain_id: u64,
        tx_hash: String,
        env: Option<String>,
    ) -> Result<Vec<Order>, QueryError> {
        let json = block(super::orderbook::tx_orders(
            WstdTransport,
            chain_id,
            &tx_hash,
            env.as_deref(),
        ))?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(value
            .as_array()
            .map(|orders| orders.iter().map(|order| to_wit_order!(order)).collect())
            .unwrap_or_default())
    }

    fn get_trades_by_owner(
        chain_id: u64,
        owner: String,
        page: Option<Pagination>,
        env: Option<String>,
    ) -> Result<Vec<Trade>, QueryError> {
        let (offset, limit) = pagination(page);
        let json = block(super::orderbook::trades_by_owner(
            WstdTransport,
            chain_id,
            &owner,
            offset,
            limit,
            env.as_deref(),
        ))?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(value
            .as_array()
            .map(|trades| trades.iter().map(|trade| to_wit_trade!(trade)).collect())
            .unwrap_or_default())
    }

    fn get_trades_by_order(
        chain_id: u64,
        order_uid: String,
        page: Option<Pagination>,
        env: Option<String>,
    ) -> Result<Vec<Trade>, QueryError> {
        let (offset, limit) = pagination(page);
        let json = block(super::orderbook::trades_by_order(
            WstdTransport,
            chain_id,
            &order_uid,
            offset,
            limit,
            env.as_deref(),
        ))?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(value
            .as_array()
            .map(|trades| trades.iter().map(|trade| to_wit_trade!(trade)).collect())
            .unwrap_or_default())
    }

    fn get_quote(
        chain_id: u64,
        request_json: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        block(super::orderbook::quote(
            WstdTransport,
            chain_id,
            &request_json,
            env.as_deref(),
        ))
    }

    fn get_native_price(
        chain_id: u64,
        token: String,
        env: Option<String>,
    ) -> Result<f64, QueryError> {
        wstd::runtime::block_on(super::orderbook::native_price(
            WstdTransport,
            chain_id,
            &token,
            env.as_deref(),
        ))
        .map_err(to_wit_error)
    }

    fn get_total_surplus(
        chain_id: u64,
        owner: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        block(super::orderbook::total_surplus(
            WstdTransport,
            chain_id,
            &owner,
            env.as_deref(),
        ))
    }

    fn get_order_competition_status(
        chain_id: u64,
        uid: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        block(super::orderbook::order_competition_status(
            WstdTransport,
            chain_id,
            &uid,
            env.as_deref(),
        ))
    }

    fn get_solver_competition(
        chain_id: u64,
        auction_id: i64,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        block(super::orderbook::solver_competition(
            WstdTransport,
            chain_id,
            auction_id,
            env.as_deref(),
        ))
    }

    fn get_solver_competition_by_tx(
        chain_id: u64,
        tx_hash: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        block(super::orderbook::solver_competition_by_tx(
            WstdTransport,
            chain_id,
            &tx_hash,
            env.as_deref(),
        ))
    }

    fn get_app_data(
        chain_id: u64,
        app_data_hash: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        block(super::orderbook::app_data(
            WstdTransport,
            chain_id,
            &app_data_hash,
            env.as_deref(),
        ))
    }

    fn version(chain_id: u64, env: Option<String>) -> Result<String, QueryError> {
        block(super::orderbook::version(
            WstdTransport,
            chain_id,
            env.as_deref(),
        ))
    }

    fn order_link(chain_id: u64, uid: String, env: Option<String>) -> Result<String, QueryError> {
        super::orderbook::order_link(WstdTransport, chain_id, &uid, env.as_deref())
            .map_err(to_wit_error)
    }
}

/// Drives a read future to completion on the synchronous lane and maps the
/// boundary error.
fn block(
    future: impl std::future::Future<Output = Result<String, super::orderbook::ReadError>>,
) -> Result<String, QueryError> {
    wstd::runtime::block_on(future).map_err(to_wit_error)
}

fn pagination(page: Option<Pagination>) -> (Option<u32>, Option<u32>) {
    page.map_or((None, None), |page| (page.offset, page.limit))
}

fn to_wit_error(error: super::orderbook::ReadError) -> QueryError {
    QueryError {
        class: map_error_class(error.class),
        message: error.message,
        retryable: error.retryable,
        retry_after_ms: error.retry_after_ms,
        error_type: error.error_type,
    }
}

const fn map_error_class(class: cow_sdk_core::ErrorClass) -> ErrorClass {
    use cow_sdk_core::ErrorClass as Source;
    match class {
        Source::Validation => ErrorClass::Validation,
        Source::RateLimited => ErrorClass::RateLimited,
        Source::Remote => ErrorClass::Remote,
        Source::Transport => ErrorClass::Transport,
        Source::Cancelled => ErrorClass::Cancelled,
        Source::Signing => ErrorClass::Signing,
        _ => ErrorClass::Internal,
    }
}

impl exports::cow::protocol::orderbook_write::Guest for Component {
    fn send_order(
        chain_id: u64,
        order_json: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        block(super::orderbook::send_order(
            WstdTransport,
            chain_id,
            &order_json,
            env.as_deref(),
        ))
    }

    fn send_cancellations(
        chain_id: u64,
        cancellations_json: String,
        env: Option<String>,
    ) -> Result<(), QueryError> {
        wstd::runtime::block_on(super::orderbook::send_cancellations(
            WstdTransport,
            chain_id,
            &cancellations_json,
            env.as_deref(),
        ))
        .map_err(to_wit_error)
    }

    fn upload_app_data(
        chain_id: u64,
        app_data_hash: String,
        full_app_data: String,
        env: Option<String>,
    ) -> Result<(), QueryError> {
        wstd::runtime::block_on(super::orderbook::upload_app_data(
            WstdTransport,
            chain_id,
            &app_data_hash,
            &full_app_data,
            env.as_deref(),
        ))
        .map_err(to_wit_error)
    }
}

impl exports::cow::protocol::trading::Guest for Component {
    fn swap(request: exports::cow::protocol::trading::SwapRequest) -> Result<String, QueryError> {
        let kind = request.kind.map(order_kind);
        block(super::core::run_swap(
            WstdTransport,
            cow::protocol::signer::sign_digest,
            swap_params(&request, kind),
        ))
    }

    fn quote(request: exports::cow::protocol::trading::SwapRequest) -> Result<String, QueryError> {
        let kind = request.kind.map(order_kind);
        block(super::core::run_quote(
            WstdTransport,
            swap_params(&request, kind),
        ))
    }

    fn post_swap_from_quote(quote_json: String) -> Result<String, QueryError> {
        block(super::core::run_post_swap_from_quote(
            WstdTransport,
            cow::protocol::signer::sign_digest,
            &quote_json,
        ))
    }

    fn post_limit(
        request: exports::cow::protocol::trading::LimitRequest,
    ) -> Result<String, QueryError> {
        let kind = request.kind.map(order_kind);
        let common = common_limit_params(&request);
        block(super::core::run_limit(
            WstdTransport,
            cow::protocol::signer::sign_digest,
            super::core::LimitParams {
                chain_id: request.chain_id,
                owner: &request.owner,
                sell_token: &request.sell_token,
                buy_token: &request.buy_token,
                sell_amount: &request.sell_amount,
                buy_amount: &request.buy_amount,
                app_code: &request.app_code,
                kind,
                env: request.env.map(cow_env),
                quote_id: request.quote_id,
                slippage_bps: request.slippage_bps,
                common,
            },
        ))
    }

    fn cow_allowance(
        chain_id: u64,
        owner: String,
        token: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        block(super::core::run_allowance(
            host_read_contract,
            chain_id,
            &owner,
            &token,
            env.as_deref(),
        ))
    }
}

/// Wraps the world's generated `contract-read` host import as a `ReadFn`.
fn host_read_contract(
    address: &str,
    method: &str,
    abi_json: &str,
    args_json: &str,
) -> Result<String, String> {
    cow::protocol::contract_read::read_contract(&cow::protocol::contract_read::ContractCall {
        address: address.to_owned(),
        method: method.to_owned(),
        abi_json: abi_json.to_owned(),
        args_json: args_json.to_owned(),
    })
}

export!(Component);
