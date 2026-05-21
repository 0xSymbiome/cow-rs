#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, and perf lints acceptable in test helper code"
)]

mod common;

use alloy_primitives::Bytes;
use cow_sdk_contracts::{
    ContractId, ContractsError, InteractionLike, InteractionStage, Registry, SettlementEncoder,
    normalize_interaction, normalize_interactions,
};
use cow_sdk_core::{Address, Amount, CowEnv, SupportedChainId, TypedDataDomain};

use common::fixture_case;

fn bytes_from_hex_literal(literal: &str) -> Bytes {
    let stripped = literal
        .strip_prefix("0x")
        .expect("hex literal must start with 0x");
    Bytes::from(hex::decode(stripped).expect("hex literal must decode"))
}

fn hex_prefixed(bytes: &Bytes) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn settlement_domain(chain_id: SupportedChainId, verifying_contract: Address) -> TypedDataDomain {
    TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        chain_id.into(),
        verifying_contract,
    )
}

#[test]
fn interaction_normalization_applies_zero_value_call_defaults() {
    let fixture = fixture_case("contracts-interaction-defaults");
    let target = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();

    let normalized = normalize_interaction(&InteractionLike::new(target, None, None));
    assert_eq!(normalized.target, target);
    assert_eq!(
        normalized.value.to_string(),
        fixture["expected"]["value"].as_str().unwrap()
    );
    assert_eq!(
        hex_prefixed(&normalized.call_data),
        fixture["expected"]["call_data"].as_str().unwrap()
    );
    assert!(
        normalized.call_data.is_empty(),
        "default calldata must be an empty byte buffer"
    );

    let explicit = normalize_interaction(&InteractionLike::new(
        normalized.target,
        Some(Amount::new("42").unwrap()),
        Some(bytes_from_hex_literal("0x12345678")),
    ));
    assert_eq!(explicit.value.to_string(), "42");
    assert_eq!(
        explicit.call_data.as_ref(),
        &[0x12, 0x34, 0x56, 0x78][..],
        "explicit calldata must round-trip byte-equal through the encoder"
    );
}

#[test]
fn batch_interaction_normalization_preserves_order() {
    let interactions = vec![
        InteractionLike::new(
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            None,
            None,
        ),
        InteractionLike::new(
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            Some(Amount::new("7").unwrap()),
            Some(bytes_from_hex_literal("0x01020304")),
        ),
    ];

    let normalized = normalize_interactions(&interactions);
    assert_eq!(normalized.len(), 2);
    assert_eq!(normalized[0].value.to_string(), "0");
    assert!(
        normalized[0].call_data.is_empty(),
        "missing calldata must normalize to an empty byte buffer"
    );
    assert_eq!(normalized[1].value.to_string(), "7");
    assert_eq!(
        normalized[1].call_data.as_ref(),
        &[0x01, 0x02, 0x03, 0x04][..],
        "explicit calldata must preserve the input bytes through normalization"
    );
    assert_eq!(normalized[1].target, interactions[1].target);
}

#[test]
fn interaction_calldata_clone_shares_backing_allocation() {
    let interaction = normalize_interaction(&InteractionLike::new(
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        None,
        Some(bytes_from_hex_literal("0xdeadbeefcafef00d")),
    ));

    let cloned = interaction.call_data.clone();
    assert_eq!(
        cloned, interaction.call_data,
        "alloy_primitives::Bytes clone must preserve the original byte sequence"
    );
    assert_eq!(
        cloned.as_ptr(),
        interaction.call_data.as_ptr(),
        "alloy_primitives::Bytes clone must reference the same backing allocation"
    );
}

#[test]
fn interaction_encoder_rejects_vault_relayer_target_for_canonical_settlement_domain() {
    let registry = Registry::default();

    for chain_id in SupportedChainId::ALL {
        for env in [CowEnv::Prod, CowEnv::Staging] {
            let settlement = registry
                .address(ContractId::Settlement, chain_id, env)
                .expect("canonical settlement must be registered for every supported env");
            let vault_relayer = registry
                .address(ContractId::VaultRelayer, chain_id, env)
                .expect("canonical vault relayer must be registered for every supported env");
            let mut encoder = SettlementEncoder::new(settlement_domain(chain_id, settlement));

            let error = encoder
                .encode_interaction(
                    &InteractionLike::new(vault_relayer, None, None),
                    InteractionStage::Intra,
                )
                .unwrap_err();

            assert!(matches!(
                error,
                ContractsError::ForbiddenInteractionTarget { target } if target == vault_relayer
            ));
        }
    }
}

#[test]
fn interaction_encoder_accepts_non_vault_target_for_canonical_settlement_domain() {
    let registry = Registry::default();
    let chain_id = SupportedChainId::Mainnet;
    let settlement = registry
        .address(ContractId::Settlement, chain_id, CowEnv::Prod)
        .expect("canonical settlement must be registered");
    let mut encoder = SettlementEncoder::new(settlement_domain(chain_id, settlement));
    let target = Address::new("0x1111111111111111111111111111111111111111").unwrap();

    encoder
        .encode_interaction(
            &InteractionLike::new(target, None, None),
            InteractionStage::Intra,
        )
        .unwrap();

    let interactions = encoder.interactions().unwrap();
    assert_eq!(
        interactions[InteractionStage::Intra as usize][0].target,
        target
    );
}

#[test]
fn interaction_encoder_does_not_cross_match_chain_or_env() {
    let registry = Registry::default();
    let chain_id = SupportedChainId::Mainnet;
    let settlement = registry
        .address(ContractId::Settlement, chain_id, CowEnv::Prod)
        .expect("canonical settlement must be registered");
    let staging_vault_relayer = registry
        .address(ContractId::VaultRelayer, chain_id, CowEnv::Staging)
        .expect("canonical staging vault relayer must be registered");
    let mut prod_encoder = SettlementEncoder::new(settlement_domain(chain_id, settlement));

    prod_encoder
        .encode_interaction(
            &InteractionLike::new(staging_vault_relayer, None, None),
            InteractionStage::Intra,
        )
        .unwrap();

    let unsupported_chain_domain = TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        424_242,
        settlement,
    );
    let prod_vault_relayer = registry
        .address(ContractId::VaultRelayer, chain_id, CowEnv::Prod)
        .expect("canonical production vault relayer must be registered");
    let mut custom_chain_encoder = SettlementEncoder::new(unsupported_chain_domain);

    custom_chain_encoder
        .encode_interaction(
            &InteractionLike::new(prod_vault_relayer, None, None),
            InteractionStage::Intra,
        )
        .unwrap();
}

#[test]
fn interaction_encoder_neutral_for_unknown_custom_settlement_domain() {
    let registry = Registry::default();
    let vault_relayer = registry
        .address(
            ContractId::VaultRelayer,
            SupportedChainId::Mainnet,
            CowEnv::Prod,
        )
        .expect("canonical vault relayer must be registered");
    let custom_settlement = Address::new("0x2222222222222222222222222222222222222222").unwrap();
    let mut encoder = SettlementEncoder::new(settlement_domain(
        SupportedChainId::Mainnet,
        custom_settlement,
    ));

    encoder
        .encode_interaction(
            &InteractionLike::new(vault_relayer, None, None),
            InteractionStage::Intra,
        )
        .unwrap();

    let interactions = encoder.interactions().unwrap();
    assert_eq!(
        interactions[InteractionStage::Intra as usize][0].target,
        vault_relayer,
    );
}
