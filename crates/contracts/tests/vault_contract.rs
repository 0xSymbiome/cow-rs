use cow_sdk_contracts::{
    VAULT_INTERFACE, grant_required_roles, required_vault_role_calls, required_vault_roles,
};
use cow_sdk_core::Address;
use sha3::{Digest, Keccak256};

const MAINNET_VAULT_ADDRESS: &str = "0xBA12222222228d8Ba445958a75a0704d566BF2C8";
const EXPECTED_MAINNET_VAULT_ROLES: [(&str, &str, &str); 2] = [
    (
        "manageUserBalance",
        "0x0e8e3e84",
        "0xeba777d811cd36c06d540d7ff2ed18ed042fd67bbf7c9afcf88c818c7ee6b498",
    ),
    (
        "batchSwap",
        "0x945bcec9",
        "0x1282ab709b2b70070f829c46bc36f76b32ad4989fecb2fcb09a1b3ce00bbfc30",
    ),
];

fn expected_role_hash(vault_bytes: &[u8], selector: [u8; 4]) -> String {
    let mut payload = [0u8; 36];
    payload[12..32].copy_from_slice(vault_bytes);
    payload[32..].copy_from_slice(&selector);
    let digest = Keccak256::digest(payload);
    format!("0x{}", alloy_primitives::hex::encode(digest))
}

fn selector_from_hex(selector: &str) -> [u8; 4] {
    let bytes = alloy_primitives::hex::decode(selector.trim_start_matches("0x"))
        .expect("selector literal must decode");
    bytes
        .try_into()
        .expect("selector literal must be exactly four bytes")
}

#[test]
fn vault_roles_match_interface_arity_and_selector_shape() {
    let vault = Address::new(MAINNET_VAULT_ADDRESS).unwrap();
    let roles = required_vault_roles(&vault).unwrap();
    assert_eq!(roles.len(), VAULT_INTERFACE.len());
    assert!(roles.iter().all(|role| role.role.starts_with("0x")));
    assert_eq!(roles[0].selector.len(), 10);
}

#[test]
fn vault_role_calls_and_grant_flow_are_stable() {
    let authorizer = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let vault = Address::new(MAINNET_VAULT_ADDRESS).unwrap();
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
        |call| serde_json::from_str::<Vec<String>>(&call.args_json).unwrap()[1]
            == relayer.to_hex_string()
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
fn vault_role_hashes_match_the_canonical_solidity_packed_layout() {
    let vault = Address::new(MAINNET_VAULT_ADDRESS).unwrap();
    let vault_bytes = alloy_primitives::hex::decode(vault.to_hex_string().trim_start_matches("0x"))
        .expect("vault literal must decode");

    let roles = required_vault_roles(&vault).unwrap();
    assert_eq!(roles.len(), EXPECTED_MAINNET_VAULT_ROLES.len());

    for (role, &(method, selector, expected_hash)) in
        roles.iter().zip(EXPECTED_MAINNET_VAULT_ROLES.iter())
    {
        let selector_bytes = selector_from_hex(selector);
        assert_eq!(role.method, method);
        assert_eq!(
            role.selector, selector,
            "{method} selector must match the canonical Solidity signature hash",
        );
        assert_eq!(
            role.role,
            expected_role_hash(&vault_bytes, selector_bytes),
            "{method} role hash must match the 36-byte packed upstream layout",
        );
        assert_eq!(
            role.role, expected_hash,
            "{method} role hash must match the canonical Mainnet Vault vector",
        );
    }

    let authorizer = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let relayer = Address::new("0x2222222222222222222222222222222222222222").unwrap();
    let abi_json = serde_json::to_string(&["function grantRole(bytes32,address)"]).unwrap();
    let calls = required_vault_role_calls(&authorizer, &abi_json, &vault, &relayer).unwrap();
    assert_eq!(calls.len(), EXPECTED_MAINNET_VAULT_ROLES.len());

    for (call, &(_, _, expected_hash)) in calls.iter().zip(EXPECTED_MAINNET_VAULT_ROLES.iter()) {
        assert_eq!(call.authorizer_address, authorizer);
        assert_eq!(call.authorizer_abi_json, abi_json);
        assert_eq!(call.method, "grantRole");

        let args = serde_json::from_str::<Vec<String>>(&call.args_json).unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], expected_hash);
        assert_eq!(args[1], relayer.to_hex_string());
    }
}
