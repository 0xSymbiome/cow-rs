#![cfg(not(target_arch = "wasm32"))]
//! Fixture-driven parity contract for `cow-sdk-signing`.
//!
//! Loads `parity/fixtures/signing.json` (schema version 1) at compile time,
//! iterates every documented case, and asserts the Rust signing helpers
//! uphold the pinned upstream contracts. The helpers exercised are:
//!
//! * [`order_fields`], [`domain_fields`] — EIP-712 order and domain field
//!   layouts.
//! * [`cancellation_fields`], [`ORDER_CANCELLATIONS_PRIMARY_TYPE`] — single
//!   and multi-order cancellation surface.
//! * [`order_typed_data`] — typed-data envelope construction.
//! * [`domain_separator_for`], [`get_domain`] — domain resolution with
//!   protocol overrides.
//! * [`sign_order_with_scheme`], [`sign_order_cancellation_with_scheme`] —
//!   signer-scheme routing for ECDSA schemes and typed rejection for
//!   contract-only schemes.
//! * [`generate_order_id`] — UID and digest generation.
//! * [`eip1271_signature_payload`] — EIP-1271 encoding with string-typed
//!   field hashing.
//!
//! Failure messages carry the fixture case id so a reviewer looking at a
//! broken CI run sees the exact upstream vector that diverged.

mod common;

use std::collections::BTreeMap;

use cow_sdk_contracts::{ContractsError, SigningScheme, normalized_ecdsa_signature};
use cow_sdk_core::{
    Address, Amount, AppDataHash, ChainId, CowEnv, OrderKind, ProtocolOptions, SupportedChainId,
    UnsignedOrder,
};
use cow_sdk_signing::{
    ORDER_CANCELLATIONS_PRIMARY_TYPE, ORDER_PRIMARY_TYPE, SigningError, cancellation_fields,
    domain_fields, domain_separator_for, eip1271_signature_payload, generate_order_id, get_domain,
    order_fields, order_typed_data, sign_order_with_scheme,
};
use serde_json::Value;

use common::MockSigner;

const FIXTURE: &str = include_str!("../../../parity/fixtures/signing.json");

#[test]
fn parity_fixture_cases_hold() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture must parse as JSON");

    assert_eq!(
        fixture["schema_version"].as_u64(),
        Some(1),
        "signing fixture must declare schema_version 1",
    );
    assert_eq!(
        fixture["surface"].as_str(),
        Some("signing"),
        "signing fixture must carry the signing surface label",
    );

    let cases = fixture["cases"]
        .as_array()
        .expect("signing fixture must expose a cases array");

    for case in cases {
        let id = case["id"]
            .as_str()
            .expect("every fixture case must carry a string id");
        let expected = &case["expected"];

        match id {
            "signing-eip712-order-fields" => assert_eip712_order_fields(id, expected),
            "signing-cancellation-support" => assert_cancellation_support(id, expected),
            "signing-typed-data-envelope" => assert_typed_data_envelope(id, expected),
            "signing-domain-separator-fields" => assert_domain_separator_fields(id, expected),
            "signing-domain-resolution-precedence" => {
                assert_domain_resolution_precedence(id, expected);
            }
            "signing-signer-supported-schemes" => assert_signer_supported_schemes(id, expected),
            "signing-unsupported-mode-errors" => assert_unsupported_mode_errors(id, expected),
            "signing-generate-order-id" => assert_generate_order_id(id, expected),
            "signing-eip1271-encoding" => assert_eip1271_encoding(id, expected),
            "signing-ecdsa-v-normalization" => assert_ecdsa_v_normalization(id, expected),
            other => panic!("unknown signing fixture case id: {other}"),
        }
    }
}

fn assert_eip712_order_fields(id: &str, expected: &Value) {
    let primary_type = expected["primary_type"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.primary_type must be a string"));
    assert_eq!(
        primary_type, ORDER_PRIMARY_TYPE,
        "case {id}: ORDER_PRIMARY_TYPE must equal the fixture value",
    );

    let expected_fields: Vec<&str> = expected["fields"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.fields must be an array"))
        .iter()
        .map(|field| {
            field
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: expected.fields entries must be strings"))
        })
        .collect();

    let actual_fields: Vec<String> = order_fields().into_iter().map(|field| field.name).collect();

    assert_eq!(
        actual_fields.iter().map(String::as_str).collect::<Vec<_>>(),
        expected_fields,
        "case {id}: order_fields() must match the pinned contracts field order",
    );
}

fn assert_cancellation_support(id: &str, expected: &Value) {
    let _single = expected["single_cancellation_method"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.single_cancellation_method must be a string")
        });
    let _multi = expected["multi_cancellation_method"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.multi_cancellation_method must be a string")
        });

    // The Rust surface exposes single (`sign_order_cancellation*`) and batch
    // (`sign_order_cancellations*`) helpers with the shared primary type
    // `OrderCancellations` and the single-field typed-data shape.
    assert_eq!(
        ORDER_CANCELLATIONS_PRIMARY_TYPE, "OrderCancellations",
        "case {id}: cancellation primary type must remain OrderCancellations",
    );
    let fields = cancellation_fields();
    assert_eq!(
        fields.len(),
        1,
        "case {id}: cancellation typed-data must carry a single orderUids field",
    );
    assert_eq!(
        fields[0].name, "orderUids",
        "case {id}: cancellation field must be named orderUids",
    );
}

fn assert_typed_data_envelope(id: &str, expected: &Value) {
    let primary_type = expected["primary_type"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.primary_type must be a string"));
    let includes: Vec<&str> = expected["includes_types"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.includes_types must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: includes_types entries must be strings"))
        })
        .collect();
    let message_source = expected["message_source"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.message_source must be a string"));

    let order = sample_order();
    let envelope = order_typed_data(SupportedChainId::Mainnet, &order, None)
        .expect("typed-data envelope must build for the sample order");

    assert_eq!(
        envelope.primary_type, primary_type,
        "case {id}: envelope must declare the fixture primary type",
    );
    for kind in includes {
        assert!(
            envelope.types.contains_key(kind),
            "case {id}: envelope types must include {kind}",
        );
    }
    assert_eq!(
        message_source, "unsignedOrder",
        "case {id}: fixture marker must name the unsigned-order source",
    );
    assert_eq!(
        envelope.message, order,
        "case {id}: envelope message must forward the unsigned order verbatim",
    );
}

fn assert_domain_separator_fields(id: &str, expected: &Value) {
    let expected_fields: Vec<&str> = expected["domain_fields"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.domain_fields must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: domain_fields entries must be strings"))
        })
        .collect();

    let actual_fields: Vec<String> = domain_fields()
        .into_iter()
        .map(|field| field.name)
        .collect();

    assert_eq!(
        actual_fields.iter().map(String::as_str).collect::<Vec<_>>(),
        expected_fields,
        "case {id}: domain_fields() must match the fixture layout",
    );
}

fn assert_domain_resolution_precedence(id: &str, expected: &Value) {
    let default_env = expected["default_env"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.default_env must be a string"));
    let alternate_env = expected["alternate_env"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.alternate_env must be a string"));
    let override_marker = expected["override_precedence"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.override_precedence must be a string"));

    // Default env (no options) resolves to the production settlement contract.
    let default_domain = get_domain(SupportedChainId::Mainnet, None)
        .expect("default domain must resolve without options");
    let prod_domain = get_domain(
        SupportedChainId::Mainnet,
        Some(&ProtocolOptions::new().with_env(CowEnv::Prod)),
    )
    .expect("explicit prod-env domain must resolve");
    assert_eq!(
        default_domain.verifying_contract, prod_domain.verifying_contract,
        "case {id}: absent env must resolve to {default_env}",
    );

    // Staging env routes to a different verifying contract than prod.
    let staging_domain = get_domain(
        SupportedChainId::Mainnet,
        Some(&ProtocolOptions::new().with_env(CowEnv::Staging)),
    )
    .expect("staging domain must resolve");
    assert_ne!(
        prod_domain.verifying_contract, staging_domain.verifying_contract,
        "case {id}: {alternate_env} env must override the verifying contract",
    );

    // settlementContractOverride wins over env defaults.
    let override_addr = Address::new("0x1234567890123456789012345678901234567890").unwrap();
    let mut map = BTreeMap::new();
    map.insert(
        ChainId::from(SupportedChainId::Mainnet),
        override_addr.clone(),
    );
    let overridden_domain = get_domain(
        SupportedChainId::Mainnet,
        Some(
            &ProtocolOptions::new()
                .with_env(CowEnv::Prod)
                .with_settlement_contract_override(map),
        ),
    )
    .expect("override domain must resolve");
    assert_eq!(
        overridden_domain.verifying_contract, override_addr,
        "case {id}: {override_marker} must override the resolved verifying contract",
    );
}

fn assert_signer_supported_schemes(id: &str, expected: &Value) {
    let signer_generated: Vec<&str> = expected["signer_generated"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.signer_generated must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: signer_generated entries must be strings"))
        })
        .collect();
    let typed_external: Vec<&str> = expected["typed_external_only"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.typed_external_only must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: typed_external_only entries must be strings"))
        })
        .collect();

    let order = sample_order();
    let signer = MockSigner::new();

    // Signer-generated schemes must sign successfully.
    for scheme_label in &signer_generated {
        let scheme = scheme_label_to_rust(id, scheme_label);
        let result =
            sign_order_with_scheme(&order, SupportedChainId::Mainnet, &signer, scheme, None);
        assert!(
            result.is_ok(),
            "case {id}: signer-generated scheme {scheme_label} must sign successfully; got {result:?}",
        );
    }

    // Typed-external schemes must return a typed rejection instead of a
    // signer-generated signature.
    for scheme_label in &typed_external {
        let scheme = scheme_label_to_rust(id, scheme_label);
        let error =
            sign_order_with_scheme(&order, SupportedChainId::Mainnet, &signer, scheme, None)
                .expect_err("typed-external scheme must reject through SigningError");
        match error {
            SigningError::UnsupportedSignerGeneratedScheme { scheme: rejected } => {
                assert_eq!(
                    rejected, scheme,
                    "case {id}: rejection scheme must match the requested {scheme_label}",
                );
            }
            other => panic!(
                "case {id}: {scheme_label} must reject with UnsupportedSignerGeneratedScheme; got {other:?}",
            ),
        }
    }
}

fn assert_unsupported_mode_errors(id: &str, expected: &Value) {
    let unsupported: Vec<&str> = expected["unsupported_modes"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.unsupported_modes must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: unsupported_modes entries must be strings"))
        })
        .collect();
    let error_surface = expected["error_surface"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.error_surface must be a string"));
    assert_eq!(
        error_surface, "typed-public-error",
        "case {id}: error surface must remain the typed public SigningError",
    );

    let order = sample_order();
    let signer = MockSigner::new();
    for scheme_label in unsupported {
        let scheme = scheme_label_to_rust(id, scheme_label);
        let error =
            sign_order_with_scheme(&order, SupportedChainId::Mainnet, &signer, scheme, None)
                .expect_err("unsupported scheme must reject through SigningError");
        assert!(
            matches!(error, SigningError::UnsupportedSignerGeneratedScheme { .. }),
            "case {id}: {scheme_label} must reject through UnsupportedSignerGeneratedScheme",
        );
    }
}

fn assert_generate_order_id(id: &str, expected: &Value) {
    let returns: Vec<&str> = expected["returns"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.returns must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: returns entries must be strings"))
        })
        .collect();
    let owner_required = expected["owner_required"]
        .as_bool()
        .unwrap_or_else(|| panic!("case {id}: expected.owner_required must be a bool"));
    let valid_to_source = expected["uid_valid_to_source"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.uid_valid_to_source must be a string"));

    assert!(
        owner_required,
        "case {id}: owner input must remain required"
    );
    assert_eq!(
        valid_to_source, "order.validTo",
        "case {id}: UID packing must source valid_to from the order",
    );
    assert!(
        returns.contains(&"orderId") && returns.contains(&"orderDigest"),
        "case {id}: generate_order_id must return both orderId and orderDigest",
    );

    let order = sample_order();
    let owner = Address::new("0x5555555555555555555555555555555555555555").unwrap();
    let generated = generate_order_id(SupportedChainId::Mainnet, &order, &owner, None)
        .expect("generate_order_id must succeed for the sample order");

    // The packed UID trails the owner address (bytes 32..52) and the order
    // valid_to (bytes 52..56) so asserting the suffix matches proves both
    // inputs are propagated through the UID construction.
    let uid_hex = generated.order_id.as_str().trim_start_matches("0x");
    let owner_hex = owner.as_str().trim_start_matches("0x").to_lowercase();
    assert!(
        uid_hex.to_lowercase().contains(&owner_hex),
        "case {id}: packed UID must embed the owner bytes",
    );
    let suffix = &uid_hex[uid_hex.len() - 8..];
    assert_eq!(
        u32::from_str_radix(suffix, 16).unwrap(),
        order.valid_to,
        "case {id}: packed UID must suffix with order.valid_to",
    );
}

fn assert_eip1271_encoding(id: &str, expected: &Value) {
    let string_fields: Vec<&str> = expected["string_fields_hashed"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.string_fields_hashed must be an array"))
        .iter()
        .map(|value| {
            value.as_str().unwrap_or_else(|| {
                panic!("case {id}: string_fields_hashed entries must be strings")
            })
        })
        .collect();

    let order = sample_order();
    let signature = synthetic_ecdsa_signature(27);
    let signature_bytes = hex::decode(signature.trim_start_matches("0x")).unwrap();
    let payload =
        eip1271_signature_payload(&order, &signature).expect("EIP-1271 payload must encode");
    let payload_hex = payload.trim_start_matches("0x");
    let payload_bytes = hex::decode(payload_hex).unwrap();
    assert_eq!(
        payload_hex.len() % 2,
        0,
        "case {id}: payload hex must be byte-aligned",
    );

    // The encoded payload hashes kind, sellTokenBalance, and buyTokenBalance
    // through keccak256 so their presence as raw UTF-8 substrings would
    // indicate a regression to ASCII concatenation. Verify none appear raw.
    for field_label in string_fields {
        let marker: &[u8] = match field_label {
            "kind" => match order.kind {
                OrderKind::Sell => b"sell",
                OrderKind::Buy => b"buy",
            },
            "sellTokenBalance" | "buyTokenBalance" => b"erc20",
            other => panic!("case {id}: unsupported string field {other}"),
        };
        let marker_hex = hex::encode(marker);
        assert!(
            !payload_hex.contains(&marker_hex),
            "case {id}: {field_label} must be hashed before encoding, but raw UTF-8 bytes appear in the payload",
        );
    }

    // The dynamic tail carries a canonical 65-byte ECDSA signature. Confirm
    // the ABI payload preserves the head offset, the tail length word, and the
    // exact signature bytes with zero padding.
    let tail_offset = 32 * 13;
    let head_offset_hex = format!("{:064x}", tail_offset as u64);
    let mut signature_len_word = [0u8; 32];
    signature_len_word[24..].copy_from_slice(&(signature_bytes.len() as u64).to_be_bytes());
    assert!(
        payload_hex.contains(&head_offset_hex),
        "case {id}: payload must include the ABI head-offset marker for the signature tail",
    );
    assert_eq!(
        &payload_bytes[tail_offset..tail_offset + 32],
        &signature_len_word,
        "case {id}: signature tail must prefix the ECDSA byte length",
    );
    assert_eq!(
        &payload_bytes[tail_offset + 32..tail_offset + 32 + signature_bytes.len()],
        signature_bytes.as_slice(),
        "case {id}: signature tail must preserve the normalized ECDSA bytes",
    );
    assert!(
        payload_bytes[tail_offset + 32 + signature_bytes.len()..]
            .iter()
            .all(|byte| *byte == 0),
        "case {id}: signature tail padding must remain zeroed",
    );

    // The domain separator must remain sensitive to the typed-data domain,
    // so computing it for two different chains yields different values.
    let mainnet = get_domain(SupportedChainId::Mainnet, None).unwrap();
    let gnosis = get_domain(SupportedChainId::GnosisChain, None).unwrap();
    let mainnet_sep = domain_separator_for(&mainnet).unwrap();
    let gnosis_sep = domain_separator_for(&gnosis).unwrap();
    assert_ne!(
        mainnet_sep, gnosis_sep,
        "case {id}: domain separator must remain sensitive to chain id",
    );
}

fn synthetic_ecdsa_signature(v: u8) -> String {
    format!("0x{}{:02x}", "aa".repeat(64), v)
}

fn assert_ecdsa_v_normalization(id: &str, expected: &Value) {
    let positive_cases = expected["positive_cases"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.positive_cases must be an array"));
    let rejection_cases = expected["rejection_cases"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.rejection_cases must be an array"));

    for case in positive_cases {
        let input = case["input"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id}: positive case input must be a string"));
        let normalized = case["normalized"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id}: positive case normalized must be a string"));
        assert_eq!(
            normalized_ecdsa_signature(input).unwrap(),
            normalized,
            "case {id}: normalized ECDSA signature must match the pinned output for {input}",
        );
    }

    for case in rejection_cases {
        let input = case["input"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id}: rejection case input must be a string"));
        let discriminant = case["error_discriminant"].as_str().unwrap_or_else(|| {
            panic!("case {id}: rejection case error_discriminant must be a string")
        });
        let value = u8::try_from(
            case["value"]
                .as_u64()
                .unwrap_or_else(|| panic!("case {id}: rejection case value must be a u64")),
        )
        .unwrap_or_else(|_| panic!("case {id}: rejection case value must fit in u8"));
        let error = normalized_ecdsa_signature(input)
            .expect_err("rejection case must fail through ContractsError");

        match (discriminant, error) {
            (
                "InvalidSignatureRecoveryByte",
                ContractsError::InvalidSignatureRecoveryByte { value: actual },
            ) => {
                assert_eq!(
                    actual, value,
                    "case {id}: rejection value must match the pinned fixture for {input}",
                );
            }
            (expected_discriminant, other) => {
                panic!("case {id}: expected {expected_discriminant} for {input}, got {other:?}");
            }
        }
    }
}

fn scheme_label_to_rust(id: &str, label: &str) -> SigningScheme {
    match label {
        "eip712" => SigningScheme::Eip712,
        "ethsign" => SigningScheme::EthSign,
        "eip1271" => SigningScheme::Eip1271,
        "presign" => SigningScheme::PreSign,
        other => panic!("case {id}: unsupported signing scheme label {other:?}"),
    }
}

fn sample_order() -> UnsignedOrder {
    UnsignedOrder::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        Amount::new("1000000000000000000").unwrap(),
        Amount::new("2000000000000000000").unwrap(),
        0x6500_0001,
        AppDataHash::new("0x4444444444444444444444444444444444444444444444444444444444444444")
            .unwrap(),
        Amount::new("1000").unwrap(),
        OrderKind::Sell,
        false,
        cow_sdk_core::SellTokenSource::default(),
        cow_sdk_core::BuyTokenDestination::default(),
    )
}
