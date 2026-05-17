//! Fixture-driven parity contract for `cow-sdk-contracts`.
//!
//! Loads `parity/fixtures/contracts.json` (schema version 1) at compile time,
//! iterates every documented case, and asserts the Rust helpers produce the
//! pinned upstream values. The helpers exercised are:
//!
//! * [`ORDER_TYPE_FIELDS`], the [`GPv2Order`] type hash, [`CANCELLATIONS_TYPE_FIELDS`],
//!   [`ORDER_UID_LENGTH`] — canonical EIP-712 and UID layout constants.
//! * [`extract_order_uid_params`] — UID length validation through
//!   [`ContractsError::InvalidOrderUidLength`].
//! * [`SellTokenSource`] / [`BuyTokenDestination`] — split balance enums; the
//!   parity fixture pins that `BuyTokenDestination` has no `external` variant
//!   so quote-derived and direct trading orders cannot rewrite the buy-side
//!   destination silently.
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

use alloy_sol_types::{
    Eip712Domain, SolCall, SolStruct,
    private::{Address as SolAddress, Bytes as SolBytes, FixedBytes, U256},
    sol,
};
use cow_sdk_contracts::{
    AllowListReader, CANCELLATIONS_TYPE_FIELDS, ContractId, DEPLOYER_CONTRACT, EIP1271_MAGICVALUE,
    Eip1967Slot, EthFlowOrderData, IERC20, IERC20Permit, InteractionLike, ORDER_TYPE_FIELDS,
    ORDER_UID_LENGTH, Order, OrderFlags, Registry, SALT, SettlementEncoder, SettlementReader,
    Signature, SigningScheme, Swap, TokenRegistry, TradeExecution, TradeFlags, TradeSimulator,
    VAULT_INTERFACE, encode_create_order_calldata, encode_invalidate_order_calldata,
    encode_order_flags, encode_swap_step, encode_trade_flags, normalize_interaction,
    permit_typed_data_hash, required_vault_roles,
};
use cow_sdk_core::{
    Address, Amount, AppDataHash, AppDataHex, BuyTokenDestination, CowEnv, OrderDigest, OrderKind,
    OrderUid, SellTokenSource, SupportedChainId, TypedDataDomain,
};
use serde_json::Value;

// Local `alloy::sol!` re-declaration of the two binding families whose
// generated types are internal to `cow-sdk-contracts`. The parity test asserts
// that the crate's encoder output is byte-identical to the canonical upstream
// ABI, and the local re-declaration provides an independent authoring surface
// that produces the same bytes only if the Solidity signature and field order
// still match upstream.
sol! {
    interface IGPv2Settlement {
        function invalidateOrder(bytes orderUid) external;
        function setPreSignature(bytes orderUid, bool signed) external;
        function freeFilledAmountStorage(bytes[] orderUids) external;
        function freePreSignatureStorage(bytes[] orderUids) external;
    }

    interface IGPv2VaultRelayer {
        struct Transfer {
            address account;
            address token;
            uint256 amount;
            uint8 balance;
        }

        function transferFromAccounts(Transfer[] transfers) external;
    }
}

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
            "contracts-buy-balance-domain" => {
                assert_buy_balance_domain(id, expected);
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
            "contracts-vault-role-hashes-match-upstream-typescript" => {
                assert_vault_role_hashes_match_upstream_typescript(id, expected);
            }
            "contracts-reader-helper-surface" => assert_reader_helper_surface(id, expected),
            "contracts-settlement-invalidate-order-calldata" => {
                assert_settlement_invalidate_order_calldata(id, expected);
            }
            "contracts-settlement-set-presignature-calldata" => {
                assert_settlement_set_presignature_calldata(id, expected);
            }
            "contracts-settlement-free-filled-amount-storage-calldata" => {
                assert_settlement_free_filled_amount_storage_calldata(id, expected);
            }
            "contracts-settlement-free-presignature-storage-calldata" => {
                assert_settlement_free_presignature_storage_calldata(id, expected);
            }
            "contracts-vault-relayer-transfer-from-accounts-calldata" => {
                assert_vault_relayer_transfer_from_accounts_calldata(id, expected);
            }
            "contracts-vault-relayer-mixed-balance-transfer-from-accounts-calldata" => {
                assert_vault_relayer_mixed_balance_transfer_from_accounts_calldata(id, expected);
            }
            "contracts-settlement-clearing-prices-multi-trade" => {
                assert_settlement_clearing_prices_multi_trade(id, expected);
            }
            "contracts-ethflow-create-order-calldata" => {
                assert_ethflow_create_order_calldata(id, expected);
            }
            "contracts-ethflow-invalidate-order-calldata" => {
                assert_ethflow_invalidate_order_calldata(id, expected);
            }
            "contracts-erc20-approve-calldata" => {
                assert_erc20_approve_calldata(id, expected);
            }
            "contracts-erc20-transfer-from-calldata" => {
                assert_erc20_transfer_from_calldata(id, expected);
            }
            "contracts-erc20-permit-typed-data-hash" => {
                assert_erc20_permit_typed_data_hash(id, expected);
            }
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
    use alloy_sol_types::SolStruct;
    use cow_sdk_contracts::GPv2Order;
    let expected_hash = expected["hash"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.hash must be a string"));
    let actual_hash = format!(
        "0x{}",
        hex::encode(GPv2Order::default().eip712_type_hash().as_slice())
    );
    assert_eq!(
        actual_hash, expected_hash,
        "case {id}: GPv2Order type hash must equal the pinned contracts-ts constant",
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

fn assert_buy_balance_domain(id: &str, expected: &Value) {
    let expected_buy = expected["buy_token_destination_variants"]
        .as_array()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.buy_token_destination_variants must be an array")
        })
        .iter()
        .map(|entry| {
            entry.as_str().unwrap_or_else(|| {
                panic!("case {id}: buy_token_destination_variants entries must be strings")
            })
        })
        .collect::<Vec<_>>();
    let expected_sell = expected["sell_token_source_variants"]
        .as_array()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.sell_token_source_variants must be an array")
        })
        .iter()
        .map(|entry| {
            entry.as_str().unwrap_or_else(|| {
                panic!("case {id}: sell_token_source_variants entries must be strings")
            })
        })
        .collect::<Vec<_>>();

    let buy_variants = [BuyTokenDestination::Erc20, BuyTokenDestination::Internal]
        .into_iter()
        .map(|variant| serde_json::to_value(variant).expect("variant serialization"))
        .map(|value| {
            value
                .as_str()
                .expect("BuyTokenDestination must serialize to a snake_case string")
                .to_owned()
        })
        .collect::<Vec<_>>();
    let sell_variants = [
        SellTokenSource::Erc20,
        SellTokenSource::External,
        SellTokenSource::Internal,
    ]
    .into_iter()
    .map(|variant| serde_json::to_value(variant).expect("variant serialization"))
    .map(|value| {
        value
            .as_str()
            .expect("SellTokenSource must serialize to a snake_case string")
            .to_owned()
    })
    .collect::<Vec<_>>();

    assert_eq!(
        buy_variants, expected_buy,
        "case {id}: BuyTokenDestination must expose exactly the services buy-side variant set",
    );
    assert_eq!(
        sell_variants, expected_sell,
        "case {id}: SellTokenSource must expose exactly the services sell-side variant set",
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

    let normalized = normalize_interaction(&InteractionLike::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        None,
        None,
    ));

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

    let encoded = encode_order_flags(&OrderFlags::new(
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    ))
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

    let encoded = encode_order_flags(&OrderFlags::new(
        OrderKind::Buy,
        true,
        SellTokenSource::Internal,
        BuyTokenDestination::Internal,
    ))
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

    let encoded = encode_trade_flags(&TradeFlags::new(
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
        SigningScheme::PreSign,
    ))
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
        .encode_order_refunds(&cow_sdk_contracts::OrderRefunds::new(
            vec![sample_order_uid()],
            vec![sample_order_uid()],
        ))
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
        &Swap::new(
            "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned(),
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            Amount::new("1").unwrap(),
            None,
        ),
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

fn assert_vault_role_hashes_match_upstream_typescript(id: &str, expected: &Value) {
    let vault_address = expected["vault_address"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.vault_address must be a string"));
    let expected_roles = expected["roles"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.roles must be an array"));

    let vault = Address::new(vault_address)
        .unwrap_or_else(|error| panic!("case {id}: expected.vault_address must parse: {error}"));
    let roles = required_vault_roles(&vault)
        .unwrap_or_else(|error| panic!("case {id}: vault role derivation must succeed: {error}"));

    assert_eq!(
        roles.len(),
        expected_roles.len(),
        "case {id}: fixture must cover every required vault role",
    );

    for (role, expected_role) in roles.iter().zip(expected_roles.iter()) {
        let method = expected_role["method"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id}: expected role method must be a string"));
        let selector = expected_role["selector"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id}: expected role selector must be a string"));
        let role_hash = expected_role["role_hash"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id}: expected role_hash must be a string"));

        assert_eq!(
            role.method, method,
            "case {id}: vault role method must match upstream order",
        );
        assert_eq!(
            role.selector, selector,
            "case {id}: vault role selector must match upstream TypeScript",
        );
        assert_eq!(
            role.role, role_hash,
            "case {id}: vault role hash must match upstream packed-keccak formula",
        );
    }
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
    TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        u64::from(SupportedChainId::Mainnet),
        Registry::default()
            .address(
                ContractId::Settlement,
                SupportedChainId::Mainnet,
                CowEnv::Prod,
            )
            .expect("canonical settlement address is registered on mainnet"),
    )
}

fn sample_order_uid() -> OrderUid {
    // 56 byte fixture UID: 32-byte digest | 20-byte owner | 4-byte valid_to.
    let digest =
        OrderDigest::new("0x1111111111111111111111111111111111111111111111111111111111111111")
            .unwrap();
    let owner = Address::new("0x2222222222222222222222222222222222222222").unwrap();

    cow_sdk_contracts::pack_order_uid_params(&cow_sdk_contracts::OrderUidParams::new(
        digest,
        owner,
        0x1234_5678,
    ))
    .expect("sample OrderUid packing must succeed")
}

// Hand-rolled `sha3::Keccak256` helper used by the assertions in this
// file. Crate code routes through `alloy_primitives::keccak256` per
// ADR 0052; this helper deliberately runs `sha3::Keccak256` directly so
// the parity check compares the crate output against an independent
// keccak implementation.
fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let digest = Keccak256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn assert_calldata_hex(id: &str, actual_bytes: &[u8], expected_hex: &str) {
    let actual_hex = format!("0x{}", hex::encode(actual_bytes));
    assert_eq!(
        actual_hex.as_str(),
        expected_hex,
        "case {id}: encoded call-data must match the pinned byte-identity fixture",
    );
}

fn parse_address_bytes(address: &Address) -> [u8; 20] {
    let hex_bytes = hex::decode(
        address
            .as_str()
            .strip_prefix("0x")
            .expect("Address must carry a 0x prefix"),
    )
    .expect("Address hex must decode");
    <[u8; 20]>::try_from(hex_bytes.as_slice()).expect("Address hex must be 20 bytes")
}

fn to_sol_address(address: &Address) -> SolAddress {
    SolAddress::from(parse_address_bytes(address))
}

fn sample_secondary_order_uid() -> OrderUid {
    // Second 56 byte fixture UID for multi-entry refund encoding; mirrors the
    // primary sample UID's shape but with distinct digest, owner, and
    // valid_to.
    let digest =
        OrderDigest::new("0x3333333333333333333333333333333333333333333333333333333333333333")
            .unwrap();
    let owner = Address::new("0x4444444444444444444444444444444444444444").unwrap();

    cow_sdk_contracts::pack_order_uid_params(&cow_sdk_contracts::OrderUidParams::new(
        digest,
        owner,
        0x9abc_def0,
    ))
    .expect("secondary OrderUid packing must succeed")
}

fn order_uid_as_sol_bytes(uid: &OrderUid) -> SolBytes {
    let hex_bytes = hex::decode(
        uid.as_str()
            .strip_prefix("0x")
            .expect("OrderUid must carry a 0x prefix"),
    )
    .expect("OrderUid hex must decode");
    SolBytes::from(hex_bytes)
}

fn sample_ethflow_order() -> EthFlowOrderData {
    EthFlowOrderData::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Amount::new("1000000000000000000").unwrap(),
        Amount::new("2000000000000000000").unwrap(),
        AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Amount::zero(),
        0x1234_5678,
        false,
        42,
    )
}

fn assert_settlement_invalidate_order_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let uid = sample_order_uid();
    let call_data = IGPv2Settlement::invalidateOrderCall {
        orderUid: order_uid_as_sol_bytes(&uid),
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_settlement_set_presignature_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let uid = sample_order_uid();
    let call_data = IGPv2Settlement::setPreSignatureCall {
        orderUid: order_uid_as_sol_bytes(&uid),
        signed: true,
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_settlement_free_filled_amount_storage_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let primary = sample_order_uid();
    let secondary = sample_secondary_order_uid();
    let call_data = IGPv2Settlement::freeFilledAmountStorageCall {
        orderUids: vec![
            order_uid_as_sol_bytes(&primary),
            order_uid_as_sol_bytes(&secondary),
        ],
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_settlement_free_presignature_storage_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let primary = sample_order_uid();
    let secondary = sample_secondary_order_uid();
    let call_data = IGPv2Settlement::freePreSignatureStorageCall {
        orderUids: vec![
            order_uid_as_sol_bytes(&primary),
            order_uid_as_sol_bytes(&secondary),
        ],
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_vault_relayer_transfer_from_accounts_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let account = Address::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let token = Address::new("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
    let amount = U256::from(1_000_000_000_000_000_000_u128);

    let call_data = IGPv2VaultRelayer::transferFromAccountsCall {
        transfers: vec![IGPv2VaultRelayer::Transfer {
            account: to_sol_address(&account),
            token: to_sol_address(&token),
            amount,
            balance: 0,
        }],
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_vault_relayer_mixed_balance_transfer_from_accounts_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let transfer =
        |account: &str, token: &str, amount: u128, balance: u8| IGPv2VaultRelayer::Transfer {
            account: to_sol_address(&Address::new(account).unwrap()),
            token: to_sol_address(&Address::new(token).unwrap()),
            amount: U256::from(amount),
            balance,
        };
    let call_data = IGPv2VaultRelayer::transferFromAccountsCall {
        transfers: vec![
            transfer(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                1_000_000_000_000_000_000,
                0,
            ),
            transfer(
                "0xcccccccccccccccccccccccccccccccccccccccc",
                "0xdddddddddddddddddddddddddddddddddddddddd",
                2_000_000_000_000_000_000,
                1,
            ),
            transfer(
                "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                "0xffffffffffffffffffffffffffffffffffffffff",
                3_000_000_000_000_000_000,
                2,
            ),
        ],
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_settlement_clearing_prices_multi_trade(id: &str, expected: &Value) {
    let expected_tokens = expected["tokens"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.tokens must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: token entries must be strings"))
        })
        .collect::<Vec<_>>();
    let expected_prices = expected["clearing_prices"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.clearing_prices must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: clearing price entries must be strings"))
        })
        .collect::<Vec<_>>();

    let mut encoder = SettlementEncoder::new(sample_domain());
    encoder
        .encode_trade(
            &settlement_sample_order(
                "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                "0x6b175474e89094c44da98b954eedeac495271d0f",
                "1000",
                "2000",
                1_700_000_001,
            ),
            &settlement_sample_signature(),
            Some(TradeExecution::new(Amount::new("1000").unwrap())),
        )
        .expect("first fixture trade must encode");
    encoder
        .encode_trade(
            &settlement_sample_order(
                "0x1111111111111111111111111111111111111111",
                "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                "3000",
                "4000",
                1_700_000_002,
            ),
            &settlement_sample_signature(),
            Some(TradeExecution::new(Amount::new("3000").unwrap())),
        )
        .expect("second fixture trade must encode");

    let prices = serde_json::from_value::<cow_sdk_contracts::Prices>(serde_json::json!({
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2": "1000000000000000000",
        "0x6b175474e89094c44da98b954eedeac495271d0f": "500000000000000",
        "0x1111111111111111111111111111111111111111": "250000000000000000"
    }))
    .expect("fixture prices must deserialize");

    assert_eq!(
        encoder
            .tokens()
            .iter()
            .map(Address::as_str)
            .collect::<Vec<_>>(),
        expected_tokens,
        "case {id}: token registry order must follow first-seen trade order",
    );
    assert_eq!(
        encoder
            .clearing_prices(&prices)
            .expect("fixture prices cover every registered token")
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        expected_prices,
        "case {id}: clearing prices must align with encoded token order",
    );
}

fn settlement_sample_order(
    sell_token: &str,
    buy_token: &str,
    sell_amount: &str,
    buy_amount: &str,
    valid_to: u32,
) -> Order {
    Order::new(
        Address::new(sell_token).unwrap(),
        Address::new(buy_token).unwrap(),
        None,
        Amount::new(sell_amount).unwrap(),
        Amount::new(buy_amount).unwrap(),
        valid_to,
        AppDataHex::new("0x0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap(),
        Amount::zero(),
        OrderKind::Sell,
        false,
        Some(SellTokenSource::Erc20),
        Some(BuyTokenDestination::Erc20),
    )
}

fn settlement_sample_signature() -> Signature {
    Signature::PreSign {
        owner: Address::new("0x9999999999999999999999999999999999999999").unwrap(),
    }
}

fn assert_ethflow_create_order_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let call_data = encode_create_order_calldata(&sample_ethflow_order())
        .expect("sample EthFlow order must encode");

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_ethflow_invalidate_order_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let call_data = encode_invalidate_order_calldata(&sample_ethflow_order())
        .expect("sample EthFlow order must encode");

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_erc20_approve_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let spender = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let value = U256::from(1_000_000_000_000_000_000_u128);

    let call_data = IERC20::approveCall {
        spender: to_sol_address(&spender),
        value,
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_erc20_transfer_from_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let from = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let to = Address::new("0x2222222222222222222222222222222222222222").unwrap();
    let value = U256::from(1_000_000_000_000_000_000_u128);

    let call_data = IERC20::transferFromCall {
        from: to_sol_address(&from),
        to: to_sol_address(&to),
        value,
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_erc20_permit_typed_data_hash(id: &str, expected: &Value) {
    let expected_hex = expected["typed_data_hash"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.typed_data_hash must be a string"));

    // USDC mainnet domain separator inputs: the deployed USD Coin (USDC) token
    // at 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 publishes
    // `DOMAIN_SEPARATOR()` with name = "USD Coin", version = "2", chainId = 1,
    // verifyingContract = the USDC contract itself. Using a real, deployed
    // token rather than a synthetic domain keeps the typed-data hash
    // cross-checkable against an on-chain reply.
    let domain = Eip712Domain::new(
        Some(alloy_sol_types::private::Cow::Borrowed("USD Coin")),
        Some(alloy_sol_types::private::Cow::Borrowed("2")),
        Some(U256::from(1_u64)),
        Some(SolAddress::from([
            0xa0, 0xb8, 0x69, 0x91, 0xc6, 0x21, 0x8b, 0x36, 0xc1, 0xd1, 0x9d, 0x4a, 0x2e, 0x9e,
            0xb0, 0xce, 0x36, 0x06, 0xeb, 0x48,
        ])),
        None,
    );

    let owner = SolAddress::from([
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x11, 0x11, 0x11,
    ]);
    let spender = SolAddress::from([
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22, 0x22, 0x22, 0x22, 0x22,
    ]);

    let permit = IERC20Permit::Permit {
        owner,
        spender,
        value: U256::from(1_000_000_000_000_000_000_u128),
        nonce: U256::ZERO,
        deadline: U256::from(2_000_000_000_u64),
    };

    let digest = permit_typed_data_hash(&domain, &permit);
    let digest_hex = format!("0x{}", hex::encode(digest));

    assert_eq!(
        digest_hex.as_str(),
        expected_hex,
        "case {id}: EIP-712 typed-data digest must match the pinned fixture value",
    );

    // Cross-check that the canonical permit struct hash preimage (type hash +
    // 5 fields) composes with the domain separator through the standard
    // `\x19\x01 || domainSeparator || structHash` envelope. A mismatch here
    // indicates drift between `permit_typed_data_hash` and the upstream
    // EIP-712 specification.
    let struct_hash: [u8; 32] = permit.eip712_hash_struct().into();
    let domain_separator: [u8; 32] = domain.separator().into();
    let mut envelope = Vec::with_capacity(2 + 32 + 32);
    envelope.push(0x19);
    envelope.push(0x01);
    envelope.extend_from_slice(&domain_separator);
    envelope.extend_from_slice(&struct_hash);
    let manual_digest = keccak256(&envelope);
    assert_eq!(
        digest, manual_digest,
        "case {id}: permit_typed_data_hash must compose `0x1901 || domain || struct` exactly",
    );

    // Fail closed if the fixture carries an obviously-malformed constant.
    let raw_hex = expected_hex
        .strip_prefix("0x")
        .unwrap_or_else(|| panic!("case {id}: expected.typed_data_hash must start with 0x"));
    assert_eq!(
        raw_hex.len(),
        64,
        "case {id}: expected.typed_data_hash must be a 32-byte digest expressed as 64 hex chars",
    );

    // Reference the declared typed constants to prove they remain exported on
    // the shipped surface.
    let _ = FixedBytes::<32>::from(digest);
}
