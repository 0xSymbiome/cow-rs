//! Canonical `GPv2` `OrderCancellations` sol! type definition.
//!
//! The macro-emitted `OrderCancellations` struct is the single source of
//! truth for the `GPv2` batch-cancellation EIP-712 schema. The canonical
//! type string is `OrderCancellations(bytes[] orderUids)`.
//!
//! The Rust struct name MUST stay `OrderCancellations` (not
//! `GPv2OrderCancellations` or any other variant) because the alloy
//! sol! macro derives the EIP-712 type-name prefix from the Rust struct
//! name; renaming it would change the type-hash bytes.

alloy_sol_types::sol! {
    /// `GPv2` batch order cancellation typed-data struct.
    #[derive(Debug, Default, PartialEq, Eq)]
    struct OrderCancellations {
        bytes[] orderUids;
    }
}
