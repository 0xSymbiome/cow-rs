#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy_signer::LocalAlloyKeystoreSigner;
use cow_sdk_contracts::{Order as ContractsOrder, SigningScheme, hash_order};
use cow_sdk_core::{
    Address, Amount, AppDataHash, AsyncSigner, BuyTokenDestination, Hash32, OrderKind,
    SellTokenSource, SupportedChainId, UnsignedOrder,
};
use cow_sdk_signing::{ORDER_PRIMARY_TYPE, get_domain, order_typed_data_payload};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const EXPECTED_ORDER_SIGNATURE: &str = "0x34bc8d9249f7f9399d1db57b96bfc3a2f935a25965fe265292142c305284c7241daf1b3049bc75da81012cf33aeac1de09ec5684bccf03afe7274262703780d01c";

#[tokio::test]
async fn sign_typed_data_payload_matches_canonical_cow_order_vector() {
    let signer = LocalAlloyKeystoreSigner::builder()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .unwrap();
    let order = sample_order();
    let payload = order_typed_data_payload(SupportedChainId::Mainnet, &order, None).unwrap();

    assert_eq!(payload.primary_type, ORDER_PRIMARY_TYPE);

    let signature = signer.sign_typed_data_payload(&payload).await.unwrap();
    let digest = order_digest(&order);
    let recovered = cow_sdk_contracts::Signature::Ecdsa {
        scheme: SigningScheme::Eip712,
        data: signature.clone(),
    }
    .recover_ecdsa_address(&digest)
    .unwrap();

    assert_eq!(signature, EXPECTED_ORDER_SIGNATURE);
    assert_eq!(
        recovered,
        signer.get_address().await.unwrap(),
        "canonical EIP-712 signature must recover to the local signer"
    );
}

fn order_digest(order: &UnsignedOrder) -> Hash32 {
    hash_order(
        &get_domain(SupportedChainId::Mainnet, None).unwrap(),
        &ContractsOrder::new(
            order.sell_token,
            order.buy_token,
            Some(order.receiver),
            order.sell_amount.clone(),
            order.buy_amount.clone(),
            order.valid_to,
            order.app_data.clone(),
            order.fee_amount.clone(),
            order.kind,
            order.partially_fillable,
            Some(order.sell_token_balance),
            Some(order.buy_token_balance),
        ),
    )
    .unwrap()
}

fn sample_order() -> UnsignedOrder {
    UnsignedOrder::new(
        Address::new("0xd057b63f5e69cf1b929b356b579cba08d7688048").unwrap(),
        Address::new("0x7b878668cd1a3adf89764d3a331e0a7bb832192d").unwrap(),
        Address::new("0xa6ddbd0de6b310819b49f680f65871bee85f517e").unwrap(),
        Amount::new("500000000000000").unwrap(),
        Amount::new("23000020000").unwrap(),
        5_000_222,
        AppDataHash::new("0x0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap(),
        Amount::new("2300000").unwrap(),
        OrderKind::Sell,
        true,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    )
}
