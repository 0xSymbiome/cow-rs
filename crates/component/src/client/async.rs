//! WASI 0.3 async client world binding. `wit_bindgen`'s generated async bindings
//! trip clippy's nursery `collection_is_never_read` inside the macro expansion
//! (async lane only — the sync lane's generated bindings are clean).
#![allow(
    clippy::collection_is_never_read,
    reason = "emitted inside the wit_bindgen async-world macro expansion, not our code"
)]

wit_bindgen::generate!({ world: "client-async" });

use cow::protocol::book::{
    EthflowData, ExecutedProtocolFee, InteractionData, OnchainOrderData, Order, OrderInteractions,
    StoredOrderQuote, Trade,
};
use cow::protocol::order::{
    BuyTokenDestination, CowEnv, OrderClass, OrderKind, OrderStatus, SellTokenSource, SigningScheme,
};

use super::orderbook::{to_wit_order, to_wit_trade};
use exports::cow::protocol::orderbook_read_async::{ErrorClass, Pagination, QueryError};
use std::time::Duration;

use cow_sdk_core::{
    HttpTransport, Redacted, TransportError, TransportErrorClass, TransportResponse, async_trait,
};
use http_body_util::BodyExt;

/// A wasip3-backed implementation of the SDK `HttpTransport` seam, over the
/// `http-compat` request and response bridge (wasi:http@0.3).
#[derive(Debug)]
struct Wasip3Transport;

impl Wasip3Transport {
    async fn send(
        &self,
        method: http::Method,
        url: &str,
        headers: &[(String, String)],
        body: Option<&str>,
    ) -> Result<TransportResponse, TransportError> {
        use wasip3::http::types::{Fields, Request, Scheme};

        let build_err = |detail: String| TransportError::Transport {
            class: TransportErrorClass::Builder,
            detail: Redacted::new(detail),
        };

        // Split "https://authority/path?query" for the low-level request.
        let rest = url
            .strip_prefix("https://")
            .ok_or_else(|| build_err("component transport requires https".to_owned()))?;
        let split = rest.find('/').unwrap_or(rest.len());
        let authority = &rest[..split];
        let path = if split < rest.len() {
            &rest[split..]
        } else {
            "/"
        };

        let header_entries: Vec<(String, Vec<u8>)> = headers
            .iter()
            .map(|(name, value)| (name.clone(), value.as_bytes().to_vec()))
            .collect();
        let fields =
            Fields::from_list(&header_entries).map_err(|error| build_err(format!("{error:?}")))?;

        let body_bytes = body.unwrap_or("").as_bytes().to_vec();
        let (mut body_tx, body_rx) = wasip3::wit_stream::new();
        let (trailers_tx, trailers_rx) = wasip3::wit_future::new(|| Ok(None));
        let (request, _done) = Request::new(fields, Some(body_rx), trailers_rx, None);
        drop(trailers_tx);

        let wasi_method: wasip3::http::types::Method = method.into();
        request
            .set_method(&wasi_method)
            .map_err(|()| build_err("set_method".to_owned()))?;
        request
            .set_scheme(Some(&Scheme::Https))
            .map_err(|()| build_err("set_scheme".to_owned()))?;
        request
            .set_authority(Some(authority))
            .map_err(|()| build_err("set_authority".to_owned()))?;
        request
            .set_path_with_query(Some(path))
            .map_err(|()| build_err("set_path".to_owned()))?;

        // Drive the body write and the send concurrently in this exported task.
        // The 0.3 async host pumps this task but not a `wit_bindgen::spawn`ed
        // subtask, so the http-compat body writer (which spawns) would deadlock.
        let write_body = async move {
            if !body_bytes.is_empty() {
                let _unwritten = body_tx.write_all(body_bytes).await;
            }
            drop(body_tx);
        };
        let ((), sent) =
            futures::future::join(write_body, wasip3::http::client::send(request)).await;
        let wasi_response = sent.map_err(|error| TransportError::Transport {
            class: TransportErrorClass::Request,
            detail: Redacted::new(format!("{error:?}")),
        })?;
        let response =
            wasip3::http_compat::http_from_wasi_response(wasi_response).map_err(|error| {
                TransportError::Transport {
                    class: TransportErrorClass::Decode,
                    detail: Redacted::new(format!("{error:?}")),
                }
            })?;
        let status = response.status().as_u16();
        let collected =
            response
                .into_body()
                .collect()
                .await
                .map_err(|error| TransportError::Transport {
                    class: TransportErrorClass::Body,
                    detail: Redacted::new(format!("{error:?}")),
                })?;
        let text = String::from_utf8_lossy(&collected.to_bytes()).to_string();
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
impl HttpTransport for Wasip3Transport {
    async fn get(
        &self,
        path: &str,
        headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.send(http::Method::GET, path, headers, None).await
    }
    async fn post(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.send(http::Method::POST, path, headers, Some(body))
            .await
    }
    async fn put(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.send(http::Method::PUT, path, headers, Some(body))
            .await
    }
    async fn delete(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.send(http::Method::DELETE, path, headers, Some(body))
            .await
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
    partner_fee: &exports::cow::protocol::trading_async::PartnerFee,
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

/// Lowers the shared optional `swap-request` fields into the lane-agnostic
/// `CommonTradeParams`, borrowing strings and slices from the generated record
/// so no field is dropped between the WIT record and the native `with_*`
/// builders.
fn common_swap_params(
    request: &exports::cow::protocol::trading_async::SwapRequest,
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
    request: &exports::cow::protocol::trading_async::LimitRequest,
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
    request: &'a exports::cow::protocol::trading_async::SwapRequest,
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

impl exports::cow::protocol::orderbook_read_async::Guest for Component {
    async fn get_order(
        chain_id: u64,
        uid: String,
        env: Option<String>,
    ) -> Result<Order, QueryError> {
        let json = super::orderbook::order(Wasip3Transport, chain_id, &uid, env.as_deref())
            .await
            .map_err(to_wit_error)?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(to_wit_order!(&value))
    }

    async fn get_orders(
        chain_id: u64,
        owner: String,
        page: Option<Pagination>,
        env: Option<String>,
    ) -> Result<Vec<Order>, QueryError> {
        let (offset, limit) = pagination(page);
        let json = super::orderbook::orders(
            Wasip3Transport,
            chain_id,
            &owner,
            offset,
            limit,
            env.as_deref(),
        )
        .await
        .map_err(to_wit_error)?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(value
            .as_array()
            .map(|orders| orders.iter().map(|order| to_wit_order!(order)).collect())
            .unwrap_or_default())
    }

    async fn get_tx_orders(
        chain_id: u64,
        tx_hash: String,
        env: Option<String>,
    ) -> Result<Vec<Order>, QueryError> {
        let json = super::orderbook::tx_orders(Wasip3Transport, chain_id, &tx_hash, env.as_deref())
            .await
            .map_err(to_wit_error)?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(value
            .as_array()
            .map(|orders| orders.iter().map(|order| to_wit_order!(order)).collect())
            .unwrap_or_default())
    }

    async fn get_trades_by_owner(
        chain_id: u64,
        owner: String,
        page: Option<Pagination>,
        env: Option<String>,
    ) -> Result<Vec<Trade>, QueryError> {
        let (offset, limit) = pagination(page);
        let json = super::orderbook::trades_by_owner(
            Wasip3Transport,
            chain_id,
            &owner,
            offset,
            limit,
            env.as_deref(),
        )
        .await
        .map_err(to_wit_error)?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(value
            .as_array()
            .map(|trades| trades.iter().map(|trade| to_wit_trade!(trade)).collect())
            .unwrap_or_default())
    }

    async fn get_trades_by_order(
        chain_id: u64,
        order_uid: String,
        page: Option<Pagination>,
        env: Option<String>,
    ) -> Result<Vec<Trade>, QueryError> {
        let (offset, limit) = pagination(page);
        let json = super::orderbook::trades_by_order(
            Wasip3Transport,
            chain_id,
            &order_uid,
            offset,
            limit,
            env.as_deref(),
        )
        .await
        .map_err(to_wit_error)?;
        let value = super::orderbook::value(&json).map_err(to_wit_error)?;
        Ok(value
            .as_array()
            .map(|trades| trades.iter().map(|trade| to_wit_trade!(trade)).collect())
            .unwrap_or_default())
    }

    async fn get_quote(
        chain_id: u64,
        request_json: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        super::orderbook::quote(Wasip3Transport, chain_id, &request_json, env.as_deref())
            .await
            .map_err(to_wit_error)
    }

    async fn get_native_price(
        chain_id: u64,
        token: String,
        env: Option<String>,
    ) -> Result<f64, QueryError> {
        super::orderbook::native_price(Wasip3Transport, chain_id, &token, env.as_deref())
            .await
            .map_err(to_wit_error)
    }

    async fn get_total_surplus(
        chain_id: u64,
        owner: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        super::orderbook::total_surplus(Wasip3Transport, chain_id, &owner, env.as_deref())
            .await
            .map_err(to_wit_error)
    }

    async fn get_order_competition_status(
        chain_id: u64,
        uid: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        super::orderbook::order_competition_status(Wasip3Transport, chain_id, &uid, env.as_deref())
            .await
            .map_err(to_wit_error)
    }

    async fn get_solver_competition(
        chain_id: u64,
        auction_id: i64,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        super::orderbook::solver_competition(Wasip3Transport, chain_id, auction_id, env.as_deref())
            .await
            .map_err(to_wit_error)
    }

    async fn get_solver_competition_by_tx(
        chain_id: u64,
        tx_hash: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        super::orderbook::solver_competition_by_tx(
            Wasip3Transport,
            chain_id,
            &tx_hash,
            env.as_deref(),
        )
        .await
        .map_err(to_wit_error)
    }

    async fn get_app_data(
        chain_id: u64,
        app_data_hash: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        super::orderbook::app_data(Wasip3Transport, chain_id, &app_data_hash, env.as_deref())
            .await
            .map_err(to_wit_error)
    }

    async fn version(chain_id: u64, env: Option<String>) -> Result<String, QueryError> {
        super::orderbook::version(Wasip3Transport, chain_id, env.as_deref())
            .await
            .map_err(to_wit_error)
    }

    async fn order_link(
        chain_id: u64,
        uid: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        super::orderbook::order_link(Wasip3Transport, chain_id, &uid, env.as_deref())
            .map_err(to_wit_error)
    }
}

impl exports::cow::protocol::orderbook_write_async::Guest for Component {
    async fn send_order(
        chain_id: u64,
        order_json: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        super::orderbook::send_order(Wasip3Transport, chain_id, &order_json, env.as_deref())
            .await
            .map_err(to_wit_error)
    }

    async fn send_cancellations(
        chain_id: u64,
        cancellations_json: String,
        env: Option<String>,
    ) -> Result<(), QueryError> {
        super::orderbook::send_cancellations(
            Wasip3Transport,
            chain_id,
            &cancellations_json,
            env.as_deref(),
        )
        .await
        .map_err(to_wit_error)
    }

    async fn upload_app_data(
        chain_id: u64,
        app_data_hash: String,
        full_app_data: String,
        env: Option<String>,
    ) -> Result<(), QueryError> {
        super::orderbook::upload_app_data(
            Wasip3Transport,
            chain_id,
            &app_data_hash,
            &full_app_data,
            env.as_deref(),
        )
        .await
        .map_err(to_wit_error)
    }
}

impl exports::cow::protocol::trading_async::Guest for Component {
    async fn swap(
        request: exports::cow::protocol::trading_async::SwapRequest,
    ) -> Result<String, QueryError> {
        let kind = request.kind.map(order_kind);
        super::core::run_swap(
            Wasip3Transport,
            cow::protocol::signer::sign_digest,
            swap_params(&request, kind),
        )
        .await
        .map_err(to_wit_error)
    }

    async fn quote(
        request: exports::cow::protocol::trading_async::SwapRequest,
    ) -> Result<String, QueryError> {
        let kind = request.kind.map(order_kind);
        super::core::run_quote(Wasip3Transport, swap_params(&request, kind))
            .await
            .map_err(to_wit_error)
    }

    async fn post_swap_from_quote(quote_json: String) -> Result<String, QueryError> {
        super::core::run_post_swap_from_quote(
            Wasip3Transport,
            cow::protocol::signer::sign_digest,
            &quote_json,
        )
        .await
        .map_err(to_wit_error)
    }

    async fn post_limit(
        request: exports::cow::protocol::trading_async::LimitRequest,
    ) -> Result<String, QueryError> {
        let kind = request.kind.map(order_kind);
        let common = common_limit_params(&request);
        super::core::run_limit(
            Wasip3Transport,
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
        )
        .await
        .map_err(to_wit_error)
    }

    async fn place_swap(
        request: exports::cow::protocol::trading_async::SwapRequest,
        owner: String,
        auth: exports::cow::protocol::trading_async::Authorization,
    ) -> Result<exports::cow::protocol::trading_async::OrderPlacement, QueryError> {
        let kind = request.kind.map(order_kind);
        let placement = super::core::run_place_swap(
            Wasip3Transport,
            cow::protocol::signer::sign_digest,
            swap_params(&request, kind),
            &owner,
            auth_params(auth),
        )
        .await
        .map_err(to_wit_error)?;
        Ok(to_wit_placement(placement))
    }

    async fn place_limit(
        request: exports::cow::protocol::trading_async::LimitRequest,
        owner: String,
        auth: exports::cow::protocol::trading_async::Authorization,
    ) -> Result<exports::cow::protocol::trading_async::OrderPlacement, QueryError> {
        let kind = request.kind.map(order_kind);
        let common = common_limit_params(&request);
        let placement = super::core::run_place_limit(
            Wasip3Transport,
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
            &owner,
            auth_params(auth),
        )
        .await
        .map_err(to_wit_error)?;
        Ok(to_wit_placement(placement))
    }

    async fn cow_allowance(
        chain_id: u64,
        owner: String,
        token: String,
        env: Option<String>,
    ) -> Result<String, QueryError> {
        super::core::run_allowance(host_read_contract, chain_id, &owner, &token, env.as_deref())
            .await
            .map_err(to_wit_error)
    }
}

/// Lowers the WIT `authorization` variant into the lane-agnostic `AuthParams`.
fn auth_params(
    auth: exports::cow::protocol::trading_async::Authorization,
) -> super::core::AuthParams {
    use exports::cow::protocol::trading_async::Authorization;
    match auth {
        Authorization::Ecdsa => super::core::AuthParams::Ecdsa,
        Authorization::Eip1271(blob) => super::core::AuthParams::Eip1271(blob),
        Authorization::PreSign => super::core::AuthParams::PreSign,
    }
}

/// Lowers the lane-agnostic `Placement` into the WIT `order-placement` variant.
fn to_wit_placement(
    placement: super::core::Placement,
) -> exports::cow::protocol::trading_async::OrderPlacement {
    use cow::protocol::trading::{PendingPlacement, SafeActivation};
    use cow::protocol::tx::TxRequest;
    use exports::cow::protocol::trading_async::OrderPlacement;
    match placement {
        super::core::Placement::Live { order_uid } => OrderPlacement::Live(order_uid),
        super::core::Placement::Pending { order_uid, calls } => {
            OrderPlacement::PendingActivation(PendingPlacement {
                order_uid,
                activation: SafeActivation {
                    calls: calls
                        .into_iter()
                        .map(|(to, data, value)| TxRequest { to, data, value })
                        .collect(),
                },
            })
        }
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

export!(Component);
