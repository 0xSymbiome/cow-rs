use alloy_signer_local::PrivateKeySigner;
use cow_sdk_alloy_signer::{ChainSet, ChainUnset, KeySourceUnset, PrivateKeySource};
use cow_sdk_core::{ChainId, SupportedChainId};

fn main() {
    key_source_unset();
    chain_unset();
    private_key_source();
    chain_set();
}

fn key_source_unset() {
    let _ = KeySourceUnset {};
}

fn chain_unset() {
    let _ = ChainUnset {};
}

fn private_key_source() {
    let signer: PrivateKeySigner =
        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
            .parse()
            .unwrap();
    let _ = PrivateKeySource { signer };
}

fn chain_set() {
    let _ = ChainSet {
        chain_id: ChainId::from(SupportedChainId::Mainnet),
    };
}
