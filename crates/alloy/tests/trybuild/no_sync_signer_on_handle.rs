use cow_sdk_alloy::AlloyClientSignerHandle;
use cow_sdk_core::Signer;

fn requires_sync_signer<S: Signer>() {}

fn main() {
    requires_sync_signer::<AlloyClientSignerHandle>();
}
