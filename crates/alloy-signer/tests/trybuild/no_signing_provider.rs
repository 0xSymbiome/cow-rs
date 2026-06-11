use cow_sdk_alloy_signer::LocalAlloySigner;
use cow_sdk_core::{SigningProvider, SupportedChainId};

fn requires_signing_provider<P: SigningProvider>(_provider: &P) {}

fn main() {
    let signer = LocalAlloySigner::builder()
        .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .unwrap();

    requires_signing_provider(&signer);
}
