//! Pinned compile-fail witness for the narrowed `PartnerFeePolicy` bps width.
//!
//! The reviewed partner-fee schema caps every basis-point field inside the
//! `u16` range; the cow-rs public contract narrows each field from `u32` to
//! `u16` so out-of-range integer literals fail at the compiler rather than at
//! the wire. The witness below proves that a `u32` value cannot be assigned
//! into the narrowed `u16` field inside `PartnerFeePolicy::Volume`.
//!
//! Cargo's default integration-test discovery only picks up `tests/*.rs`, so
//! this source is never compiled by `cargo test --workspace`. The sibling
//! contract suite `crates/app-data/tests/partner_fee_contract.rs` asserts the
//! source and its captured `stderr` remain pinned in the tree.

use cow_sdk_app_data::PartnerFeePolicy;
use cow_sdk_core::Address;

fn main() {
    let recipient = Address::new("0x0101010101010101010101010101010101010101").unwrap();

    // Assigning a `u32` literal above the `u16` range into the narrowed
    // `volume_bps` field must fail to compile so the published [1, 100] cap
    // on partner fees cannot be silently widened by a caller-supplied value.
    let _policy = PartnerFeePolicy::Volume {
        volume_bps: 100_000_u32,
        recipient,
    };
}
