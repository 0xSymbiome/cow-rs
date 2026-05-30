#![allow(
    clippy::redundant_clone,
    clippy::too_many_lines,
    reason = "wire contract fixtures stay inline so byte identity remains reviewable"
)]

use serde::{Serialize, de::DeserializeOwned};

use cow_sdk_orderbook::{
    Amount, CompetitionOrderStatus, Order, OrderCreation, OrderKind, OrderQuoteResponse,
    OrderQuoteSide, QuoteData, SigningScheme, TotalSurplus, Trade,
};

mod common;

use common::{address, app_data_hash};

const ADDRESS_1: &str = "0x0000000000000000000000000000000000000001";
const ADDRESS_2: &str = "0x0000000000000000000000000000000000000002";
const ADDRESS_3: &str = "0x0000000000000000000000000000000000000003";
const APP_DATA: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const ORDER_UID: &str = "0x59920c85de0162e9e55df8d396e75f3b6b7c2dfdb535f03e5c807731c31585eaff714b8b0e2700303ec912bd40496c3997ceea2b616d6710";
const SIGNATURE: &str = "0x1234";

fn assert_wire_roundtrip<T>(wire: &str)
where
    T: DeserializeOwned + Serialize,
{
    let typed = serde_json::from_str::<T>(wire).expect("wire fixture must deserialize");
    let serialized = serde_json::to_string(&typed).expect("typed DTO must serialize");

    assert_eq!(serialized, wire);
}

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("test amount literal must be valid")
}

#[test]
fn promoted_amount_dtos_roundtrip_byte_identical() {
    assert_wire_roundtrip::<OrderQuoteSide>(
        r#"{"kind":"sell","sellAmountBeforeFee":"1000000000000000000"}"#,
    );

    assert_wire_roundtrip::<OrderQuoteSide>(
        r#"{"kind":"buy","buyAmountAfterFee":"2000000000000000000"}"#,
    );

    assert_wire_roundtrip::<QuoteData>(
        r#"{"sellToken":"0x0000000000000000000000000000000000000001","buyToken":"0x0000000000000000000000000000000000000002","receiver":"0x0000000000000000000000000000000000000003","sellAmount":"1000000000000000000","buyAmount":"2000000000000000000","validTo":1700000000,"appData":"0x0000000000000000000000000000000000000000000000000000000000000000","feeAmount":"300000000000000000","kind":"sell","partiallyFillable":true,"sellTokenBalance":"erc20","buyTokenBalance":"internal","gasAmount":"150000","gasPrice":"15000000000","sellTokenPrice":"400000000000000","signingScheme":"eip712"}"#,
    );

    assert_wire_roundtrip::<OrderCreation>(
        r#"{"sellToken":"0x0000000000000000000000000000000000000001","buyToken":"0x0000000000000000000000000000000000000002","receiver":"0x0000000000000000000000000000000000000003","sellAmount":"1000000000000000000","buyAmount":"2000000000000000000","validTo":1700000000,"appData":"{\"version\":\"0.1.0\"}","appDataHash":"0x0000000000000000000000000000000000000000000000000000000000000000","feeAmount":"0","kind":"buy","partiallyFillable":true,"sellTokenBalance":"external","buyTokenBalance":"internal","signingScheme":"eip712","signature":"0x1234","from":"0x0000000000000000000000000000000000000003","quoteId":7}"#,
    );

    assert_wire_roundtrip::<Order>(
        r#"{"sellToken":"0x0000000000000000000000000000000000000001","buyToken":"0x0000000000000000000000000000000000000002","receiver":"0x0000000000000000000000000000000000000003","sellAmount":"1000000000000000000","buyAmount":"2000000000000000000","validTo":1700000000,"appData":"0x0000000000000000000000000000000000000000000000000000000000000000","feeAmount":"0","kind":"sell","partiallyFillable":true,"sellTokenBalance":"internal","buyTokenBalance":"erc20","signingScheme":"presign","signature":"0x1234","from":"0x0000000000000000000000000000000000000003","quoteId":7,"class":"limit","owner":"0x0000000000000000000000000000000000000003","uid":"0x59920c85de0162e9e55df8d396e75f3b6b7c2dfdb535f03e5c807731c31585eaff714b8b0e2700303ec912bd40496c3997ceea2b616d6710","creationDate":"2020-12-03T18:35:18.814523Z","availableBalance":"999999999999999999","executedSellAmount":"100000000000000000","executedSellAmountBeforeFees":"90000000000000000","executedBuyAmount":"200000000000000000","executedFee":"3000000000000000","executedFeeAmount":"4000000000000000","invalidated":false,"status":"fulfilled","onchainUser":"0x0000000000000000000000000000000000000001","ethflowData":{"refundTxHash":"0x1111111111111111111111111111111111111111111111111111111111111111","userValidTo":1700000100},"settlementContract":"0x0000000000000000000000000000000000000003","totalFee":"3000000000000000"}"#,
    );

    assert_wire_roundtrip::<Trade>(
        r#"{"blockNumber":1,"logIndex":2,"orderUid":"0x59920c85de0162e9e55df8d396e75f3b6b7c2dfdb535f03e5c807731c31585eaff714b8b0e2700303ec912bd40496c3997ceea2b616d6710","owner":"0x0000000000000000000000000000000000000003","sellToken":"0x0000000000000000000000000000000000000001","buyToken":"0x0000000000000000000000000000000000000002","sellAmount":"100000000000000000","sellAmountBeforeFees":"90000000000000000","buyAmount":"200000000000000000","txHash":"0x1111111111111111111111111111111111111111111111111111111111111111"}"#,
    );

    assert_wire_roundtrip::<TotalSurplus>(r#"{"totalSurplus":"12345678901234567890"}"#);

    assert_wire_roundtrip::<CompetitionOrderStatus>(
        r#"{"type":"solved","value":[{"solver":"solver-a","executedAmounts":{"sell":"100000000000000000","buy":"200000000000000000"}}]}"#,
    );
}

#[test]
fn promoted_amount_fields_reject_malformed_wire_amounts() {
    let malformed_quote = format!(
        r#"{{"sellToken":"{ADDRESS_1}","buyToken":"{ADDRESS_2}","sellAmount":"not-a-decimal","buyAmount":"1","validTo":1700000000,"appData":"{APP_DATA}","feeAmount":"0","kind":"sell","partiallyFillable":false,"sellTokenBalance":"erc20","buyTokenBalance":"erc20"}}"#
    );
    let quote_error =
        serde_json::from_str::<QuoteData>(&malformed_quote).expect_err("quote must fail");
    assert!(
        quote_error.to_string().contains("amount"),
        "quote error should retain amount context: {quote_error}"
    );

    let malformed_order = format!(
        r#"{{"sellToken":"{ADDRESS_1}","buyToken":"{ADDRESS_2}","receiver":"{ADDRESS_3}","sellAmount":"1","buyAmount":"2","validTo":1700000000,"appData":"{APP_DATA}","feeAmount":"x","kind":"sell","partiallyFillable":false,"sellTokenBalance":"erc20","buyTokenBalance":"erc20","signingScheme":"eip712","signature":"{SIGNATURE}","owner":"{ADDRESS_3}","uid":"{ORDER_UID}","executedSellAmount":"0","executedBuyAmount":"0","invalidated":false,"status":"open","class":"market","totalFee":"0"}}"#
    );
    let order_error = serde_json::from_str::<Order>(&malformed_order).expect_err("order must fail");
    assert!(
        order_error.to_string().contains("amount"),
        "order error should retain amount context: {order_error}"
    );

    let side_error = serde_json::from_str::<OrderQuoteSide>(
        r#"{"kind":"sell","sellAmountBeforeFee":"not-a-decimal"}"#,
    )
    .expect_err("quote side must fail");
    assert!(
        side_error.to_string().contains("amount"),
        "quote-side error should retain amount context: {side_error}"
    );
}

#[test]
fn typed_amount_builders_keep_decimal_string_wire_shape() {
    let quote = QuoteData::new(
        address(ADDRESS_1),
        address(ADDRESS_2),
        amount("1000000000000000000"),
        amount("2000000000000000000"),
        1_700_000_000,
        app_data_hash(APP_DATA),
        OrderKind::Sell,
    )
    .with_network_cost_amount(amount("300000000000000000"));
    let quote_value = serde_json::to_value(&quote).expect("quote must serialize");

    assert_eq!(quote_value["sellAmount"], "1000000000000000000");
    assert_eq!(quote_value["buyAmount"], "2000000000000000000");
    assert_eq!(quote_value["feeAmount"], "300000000000000000");
    assert!(quote_value["sellAmount"].is_string());

    let order = OrderCreation::new(
        address(ADDRESS_1),
        address(ADDRESS_2),
        amount("1000000000000000000"),
        amount("2000000000000000000"),
        1_700_000_000,
        OrderKind::Sell,
        SigningScheme::Eip712,
        SIGNATURE,
        address(ADDRESS_3),
    );
    let order_value = serde_json::to_value(&order).expect("order must serialize");

    assert_eq!(order_value["sellAmount"], "1000000000000000000");
    assert_eq!(order_value["buyAmount"], "2000000000000000000");
    assert_eq!(order_value["feeAmount"], "0");
    assert!(order_value["buyAmount"].is_string());
}

#[test]
fn order_quote_response_amount_fields_deserialize_through_typed_amount() {
    let response: OrderQuoteResponse = serde_json::from_str(include_str!(
        "../../../parity/fixtures/orderbook/order_quote_response.json"
    ))
    .expect("quote response fixture must deserialize");

    assert_eq!(response.quote.sell_amount, amount("1000000000000000000"));
    assert_eq!(response.quote.buy_amount, amount("2000000000000000000"));
    assert_eq!(
        response.quote.network_cost_amount(),
        &amount("3000000000000000")
    );

    let value = serde_json::to_value(&response).expect("quote response must serialize");
    assert_eq!(value["quote"]["sellAmount"], "1000000000000000000");
    assert_eq!(value["quote"]["buyAmount"], "2000000000000000000");
    assert_eq!(value["quote"]["feeAmount"], "3000000000000000");
}
