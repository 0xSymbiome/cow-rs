use cow_sdk_alloy_signer::LocalAlloyKeystoreSigner;
use cow_sdk_core::{AsyncSigningProvider, SupportedChainId};

fn requires_async_signing_provider<P: AsyncSigningProvider>(_provider: &P) {}

fn main() {
    let signer = LocalAlloyKeystoreSigner::builder()
        .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .unwrap();

    requires_async_signing_provider(&signer);
}
