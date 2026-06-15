use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::Signer;

fn requires_signer<S: Signer>() {}

fn main() {
    requires_signer::<AlloyClient>();
}
