#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_wasm::exports::{
    AllowanceParametersInput, ApprovalParametersInput, LimitTradeParametersInput, OrderBookClient,
    OrderBookClientConfig, OrderKindDto, OrderTraderParametersInput, PaginationOptions,
    SwapParametersInput, TokenBalanceDto, TradesQueryInput, TradingClient, build_cancel_order_tx,
    build_presign_tx, compute_order_uid,
};
use js_sys::{Function, Object, Reflect};
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

use crate::common::{
    ADDR_BUY, ADDR_OWNER, ADDR_RECEIVER, ADDR_SELL, CHAIN_MAINNET, ECDSA_SIGNATURE, HASH_APP_DATA,
    wasm_order_input,
};

wasm_bindgen_test_configure!(run_in_browser);

const NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn set_js(target: &Object, key: &str, value: &JsValue) {
    Reflect::set(target, &JsValue::from_str(key), value).expect("test object field should be set");
}

fn callback_transport(callback: &Function) -> Object {
    let transport = Object::new();
    set_js(&transport, "kind", &JsValue::from_str("callback"));
    set_js(&transport, "callback", callback.as_ref());
    transport
}

fn orderbook_config_with_api_key(callback: &Function, api_key: &str) -> OrderBookClientConfig {
    let config = Object::new();
    set_js(
        &config,
        "chainId",
        &JsValue::from_f64(f64::from(CHAIN_MAINNET)),
    );
    set_js(&config, "apiKey", &JsValue::from_str(api_key));
    set_js(&config, "transport", callback_transport(callback).as_ref());
    JsValue::from(config).unchecked_into()
}

fn recorded_requests() -> Vec<Value> {
    json(js_sys::eval("globalThis.__cowCoverageRequests").unwrap())
        .as_array()
        .expect("recorded requests should be an array")
        .clone()
}

fn generated_order_uid() -> String {
    let value =
        json(compute_order_uid(wasm_order_input(), CHAIN_MAINNET, ADDR_OWNER.to_owned()).unwrap());
    value["value"]["orderUid"].as_str().unwrap().to_owned()
}

fn future_valid_to() -> u32 {
    (js_sys::Date::now() / 1000.0) as u32 + 3_600
}

/// Builds a `/api/v1/quote` response body that deserializes into the native
/// `OrderQuoteResponse`, carrying `id` so the posted order's `quoteId` is
/// asserted end-to-end through a real `getQuote` round-trip.
fn quote_response_json(id: i64, valid_to: u32) -> String {
    serde_json::json!({
        "quote": {
            "sellToken": ADDR_SELL,
            "buyToken": ADDR_BUY,
            "receiver": ADDR_RECEIVER,
            "sellAmount": "98646335338956442",
            "buyAmount": "30000000000000000000",
            "validTo": valid_to,
            "appData": HASH_APP_DATA,
            "feeAmount": "1353664661043558",
            "kind": "sell",
            "partiallyFillable": false,
            "sellTokenBalance": "erc20",
            "buyTokenBalance": "erc20"
        },
        "from": ADDR_OWNER,
        "expiration": "2099-01-01T00:00:00.000Z",
        "id": id,
        "verified": true
    })
    .to_string()
}

/// Fetch callback that serves a quote for `POST /api/v1/quote` — echoing back
/// the request's pinned `appDataHash` so the quote-echo gate (ADR 0058)
/// reconciles the response against the request like a real orderbook — echoes
/// the app-data hash for the `PUT /api/v1/app_data/:hash` upload (the orderbook
/// verifies the server echo against the URL hash), and returns `uid` for every
/// order creation.
fn swap_quote_fetch_callback(uid: &str, quote_body: &str) -> Function {
    callback(
        "request",
        &format!(
            r#"
            globalThis.__cowCoverageRequests = globalThis.__cowCoverageRequests || [];
            globalThis.__cowCoverageRequests.push(request);
            if (request.method === "PUT" && request.url.includes("/api/v1/app_data/")) {{
              const appDataHash = request.url.split("/api/v1/app_data/")[1].split(/[?#]/)[0];
              return {{ status: 200, headers: {{}}, body: JSON.stringify(appDataHash) }};
            }}
            if (request.method === "POST" && request.url.includes("/api/v1/quote")) {{
              const reqBody = JSON.parse(request.body);
              const reqHash = reqBody.appDataHash || reqBody.appData;
              const resp = JSON.parse({quote_body:?});
              if (reqHash) {{ resp.quote.appData = reqHash; }}
              return {{ status: 200, headers: {{}}, body: JSON.stringify(resp) }};
            }}
            return {{ status: 200, headers: {{}}, body: JSON.stringify("{uid}") }};
            "#,
        ),
    )
}

/// Parsed bodies of every recorded `POST /api/v1/orders` request, in order.
fn recorded_order_creation_bodies() -> Vec<Value> {
    recorded_requests()
        .into_iter()
        .filter(|request| {
            request["method"].as_str() == Some("POST")
                && request["url"]
                    .as_str()
                    .is_some_and(|url| url.contains("/api/v1/orders"))
        })
        .map(|request| {
            serde_json::from_str(request["body"].as_str().expect("order request body"))
                .expect("order creation body should be JSON")
        })
        .collect()
}

fn signer_callback() -> Function {
    callback(
        "envelope",
        &format!(
            "globalThis.__cowCoverageSignerEnvelope = envelope; return '{}';",
            ECDSA_SIGNATURE
        ),
    )
}

fn limit_params(valid_to: u32) -> LimitTradeParametersInput {
    LimitTradeParametersInput {
        kind: OrderKindDto::Sell,
        owner: None,
        sell_token: ADDR_SELL.to_owned(),
        buy_token: ADDR_BUY.to_owned(),
        sell_amount: "1000000000000000000".to_owned(),
        buy_amount: "2000000000000000000".to_owned(),
        quote_id: None,
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        sell_token_balance: Some(TokenBalanceDto::Erc20),
        buy_token_balance: Some(TokenBalanceDto::Erc20),
        slippage_bps: Some(0),
        receiver: Some(ADDR_RECEIVER.to_owned()),
        valid_for: None,
        valid_to: Some(valid_to),
        partner_fee: None,
    }
}

#[wasm_bindgen_test]
async fn orderbook_lists_support_api_key_routing_and_owner_trade_pagination() {
    js_sys::eval("globalThis.__cowCoverageRequests = []").unwrap();
    let fetch = callback(
        "request",
        "globalThis.__cowCoverageRequests.push(request);
         return { status: 200, headers: {}, body: '[]' };",
    );
    let client = OrderBookClient::new(orderbook_config_with_api_key(&fetch, "test-api-key"))
        .expect("orderbook config should accept apiKey");

    let orders = json(
        client
            .orders(
                ADDR_OWNER.to_owned(),
                Some(PaginationOptions {
                    offset: Some(7),
                    limit: Some(13),
                }),
                None,
            )
            .await
            .unwrap(),
    );
    let trades = json(
        client
            .trades(
                TradesQueryInput {
                    owner: Some(ADDR_OWNER.to_owned()),
                    order_uid: None,
                    offset: Some(3),
                    limit: Some(5),
                },
                None,
            )
            .await
            .unwrap(),
    );
    let requests = recorded_requests();

    assert_eq!(orders["value"].as_array().unwrap().len(), 0);
    assert_eq!(trades["value"].as_array().unwrap().len(), 0);
    assert!(
        requests[0]["url"]
            .as_str()
            .unwrap()
            .starts_with("https://partners.cow.fi/mainnet")
    );
    assert!(
        requests[0]["url"]
            .as_str()
            .unwrap()
            .contains(&format!("/api/v1/account/{ADDR_OWNER}/orders"))
    );
    assert!(requests[0]["url"].as_str().unwrap().contains("offset=7"));
    assert!(requests[0]["url"].as_str().unwrap().contains("limit=13"));
    assert!(
        requests[1]["url"]
            .as_str()
            .unwrap()
            .contains(&format!("owner={ADDR_OWNER}"))
    );
    assert!(requests[1]["url"].as_str().unwrap().contains("offset=3"));
    assert!(requests[1]["url"].as_str().unwrap().contains("limit=5"));
    assert!(
        requests[0]["headers"]
            .as_object()
            .unwrap()
            .iter()
            .any(|(name, value)| name.eq_ignore_ascii_case("x-api-key")
                && value.as_str() == Some("test-api-key"))
    );
}

#[wasm_bindgen_test]
async fn trading_posts_swap_from_quote_and_limit_orders_through_typed_signers() {
    js_sys::eval("globalThis.__cowCoverageRequests = []").unwrap();
    let uid = generated_order_uid();
    let valid_to = future_valid_to();
    let fetch = swap_quote_fetch_callback(&uid, &quote_response_json(91, valid_to));
    let client = TradingClient::new(crate::common::trading_config(
        CHAIN_MAINNET,
        None,
        "CoW Swap",
        &fetch,
    ))
    .unwrap();

    // Fetch a quote, then post it back unchanged: this exercises the real
    // `QuoteResults` serialization round-trip between `getQuote` and
    // `postSwapOrderFromQuote`, the documented host workflow.
    let quote_envelope = client
        .quote(
            SwapParametersInput {
                kind: OrderKindDto::Sell,
                owner: Some(ADDR_OWNER.to_owned()),
                sell_token: ADDR_SELL.to_owned(),
                buy_token: ADDR_BUY.to_owned(),
                // sellAmountBeforeFee: reconciles with the canned response's
                // fixed leg (sellAmount 98646335338956442 + feeAmount
                // 1353664661043558) so the quote-echo gate accepts it.
                amount: "100000000000000000".to_owned(),
                env: None,
                partially_fillable: false,
                sell_token_balance: Some(TokenBalanceDto::Erc20),
                buy_token_balance: Some(TokenBalanceDto::Erc20),
                slippage_bps: Some(50),
                receiver: Some(ADDR_RECEIVER.to_owned()),
                valid_for: None,
                valid_to: Some(valid_to),
                partner_fee: None,
            },
            None,
        )
        .await
        .unwrap();
    let quote_results = Reflect::get(&quote_envelope, &JsValue::from_str("value"))
        .expect("getQuote envelope should expose the QuoteResults value");

    // The native-sell `…FromQuote` builder fails closed on a quote that was not
    // requested for a native-currency sell: this swap quote sells an ERC-20, so
    // its provenance check rejects it before deriving any EthFlow transaction.
    let swap_quote_for_native = Reflect::get(&quote_envelope, &JsValue::from_str("value"))
        .expect("getQuote envelope should expose the QuoteResults value");
    client
        .build_sell_native_currency_tx_from_quote(
            swap_quote_for_native,
            ADDR_OWNER.to_owned(),
            None,
        )
        .await
        .expect_err("a non-native-currency-sell quote must be rejected by the from-quote builder");

    // Both posts sign and then run the post-sign owner-recovery gate (ADR 0015).
    // This wasm harness signs with a fixed canned signature that does not
    // recover to ADDR_OWNER, so the gate fails closed — exactly as it would for
    // a browser wallet that signed for the wrong account. The happy-path post is
    // covered natively with real keys (`crates/trading/tests/post_contract.rs`);
    // here we cover the wasm-unique boundary: the `QuoteResults` serialization
    // round-trip is accepted and projected into a signable `Order`, and the
    // gate's rejection surfaces through the wasm trading client before any order
    // is created.
    client
        .post_swap_order_from_quote(
            quote_results,
            ADDR_OWNER.to_owned(),
            signer_callback(),
            None,
        )
        .await
        .expect_err("the canned wasm signer cannot satisfy the owner-recovery gate");
    client
        .post_limit_order(
            limit_params(valid_to),
            ADDR_OWNER.to_owned(),
            signer_callback(),
            None,
        )
        .await
        .expect_err("the canned wasm signer cannot satisfy the owner-recovery gate");

    // Signing ran before the gate — the round-tripped `QuoteResults` reached the
    // signer as a canonical EIP-712 `Order` envelope — and no order body was
    // ever submitted for creation.
    let signer_envelope = json(js_sys::eval("globalThis.__cowCoverageSignerEnvelope").unwrap());
    assert_eq!(signer_envelope["primaryType"], "Order");
    assert!(
        recorded_order_creation_bodies().is_empty(),
        "the owner-recovery gate must reject before any order is created",
    );
}

#[wasm_bindgen_test]
async fn trading_exposes_allowance_and_transaction_builders() {
    let fetch = callback(
        "request",
        "return { status: 200, headers: {}, body: '{}' };",
    );
    let client = TradingClient::new(crate::common::trading_config(
        CHAIN_MAINNET,
        None,
        "CoW Swap",
        &fetch,
    ))
    .unwrap();
    let allowance_reader = callback(
        "request",
        "globalThis.__cowAllowanceRead = request; return '57';",
    );

    let allowance = json(
        client
            .cow_protocol_allowance(
                AllowanceParametersInput {
                    token_address: ADDR_SELL.to_owned(),
                    owner: ADDR_OWNER.to_owned(),
                    chain_id: None,
                    env: None,
                    vault_relayer_override: None,
                },
                allowance_reader,
                None,
            )
            .await
            .unwrap(),
    );
    let allowance_request = json(js_sys::eval("globalThis.__cowAllowanceRead").unwrap());

    let mut native_order = wasm_order_input();
    native_order.sell_token = NATIVE_TOKEN.to_owned();
    native_order.valid_to = future_valid_to();
    native_order.app_data = HASH_APP_DATA.to_owned();
    let ethflow = json(
        client
            .build_sell_native_currency_tx(native_order, 77.0, ADDR_OWNER.to_owned(), None)
            .await
            .unwrap(),
    );
    let approval = json(
        client
            .build_approval_tx(
                ApprovalParametersInput {
                    token_address: ADDR_SELL.to_owned(),
                    amount: "1000000000000000000".to_owned(),
                    vault_relayer_override: None,
                },
                None,
            )
            .await
            .unwrap(),
    );
    let order_uid = generated_order_uid();
    let presign = json(
        build_presign_tx(OrderTraderParametersInput {
            order_uid: order_uid.clone(),
            chain_id: Some(CHAIN_MAINNET),
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        })
        .unwrap(),
    );
    let cancel = json(
        build_cancel_order_tx(OrderTraderParametersInput {
            order_uid,
            chain_id: Some(CHAIN_MAINNET),
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        })
        .unwrap(),
    );

    assert_eq!(allowance["value"], "57");
    assert_eq!(allowance_request["method"], "allowance");
    assert_eq!(allowance_request["address"], ADDR_SELL);
    assert_eq!(
        ethflow["value"]["transaction"]["value"],
        "1000000000000000000"
    );
    assert!(
        ethflow["value"]["transaction"]["data"]
            .as_str()
            .unwrap()
            .starts_with("0x")
    );
    assert_eq!(ethflow["value"]["orderUid"].as_str().unwrap().len(), 114);
    assert_eq!(presign["value"]["value"], "0");
    assert_eq!(cancel["value"]["value"], "0");
    assert!(presign["value"]["data"].as_str().unwrap().starts_with("0x"));
    assert!(cancel["value"]["data"].as_str().unwrap().starts_with("0x"));
    assert_eq!(approval["value"]["to"], ADDR_SELL);
    assert_eq!(approval["value"]["value"], "0");
    assert!(
        approval["value"]["data"]
            .as_str()
            .unwrap()
            .starts_with("0x")
    );
}

const SOLVER_COMPETITION_TX: &str =
    "0x1111111111111111111111111111111111111111111111111111111111111111";

#[wasm_bindgen_test]
async fn orderbook_reads_solver_competition_over_v2_routes() {
    js_sys::eval("globalThis.__cowCoverageRequests = []").unwrap();
    // A v2 SolverCompetitionResponse body that deserializes into the native
    // `SolverCompetitionResponse`, exercised end-to-end through both reads.
    let body = r#"{"auctionId":123,"auctionStartBlock":100,"auctionDeadlineBlock":110,"transactionHashes":["0x1111111111111111111111111111111111111111111111111111111111111111"],"referenceScores":{"0x2222222222222222222222222222222222222222":"42"},"auction":{"orders":[],"prices":{"0x2222222222222222222222222222222222222222":"7"}},"solutions":[{"solverAddress":"0x3333333333333333333333333333333333333333","score":"99","ranking":0,"clearingPrices":{},"orders":[],"isWinner":true,"filteredOut":false}]}"#;
    let fetch = callback(
        "request",
        &format!(
            "globalThis.__cowCoverageRequests.push(request);
             return {{ status: 200, headers: {{}}, body: {body:?} }};"
        ),
    );
    let client =
        OrderBookClient::new(crate::common::orderbook_config(CHAIN_MAINNET, None, &fetch)).unwrap();

    let by_id = json(client.solver_competition(123.0, None).await.unwrap());
    let by_hash = json(
        client
            .solver_competition_by_tx_hash(SOLVER_COMPETITION_TX.to_owned(), None)
            .await
            .unwrap(),
    );
    let requests = recorded_requests();

    // The native typed response round-trips through the envelope unchanged.
    assert_eq!(by_id["value"]["auctionId"], 123);
    assert_eq!(by_id["value"]["solutions"][0]["isWinner"], true);
    assert_eq!(by_id["value"]["solutions"][0]["score"], "99");
    assert_eq!(by_hash["value"]["auctionId"], 123);
    // Both reads target the v2 routes the services backend serves.
    assert!(
        requests[0]["url"]
            .as_str()
            .unwrap()
            .contains("/api/v2/solver_competition/123")
    );
    assert!(
        requests[1]["url"]
            .as_str()
            .unwrap()
            .contains("/api/v2/solver_competition/by_tx_hash/")
    );
}
