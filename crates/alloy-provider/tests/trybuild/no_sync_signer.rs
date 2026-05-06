use cow_sdk_alloy_provider::RpcAlloyProvider;
use cow_sdk_core::Signer;

fn main() {
    fn assert_impl<T: Signer>() {}
    assert_impl::<RpcAlloyProvider>();
}
