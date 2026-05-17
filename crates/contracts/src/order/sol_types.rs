//! Canonical `GPv2` `Order` sol! type definition.
//!
//! The macro-emitted `Order` struct is the single source of truth for
//! the `GPv2` EIP-712 order schema. The canonical type string is
//! `Order(address sellToken,address buyToken,address receiver,uint256
//! sellAmount,uint256 buyAmount,uint32 validTo,bytes32 appData,uint256
//! feeAmount,string kind,bool partiallyFillable,string sellTokenBalance,
//! string buyTokenBalance)`, which keccak256-hashes to the protocol
//! constant
//! `0xd5a25ba2e97094ad7d83dc28a6572da797d6b3e7fc6663bd93efb789fc17e489`.
//!
//! The Rust struct name MUST stay `Order` (not `GPv2Order` or any other
//! variant) because the alloy sol! macro derives the EIP-712 type-name
//! prefix from the Rust struct name; renaming it would change the
//! type-hash bytes.

alloy_sol_types::sol! {
    /// `GPv2` settlement `Order` typed-data struct.
    #[derive(Debug, Default, PartialEq, Eq)]
    struct Order {
        address sellToken;
        address buyToken;
        address receiver;
        uint256 sellAmount;
        uint256 buyAmount;
        uint32 validTo;
        bytes32 appData;
        uint256 feeAmount;
        string kind;
        bool partiallyFillable;
        string sellTokenBalance;
        string buyTokenBalance;
    }
}

#[cfg(test)]
mod tests {
    use super::Order;
    use alloy_primitives::b256;
    use alloy_sol_types::SolStruct;

    /// Pins the macro-emitted `GPv2` `Order` type hash to the deployed
    /// protocol constant.
    #[test]
    fn order_type_hash_matches_protocol_constant() {
        let expected = b256!("0xd5a25ba2e97094ad7d83dc28a6572da797d6b3e7fc6663bd93efb789fc17e489");
        let sample = Order::default();
        assert_eq!(sample.eip712_type_hash(), expected);
    }
}
