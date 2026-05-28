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

use alloy_primitives::Bytes;
use alloy_sol_types::{SolCall, sol};

use cow_sdk_core::{Address, Amount, AppDataHash, UnsignedOrder};

use crate::ContractsError;
use crate::interaction::Interaction;
use crate::order::hash::reject_zero_receiver;

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
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::ZeroReceiver`] when `receiver` is the
    /// zero address. The deployed `CoWSwapEthFlow` contract reverts both
    /// `createOrder` and `invalidateOrder` calldata with
    /// `ReceiverMustBeSet()` on this input (selector `0xefc9ccdf`),
    /// raised from `EthFlowOrder.toCoWSwapOrder` in the upstream
    /// `cowprotocol/ethflowcontract` Solidity surface, so the SDK
    /// refuses to produce the calldata in the first place.
    // Mirrors the full current public field set so callers can migrate off
    // struct literals without losing explicit control over any wire field.
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the public field set so callers can migrate off struct-literal construction without losing explicit control over any wire field"
    )]
    pub fn new(
        buy_token: Address,
        receiver: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        app_data: AppDataHash,
        fee_amount: Amount,
        valid_to: u32,
        partially_fillable: bool,
        quote_id: i64,
    ) -> Result<Self, ContractsError> {
        reject_zero_receiver(&receiver)?;
        Ok(Self {
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            app_data,
            fee_amount,
            valid_to,
            partially_fillable,
            quote_id,
        })
    }

    /// Builds an `EthFlowOrderData` payload from a pre-signature unsigned order
    /// and the originating quote id.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::ZeroReceiver`] under the same condition
    /// as [`Self::new`].
    pub fn from_unsigned_order(
        order: &UnsignedOrder,
        quote_id: i64,
    ) -> Result<Self, ContractsError> {
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

/// Builds the `ICoWSwapEthFlow::EthFlowOrderData` sol-typed struct from the
/// cow [`EthFlowOrderData`] value.
///
/// This helper is intentionally **not** declared `const fn`. The alloy
/// `From<[u8; N]>` impls on [`alloy_primitives::Address`] and
/// [`alloy_primitives::FixedBytes`] go through `derive_more::From` or
/// the `wrap_fixed_bytes!` macro, both of which generate plain
/// `fn from(...)` rather than `const fn from(...)`. Const-trait support
/// is not yet stable on the Rust toolchain this crate targets
/// (RFC 3762 tracks the path).
///
/// The only const-callable workaround would use the cow newtype
/// field-access escape hatch (e.g. `EthFlowOrderData.buy_token.into_alloy().0.0`),
/// which is documented under ADR 0052 as a non-stable forward-compatibility
/// surface. Promotion would also buy nothing in practice because every
/// public caller of this helper routes through `abi_encode`, which
/// heap-allocates a `Vec<u8>` and is never const-callable; the
/// pre-encoding step has no observable cost difference between `fn` and
/// `const fn`.
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

/// Function selector for the `CoWSwapEthFlow` `wrapAll()` entrypoint.
///
/// Equals `keccak256("wrapAll()")[..4]`. The eth-flow contract wraps its entire
/// native-asset balance into the wrapped-native token through this call; an
/// on-chain order placed through eth-flow is preceded by a `wrapAll()`
/// pre-interaction targeting the eth-flow contract.
pub const WRAP_ALL_SELECTOR: [u8; 4] = [0x4c, 0x84, 0xc1, 0xc8];

/// Builds the `wrapAll()` pre-interaction targeting a `CoWSwapEthFlow` contract.
///
/// The interaction calls `wrapAll()` on `eth_flow` with no arguments and zero
/// native value; it is the pre-interaction associated with an eth-flow on-chain
/// order.
#[must_use]
pub fn wrap_all_interaction(eth_flow: Address) -> Interaction {
    Interaction::new(
        eth_flow,
        Amount::ZERO,
        Bytes::from(WRAP_ALL_SELECTOR.to_vec()),
    )
}

/// Decoded trailing data carried by an eth-flow `OrderPlacement` event.
///
/// The eth-flow contract sets the `OrderPlacement` event's trailing `data`
/// field to `abi.encodePacked(int64 quoteId, uint32 userValidTo)` — a 12-byte,
/// big-endian payload that carries the originating quote id and the trader's
/// real (pre-clamp) order expiry, neither of which survives in the on-chain
/// `GPv2` order whose `validTo` is fixed to `u32::MAX`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EthFlowOnchainData {
    /// Originating quote id; signed, and may be negative.
    pub quote_id: i64,
    /// The trader's real order expiry, before eth-flow clamps the on-chain
    /// order's `validTo` to `u32::MAX`.
    pub user_valid_to: u32,
}

/// Parses the 12-byte eth-flow `OrderPlacement` trailing data field.
///
/// Decodes `abi.encodePacked(int64 quoteId, uint32 userValidTo)` exactly: bytes
/// `[0..8]` are the big-endian signed `quoteId`, bytes `[8..12]` the big-endian
/// `userValidTo`.
///
/// # Errors
///
/// Returns [`ContractsError::InvalidDecodedLength`] when `data` is not exactly
/// 12 bytes.
///
/// # Panics
///
/// Cannot panic in practice: the `[u8; 12]` conversion above guarantees the
/// subsequent fixed-width `[..8]` and `[8..]` slice conversions are exact. The
/// `expect` calls document that unreachability proof at the call site.
pub fn parse_eth_flow_onchain_data(data: &[u8]) -> Result<EthFlowOnchainData, ContractsError> {
    let bytes: [u8; 12] = data
        .try_into()
        .map_err(|_| ContractsError::InvalidDecodedLength {
            field: "eth-flow onchain order data",
            expected: 12,
            actual: data.len(),
        })?;
    let quote_id = i64::from_be_bytes(
        bytes[..8]
            .try_into()
            .expect("slice length 8 is guaranteed by the 12-byte array above"),
    );
    let user_valid_to = u32::from_be_bytes(
        bytes[8..]
            .try_into()
            .expect("slice length 4 is guaranteed by the 12-byte array above"),
    );
    Ok(EthFlowOnchainData {
        quote_id,
        user_valid_to,
    })
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
            Amount::ZERO,
            0x1234_5678,
            false,
            42,
        )
        .expect("sample order uses non-zero receiver")
    }

    fn sample_unsigned_order(receiver: Address) -> UnsignedOrder {
        use cow_sdk_core::{BuyTokenDestination, OrderKind, SellTokenSource};
        UnsignedOrder::new(
            Address::new("0x3333333333333333333333333333333333333333").unwrap(),
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            receiver,
            Amount::new("1000000000000000000").unwrap(),
            Amount::new("2000000000000000000").unwrap(),
            0x1234_5678,
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
            Amount::ZERO,
            OrderKind::Sell,
            false,
            SellTokenSource::Erc20,
            BuyTokenDestination::Erc20,
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
        alloy_primitives::hex::encode(&bytes[start..start + 32])
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
        let word_hex = alloy_primitives::hex::encode(&encoded[4 + 8 * 32..4 + 9 * 32]);
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

    #[test]
    fn new_rejects_zero_receiver_with_zero_receiver_error() {
        let result = EthFlowOrderData::new(
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::ZERO,
            Amount::new("1000000000000000000").unwrap(),
            Amount::new("2000000000000000000").unwrap(),
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
            Amount::ZERO,
            0x1234_5678,
            false,
            42,
        );
        assert!(
            matches!(result, Err(ContractsError::ZeroReceiver)),
            "EthFlowOrderData::new must reject Address::ZERO with ContractsError::ZeroReceiver; got {result:?}",
        );
    }

    #[test]
    fn from_unsigned_order_rejects_zero_receiver_with_zero_receiver_error() {
        let order = sample_unsigned_order(Address::ZERO);
        let result = EthFlowOrderData::from_unsigned_order(&order, 42);
        assert!(
            matches!(result, Err(ContractsError::ZeroReceiver)),
            "EthFlowOrderData::from_unsigned_order must reject Address::ZERO with ContractsError::ZeroReceiver; got {result:?}",
        );
    }

    #[test]
    fn zero_receiver_invariant_matches_ethflow_on_chain_revert_selector() {
        // The cow-sdk-contracts `ContractsError::ZeroReceiver` variant
        // pre-empts the upstream `CoWSwapEthFlow` contract's
        // `ReceiverMustBeSet()` revert (selector 0xefc9ccdf) at
        // calldata-construction time. This test pins the selector by
        // re-deriving it from first principles via
        // `alloy_primitives::keccak256("ReceiverMustBeSet()")[0..4]`.
        // If the assertion fails, the upstream contract has changed its
        // error signature and ADR 0020's construction-time invariant
        // needs review.
        let derived: [u8; 4] = alloy_primitives::keccak256(b"ReceiverMustBeSet()").0[..4]
            .try_into()
            .expect("keccak256 output is always 32 bytes, slicing [..4] is infallible");
        assert_eq!(
            derived,
            [0xef, 0xc9, 0xcc, 0xdf],
            "upstream `error ReceiverMustBeSet()` selector must remain 0xefc9ccdf",
        );
    }
}
