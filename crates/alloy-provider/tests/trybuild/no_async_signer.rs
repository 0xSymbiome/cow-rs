use cow_sdk_alloy_provider::RpcAlloyProvider;
use cow_sdk_core::AsyncSigner;

fn main() {
    fn assert_impl<T: AsyncSigner>() {}
    assert_impl::<RpcAlloyProvider>();
}
