use cow_sdk_alloy::AlloyClientSignerHandle;
use cow_sdk_core::AsyncProvider;

fn requires_async_provider<P: AsyncProvider>() {}

fn main() {
    requires_async_provider::<AlloyClientSignerHandle>();
}
