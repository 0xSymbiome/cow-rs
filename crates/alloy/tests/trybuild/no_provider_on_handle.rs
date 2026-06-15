use cow_sdk_alloy::AlloyClientSignerHandle;
use cow_sdk_core::Provider;

fn requires_provider<P: Provider>() {}

fn main() {
    requires_provider::<AlloyClientSignerHandle>();
}
