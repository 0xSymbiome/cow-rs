mod common;

use cow_sdk_contracts::{Order as ContractsOrder, OrderUidParams, SigningScheme, hash_order};
use cow_sdk_core::{Address, SupportedChainId};
use cow_sdk_signing::{
    GeneratedOrderId, ORDER_PRIMARY_TYPE, SigningError, generate_order_id, get_domain,
    order_typed_data, sign_order, sign_order_async, sign_order_with_scheme,
    sign_order_with_scheme_async,
};

use common::{MockSigner, fixture_case, sample_order};

#[test]
fn order_typed_data_matches_fixture_contract_and_consumer_shape() {
    let order = sample_order();
    let typed = order_typed_data(SupportedChainId::Mainnet, &order, None).unwrap();
    let fields_case = fixture_case("signing-eip712-order-fields");
    let typed_data_case = fixture_case("signing-typed-data-envelope");

    assert_eq!(typed.primary_type, ORDER_PRIMARY_TYPE);
    assert_eq!(
        typed.types["Order"]
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        fields_case["expected"]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| field.as_str().unwrap())
            .collect::<Vec<_>>()
    );
    let actual_type_names = typed.types.keys().map(String::as_str).collect::<Vec<_>>();
    let expected_type_names = typed_data_case["expected"]["includes_types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<Vec<_>>();
    for type_name in expected_type_names {
        assert!(actual_type_names.contains(&type_name));
    }
    assert_eq!(typed.message, order);
}

#[test]
fn sign_order_uses_typed_data_for_eip712_and_digest_for_ethsign() {
    let order = sample_order();
    let signer = MockSigner::new();

    let typed_result = sign_order(&order, SupportedChainId::Sepolia, &signer, None).unwrap();
    assert_eq!(typed_result.signing_scheme, SigningScheme::Eip712);
    assert_eq!(typed_result.signature, signer.typed_data_signature);
    assert_eq!(signer.calls.borrow().typed_data.len(), 1);
    assert!(signer.calls.borrow().messages.is_empty());

    let ethsign_result = sign_order_with_scheme(
        &order,
        SupportedChainId::Sepolia,
        &signer,
        SigningScheme::EthSign,
        None,
    )
    .unwrap();
    assert_eq!(ethsign_result.signing_scheme, SigningScheme::EthSign);
    assert_eq!(ethsign_result.signature, signer.message_signature);

    let expected_digest = hash_order(
        &get_domain(SupportedChainId::Sepolia, None).unwrap(),
        &contracts_order(&order),
    )
    .unwrap();

    assert_eq!(
        format!("0x{}", hex::encode(&signer.calls.borrow().messages[0])),
        expected_digest.as_str()
    );
}

#[tokio::test]
async fn async_sign_order_paths_match_sync_signing_behavior() {
    let order = sample_order();
    let signer = MockSigner::new();

    let typed_result = sign_order_async(&order, SupportedChainId::Sepolia, &signer, None)
        .await
        .unwrap();
    assert_eq!(typed_result.signing_scheme, SigningScheme::Eip712);
    assert_eq!(typed_result.signature, signer.typed_data_signature);

    let ethsign_result = sign_order_with_scheme_async(
        &order,
        SupportedChainId::Sepolia,
        &signer,
        SigningScheme::EthSign,
        None,
    )
    .await
    .unwrap();
    assert_eq!(ethsign_result.signing_scheme, SigningScheme::EthSign);
    assert_eq!(ethsign_result.signature, signer.message_signature);
}

#[test]
fn unsupported_local_signer_modes_fail_with_typed_errors() {
    let order = sample_order();
    let signer = MockSigner::new();

    for scheme in [SigningScheme::Eip1271, SigningScheme::PreSign] {
        let error =
            sign_order_with_scheme(&order, SupportedChainId::Mainnet, &signer, scheme, None)
                .unwrap_err();

        assert_eq!(
            error,
            SigningError::UnsupportedSignerGeneratedScheme { scheme }
        );
    }

    assert!(signer.calls.borrow().typed_data.is_empty());
    assert!(signer.calls.borrow().messages.is_empty());
}

#[test]
fn generate_order_id_reuses_contract_hashing_and_uid_packing() {
    let order = sample_order();
    let owner = Address::new("0x1111111111111111111111111111111111111111").unwrap();

    let generated = generate_order_id(SupportedChainId::Sepolia, &order, &owner, None).unwrap();
    let expected_digest = hash_order(
        &get_domain(SupportedChainId::Sepolia, None).unwrap(),
        &contracts_order(&order),
    )
    .unwrap();
    let expected_uid = cow_sdk_contracts::pack_order_uid_params(&OrderUidParams {
        order_digest: expected_digest.clone(),
        owner: owner.clone(),
        valid_to: order.valid_to,
    })
    .unwrap();

    assert_eq!(
        generated,
        GeneratedOrderId {
            order_id: expected_uid,
            order_digest: expected_digest,
        }
    );
}

fn contracts_order(order: &cow_sdk_core::UnsignedOrder) -> ContractsOrder {
    ContractsOrder {
        sell_token: order.sell_token.clone(),
        buy_token: order.buy_token.clone(),
        receiver: Some(order.receiver.clone()),
        sell_amount: order.sell_amount.clone(),
        buy_amount: order.buy_amount.clone(),
        valid_to: order.valid_to,
        app_data: order.app_data.clone(),
        fee_amount: order.fee_amount.clone(),
        kind: order.kind,
        partially_fillable: order.partially_fillable,
        sell_token_balance: Some(order.sell_token_balance),
        buy_token_balance: Some(order.buy_token_balance),
    }
}
