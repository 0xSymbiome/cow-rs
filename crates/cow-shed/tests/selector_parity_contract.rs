use alloy_sol_types::SolCall;
use cow_sdk_cow_shed::bindings::{COWShed, COWShedFactory};

#[cfg(feature = "cow-shed-gnosis")]
use cow_sdk_cow_shed::bindings::COWShedForComposableCoW;

#[test]
fn factory_and_proxy_selectors_match_canonical_signatures() {
    assert_eq!(
        COWShedFactory::initializeProxyCall::SELECTOR,
        hex4("66b14069")
    );
    assert_eq!(COWShedFactory::executeHooksCall::SELECTOR, hex4("a8481abe"));
    assert_eq!(COWShed::executeHooksCall::SELECTOR, hex4("ffdacefc"));
    assert_eq!(
        COWShed::executePreSignedHooksCall::SELECTOR,
        hex4("0dfdefe8")
    );
    assert_eq!(COWShed::preSignHooksCall::SELECTOR, hex4("dd0064b3"));
    assert_eq!(COWShed::isPreSignedHooksCall::SELECTOR, hex4("4f0b914b"));
}

#[cfg(feature = "cow-shed-gnosis")]
#[test]
fn gnosis_forwarder_selector_matches_erc1271() {
    assert_eq!(
        COWShedForComposableCoW::isValidSignatureCall::SELECTOR,
        hex4("1626ba7e")
    );
}

fn hex4(value: &str) -> [u8; 4] {
    let bytes = alloy_primitives::hex::decode(value).expect("selector fixture parses");
    let mut out = [0_u8; 4];
    out.copy_from_slice(&bytes);
    out
}
