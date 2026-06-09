#![cfg(not(target_arch = "wasm32"))]

mod common;

use cow_sdk_core::{Address, SupportedChainId};
use cow_sdk_wasm::helpers::{
    app_data, chains,
    dto::{AppDataDocInput, OrderKindDto, TokenBalanceDto, generated_order_uid_dto},
    errors::PureError,
    signing,
};
use serde_json::json;

use crate::common::{
    ADDR_OWNER, ADDR_SELL, APP_DATA_CONTENT, CHAIN_GNOSIS, CHAIN_MAINNET, CHAIN_UNSUPPORTED,
    CID_APP_DATA, CID_APP_DATA_TWO, ECDSA_SIGNATURE, EIP1271_SIGNATURE, HASH_APP_DATA,
    HASH_APP_DATA_TWO, host_app_data_input, host_order_input,
};

#[test]
fn supported_chain_ids_include_expected_core_networks_in_order() {
    let ids = chains::supported_chain_ids();
    assert_eq!(ids.first(), Some(&CHAIN_MAINNET));
    assert!(ids.contains(&CHAIN_GNOSIS));
    assert_eq!(ids.last(), Some(&11_155_111));
}

#[test]
fn supported_chain_parses_mainnet_and_gnosis() {
    assert_eq!(
        chains::supported_chain(CHAIN_MAINNET).unwrap(),
        SupportedChainId::Mainnet
    );
    assert_eq!(
        chains::supported_chain(CHAIN_GNOSIS).unwrap(),
        SupportedChainId::GnosisChain
    );
}

#[test]
fn supported_chain_rejects_unknown_chain_with_typed_error() {
    assert_eq!(
        chains::supported_chain(CHAIN_UNSUPPORTED).unwrap_err(),
        PureError::UnsupportedChain {
            chain_id: CHAIN_UNSUPPORTED
        }
    );
}

#[test]
fn env_parser_defaults_to_production() {
    assert_eq!(
        chains::env_from_str(None).unwrap(),
        cow_sdk_core::CowEnv::Prod
    );
    assert_eq!(
        chains::env_from_str(Some("production")).unwrap(),
        cow_sdk_core::CowEnv::Prod
    );
}

#[test]
fn env_parser_accepts_staging_aliases() {
    assert_eq!(
        chains::env_from_str(Some("staging")).unwrap(),
        cow_sdk_core::CowEnv::Staging
    );
    assert_eq!(
        chains::env_from_str(Some("barn")).unwrap(),
        cow_sdk_core::CowEnv::Staging
    );
}

#[test]
fn env_parser_rejects_unknown_values() {
    assert!(matches!(
        chains::env_from_str(Some("dev")),
        Err(PureError::UnknownEnumValue { field, value }) if field == "env" && value == "dev"
    ));
}

#[test]
fn deployment_addresses_resolve_for_mainnet() {
    let addresses = chains::deployment_addresses(CHAIN_MAINNET, None).unwrap();
    assert!(addresses.settlement.starts_with("0x"));
    assert!(addresses.vault_relayer.starts_with("0x"));
    assert!(addresses.eth_flow.starts_with("0x"));
}

#[test]
fn deployment_addresses_reject_unsupported_chain() {
    assert!(matches!(
        chains::deployment_addresses(CHAIN_UNSUPPORTED, None),
        Err(PureError::UnsupportedChain { chain_id }) if chain_id == CHAIN_UNSUPPORTED
    ));
}

#[test]
fn domain_separator_is_stable_hex() {
    let separator = chains::domain_separator(CHAIN_MAINNET).unwrap();
    assert_eq!(separator.len(), 66);
    assert!(separator.starts_with("0x"));
}

#[test]
fn order_input_parses_to_unsigned_order() {
    let order = host_order_input().to_unsigned_order().unwrap();
    assert_eq!(order.sell_token.to_hex_string(), ADDR_SELL);
    assert_eq!(order.kind, cow_sdk_core::OrderKind::Sell);
}

#[test]
fn order_input_rejects_malformed_address() {
    let mut input = host_order_input();
    input.sell_token = "0x1234".to_owned();
    assert!(matches!(
        input.to_unsigned_order(),
        Err(PureError::InvalidInput { field, .. }) if field == "sellToken"
    ));
}

#[test]
fn order_input_rejects_external_buy_balance() {
    let mut input = host_order_input();
    input.buy_token_balance = TokenBalanceDto::External;
    assert!(matches!(
        input.to_unsigned_order(),
        Err(PureError::UnknownEnumValue { field, value })
            if field == "buyTokenBalance" && value == "external"
    ));
}

#[test]
fn token_balance_maps_to_expected_sell_sources() {
    assert_eq!(
        serde_json::to_string(&TokenBalanceDto::Erc20).unwrap(),
        "\"erc20\""
    );
    assert_eq!(
        serde_json::to_string(&TokenBalanceDto::External).unwrap(),
        "\"external\""
    );
    assert_eq!(
        serde_json::to_string(&TokenBalanceDto::Internal).unwrap(),
        "\"internal\""
    );
}

#[test]
fn order_kind_serde_shape_is_lowercase() {
    assert_eq!(
        serde_json::to_string(&OrderKindDto::Sell).unwrap(),
        "\"sell\""
    );
    assert_eq!(
        serde_json::to_string(&OrderKindDto::Buy).unwrap(),
        "\"buy\""
    );
}

#[test]
fn app_data_doc_input_requires_object_metadata() {
    let input = AppDataDocInput {
        app_code: "CoW Swap".to_owned(),
        metadata: json!("not an object"),
        version: "0.7.0".to_owned(),
        environment: None,
    };
    assert!(matches!(
        app_data::document_from_input(input),
        Err(PureError::InvalidInput { field, .. }) if field == "metadata"
    ));
}

#[test]
fn app_data_info_returns_canonical_hash_content_and_cid() {
    let doc = app_data::document_from_input(host_app_data_input()).unwrap();
    let info = app_data::app_data_info(&doc).unwrap();
    assert_eq!(info.app_data_hex, HASH_APP_DATA);
    assert_eq!(info.cid, CID_APP_DATA);
    assert_eq!(info.app_data_content, APP_DATA_CONTENT);
}

#[test]
fn app_data_doc_validation_reports_success() {
    let doc = app_data::document_from_input(host_app_data_input()).unwrap();
    let result = app_data::validate_app_data_doc(&doc);
    assert!(result.success);
    assert!(result.errors.is_none());
}

#[test]
fn app_data_hex_and_cid_round_trip_for_two_vectors() {
    assert_eq!(
        app_data::app_data_hex_to_cid(HASH_APP_DATA).unwrap(),
        CID_APP_DATA
    );
    assert_eq!(
        app_data::cid_to_app_data_hex(CID_APP_DATA).unwrap(),
        HASH_APP_DATA
    );
    assert_eq!(
        app_data::app_data_hex_to_cid(HASH_APP_DATA_TWO).unwrap(),
        CID_APP_DATA_TWO
    );
    assert_eq!(
        app_data::cid_to_app_data_hex(CID_APP_DATA_TWO).unwrap(),
        HASH_APP_DATA_TWO
    );
}

#[test]
fn typed_data_payload_matches_signing_module_output() {
    let order = host_order_input().to_unsigned_order().unwrap();
    let chain = chains::supported_chain(CHAIN_MAINNET).unwrap();
    let wasm_payload = signing::order_typed_data_payload(chain, &order).unwrap();
    let native_payload =
        cow_sdk_signing::domain::order_typed_data_payload(chain, &order, None).unwrap();
    assert_eq!(wasm_payload.domain, native_payload.domain);
    assert_eq!(wasm_payload.primary_type, native_payload.primary_type);
    assert_eq!(wasm_payload.message_json(), native_payload.message_json());
}

#[test]
fn generated_order_uid_uses_canonical_strings() {
    let order = host_order_input().to_unsigned_order().unwrap();
    let chain = chains::supported_chain(CHAIN_MAINNET).unwrap();
    let owner = Address::new(ADDR_OWNER).unwrap();
    let generated = signing::generate_order_id(chain, &order, &owner).unwrap();
    let dto = generated_order_uid_dto(&generated);
    assert_eq!(dto.order_uid, generated.order_id.to_hex_string());
    assert_eq!(dto.order_digest, generated.order_digest.to_hex_string());
    assert_eq!(dto.order_uid.len(), 114);
    assert_eq!(dto.order_digest.len(), 66);
}

#[test]
fn eip1271_payload_matches_signing_module_output_and_vector() {
    let order = host_order_input().to_unsigned_order().unwrap();
    let wasm_payload = signing::eip1271_signature_payload(&order, ECDSA_SIGNATURE).unwrap();
    let native_payload =
        cow_sdk_signing::eip1271_signature_payload(&order, ECDSA_SIGNATURE).unwrap();
    assert_eq!(wasm_payload, native_payload);
    assert_eq!(wasm_payload, EIP1271_SIGNATURE);
}

#[test]
fn pure_error_display_strings_are_stable() {
    let invalid = PureError::InvalidInput {
        field: "sellToken".to_owned(),
        message: "invalid address".to_owned(),
    };
    let unknown = PureError::UnknownEnumValue {
        field: "kind".to_owned(),
        value: "swap".to_owned(),
    };
    let unsupported = PureError::UnsupportedChain { chain_id: 13_337 };

    assert_eq!(
        invalid.to_string(),
        "invalid input for sellToken: invalid address"
    );
    assert_eq!(unknown.to_string(), "unknown enum value 'swap' for kind");
    assert_eq!(unsupported.to_string(), "unsupported chain id: 13337");
}

#[test]
fn wasm_version_matches_package_version() {
    assert_eq!(chains::wasm_version(), env!("CARGO_PKG_VERSION"));
}
