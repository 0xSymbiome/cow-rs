mod common;

use cow_sdk_contracts::{
    Eip1967Slot, IEip173Proxy, admin_address, implementation_address, owner_address,
};
use cow_sdk_core::Address;
use sha3::{Digest, Keccak256};

use common::{MockProvider, fixture_case};

#[tokio::test]
async fn proxy_constants_and_storage_readers_match_contract_surface() {
    let fixture = fixture_case("contracts-proxy-storage-slots");
    assert_eq!(
        Eip1967Slot::Implementation.as_hex_str(),
        fixture["expected"]["implementation_slot"].as_str().unwrap()
    );
    assert_eq!(
        Eip1967Slot::Admin.as_hex_str(),
        fixture["expected"]["owner_slot"].as_str().unwrap(),
        "fixture `owner_slot` field stores the EIP-1967 admin-slot hash by spec",
    );

    let proxy = Address::new("0x1234567890123456789012345678901234567890").unwrap();
    let provider = MockProvider::new();
    provider.set_storage(
        &proxy,
        Eip1967Slot::Implementation.as_hex_str(),
        "0x0000000000000000000000001111111111111111111111111111111111111111",
    );
    provider.set_storage(
        &proxy,
        Eip1967Slot::Admin.as_hex_str(),
        "0x0000000000000000000000002222222222222222222222222222222222222222",
    );

    assert_eq!(
        implementation_address(&provider, &proxy)
            .await
            .unwrap()
            .to_hex_string(),
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        admin_address(&provider, &proxy)
            .await
            .unwrap()
            .to_hex_string(),
        "0x2222222222222222222222222222222222222222"
    );
    assert_eq!(
        owner_address(&provider, &proxy)
            .await
            .unwrap()
            .to_hex_string(),
        admin_address(&provider, &proxy)
            .await
            .unwrap()
            .to_hex_string(),
        "owner_address is the legacy alias for admin_address",
    );
}

#[test]
fn eip1967_slot_bytes_match_the_canonical_hex_payload() {
    let admin_bytes = Eip1967Slot::Admin.as_bytes();
    assert_eq!(
        format!("0x{}", alloy_primitives::hex::encode(admin_bytes)),
        Eip1967Slot::Admin.as_hex_str(),
        "admin slot bytes must round-trip through the typed hex accessor",
    );

    let implementation_bytes = Eip1967Slot::Implementation.as_bytes();
    assert_eq!(
        format!("0x{}", alloy_primitives::hex::encode(implementation_bytes)),
        Eip1967Slot::Implementation.as_hex_str(),
        "implementation slot bytes must round-trip through the typed hex accessor",
    );
}

#[test]
fn eip1967_slot_hex_strings_match_their_byte_forms() {
    // Pins the relationship between the `fixed_bytes!`-emitted byte
    // form (the typed source of truth) and the parallel `&'static str`
    // hex form (consumed by the `Provider::get_storage_at(slot: &str)`
    // trait-shape stability seam). If a future contributor edits one
    // without the other, this round-trip assertion catches the drift
    // before the canonical-keccak parity test below does.
    for slot in [Eip1967Slot::Admin, Eip1967Slot::Implementation] {
        assert_eq!(
            slot.as_hex_str(),
            format!("{:#x}", slot.as_bytes()),
            "EIP-1967 slot hex string must equal the LowerHex rendering of the byte form for {slot:?}",
        );
    }
}

#[test]
fn eip173_proxy_interface_exposes_the_expected_function_selectors() {
    use alloy_sol_types::SolCall;

    assert_eq!(IEip173Proxy::ownerCall::SIGNATURE, "owner()");
    assert_eq!(
        IEip173Proxy::transferOwnershipCall::SIGNATURE,
        "transferOwnership(address)",
    );
    assert_eq!(
        IEip173Proxy::supportsInterfaceCall::SIGNATURE,
        "supportsInterface(bytes4)",
    );
}

fn canonical_eip1967_slot(label: &str) -> String {
    let mut bytes: [u8; 32] = Keccak256::digest(label.as_bytes()).into();
    for byte in bytes.iter_mut().rev() {
        if *byte == 0 {
            *byte = u8::MAX;
        } else {
            *byte -= 1;
            break;
        }
    }
    format!("0x{}", alloy_primitives::hex::encode(bytes))
}

#[test]
fn eip1967_slot_constants_match_canonical_keccak_minus_one() {
    assert_eq!(
        Eip1967Slot::Implementation.as_hex_str(),
        canonical_eip1967_slot("eip1967.proxy.implementation")
    );
    assert_eq!(
        Eip1967Slot::Admin.as_hex_str(),
        canonical_eip1967_slot("eip1967.proxy.admin")
    );
}
