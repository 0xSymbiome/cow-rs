use cow_sdk_alloy_provider::RpcAlloyProvider;
use cow_sdk_core::AsyncSigningProvider;

fn main() {
    fn assert_impl<T: AsyncSigningProvider>() {}
    assert_impl::<RpcAlloyProvider>();
}
