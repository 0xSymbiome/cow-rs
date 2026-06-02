#[path = "support/order_sign_submit_smoke.rs"]
mod order_sign_submit_smoke;

fn main() {
    let _ = order_sign_submit_smoke::smoke_hash_and_uid();
}
