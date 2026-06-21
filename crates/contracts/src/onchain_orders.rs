//! Typed bindings and a fail-closed decoder for the `CoWSwapOnchainOrders`
//! event surface.
//!
//! `CoWSwapOnchainOrders` is the upstream mixin that on-chain order routers —
//! most notably `CoWSwapEthFlow` — use to broadcast a freshly created order so
//! off-chain consumers can reconstruct and track it without re-reading
//! transaction call-data. It emits two events:
//!
//! * `OrderPlacement` — carries the full `GPv2` order, the on-chain signature
//!   (scheme plus payload), and an opaque trailing data field.
//! * `OrderInvalidation` — carries the 56-byte order UID being invalidated.
//!
//! [`decode_order_placement`] and [`decode_order_invalidation`] turn raw logs
//! into typed Rust values. Both are *fail-closed*: a malformed log returns a
//! typed [`ContractsError`] and no input — however adversarial — can panic the
//! decoder. The topic set is validated before ABI decoding, the on-chain signing
//! scheme is range-checked, owner resolution length-checks the EIP-1271 payload
//! rather than slicing it blindly, and the order markers are mapped through the
//! canonical label tables.
//!
//! On the event ABI the `GPv2` order `kind` / `sellTokenBalance` /
//! `buyTokenBalance` members are `bytes32` markers (the keccak-256 of the label)
//! rather than the EIP-712 `string` typing used by the order type hash; the
//! decoder maps those markers back to the typed enums.
//!
//! These bindings are authored inline as `alloy::sol!` against the upstream
//! cowprotocol/ethflowcontract `src/mixins/CoWSwapOnchainOrders.sol` and
//! `src/interfaces/ICoWSwapOnchainOrders.sol` surface, pinned by commit in
//! `parity/source-lock.yaml` and proven by the crate parity tests.

use alloy_primitives::{Bytes, LogData};
use alloy_sol_types::{SolEvent, sol};

use cow_sdk_core::{
    Address, Amount, AppDataHash, CowEnv, OrderData, OrderUid, SupportedChainId, TypedDataDomain,
};

use crate::SigningScheme;
use crate::deployments::{ContractId, Registry};
use crate::errors::ContractsError;
use crate::order::compute_order_uid;
use crate::primitives::{
    buy_balance_from_marker, check_topics, order_kind_from_marker, order_uid_from_bytes,
    sell_balance_from_marker,
};

sol! {
    // Canonical CoWSwapOnchainOrders event surface, mirroring cowprotocol/
    // ethflowcontract `src/mixins/CoWSwapOnchainOrders.sol` and
    // `src/interfaces/ICoWSwapOnchainOrders.sol` (pinned by commit in
    // `parity/source-lock.yaml`). The `order` tuple mirrors GPv2Order.Data,
    // whose kind / sellTokenBalance / buyTokenBalance members are bytes32 label
    // markers on the event ABI. Topic-0 hashes are proven against the crate
    // parity tests.
    #[sol(rename_all = "camelcase")]
    interface ICoWSwapOnchainOrders {
        struct GPv2OrderData {
            address sellToken;
            address buyToken;
            address receiver;
            uint256 sellAmount;
            uint256 buyAmount;
            uint32 validTo;
            bytes32 appData;
            uint256 feeAmount;
            bytes32 kind;
            bool partiallyFillable;
            bytes32 sellTokenBalance;
            bytes32 buyTokenBalance;
        }

        struct OnchainSignature {
            uint8 scheme;
            bytes data;
        }

        event OrderPlacement(
            address indexed sender,
            GPv2OrderData order,
            OnchainSignature signature,
            bytes data
        );

        event OrderInvalidation(bytes orderUid);
    }
}

/// On-chain order signing schemes supported by `CoWSwapOnchainOrders`.
///
/// This is the on-chain subset of [`SigningScheme`]: an order broadcast through
/// `CoWSwapOnchainOrders` is always validated either by an EIP-1271 smart
/// contract or by an on-chain pre-signature, never by an off-chain ECDSA
/// signature. The discriminants match the upstream `OnchainSigningScheme`
/// Solidity enum.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OnchainSigningScheme {
    /// EIP-1271 smart-contract signature; the owner is carried in the signature
    /// payload.
    Eip1271 = 0,
    /// On-chain pre-signature; the owner is the account that placed the order.
    PreSign = 1,
}

impl TryFrom<u8> for OnchainSigningScheme {
    type Error = ContractsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Eip1271),
            1 => Ok(Self::PreSign),
            other => Err(ContractsError::UnsupportedSigningScheme(other)),
        }
    }
}

impl From<OnchainSigningScheme> for SigningScheme {
    fn from(scheme: OnchainSigningScheme) -> Self {
        match scheme {
            OnchainSigningScheme::Eip1271 => Self::Eip1271,
            OnchainSigningScheme::PreSign => Self::PreSign,
        }
    }
}

/// A decoded `CoWSwapOnchainOrders::OrderPlacement` event.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnchainOrderPlacement {
    /// Account that triggered the on-chain order creation (the event's indexed
    /// `sender`). This is not necessarily the order owner.
    pub sender: Address,
    /// The reconstructed `GPv2` order.
    pub order: OrderData,
    /// On-chain signing scheme used to validate the order.
    pub signing_scheme: OnchainSigningScheme,
    /// Raw on-chain signature payload. For [`OnchainSigningScheme::Eip1271`]
    /// this is the 20-byte owner address; for [`OnchainSigningScheme::PreSign`]
    /// it carries scheme-specific data.
    pub signature_data: Bytes,
    /// Opaque trailing data field. For eth-flow placements this is the packed
    /// `(quoteId, userValidTo)` trailer parsed by
    /// [`crate::eth_flow::parse_eth_flow_onchain_data`].
    pub data: Bytes,
}

impl OnchainOrderPlacement {
    /// Resolves the order owner from the on-chain signature.
    ///
    /// For [`OnchainSigningScheme::PreSign`] the owner is the event `sender`.
    /// For [`OnchainSigningScheme::Eip1271`] the owner is the 20-byte address
    /// carried in [`Self::signature_data`].
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::InvalidDecodedLength`] when the EIP-1271
    /// signature payload is not exactly 20 bytes.
    pub fn resolve_owner(&self) -> Result<Address, ContractsError> {
        match self.signing_scheme {
            OnchainSigningScheme::PreSign => Ok(self.sender),
            OnchainSigningScheme::Eip1271 => {
                let data = self.signature_data.as_ref();
                let owner: [u8; 20] =
                    data.try_into()
                        .map_err(|_| ContractsError::InvalidDecodedLength {
                            field: "onchain EIP-1271 order owner",
                            expected: 20,
                            actual: data.len(),
                        })?;
                Ok(Address::from_bytes(owner))
            }
        }
    }

    /// Computes the 56-byte order UID for this placement against `domain`.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] when owner resolution fails.
    pub fn order_uid(&self, domain: &TypedDataDomain) -> Result<OrderUid, ContractsError> {
        let owner = self.resolve_owner()?;
        Ok(compute_order_uid(domain, &self.order, &owner))
    }

    /// Computes the 56-byte order UID using the canonical settlement domain for
    /// `chain_id` and `env`.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] when the settlement domain cannot be resolved,
    /// owner resolution fails, or order hashing fails.
    pub fn order_uid_for_chain(
        &self,
        chain_id: SupportedChainId,
        env: CowEnv,
    ) -> Result<OrderUid, ContractsError> {
        self.order_uid(&settlement_domain(chain_id, env)?)
    }
}

/// A decoded `CoWSwapOnchainOrders::OrderInvalidation` event.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnchainOrderInvalidation {
    /// The 56-byte UID of the order being invalidated.
    pub order_uid: OrderUid,
}

/// Decodes a `CoWSwapOnchainOrders::OrderPlacement` log into typed Rust.
///
/// Fail-closed: validates the topic set, ABI body, on-chain signing scheme, and
/// order markers, returning a typed [`ContractsError`] on any malformed input.
/// The decoder never panics.
///
/// # Errors
///
/// Returns [`ContractsError::UnexpectedEventTopics`] when the topic set does not
/// match the `OrderPlacement` signature, [`ContractsError::Abi`] when the ABI
/// body is malformed, [`ContractsError::UnsupportedSigningScheme`] for an
/// unknown on-chain signing scheme, and [`ContractsError::UnknownOrderMarker`]
/// for an unrecognized order-kind or balance marker.
pub fn decode_order_placement(log: &LogData) -> Result<OnchainOrderPlacement, ContractsError> {
    check_topics(
        log,
        ICoWSwapOnchainOrders::OrderPlacement::SIGNATURE_HASH,
        2,
        "OrderPlacement",
    )?;
    let event = ICoWSwapOnchainOrders::OrderPlacement::decode_raw_log_validate(
        log.topics().iter().copied(),
        log.data.as_ref(),
    )?;
    Ok(OnchainOrderPlacement {
        sender: Address::from_bytes(event.sender.into_array()),
        order: reconstruct_order(&event.order)?,
        signing_scheme: OnchainSigningScheme::try_from(event.signature.scheme)?,
        signature_data: event.signature.data,
        data: event.data,
    })
}

/// Decodes a `CoWSwapOnchainOrders::OrderInvalidation` log into typed Rust.
///
/// Fail-closed: validates the topic set and the 56-byte UID length.
///
/// # Errors
///
/// Returns [`ContractsError::UnexpectedEventTopics`] when the topic set does not
/// match the `OrderInvalidation` signature, [`ContractsError::Abi`] when the ABI
/// body is malformed, and [`ContractsError::InvalidOrderUidLength`] when the
/// decoded UID is not exactly 56 bytes.
pub fn decode_order_invalidation(
    log: &LogData,
) -> Result<OnchainOrderInvalidation, ContractsError> {
    check_topics(
        log,
        ICoWSwapOnchainOrders::OrderInvalidation::SIGNATURE_HASH,
        1,
        "OrderInvalidation",
    )?;
    let event = ICoWSwapOnchainOrders::OrderInvalidation::decode_raw_log_validate(
        log.topics().iter().copied(),
        log.data.as_ref(),
    )?;
    Ok(OnchainOrderInvalidation {
        order_uid: order_uid_from_bytes(event.orderUid.as_ref())?,
    })
}

impl TryFrom<&LogData> for OnchainOrderPlacement {
    type Error = ContractsError;

    /// Decodes a `CoWSwapOnchainOrders` `OrderPlacement` log; see
    /// [`decode_order_placement`].
    fn try_from(log: &LogData) -> Result<Self, Self::Error> {
        decode_order_placement(log)
    }
}

impl TryFrom<&LogData> for OnchainOrderInvalidation {
    type Error = ContractsError;

    /// Decodes a `CoWSwapOnchainOrders` `OrderInvalidation` log; see
    /// [`decode_order_invalidation`].
    fn try_from(log: &LogData) -> Result<Self, Self::Error> {
        decode_order_invalidation(log)
    }
}

fn reconstruct_order(
    order: &ICoWSwapOnchainOrders::GPv2OrderData,
) -> Result<OrderData, ContractsError> {
    // `address(0)` on the wire is the GPv2 RECEIVER_SAME_AS_OWNER sentinel. The
    // concrete `OrderData` carries it verbatim and `hash_order` hashes the zero
    // word directly, preserving the on-chain digest byte-for-byte.
    Ok(OrderData::new(
        Address::from_bytes(order.sellToken.into_array()),
        Address::from_bytes(order.buyToken.into_array()),
        Address::from_bytes(order.receiver.into_array()),
        Amount::from_u256(order.sellAmount),
        Amount::from_u256(order.buyAmount),
        order.validTo,
        AppDataHash::from_bytes(order.appData.0),
        Amount::from_u256(order.feeAmount),
        order_kind_from_marker(order.kind)?,
        order.partiallyFillable,
        sell_balance_from_marker(order.sellTokenBalance)?,
        buy_balance_from_marker(order.buyTokenBalance)?,
    ))
}

fn settlement_domain(
    chain_id: SupportedChainId,
    env: CowEnv,
) -> Result<TypedDataDomain, ContractsError> {
    let settlement = Registry::default()
        .address(ContractId::Settlement, chain_id, env)
        .ok_or(ContractsError::UnsupportedChain(u64::from(chain_id)))?;
    Ok(TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        u64::from(chain_id),
        settlement,
    ))
}
