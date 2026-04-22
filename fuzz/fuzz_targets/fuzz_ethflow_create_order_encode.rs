#![no_main]

//! Fuzz target for `CoWSwapEthFlow.createOrder(EthFlowOrderData)` round-trip.
//!
//! Drives arbitrary `EthFlowOrderData` field combinations through the
//! `alloy::sol!`-generated `ICoWSwapEthFlow::createOrderCall` encoder,
//! decodes the produced bytes back through the matching decoder, and
//! asserts the decoded struct is field-wise equal to the input. The
//! invariant proves the ABI encoder and decoder are inverses on every
//! well-typed `EthFlowOrderData` shape.
//!
//! Inputs are derived via [`arbitrary::Arbitrary`] on the nine
//! `EthFlowOrderData` fields. `u128` is used for the three `uint256`
//! amounts because a u128 fully spans the ABI head word of each
//! amount and keeps every run bounded.

use alloy_sol_types::{
    SolCall,
    private::{Address, FixedBytes, U256},
    sol,
};
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};

sol! {
    interface ICoWSwapEthFlow {
        struct EthFlowOrderData {
            address buyToken;
            address receiver;
            uint256 sellAmount;
            uint256 buyAmount;
            bytes32 appData;
            uint256 feeAmount;
            uint32 validTo;
            bool partiallyFillable;
            int64 quoteId;
        }

        function createOrder(EthFlowOrderData order)
            external
            payable
            returns (bytes32 orderHash);
    }
}

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    buy_token: [u8; 20],
    receiver: [u8; 20],
    sell_amount: u128,
    buy_amount: u128,
    app_data: [u8; 32],
    fee_amount: u128,
    valid_to: u32,
    partially_fillable: bool,
    quote_id: i64,
}

fuzz_target!(|input: FuzzInput| {
    let order = ICoWSwapEthFlow::EthFlowOrderData {
        buyToken: Address::from(input.buy_token),
        receiver: Address::from(input.receiver),
        sellAmount: U256::from(input.sell_amount),
        buyAmount: U256::from(input.buy_amount),
        appData: FixedBytes::from(input.app_data),
        feeAmount: U256::from(input.fee_amount),
        validTo: input.valid_to,
        partiallyFillable: input.partially_fillable,
        quoteId: input.quote_id,
    };

    let encoded = ICoWSwapEthFlow::createOrderCall {
        order: order.clone(),
    }
    .abi_encode();
    let decoded = ICoWSwapEthFlow::createOrderCall::abi_decode_validate(&encoded)
        .expect("createOrder call-data must round-trip through the matching decoder");

    assert_eq!(
        decoded.order.buyToken, order.buyToken,
        "buyToken must round-trip",
    );
    assert_eq!(
        decoded.order.receiver, order.receiver,
        "receiver must round-trip",
    );
    assert_eq!(
        decoded.order.sellAmount, order.sellAmount,
        "sellAmount must round-trip",
    );
    assert_eq!(
        decoded.order.buyAmount, order.buyAmount,
        "buyAmount must round-trip",
    );
    assert_eq!(
        decoded.order.appData, order.appData,
        "appData must round-trip",
    );
    assert_eq!(
        decoded.order.feeAmount, order.feeAmount,
        "feeAmount must round-trip",
    );
    assert_eq!(
        decoded.order.validTo, order.validTo,
        "validTo must round-trip",
    );
    assert_eq!(
        decoded.order.partiallyFillable, order.partiallyFillable,
        "partiallyFillable must round-trip",
    );
    assert_eq!(
        decoded.order.quoteId, order.quoteId,
        "quoteId must round-trip",
    );
});
