use alloy_primitives::Bytes;
use alloy_sol_types::{
    SolCall,
    private::{Address as SolAddress, FixedBytes, U256},
    sol,
};
use cow_sdk_contracts::{
    BatchSwapStep, ContractAddresses, ContractName, Eip1271SignatureData,
    Eip1271VerificationRequest, EthFlowOrderData, GrantRoleCall, Interaction, InteractionLike,
    OrderCancellations, OrderFlags, OrderRefunds, OrderTypeField, OrderUidParams,
    RequiredVaultRole, Signature, SigningScheme, Swap, SwapExecution, Trade, TradeExecution,
    TradeFlags, TradeSimulation, TradeSimulationBalanceDelta, TradeSimulationResult,
    encode_create_order_calldata,
};
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, Hash32, HexData, OrderDigest, OrderKind,
    OrderUid, SellTokenSource, SignedAmount,
};
use serde::Deserialize;
use serde::Serialize;

const CONTRACT_SIGNATURE_SOURCE: &str = include_str!("../src/signature.rs");
const ORDERBOOK_TYPES_SOURCE: &str = include_str!("../../orderbook/src/types/enums.rs");

const ADDR1: &str = "0x1111111111111111111111111111111111111111";
const ADDR2: &str = "0x2222222222222222222222222222222222222222";
const ADDR3: &str = "0x3333333333333333333333333333333333333333";
const ADDR4: &str = "0x4444444444444444444444444444444444444444";
const APP_DATA: &str = concat!(
    "0x", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa",
    "aaaaaaaa",
);
const DIGEST: &str = concat!(
    "0x", "dddddddd", "dddddddd", "dddddddd", "dddddddd", "dddddddd", "dddddddd", "dddddddd",
    "dddddddd",
);
const HASH: &str = concat!(
    "0x", "bbbbbbbb", "bbbbbbbb", "bbbbbbbb", "bbbbbbbb", "bbbbbbbb", "bbbbbbbb", "bbbbbbbb",
    "bbbbbbbb",
);
const UID1: &str = concat!(
    "0x", "dddddddd", "dddddddd", "dddddddd", "dddddddd", "dddddddd", "dddddddd", "dddddddd",
    "dddddddd", "eeeeeeee", "eeeeeeee", "eeeeeeee", "eeeeeeee", "eeeeeeee", "0000002a",
);
const UID2: &str = concat!(
    "0x", "ffffffff", "ffffffff", "ffffffff", "ffffffff", "ffffffff", "ffffffff", "ffffffff",
    "ffffffff", "11111111", "11111111", "11111111", "11111111", "11111111", "0000002b",
);

sol! {
    #[sol(rename_all = "camelcase")]
    interface LocalEthFlow {
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

        function createOrder(EthFlowOrderData calldata order) external payable;
    }
}

fn assert_json_bytes<T>(value: &T, expected: &str)
where
    T: Serialize,
{
    let actual = serde_json::to_string(value).expect("DTO serialization must succeed");
    assert_eq!(actual, expected);
}

fn address(value: &str) -> Address {
    Address::new(value).expect("address literal must stay valid")
}

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("amount literal must stay valid")
}

fn app_data() -> AppDataHash {
    AppDataHash::new(APP_DATA).expect("app-data literal must stay valid")
}

fn hash32() -> Hash32 {
    Hash32::new(HASH).expect("hash literal must stay valid")
}

fn order_digest() -> OrderDigest {
    OrderDigest::new(DIGEST).expect("order-digest literal must stay valid")
}

fn order_uid(value: &str) -> OrderUid {
    OrderUid::new(value).expect("order UID literal must stay valid")
}

fn signed_amount(value: &str) -> SignedAmount {
    SignedAmount::new(value).expect("signed amount literal must stay valid")
}

#[test]
fn order_type_field_new_preserves_wire_shape() {
    let field = OrderTypeField::new("sellToken", "address");

    assert_json_bytes(&field, r#"{"name":"sellToken","type":"address"}"#);
}

#[test]
fn order_uid_params_new_preserves_wire_shape() {
    let params = OrderUidParams::new(order_digest(), address(ADDR3), 44);
    let expected = format!("{{\"orderDigest\":\"{DIGEST}\",\"owner\":\"{ADDR3}\",\"validTo\":44}}");

    assert_json_bytes(&params, &expected);
}

#[test]
fn order_cancellations_new_preserves_wire_shape() {
    let cancellations = OrderCancellations::new(vec![order_uid(UID1), order_uid(UID2)]);
    let expected = format!("{{\"orderUids\":[\"{UID1}\",\"{UID2}\"]}}");

    assert_json_bytes(&cancellations, &expected);
}

#[test]
fn eip1271_signature_data_new_preserves_wire_shape() {
    let data = Eip1271SignatureData::new(address(ADDR4), "0x1234".to_owned());
    let expected = format!("{{\"verifier\":\"{ADDR4}\",\"signature\":\"0x1234\"}}");

    assert_json_bytes(&data, &expected);
}

#[test]
fn eip1271_verification_request_new_preserves_wire_shape() {
    let request = Eip1271VerificationRequest::new(
        address(ADDR4),
        hash32(),
        HexData::new("0xcafe").expect("signature hex must stay valid"),
    );
    let expected =
        format!("{{\"verifier\":\"{ADDR4}\",\"digest\":\"{HASH}\",\"signature\":\"0xcafe\"}}");

    assert_json_bytes(&request, &expected);
}

#[test]
fn signature_enum_preserves_wire_shape() {
    let signature = Signature::PreSign {
        owner: address(ADDR1),
    };
    let expected = format!("{{\"kind\":\"preSign\",\"owner\":\"{ADDR1}\"}}");

    assert_json_bytes(&signature, &expected);
}

#[test]
fn interaction_new_preserves_wire_shape() {
    let interaction =
        Interaction::new(address(ADDR1), amount("5"), Bytes::from_static(b"\xbe\xef"));
    let expected = format!("{{\"target\":\"{ADDR1}\",\"value\":\"5\",\"callData\":\"0xbeef\"}}");

    assert_json_bytes(&interaction, &expected);
}

#[test]
fn interaction_like_new_preserves_wire_shape() {
    let interaction = InteractionLike::new(address(ADDR2), None, None);
    let expected = format!("{{\"target\":\"{ADDR2}\"}}");

    assert_json_bytes(&interaction, &expected);
}

#[test]
fn swap_new_preserves_wire_shape() {
    let swap = Swap::new(
        "pool-1".to_owned(),
        address(ADDR1),
        address(ADDR2),
        amount("7"),
        Some(Bytes::from_static(b"\xab\xcd")),
    );
    let expected = format!(
        "{{\"poolId\":\"pool-1\",\"assetIn\":\"{ADDR1}\",\"assetOut\":\"{ADDR2}\",\
         \"amount\":\"7\",\"userData\":\"0xabcd\"}}"
    );

    assert_json_bytes(&swap, &expected);
}

#[test]
fn batch_swap_step_new_preserves_wire_shape() {
    let step = BatchSwapStep::new(
        "pool-1".to_owned(),
        1,
        2,
        amount("8"),
        Bytes::from_static(b"\xde\xad"),
    );

    assert_json_bytes(
        &step,
        r#"{"poolId":"pool-1","assetInIndex":1,"assetOutIndex":2,"amount":"8","userData":"0xdead"}"#,
    );
}

#[test]
fn swap_execution_new_preserves_wire_shape() {
    let execution = SwapExecution::new(amount("9"));

    assert_json_bytes(&execution, r#"{"limitAmount":"9"}"#);
}

#[test]
fn order_flags_new_preserves_wire_shape() {
    let flags = OrderFlags::new(
        OrderKind::Buy,
        true,
        SellTokenSource::Internal,
        BuyTokenDestination::Internal,
    );

    assert_json_bytes(
        &flags,
        r#"{"kind":"buy","partiallyFillable":true,"sellTokenBalance":"internal","buyTokenBalance":"internal"}"#,
    );
}

#[test]
fn trade_flags_new_preserves_wire_shape() {
    let flags = TradeFlags::new(
        OrderKind::Sell,
        false,
        SellTokenSource::External,
        BuyTokenDestination::Erc20,
        SigningScheme::PreSign,
    );

    assert_json_bytes(
        &flags,
        r#"{"kind":"sell","partiallyFillable":false,"sellTokenBalance":"external","buyTokenBalance":"erc20","signingScheme":"PreSign"}"#,
    );
}

#[test]
fn trade_execution_new_preserves_wire_shape() {
    let execution = TradeExecution::new(amount("10"));

    assert_json_bytes(&execution, r#"{"executedAmount":"10"}"#);
}

#[test]
fn order_refunds_new_preserves_wire_shape() {
    let refunds = OrderRefunds::new(vec![order_uid(UID1)], vec![order_uid(UID2)]);
    let expected = format!("{{\"filledAmounts\":[\"{UID1}\"],\"preSignatures\":[\"{UID2}\"]}}");

    assert_json_bytes(&refunds, &expected);
}

#[test]
fn trade_new_preserves_wire_shape() {
    let trade = Trade::new(
        1,
        2,
        address(ADDR3),
        amount("11"),
        amount("12"),
        99,
        app_data(),
        amount("13"),
        31,
        amount("14"),
        "0x1234".to_owned(),
    );
    let expected = format!(
        "{{\"sellTokenIndex\":1,\"buyTokenIndex\":2,\"receiver\":\"{ADDR3}\",\
         \"sellAmount\":\"11\",\"buyAmount\":\"12\",\"validTo\":99,\"appData\":\"{APP_DATA}\",\
         \"feeAmount\":\"13\",\"flags\":31,\"executedAmount\":\"14\",\"signature\":\"0x1234\"}}"
    );

    assert_json_bytes(&trade, &expected);
}

#[test]
fn eth_flow_order_data_new_preserves_abi_shape() {
    let order = EthFlowOrderData::new(
        address(ADDR1),
        address(ADDR2),
        amount("15"),
        amount("16"),
        app_data(),
        Amount::ZERO,
        77,
        false,
        7,
    )
    .expect("non-zero receiver fixture must construct successfully");

    let actual = encode_create_order_calldata(&order);
    let expected = LocalEthFlow::createOrderCall {
        order: LocalEthFlow::EthFlowOrderData {
            buyToken: SolAddress::from([0x11; 20]),
            receiver: SolAddress::from([0x22; 20]),
            sellAmount: U256::from(15_u64),
            buyAmount: U256::from(16_u64),
            appData: FixedBytes::from([0xaa; 32]),
            feeAmount: U256::ZERO,
            validTo: 77,
            partiallyFillable: false,
            quoteId: 7,
        },
    }
    .abi_encode();

    assert_eq!(actual, expected);
}

#[test]
fn trade_simulation_new_preserves_wire_shape() {
    let simulation = TradeSimulation::new(
        address(ADDR1),
        address(ADDR2),
        Some(address(ADDR3)),
        amount("17"),
        amount("18"),
        Some(SellTokenSource::External),
        Some(BuyTokenDestination::Internal),
        address(ADDR4),
    );
    let expected = format!(
        "{{\"sellToken\":\"{ADDR1}\",\"buyToken\":\"{ADDR2}\",\"receiver\":\"{ADDR3}\",\
         \"sellAmount\":\"17\",\"buyAmount\":\"18\",\"sellTokenBalance\":\"external\",\
         \"buyTokenBalance\":\"internal\",\"owner\":\"{ADDR4}\"}}"
    );

    assert_json_bytes(&simulation, &expected);
}

#[test]
fn trade_simulation_balance_delta_new_preserves_wire_shape() {
    let delta = TradeSimulationBalanceDelta::new(signed_amount("-5"), signed_amount("7"));

    assert_json_bytes(&delta, r#"{"sellTokenDelta":"-5","buyTokenDelta":"7"}"#);
}

#[test]
fn trade_simulation_result_new_preserves_wire_shape() {
    let result = TradeSimulationResult::new(
        amount("21"),
        amount("22"),
        TradeSimulationBalanceDelta::new(signed_amount("-1"), signed_amount("2")),
        TradeSimulationBalanceDelta::new(signed_amount("3"), signed_amount("-4")),
    );

    assert_json_bytes(
        &result,
        r#"{"gasUsed":"21","executedBuyAmount":"22","contractBalance":{"sellTokenDelta":"-1","buyTokenDelta":"2"},"ownerBalance":{"sellTokenDelta":"3","buyTokenDelta":"-4"}}"#,
    );
}

#[test]
fn required_vault_role_new_preserves_wire_shape() {
    let role = RequiredVaultRole::new(
        "swap".to_owned(),
        "0x12345678".to_owned(),
        "0xdeadbeef".to_owned(),
    );

    assert_json_bytes(
        &role,
        r#"{"method":"swap","selector":"0x12345678","role":"0xdeadbeef"}"#,
    );
}

#[test]
fn grant_role_call_new_preserves_wire_shape() {
    let call = GrantRoleCall::new(
        address(ADDR1),
        "[]".to_owned(),
        "grantRole".to_owned(),
        "[]".to_owned(),
    );
    let expected = format!(
        "{{\"authorizerAddress\":\"{ADDR1}\",\"authorizerAbiJson\":\"[]\",\
         \"method\":\"grantRole\",\"argsJson\":\"[]\"}}"
    );

    assert_json_bytes(&call, &expected);
}

#[test]
fn contract_addresses_new_preserves_wire_shape() {
    let addresses = ContractAddresses::new(address(ADDR1), address(ADDR2), address(ADDR3));
    let expected = format!(
        "{{\"settlement\":\"{ADDR1}\",\"vaultRelayer\":\"{ADDR2}\",\"ethFlow\":\"{ADDR3}\"}}"
    );

    assert_json_bytes(&addresses, &expected);
}

#[test]
fn contract_name_enum_preserves_wire_shape() {
    let name = ContractName::TradeSimulator;

    assert_json_bytes(&name, r#""tradeSimulator""#);
}

#[test]
fn adr_0027_signature_family_non_exhaustive() {
    assert_enum_has_non_exhaustive(CONTRACT_SIGNATURE_SOURCE, "SigningScheme");
    assert_enum_has_non_exhaustive(CONTRACT_SIGNATURE_SOURCE, "Signature");
    assert_enum_has_non_exhaustive(ORDERBOOK_TYPES_SOURCE, "SigningScheme");
}

#[test]
fn enum_policy_manifest_entries_match_expected_markers() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("contracts crate must live under crates/contracts");
    let manifest_path = repo_root.join(".github/config/enum-policy.yaml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", manifest_path.display()));
    let policy: EnumPolicyManifest =
        serde_yaml::from_str(&manifest).expect("enum-policy.yaml must parse");

    assert_eq!(
        policy.version, 1,
        "enum-policy schema version must stay at 1"
    );
    assert!(
        !policy.enums.is_empty(),
        "enum-policy.yaml must classify at least one enum",
    );

    for entry in policy.enums {
        if entry.planned {
            // Reserved entry: the source-of-truth Rust definition lands in a
            // later capability landing. Skip the file/line check; the
            // parity-maintainer `validate-enum-policy` subcommand checks the
            // catalog presence separately.
            continue;
        }
        let source_path = repo_root.join(&entry.file);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        let has_marker = enum_has_non_exhaustive(&source, &entry.name);
        match entry.expected_marker.as_str() {
            "non_exhaustive" => assert!(
                has_marker,
                "{} in {} must carry #[non_exhaustive]",
                entry.name, entry.file,
            ),
            "exhaustive" => assert!(
                !has_marker,
                "{} in {} must not carry #[non_exhaustive]",
                entry.name, entry.file,
            ),
            marker => panic!(
                "{} in enum-policy.yaml has unsupported expected_marker `{marker}`",
                entry.name,
            ),
        }
    }
}

#[derive(Deserialize)]
struct EnumPolicyManifest {
    version: u32,
    enums: Vec<EnumPolicyEntry>,
}

#[derive(Deserialize)]
struct EnumPolicyEntry {
    name: String,
    file: String,
    expected_marker: String,
    #[serde(default)]
    planned: bool,
}

fn assert_enum_has_non_exhaustive(source: &str, enum_name: &str) {
    assert!(
        enum_has_non_exhaustive(source, enum_name),
        "public enum `{enum_name}` must carry #[non_exhaustive]",
    );
}

fn enum_has_non_exhaustive(source: &str, enum_name: &str) -> bool {
    let enum_start = source
        .find(&format!("pub enum {enum_name}"))
        .unwrap_or_else(|| panic!("public enum `{enum_name}` must exist"));
    let preceding = &source[..enum_start];
    let item_header_start = preceding
        .rfind("\n///")
        .or_else(|| preceding.rfind("\n#["))
        .map_or(0, |position| position + 1);
    let item_header = &preceding[item_header_start..];

    item_header.contains("#[non_exhaustive]")
}
