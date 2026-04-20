mod common;

use cow_sdk_contracts::{
    VAULT_INTERFACE, grant_required_roles, required_vault_role_calls, required_vault_roles,
};
use cow_sdk_core::Address;
use sha3::{Digest, Keccak256};

use common::fixture_case;

fn expected_role_hash(vault_bytes: &[u8], selector: [u8; 4]) -> String {
    let mut payload = [0u8; 64];
    payload[12..32].copy_from_slice(vault_bytes);
    payload[32..36].copy_from_slice(&selector);
    let digest = Keccak256::digest(payload);
    format!("0x{}", hex::encode(digest))
}

#[test]
fn vault_roles_cover_the_expected_methods() {
    let fixture = fixture_case("contracts-vault-required-methods");
    let methods = fixture["expected"]["methods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();

    let vault = Address::new("0xBA12222222228d8Ba445958a75a0704d566BF2C8").unwrap();
    let roles = required_vault_roles(&vault).unwrap();
    assert_eq!(roles.len(), VAULT_INTERFACE.len());
    assert_eq!(
        roles
            .iter()
            .map(|role| role.method.clone())
            .collect::<Vec<_>>(),
        methods
    );
    assert!(roles.iter().all(|role| role.role.starts_with("0x")));
    assert_eq!(roles[0].selector.len(), 10);
}

#[test]
fn vault_role_calls_and_grant_flow_are_stable() {
    let authorizer = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let vault = Address::new("0xBA12222222228d8Ba445958a75a0704d566BF2C8").unwrap();
    let relayer = Address::new("0x2222222222222222222222222222222222222222").unwrap();
    let abi_json = serde_json::to_string(&["function grantRole(bytes32,address)"]).unwrap();

    let calls = required_vault_role_calls(&authorizer, &abi_json, &vault, &relayer).unwrap();
    assert_eq!(calls.len(), 2);
    assert!(calls.iter().all(|call| call.method == "grantRole"));
    assert!(
        calls
            .iter()
            .all(|call| call.authorizer_address == authorizer)
    );
    assert!(
        calls
            .iter()
            .all(|call| call.authorizer_abi_json == abi_json)
    );
    assert!(calls.iter().all(
        |call| serde_json::from_str::<Vec<String>>(&call.args_json).unwrap()[1] == relayer.as_str()
    ));

    let mut granted = Vec::new();
    grant_required_roles(&authorizer, &abi_json, &vault, &relayer, |call| {
        granted.push(call.method.clone());
        Ok::<_, std::io::Error>(())
    })
    .unwrap();
    assert_eq!(
        granted,
        vec!["grantRole".to_owned(), "grantRole".to_owned()]
    );

    let error = grant_required_roles(&authorizer, &abi_json, &vault, &relayer, |_call| {
        Err::<(), _>(std::io::Error::other("grant failed"))
    });
    assert!(error.is_err());
}

#[test]
fn vault_role_hashes_match_the_canonical_abi_encode_byte_layout() {
    // The role hash is defined as `keccak256(abi.encode(vault_address, selector))`.
    // abi.encode of a `(address, bytes4)` tuple produces two 32-byte words:
    //
    //   word 0 (address): 12 zero bytes | 20-byte address
    //   word 1 (bytes4) : 4-byte selector | 28 zero trailing bytes
    //
    // Total: 64 bytes. Compute the expected hash through a hand-rolled byte
    // layout and confirm every role produced by `required_vault_roles` matches
    // that canonical shape; a regression to any other encoding rule would
    // invalidate every already-granted Balancer Authorizer role in production.
    let vault = Address::new("0xBA12222222228d8Ba445958a75a0704d566BF2C8").unwrap();
    let vault_bytes =
        hex::decode(vault.as_str().trim_start_matches("0x")).expect("vault literal must decode");

    let manage_user_balance_selector = {
        let signature = "manageUserBalance((uint8,address,uint256,address,address)[])";
        let digest = Keccak256::digest(signature.as_bytes());
        [digest[0], digest[1], digest[2], digest[3]]
    };
    let batch_swap_selector = {
        let signature = "batchSwap(uint8,(bytes32,uint256,uint256,uint256,bytes)[],address[],(address,bool,address,bool),int256[],uint256)";
        let digest = Keccak256::digest(signature.as_bytes());
        [digest[0], digest[1], digest[2], digest[3]]
    };

    let roles = required_vault_roles(&vault).unwrap();
    assert_eq!(roles.len(), 2);

    assert_eq!(roles[0].method, "manageUserBalance");
    assert_eq!(
        roles[0].selector,
        format!("0x{}", hex::encode(manage_user_balance_selector)),
        "manageUserBalance selector must match the canonical Solidity signature hash",
    );
    assert_eq!(
        roles[0].role,
        expected_role_hash(&vault_bytes, manage_user_balance_selector),
        "manageUserBalance role hash must match keccak256(abi.encode(vault, selector))",
    );

    assert_eq!(roles[1].method, "batchSwap");
    assert_eq!(
        roles[1].selector,
        format!("0x{}", hex::encode(batch_swap_selector)),
        "batchSwap selector must match the canonical Solidity signature hash",
    );
    assert_eq!(
        roles[1].role,
        expected_role_hash(&vault_bytes, batch_swap_selector),
        "batchSwap role hash must match keccak256(abi.encode(vault, selector))",
    );
}
