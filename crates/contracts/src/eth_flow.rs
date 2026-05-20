//! Typed ABI bindings for the `CoWSwapEthFlow` contract.
//!
//! The `CoWSwapEthFlow` contract wraps the native asset into the canonical
//! wrapped-native token and creates the matching EIP-712 order on behalf of
//! the trader. The same contract supports on-chain invalidation of a live
//! EthFlow order by taking the full `EthFlowOrderData` payload (distinct from
//! `GPv2Settlement::invalidateOrder(bytes)`, which takes a packed order UID).
//!
//! Bindings are generated from the canonical upstream Solidity surface via
//! the `alloy::sol!` macro. The Solidity excerpt used to author the bindings
//! lives under `crates/contracts/abi/eth-flow/` for provenance.

use alloy_sol_types::{SolCall, sol};

use cow_sdk_core::{Address, Amount, AppDataHash, UnsignedOrder};

sol! {
    // Canonical CoWSwapEthFlow ABI surface. Signatures are reproduced verbatim
    // from the mainnet-deployed CoWSwapEthFlow contract (upstream source at
    // https://github.com/cowprotocol/ethflowcontract). The Solidity excerpt
    // used to author these bindings is committed under
    // `crates/contracts/abi/eth-flow/` for provenance.
    #[sol(rename_all = "camelcase")]
    interface ICoWSwapEthFlow {
        struct EthFlowOrderData {
            address buyToken;
            address receiver;
            uint256 sellAmount;
            uint256 buyAmount;
            bytes32 appData;
            uint256 feeAmount;
            uint32 validTo;
            bool partiallyFillable;
            int64 quoteId;
        }

        function createOrder(EthFlowOrderData calldata order)
            external
            payable
            returns (bytes32 orderHash);

        function invalidateOrder(EthFlowOrderData calldata order) external;
    }
}

/// Canonical `CoWSwapEthFlow` order-data payload used by both
/// [`encode_create_order_calldata`] and [`encode_invalidate_order_calldata`].
///
/// Field order mirrors the upstream on-chain `EthFlowOrder.Data` struct.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthFlowOrderData {
    /// Buy-token address.
    pub buy_token: Address,
    /// Receiver address.
    pub receiver: Address,
    /// Sell amount (wrapped-native atomic units).
    pub sell_amount: Amount,
    /// Buy amount (atomic units of the buy token).
    pub buy_amount: Amount,
    /// App-data keccak-256 hash.
    pub app_data: AppDataHash,
    /// Fee amount (always zero on the live protocol; surfaced for wire parity).
    pub fee_amount: Amount,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// Whether partial fills are allowed.
    pub partially_fillable: bool,
    /// Quote id linking the transaction back to its originating quote.
    pub quote_id: i64,
}

impl EthFlowOrderData {
    /// Creates an `EthFlowOrderData` payload.
    #[must_use]
    // Mirrors the full current public field set so callers can migrate off
    // struct literals without losing explicit control over any wire field.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        buy_token: Address,
        receiver: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        app_data: AppDataHash,
        fee_amount: Amount,
        valid_to: u32,
        partially_fillable: bool,
        quote_id: i64,
    ) -> Self {
        Self {
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            app_data,
            fee_amount,
            valid_to,
            partially_fillable,
            quote_id,
        }
    }

    /// Builds an `EthFlowOrderData` payload from a pre-signature unsigned order
    /// and the originating quote id.
    #[must_use]
    pub const fn from_unsigned_order(order: &UnsignedOrder, quote_id: i64) -> Self {
        Self::new(
            order.buy_token,
            order.receiver,
            order.sell_amount,
            order.buy_amount,
            order.app_data,
            order.fee_amount,
            order.valid_to,
            order.partially_fillable,
            quote_id,
        )
    }
}

/// Returns the ABI-encoded `createOrder(EthFlowOrderData)` call-data for the
/// `CoWSwapEthFlow` contract.
///
/// Infallible: the cow [`Amount`] / [`AppDataHash`] newtypes enforce the
/// `uint256` and 32-byte boundaries at construction per ADR 0052, so the
/// alloy-sol `abi_encode` call cannot fail by construction.
#[must_use]
pub fn encode_create_order_calldata(order: &EthFlowOrderData) -> Vec<u8> {
    ICoWSwapEthFlow::createOrderCall {
        order: to_sol_struct(order),
    }
    .abi_encode()
}

/// Returns the ABI-encoded `invalidateOrder(EthFlowOrderData)` call-data for the
/// `CoWSwapEthFlow` contract.
///
/// This is distinct from the `GPv2Settlement::invalidateOrder(bytes orderUid)`
/// call: `EthFlow` on-chain cancellation takes the full order payload back,
/// while the settlement-level invalidation only needs the packed UID.
///
/// Infallible: the cow [`Amount`] / [`AppDataHash`] newtypes enforce the
/// `uint256` and 32-byte boundaries at construction per ADR 0052, so the
/// alloy-sol `abi_encode` call cannot fail by construction.
#[must_use]
pub fn encode_invalidate_order_calldata(order: &EthFlowOrderData) -> Vec<u8> {
    ICoWSwapEthFlow::invalidateOrderCall {
        order: to_sol_struct(order),
    }
    .abi_encode()
}

fn to_sol_struct(order: &EthFlowOrderData) -> ICoWSwapEthFlow::EthFlowOrderData {
    use alloy_sol_types::private::{Address as SolAddress, FixedBytes};

    // The cow `Amount` newtype is `#[repr(transparent)]` over
    // `alloy_primitives::U256`, so the conversion to the sol `U256`
    // surface is a single deref of the inner U256 with no intermediate
    // bigint allocation and no overflow guard required. The same holds
    // for `AppDataHash` over `B256`, so `as_alloy().0` exposes the
    // packed 32-byte payload directly.
    let buy_token_bytes = order.buy_token.into_alloy().0.0;
    let receiver_bytes = order.receiver.into_alloy().0.0;
    let app_data_bytes = order.app_data.as_alloy().0;

    ICoWSwapEthFlow::EthFlowOrderData {
        buyToken: SolAddress::from(buy_token_bytes),
        receiver: SolAddress::from(receiver_bytes),
        sellAmount: *order.sell_amount.as_u256(),
        buyAmount: *order.buy_amount.as_u256(),
        appData: FixedBytes::from(app_data_bytes),
        feeAmount: *order.fee_amount.as_u256(),
        validTo: order.valid_to,
        partiallyFillable: order.partially_fillable,
        quoteId: order.quote_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::{Digest, Keccak256};

    fn sample_order() -> EthFlowOrderData {
        EthFlowOrderData::new(
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            Amount::new("1000000000000000000").unwrap(),
            Amount::new("2000000000000000000").unwrap(),
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
            Amount::zero(),
            0x1234_5678,
            false,
            42,
        )
    }

    fn canonical_create_order_selector() -> [u8; 4] {
        let signature =
            "createOrder((address,address,uint256,uint256,bytes32,uint256,uint32,bool,int64))";
        let digest = Keccak256::digest(signature.as_bytes());
        [digest[0], digest[1], digest[2], digest[3]]
    }

    fn canonical_invalidate_order_selector() -> [u8; 4] {
        let signature =
            "invalidateOrder((address,address,uint256,uint256,bytes32,uint256,uint32,bool,int64))";
        let digest = Keccak256::digest(signature.as_bytes());
        [digest[0], digest[1], digest[2], digest[3]]
    }

    fn word_hex(bytes: &[u8], index: usize) -> String {
        let start = 4 + index * 32;
        hex::encode(&bytes[start..start + 32])
    }

    #[test]
    fn create_order_calldata_starts_with_the_canonical_upstream_selector() {
        let order = sample_order();
        let encoded = encode_create_order_calldata(&order);
        assert_eq!(
            &encoded[..4],
            canonical_create_order_selector(),
            "createOrder selector must match the upstream EthFlowOrder.Data field order",
        );
        // (address, address, uint256, uint256, bytes32, uint256, uint32, bool, int64)
        // is a static tuple whose head consumes 9 * 32 = 288 bytes; plus the 4-byte
        // selector gives 292 bytes of total call-data.
        assert_eq!(
            encoded.len(),
            4 + 9 * 32,
            "createOrder call-data must be selector + 9 head words for the static struct layout",
        );
    }

    #[test]
    fn invalidate_order_calldata_starts_with_the_canonical_upstream_selector() {
        let order = sample_order();
        let encoded = encode_invalidate_order_calldata(&order);
        assert_eq!(
            &encoded[..4],
            canonical_invalidate_order_selector(),
            "invalidateOrder(EthFlowOrderData) selector must match the upstream field order",
        );
        assert_eq!(
            encoded.len(),
            4 + 9 * 32,
            "invalidateOrder call-data must be selector + 9 head words",
        );
    }

    #[test]
    fn encoded_struct_head_follows_the_upstream_field_order() {
        let order = sample_order();
        let encoded = encode_create_order_calldata(&order);

        // word 0: buyToken (right-aligned 20-byte address)
        assert_eq!(
            word_hex(&encoded, 0),
            "0000000000000000000000001111111111111111111111111111111111111111",
        );
        // word 1: receiver
        assert_eq!(
            word_hex(&encoded, 1),
            "0000000000000000000000002222222222222222222222222222222222222222",
        );
        // word 2: sellAmount (1e18)
        assert_eq!(
            word_hex(&encoded, 2),
            "0000000000000000000000000000000000000000000000000de0b6b3a7640000",
        );
        // word 3: buyAmount (2e18)
        assert_eq!(
            word_hex(&encoded, 3),
            "0000000000000000000000000000000000000000000000001bc16d674ec80000",
        );
        // word 4: appData (bytes32, left-aligned)
        assert_eq!(
            word_hex(&encoded, 4),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        // word 5: feeAmount (0)
        assert_eq!(
            word_hex(&encoded, 5),
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        // word 6: validTo (0x12345678, right-aligned)
        assert_eq!(
            word_hex(&encoded, 6),
            "0000000000000000000000000000000000000000000000000000000012345678",
        );
        // word 7: partiallyFillable (false)
        assert_eq!(
            word_hex(&encoded, 7),
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        // word 8: quoteId (int64 42, sign-extended)
        assert_eq!(
            word_hex(&encoded, 8),
            "000000000000000000000000000000000000000000000000000000000000002a",
        );
    }

    #[test]
    fn negative_quote_id_sign_extends_to_the_full_256_bit_word() {
        let mut order = sample_order();
        order.quote_id = -1;
        let encoded = encode_create_order_calldata(&order);
        let word_hex = hex::encode(&encoded[4 + 8 * 32..4 + 9 * 32]);
        assert_eq!(
            word_hex, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "negative int64 quote id must sign-extend to a full 256-bit two's-complement word",
        );
    }

    #[test]
    fn u256_amount_encoding_preserves_max_value() {
        // The cow `Amount` newtype is `#[repr(transparent)]` over
        // `alloy_primitives::U256` per ADR 0052, so the `uint256` ceiling
        // is enforced by the type system at construction and the
        // ABI-encoded sellAmount cannot exceed 32 bytes; the historical
        // `Amount::from_atoms(BigUint::from(1u8) << 256usize)` overflow
        // arm collapses into a compile-time impossibility and is no
        // longer needed at runtime.
        let mut max_order = sample_order();
        max_order.sell_amount = Amount::from_u256(alloy_primitives::U256::MAX);

        let encoded = encode_create_order_calldata(&max_order);
        assert_eq!(
            word_hex(&encoded, 2),
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "sellAmount must preserve all 32 bytes of the maximum uint256",
        );
    }
}
