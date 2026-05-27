use cow_sdk_alloy_provider::RpcAlloyProvider;
use cow_sdk_core::SigningProvider;

fn main() {
    fn assert_impl<T: SigningProvider>() {}
    assert_impl::<RpcAlloyProvider>();
}
