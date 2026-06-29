//! `ComposableCoW` registry bindings: conditional-order identity and authorization.
//!
//! [`ConditionalOrderParams`] identifies a conditional order for an owner, and
//! [`conditional_order_id`] reproduces the on-chain `ComposableCoW.hash`. The
//! `create` / `createWithContext` / `remove` encoders build the call-data that
//! authorizes or cancels an order; the owner submits it from its smart-contract
//! account.
//!
//! Bindings are authored as `alloy::sol!` against the `ComposableCoW` Solidity
//! surface, pinned by commit in `parity/source-lock.yaml`.

use alloy_primitives::keccak256;
use alloy_sol_types::{SolCall, SolValue, sol};

use cow_sdk_core::{Address, Hash32, HexData, address};

/// `ComposableCoW` registry deployment — a CREATE2 singleton identical on every
/// supported chain (`mapAddressToSupportedNetworks` in the upstream config).
pub const COMPOSABLE_COW: Address = address!("0xfdafc9d1902f4e0b84f65f49f244b32b31013b74");

/// `ExtensibleFallbackHandler` deployment — a CREATE2 singleton identical on
/// every supported chain.
///
/// A Safe sets this as its fallback handler and points a domain verifier at
/// [`COMPOSABLE_COW`] before authorizing conditional orders.
pub const EXTENSIBLE_FALLBACK_HANDLER: Address =
    address!("0x2f55e8b20d0b9fefa187aa7d00b6cbe563605bf5");

sol! {
    // Canonical ComposableCoW surface. Signatures mirror cowprotocol/
    // composable-cow `src/ComposableCoW.sol` and `src/interfaces/
    // IConditionalOrder.sol`, pinned by commit in `parity/source-lock.yaml`.
    #[sol(rename_all = "camelcase")]
    interface ComposableCoW {
        struct ConditionalOrderParams {
            address handler;
            bytes32 salt;
            bytes staticInput;
        }

        function create(ConditionalOrderParams params, bool dispatch);
        function createWithContext(
            ConditionalOrderParams params,
            address factory,
            bytes data,
            bool dispatch
        );
        function remove(bytes32 singleOrderHash);
        function hash(ConditionalOrderParams params) external pure returns (bytes32);
    }
}

/// Parameters that uniquely identify a conditional order for an owner.
///
/// `H(handler || salt || staticInput)` must be unique per owner. The cow-typed
/// fields cross the alloy seam through [`Address`] / [`Hash32`] / [`HexData`] and
/// [`abi_encode`](ConditionalOrderParams::abi_encode) reproduces the on-chain
/// struct encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalOrderParams {
    /// Conditional-order handler contract (for TWAP,
    /// [`TWAP_HANDLER`](crate::composable::TWAP_HANDLER)).
    pub handler: Address,
    /// 32-byte salt distinguishing two otherwise-identical orders.
    pub salt: Hash32,
    /// Handler-specific static input (for TWAP, the encoded
    /// [`TwapStaticInput`](crate::composable::TwapStaticInput)).
    pub static_input: HexData,
}

impl ConditionalOrderParams {
    /// Builds the `ComposableCoW::ConditionalOrderParams` sol-typed struct.
    fn to_sol_struct(&self) -> ComposableCoW::ConditionalOrderParams {
        use alloy_sol_types::private::{Address as SolAddress, FixedBytes};

        ComposableCoW::ConditionalOrderParams {
            handler: SolAddress::from(self.handler.into_alloy().0.0),
            salt: FixedBytes::from(self.salt.as_alloy().0),
            staticInput: self.static_input.as_alloy().clone(),
        }
    }

    /// Returns `abi.encode(params)` — the struct ABI encoding the on-chain
    /// `ComposableCoW.hash` keccak-hashes.
    #[must_use]
    pub fn abi_encode(&self) -> Vec<u8> {
        self.to_sol_struct().abi_encode()
    }
}

/// Returns the conditional-order id: `keccak256(abi.encode(params))`.
///
/// Byte-identical to the on-chain `ComposableCoW.hash(params)` (pure view), the
/// key under which a single order is authorized and the `ctx` cabinet slot.
#[must_use]
pub fn conditional_order_id(params: &ConditionalOrderParams) -> Hash32 {
    Hash32::from_bytes(keccak256(params.abi_encode()).0)
}

/// Returns the ABI-encoded `create(params, dispatch)` call-data.
///
/// Authorizes a single conditional order whose start time does not depend on a
/// value factory. For a start-at-mining-time TWAP use
/// [`encode_create_with_context_calldata`].
#[must_use]
pub fn encode_create_calldata(params: &ConditionalOrderParams, dispatch: bool) -> Vec<u8> {
    ComposableCoW::createCall {
        params: params.to_sol_struct(),
        dispatch,
    }
    .abi_encode()
}

/// Returns the ABI-encoded `createWithContext(params, factory, data, dispatch)`
/// call-data.
///
/// Authorizes a single conditional order and stores a value from `factory` in
/// the cabinet slot keyed by the order id. A start-at-mining-time TWAP passes
/// [`CURRENT_BLOCK_TIMESTAMP_FACTORY`](crate::composable::CURRENT_BLOCK_TIMESTAMP_FACTORY)
/// so the handler reads the start time from the block timestamp at authorization.
#[must_use]
pub fn encode_create_with_context_calldata(
    params: &ConditionalOrderParams,
    factory: Address,
    data: &[u8],
    dispatch: bool,
) -> Vec<u8> {
    ComposableCoW::createWithContextCall {
        params: params.to_sol_struct(),
        factory: alloy_sol_types::private::Address::from(factory.into_alloy().0.0),
        data: data.to_vec().into(),
        dispatch,
    }
    .abi_encode()
}

/// Returns the ABI-encoded `remove(singleOrderHash)` call-data that cancels a
/// single conditional order, where `single_order_hash` is its
/// [`conditional_order_id`].
#[must_use]
pub fn encode_remove_calldata(single_order_hash: Hash32) -> Vec<u8> {
    ComposableCoW::removeCall {
        singleOrderHash: alloy_sol_types::private::FixedBytes::from(single_order_hash.as_alloy().0),
    }
    .abi_encode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composable::{CURRENT_BLOCK_TIMESTAMP_FACTORY, TWAP_HANDLER};
    use sha3::{Digest, Keccak256};

    fn sample_params() -> ConditionalOrderParams {
        ConditionalOrderParams {
            handler: TWAP_HANDLER,
            salt: Hash32::from_bytes([0x01; 32]),
            static_input: HexData::new("0xdeadbeef").unwrap(),
        }
    }

    fn selector(signature: &str) -> [u8; 4] {
        let digest = Keccak256::digest(signature.as_bytes());
        [digest[0], digest[1], digest[2], digest[3]]
    }

    #[test]
    fn conditional_order_id_matches_onchain_hash_formula() {
        // ComposableCoW.hash(params) = keccak256(abi.encode(params)).
        let params = sample_params();
        let expected = Hash32::from_bytes(keccak256(params.abi_encode()).0);
        assert_eq!(conditional_order_id(&params), expected);
    }

    #[test]
    fn create_with_context_selector_matches_upstream_signature() {
        let calldata = encode_create_with_context_calldata(
            &sample_params(),
            CURRENT_BLOCK_TIMESTAMP_FACTORY,
            &[],
            true,
        );
        assert_eq!(
            calldata[..4],
            selector("createWithContext((address,bytes32,bytes),address,bytes,bool)"),
        );
        // Cross-checks the selector recorded by the CoW Swap frontend's Safe
        // transaction filter.
        assert_eq!(&calldata[..4], &[0x0d, 0x0d, 0x98, 0x00]);
    }

    #[test]
    fn create_and_remove_selectors_match_upstream_signatures() {
        let create = encode_create_calldata(&sample_params(), true);
        assert_eq!(
            create[..4],
            selector("create((address,bytes32,bytes),bool)")
        );
        let remove = encode_remove_calldata(Hash32::from_bytes([0x02; 32]));
        assert_eq!(remove[..4], selector("remove(bytes32)"));
    }
}
