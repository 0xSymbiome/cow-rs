mod common;

use cow_sdk_contracts::{
    VAULT_INTERFACE, grant_required_roles, required_vault_role_calls, required_vault_roles,
};
use cow_sdk_core::Address;

use common::fixture_case;

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
