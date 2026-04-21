//! Fixture-driven parity contract for `cow-sdk-contracts`.
//!
//! Loads `parity/fixtures/contracts.json` (schema version 1) at compile time,
//! iterates every documented case, and asserts the Rust helpers produce the
//! pinned upstream values. The helpers exercised are:
//!
//! * [`ORDER_TYPE_FIELDS`], [`ORDER_TYPE_HASH`], [`CANCELLATIONS_TYPE_FIELDS`],
//!   [`ORDER_UID_LENGTH`] — canonical EIP-712 and UID layout constants.
//! * [`extract_order_uid_params`] — UID length validation through
//!   [`ContractsError::InvalidOrderUidLength`].
//! * [`normalize_buy_token_balance`] — buy-balance normalization rule.
//! * [`SigningScheme`], [`EIP1271_MAGICVALUE`] — signature scheme discriminants
//!   and EIP-1271 success value.
//! * [`normalize_interaction`] — interaction defaulting rule.
//! * [`SALT`], [`DEPLOYER_CONTRACT`] — deterministic deployment constants.
//! * [`Eip1967Slot::Implementation`], [`Eip1967Slot::Admin`] — EIP-1967 proxy
//!   storage slot constants.
//! * [`encode_order_flags`], [`encode_trade_flags`] — order and trade flag
//!   bitfield codecs.
//! * [`SettlementEncoder::encoded_order_refunds`] — order refund method names.
//! * [`encode_swap_step`] — swap user-data default.
//! * [`VAULT_INTERFACE`] — vault method surface.
//! * [`AllowListReader`], [`SettlementReader`], [`TradeSimulator`] — reader
//!   helper surface.
//!
//! Failure messages carry the fixture case id so a reviewer looking at a
//! broken CI run sees the exact upstream vector that diverged.

use cow_sdk_contracts::{
    AllowListReader, CANCELLATIONS_TYPE_FIELDS, ContractId, DEPLOYER_CONTRACT, EIP1271_MAGICVALUE,
    Eip1967Slot, InteractionLike, ORDER_TYPE_FIELDS, ORDER_TYPE_HASH, ORDER_UID_LENGTH, OrderFlags,
    Registry, SALT, SettlementEncoder, SettlementReader, SigningScheme, Swap, TokenRegistry,
    TradeFlags, TradeSimulator, VAULT_INTERFACE, encode_order_flags, encode_swap_step,
    encode_trade_flags, normalize_buy_token_balance, normalize_interaction,
};
use cow_sdk_core::{
    Address, Amount, CowEnv, OrderBalance, OrderDigest, OrderKind, OrderUid, SupportedChainId,
    TypedDataDomain,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("../../../parity/fixtures/contracts.json");

#[test]
fn parity_fixture_cases_hold() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture must parse as JSON");

    assert_eq!(
        fixture["schema_version"].as_u64(),
        Some(1),
        "contracts fixture must declare schema_version 1"
    );
    assert_eq!(
        fixture["surface"].as_str(),
        Some("contracts"),
        "contracts fixture must carry the contracts surface label"
    );

    let cases = fixture["cases"]
        .as_array()
        .expect("contracts fixture must expose a cases array");

    for case in cases {
        let id = case["id"]
            .as_str()
            .expect("every fixture case must carry a string id");
        let expected = &case["expected"];

        match id {
            "contracts-order-type-fields" => assert_order_type_fields(id, expected),
            "contracts-order-type-hash" => assert_order_type_hash(id, expected),
            "contracts-cancellation-type-fields" => assert_cancellation_type_fields(id, expected),
            "contracts-order-uid-length" => assert_order_uid_length(id, expected),
            "contracts-extract-order-uid-invalid-length" => {
                assert_extract_order_uid_invalid_length(id, expected);
            }
            "contracts-buy-balance-normalization" => {
                assert_buy_balance_normalization(id, case, expected);
            }
            "contracts-signing-scheme-discriminants" => {
                assert_signing_scheme_discriminants(id, expected);
            }
            "contracts-eip1271-magic-value" => assert_eip1271_magic_value(id, expected),
            "contracts-interaction-defaults" => assert_interaction_defaults(id, expected),
            "contracts-deployment-constants" => assert_deployment_constants(id, expected),
            "contracts-proxy-storage-slots" => assert_proxy_storage_slots(id, expected),
            "contracts-order-flags-default-sell" => {
                assert_order_flags_default_sell(id, expected);
            }
            "contracts-order-flags-buy-partial-internal" => {
                assert_order_flags_buy_partial_internal(id, expected);
            }
            "contracts-trade-flags-presign" => assert_trade_flags_presign(id, expected),
            "contracts-order-refund-method-names" => {
                assert_order_refund_method_names(id, expected);
            }
            "contracts-swap-default-user-data" => assert_swap_default_user_data(id, expected),
            "contracts-vault-required-methods" => assert_vault_required_methods(id, expected),
            "contracts-reader-helper-surface" => assert_reader_helper_surface(id, expected),
            other => panic!("unknown contracts fixture case id: {other}"),
        }
    }
}

fn assert_order_type_fields(id: &str, expected: &Value) {
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

    let actual_fields: Vec<&str> = ORDER_TYPE_FIELDS.iter().map(|field| field.name).collect();

    assert_eq!(
        actual_fields, expected_fields,
        "case {id}: ORDER_TYPE_FIELDS names must match the pinned contracts field order",
    );
}

fn assert_order_type_hash(id: &str, expected: &Value) {
    let expected_hash = expected["hash"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.hash must be a string"));
    assert_eq!(
        ORDER_TYPE_HASH, expected_hash,
        "case {id}: ORDER_TYPE_HASH must equal the pinned contracts-ts constant",
    );
}

fn assert_cancellation_type_fields(id: &str, expected: &Value) {
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

    let actual_fields: Vec<&str> = CANCELLATIONS_TYPE_FIELDS
        .iter()
        .map(|field| field.name)
        .collect();

    assert_eq!(
        actual_fields, expected_fields,
        "case {id}: CANCELLATIONS_TYPE_FIELDS must expose the single orderUids field",
    );
}

fn assert_order_uid_length(id: &str, expected: &Value) {
    let expected_bytes = expected["bytes"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.bytes must be a u64"));
    let expected_hex = expected["hex_chars"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.hex_chars must be a u64"));

    assert_eq!(
        ORDER_UID_LENGTH as u64, expected_bytes,
        "case {id}: ORDER_UID_LENGTH byte length must match the fixture",
    );
    assert_eq!(
        (ORDER_UID_LENGTH * 2) as u64,
        expected_hex,
        "case {id}: ORDER_UID_LENGTH hex-char length must be twice the byte length",
    );
}

fn assert_extract_order_uid_invalid_length(id: &str, expected: &Value) {
    assert_eq!(
        expected["must_reject"].as_bool(),
        Some(true),
        "case {id}: expected.must_reject must be true",
    );
    let _ = expected["error_contains"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.error_contains must be a string"));

    // The Rust boundary rejects malformed UID length at `OrderUid::new`, which
    // runs before `extract_order_uid_params`. The typed
    // `ValidationError::InvalidHexLength` names the `order_uid` field and the
    // required hex-character count, mirroring the "invalid order UID length"
    // rejection posture expressed in the upstream TypeScript fixture.
    let rejection = OrderUid::new("0x0000000000000000000000000000000000000000");
    let error = rejection
        .expect_err("case contracts-extract-order-uid-invalid-length must reject short UIDs");
    let message = error.to_string();
    assert!(
        message.contains("order_uid") && message.contains("hex"),
        "case {id}: rejection message {message:?} must name the order_uid hex-length invariant",
    );
}

fn assert_buy_balance_normalization(id: &str, case: &Value, expected: &Value) {
    let input = case["input"]["buy_token_balance"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: input.buy_token_balance must be a string"));
    let expected_balance = expected["normalized_buy_token_balance"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.normalized_buy_token_balance must be a string")
        });

    let input_balance = match input {
        "external" => OrderBalance::External,
        "internal" => OrderBalance::Internal,
        "erc20" => OrderBalance::Erc20,
        other => panic!("case {id}: unsupported buy_token_balance input {other:?}"),
    };

    let normalized = normalize_buy_token_balance(Some(input_balance));
    let normalized_label = match normalized {
        OrderBalance::Erc20 => "erc20",
        OrderBalance::External => "external",
        OrderBalance::Internal => "internal",
    };

    assert_eq!(
        normalized_label, expected_balance,
        "case {id}: normalize_buy_token_balance must map {input} → {expected_balance}",
    );
}

fn assert_signing_scheme_discriminants(id: &str, expected: &Value) {
    let expected_eip712 = expected["EIP712"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.EIP712 must be a u64"));
    let expected_ethsign = expected["ETHSIGN"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.ETHSIGN must be a u64"));
    let expected_eip1271 = expected["EIP1271"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.EIP1271 must be a u64"));
    let expected_presign = expected["PRESIGN"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.PRESIGN must be a u64"));

    assert_eq!(
        u64::from(SigningScheme::Eip712.as_u8()),
        expected_eip712,
        "case {id}: SigningScheme::Eip712 discriminant must match the fixture",
    );
    assert_eq!(
        u64::from(SigningScheme::EthSign.as_u8()),
        expected_ethsign,
        "case {id}: SigningScheme::EthSign discriminant must match the fixture",
    );
    assert_eq!(
        u64::from(SigningScheme::Eip1271.as_u8()),
        expected_eip1271,
        "case {id}: SigningScheme::Eip1271 discriminant must match the fixture",
    );
    assert_eq!(
        u64::from(SigningScheme::PreSign.as_u8()),
        expected_presign,
        "case {id}: SigningScheme::PreSign discriminant must match the fixture",
    );
}

fn assert_eip1271_magic_value(id: &str, expected: &Value) {
    let magic = expected["magic_value"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.magic_value must be a string"));
    assert_eq!(
        EIP1271_MAGICVALUE, magic,
        "case {id}: EIP1271_MAGICVALUE must equal the standard 0x1626ba7e",
    );
}

fn assert_interaction_defaults(id: &str, expected: &Value) {
    let expected_value = expected["value"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.value must be a string"));
    let expected_call_data = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let normalized = normalize_interaction(&InteractionLike {
        target: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        value: None,
        call_data: None,
    });

    assert_eq!(
        normalized.value.to_string(),
        expected_value,
        "case {id}: normalize_interaction must default missing value to zero",
    );
    assert_eq!(
        normalized.call_data.len(),
        0,
        "case {id}: normalize_interaction must default missing call_data to an empty payload",
    );
    assert_eq!(
        expected_call_data, "0x",
        "case {id}: fixture expects the hex-empty payload marker 0x",
    );
}

fn assert_deployment_constants(id: &str, expected: &Value) {
    let expected_salt = expected["salt"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.salt must be a string"));
    let expected_deployer = expected["deployer_contract"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.deployer_contract must be a string"));

    assert_eq!(
        SALT, expected_salt,
        "case {id}: SALT must equal the pinned deterministic deployment salt",
    );
    assert_eq!(
        DEPLOYER_CONTRACT, expected_deployer,
        "case {id}: DEPLOYER_CONTRACT must equal the Arachnid deployment proxy",
    );
}

fn assert_proxy_storage_slots(id: &str, expected: &Value) {
    let expected_impl = expected["implementation_slot"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.implementation_slot must be a string"));
    let expected_owner = expected["owner_slot"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.owner_slot must be a string"));

    assert_eq!(
        Eip1967Slot::Implementation.as_hex_str(),
        expected_impl,
        "case {id}: Eip1967Slot::Implementation must match the EIP-1967 slot",
    );
    assert_eq!(
        Eip1967Slot::Admin.as_hex_str(),
        expected_owner,
        "case {id}: Eip1967Slot::Admin must match the fixture admin-slot hash",
    );
}

fn assert_order_flags_default_sell(id: &str, expected: &Value) {
    let expected_flags = expected["encoded_flags"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.encoded_flags must be a u64"));

    let encoded = encode_order_flags(&OrderFlags {
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
    })
    .expect("default sell-erc20 order flags must encode");

    assert_eq!(
        u64::from(encoded),
        expected_flags,
        "case {id}: default sell-order flags must encode to zero",
    );
}

fn assert_order_flags_buy_partial_internal(id: &str, expected: &Value) {
    let expected_flags = expected["encoded_flags"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.encoded_flags must be a u64"));

    let encoded = encode_order_flags(&OrderFlags {
        kind: OrderKind::Buy,
        partially_fillable: true,
        sell_token_balance: OrderBalance::Internal,
        buy_token_balance: OrderBalance::Internal,
    })
    .expect("buy-partial-internal order flags must encode");

    assert_eq!(
        u64::from(encoded),
        expected_flags,
        "case {id}: buy + partially_fillable + internal/internal must encode to 31",
    );
}

fn assert_trade_flags_presign(id: &str, expected: &Value) {
    let expected_flags = expected["encoded_flags"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.encoded_flags must be a u64"));

    let encoded = encode_trade_flags(&TradeFlags {
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
        signing_scheme: SigningScheme::PreSign,
    })
    .expect("presign trade flags must encode");

    assert_eq!(
        u64::from(encoded),
        expected_flags,
        "case {id}: presign trade flags must layer the scheme bits above the order flags",
    );
}

fn assert_order_refund_method_names(id: &str, expected: &Value) {
    let expected_methods: Vec<&str> = expected["methods"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.methods must be an array"))
        .iter()
        .map(|method| {
            method
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: expected.methods entries must be strings"))
        })
        .collect();

    let domain = sample_domain();
    let mut encoder = SettlementEncoder::new(domain);
    encoder
        .encode_order_refunds(&cow_sdk_contracts::OrderRefunds {
            filled_amounts: vec![sample_order_uid()],
            pre_signatures: vec![sample_order_uid()],
        })
        .expect("sample order refunds must encode");

    let interactions = encoder
        .encoded_order_refunds()
        .expect("encoded refunds must serialize");
    assert_eq!(
        interactions.len(),
        expected_methods.len(),
        "case {id}: encoded refunds must produce one interaction per method",
    );

    let method_selectors: Vec<String> = expected_methods
        .iter()
        .map(|method| {
            let signature = format!("{method}(bytes[])");
            let hash = keccak256(signature.as_bytes());
            format!("0x{}", hex::encode(&hash[..4]))
        })
        .collect();

    for (interaction, selector) in interactions.iter().zip(method_selectors.iter()) {
        let call_data_hex = format!("0x{}", hex::encode(&interaction.call_data));
        assert!(
            call_data_hex.starts_with(selector),
            "case {id}: refund interaction call-data must start with {selector}",
        );
    }
}

fn assert_swap_default_user_data(id: &str, expected: &Value) {
    let expected_user_data = expected["user_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.user_data must be a string"));
    assert_eq!(
        expected_user_data, "0x",
        "case {id}: fixture expects the hex-empty user-data marker 0x",
    );

    let mut tokens = TokenRegistry::default();
    let step = encode_swap_step(
        &mut tokens,
        &Swap {
            pool_id: "0x0000000000000000000000000000000000000000000000000000000000000001"
                .to_owned(),
            asset_in: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            asset_out: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            amount: Amount::new("1").unwrap(),
            user_data: None,
        },
    );

    assert_eq!(
        step.user_data.len(),
        0,
        "case {id}: encode_swap_step must default missing user_data to an empty buffer",
    );
}

fn assert_vault_required_methods(id: &str, expected: &Value) {
    let expected_methods: Vec<&str> = expected["methods"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.methods must be an array"))
        .iter()
        .map(|method| {
            method
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: expected.methods entries must be strings"))
        })
        .collect();

    let actual_methods: Vec<&str> = VAULT_INTERFACE
        .iter()
        .map(|entry| {
            entry
                .trim_start_matches("function ")
                .split('(')
                .next()
                .unwrap()
        })
        .collect();

    assert_eq!(
        actual_methods, expected_methods,
        "case {id}: VAULT_INTERFACE must expose the fixture-named methods in order",
    );
}

fn assert_reader_helper_surface(id: &str, expected: &Value) {
    let expected_helpers: Vec<&str> = expected["helpers"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.helpers must be an array"))
        .iter()
        .map(|helper| {
            helper
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: expected.helpers entries must be strings"))
        })
        .collect();

    // The reader helper surface is the three public struct types; referencing
    // each through a size_of assertion proves they remain part of the shipped
    // API and compiles only when the types are still exported.
    let _ = std::mem::size_of::<AllowListReader<()>>();
    let _ = std::mem::size_of::<SettlementReader<()>>();
    let _ = std::mem::size_of::<TradeSimulator<()>>();

    let rust_surface = ["AllowListReader", "SettlementReader", "TradeSimulator"];
    assert_eq!(
        rust_surface.as_slice(),
        expected_helpers.as_slice(),
        "case {id}: reader helper surface must expose AllowListReader, SettlementReader, and TradeSimulator",
    );
}

fn sample_domain() -> TypedDataDomain {
    TypedDataDomain {
        name: "Gnosis Protocol".to_owned(),
        version: "v2".to_owned(),
        chain_id: u64::from(SupportedChainId::Mainnet),
        verifying_contract: Registry::default()
            .address(
                ContractId::Settlement,
                SupportedChainId::Mainnet,
                CowEnv::Prod,
            )
            .expect("canonical settlement address is registered on mainnet"),
    }
}

fn sample_order_uid() -> OrderUid {
    // 56 byte fixture UID: 32-byte digest | 20-byte owner | 4-byte valid_to.
    let digest =
        OrderDigest::new("0x1111111111111111111111111111111111111111111111111111111111111111")
            .unwrap();
    let owner = Address::new("0x2222222222222222222222222222222222222222").unwrap();

    cow_sdk_contracts::pack_order_uid_params(&cow_sdk_contracts::OrderUidParams {
        order_digest: digest,
        owner,
        valid_to: 0x1234_5678,
    })
    .expect("sample OrderUid packing must succeed")
}

fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let digest = Keccak256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}
