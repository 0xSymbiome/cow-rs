mod common;

use cow_sdk_contracts::{
    EIP173_PROXY_ABI, IMPLEMENTATION_STORAGE_SLOT, OWNER_STORAGE_SLOT, implementation_address,
    owner_address, proxy_interface,
};
use cow_sdk_core::Address;

use common::{MockProvider, fixture_case};

#[test]
fn proxy_constants_and_storage_readers_match_contract_surface() {
    let fixture = fixture_case("contracts-proxy-storage-slots");
    assert_eq!(
        IMPLEMENTATION_STORAGE_SLOT,
        fixture["expected"]["implementation_slot"].as_str().unwrap()
    );
    assert_eq!(
        OWNER_STORAGE_SLOT,
        fixture["expected"]["owner_slot"].as_str().unwrap()
    );

    let proxy = Address::new("0x1234567890123456789012345678901234567890").unwrap();
    let provider = MockProvider::new();
    provider.set_storage(
        &proxy,
        IMPLEMENTATION_STORAGE_SLOT,
        "0x0000000000000000000000001111111111111111111111111111111111111111",
    );
    provider.set_storage(
        &proxy,
        OWNER_STORAGE_SLOT,
        "0x0000000000000000000000002222222222222222222222222222222222222222",
    );

    assert_eq!(
        implementation_address(&provider, &proxy).unwrap().as_str(),
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        owner_address(&provider, &proxy).unwrap().as_str(),
        "0x2222222222222222222222222222222222222222"
    );
}

#[test]
fn proxy_interface_exposes_eip173_abi_handle() {
    let proxy = Address::new("0x1234567890123456789012345678901234567890").unwrap();
    let handle = proxy_interface(&proxy).unwrap();
    let abi: Vec<String> = serde_json::from_str(&handle.abi_json).unwrap();

    assert_eq!(handle.address, proxy);
    assert_eq!(abi.len(), EIP173_PROXY_ABI.len());
    assert_eq!(abi[0], EIP173_PROXY_ABI[0]);
    assert_eq!(abi[1], EIP173_PROXY_ABI[1]);
}
