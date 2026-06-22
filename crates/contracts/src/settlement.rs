//! `GPv2Settlement` ABI binding and fail-closed event decoding.
//!
//! This module owns the typed `GPv2Settlement` call binding (`IGPv2Settlement`)
//! together with the [`encode_set_pre_signature`] / [`encode_invalidate_order`]
//! call-data builders over it, and a fail-closed decoder for the settlement
//! event surface.
//!
//! The deployed settlement contract emits five events: `Trade` (one per filled
//! order), `Interaction` (one per executed solver interaction), `Settlement`
//! (one per completed batch), `OrderInvalidated` (an on-chain cancellation of a
//! signed order), and `PreSignature` (inherited from the `GPv2Signing` mixin,
//! emitted when an order pre-signature is set or revoked).
//!
//! [`decode_settlement_log`] turns a raw log into a typed [`SettlementEvent`].
//! Like the on-chain order decoder, it is *fail-closed* and *provider-free*: it
//! accepts borrowed `alloy_primitives::LogData` with no `Provider` or network
//! dependency, validates the topic set against the generated `SIGNATURE_HASH`
//! and the indexed arity before ABI decoding, and length-checks the 56-byte
//! order UID. Every malformed input returns a typed [`ContractsError`]; no log,
//! however adversarial, can panic the decoder.
//!
//! Both ABI surfaces mirror cowprotocol/contracts
//! `src/contracts/GPv2Settlement.sol` (pinned by commit in
//! `parity/source-lock.yaml`); `PreSignature` is the inherited `GPv2Signing`
//! mixin event. Every topic-0 hash is byte-locked against an independent
//! keccak-256 of the canonical event signature in the crate integration tests.

use alloy_primitives::{Bytes, LogData};
use alloy_sol_types::{SolCall, SolEvent, sol};

use cow_sdk_core::{Address, Amount, OrderUid};

use crate::errors::ContractsError;
use crate::primitives::{check_topics, order_uid_from_bytes};

sol! {
    // Canonical GPv2Settlement call surface the SDK encodes. Signatures mirror
    // the mainnet-deployed GPv2Settlement contract at
    // 0x9008D19f58AAbD9eD0D60971565AA8510560ab41, whose source is
    // cowprotocol/contracts `src/contracts/GPv2Settlement.sol`, pinned by commit
    // in `parity/source-lock.yaml`. Consumers encode the `setPreSignature` and
    // `invalidateOrder` calls from this binding; the call selectors are proven
    // against the fixtures under `parity/fixtures/` and the crate parity tests.
    // The solver-only `settle` entry point and its trade/interaction tuples are
    // deliberately out of scope: this is an order-lifecycle SDK, not a solver.
    #[sol(rename_all = "camelcase")]
    interface IGPv2Settlement {
        function invalidateOrder(bytes calldata orderUid) external;

        function setPreSignature(bytes calldata orderUid, bool signed) external;

        function freeFilledAmountStorage(bytes[] calldata orderUids) external;

        function freePreSignatureStorage(bytes[] calldata orderUids) external;
    }
}

/// Returns the ABI-encoded `setPreSignature(bytes orderUid, bool signed)`
/// call-data for the `GPv2Settlement` contract.
///
/// Pass `signed = true` to register a pre-signature for `order_uid` and
/// `signed = false` to revoke it; the 56-byte UID is the dynamic `bytes`
/// argument. The selector and argument encoding are byte-locked by the
/// settlement call-data fixtures under `parity/fixtures/` and the crate parity
/// tests.
///
/// Infallible: the call shape is fixed and the cow [`OrderUid`] newtype enforces
/// the 56-byte length at construction, so the alloy-sol `abi_encode` cannot fail.
#[must_use]
pub fn encode_set_pre_signature(order_uid: &OrderUid, signed: bool) -> Vec<u8> {
    IGPv2Settlement::setPreSignatureCall {
        orderUid: Bytes::from(order_uid.as_slice().to_vec()),
        signed,
    }
    .abi_encode()
}

/// Returns the ABI-encoded `invalidateOrder(bytes orderUid)` call-data for the
/// `GPv2Settlement` contract.
///
/// This is the settlement-level on-chain cancellation of a signed order and
/// takes only the packed 56-byte order UID — distinct from
/// [`encode_invalidate_order_calldata`](crate::eth_flow::encode_invalidate_order_calldata),
/// which cancels an eth-flow order by taking the full order payload back. The
/// selector and argument encoding are byte-locked by the settlement call-data
/// fixtures under `parity/fixtures/` and the crate parity tests.
///
/// Infallible: the call shape is fixed and the cow [`OrderUid`] newtype enforces
/// the 56-byte length at construction, so the alloy-sol `abi_encode` cannot fail.
#[must_use]
pub fn encode_invalidate_order(order_uid: &OrderUid) -> Vec<u8> {
    IGPv2Settlement::invalidateOrderCall {
        orderUid: Bytes::from(order_uid.as_slice().to_vec()),
    }
    .abi_encode()
}

sol! {
    // Canonical GPv2Settlement event surface. The four settlement events
    // mirror cowprotocol/contracts `src/contracts/GPv2Settlement.sol` and
    // `PreSignature` is the inherited GPv2Signing mixin event, both pinned by
    // commit in `parity/source-lock.yaml`. Every topic-0 hash is byte-locked
    // against an independent keccak-256 of the canonical signature in tests.
    #[sol(rename_all = "camelcase")]
    interface IGPv2SettlementEvents {
        event Trade(
            address indexed owner,
            address sellToken,
            address buyToken,
            uint256 sellAmount,
            uint256 buyAmount,
            uint256 feeAmount,
            bytes orderUid
        );

        event Interaction(address indexed target, uint256 value, bytes4 selector);

        event Settlement(address indexed solver);

        event OrderInvalidated(address indexed owner, bytes orderUid);

        event PreSignature(address indexed owner, bytes orderUid, bool signed);
    }
}

/// A decoded `GPv2Settlement` (or inherited `GPv2Signing`) event.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettlementEvent {
    /// A user order was executed in a settlement.
    Trade {
        /// Order owner, recovered from the indexed event topic.
        owner: Address,
        /// Sell token traded.
        sell_token: Address,
        /// Buy token traded.
        buy_token: Address,
        /// Executed sell amount.
        sell_amount: Amount,
        /// Executed buy amount.
        buy_amount: Amount,
        /// Executed fee amount.
        fee_amount: Amount,
        /// 56-byte order UID of the filled order.
        order_uid: OrderUid,
    },
    /// A solver interaction was executed during a settlement. Only the first
    /// four selector bytes of the interaction calldata are logged on-chain.
    Interaction {
        /// Interaction target contract.
        target: Address,
        /// Native value forwarded with the interaction.
        value: Amount,
        /// First four bytes of the interaction calldata (the function selector).
        selector: [u8; 4],
    },
    /// A settlement batch completed.
    Settlement {
        /// Authorized solver that submitted the batch.
        solver: Address,
    },
    /// An off-chain signed order was invalidated on-chain by its owner.
    OrderInvalidated {
        /// Owner that invalidated the order.
        owner: Address,
        /// 56-byte order UID that was invalidated.
        order_uid: OrderUid,
    },
    /// An order pre-signature was set or revoked.
    PreSignature {
        /// Owner whose pre-signature changed.
        owner: Address,
        /// 56-byte order UID affected.
        order_uid: OrderUid,
        /// `true` when the order is now pre-signed, `false` when revoked.
        signed: bool,
    },
}

/// Decodes a `GPv2Settlement` event log into a typed [`SettlementEvent`].
///
/// Fail-closed and provider-free: the topic set is validated against the
/// matching event's `SIGNATURE_HASH` and indexed arity before ABI decoding, and
/// every decoded `bytes orderUid` is length-checked to 56 bytes. The decoder
/// borrows the log bytes and performs no I/O, so one implementation serves
/// native, browser, and any RPC client. The owner field on `Trade`,
/// `OrderInvalidated`, and `PreSignature` is recovered from the indexed topic.
///
/// # Errors
///
/// Returns [`ContractsError::UnexpectedEventTopics`] when the topic set does
/// not match any known settlement event signature, [`ContractsError::Abi`] when
/// the ABI body is malformed, and [`ContractsError::InvalidOrderUidLength`] when
/// a decoded order UID is not exactly 56 bytes.
pub fn decode_settlement_log(log: &LogData) -> Result<SettlementEvent, ContractsError> {
    let topic0 = log
        .topics()
        .first()
        .copied()
        .ok_or(ContractsError::UnexpectedEventTopics {
            event: "settlement",
        })?;

    if topic0 == IGPv2SettlementEvents::Trade::SIGNATURE_HASH {
        check_topics(
            log,
            IGPv2SettlementEvents::Trade::SIGNATURE_HASH,
            2,
            "Trade",
        )?;
        let event = IGPv2SettlementEvents::Trade::decode_raw_log_validate(
            log.topics().iter().copied(),
            log.data.as_ref(),
        )?;
        Ok(SettlementEvent::Trade {
            owner: Address::from_bytes(event.owner.into_array()),
            sell_token: Address::from_bytes(event.sellToken.into_array()),
            buy_token: Address::from_bytes(event.buyToken.into_array()),
            sell_amount: Amount::from_u256(event.sellAmount),
            buy_amount: Amount::from_u256(event.buyAmount),
            fee_amount: Amount::from_u256(event.feeAmount),
            order_uid: order_uid_from_bytes(&event.orderUid)?,
        })
    } else if topic0 == IGPv2SettlementEvents::Interaction::SIGNATURE_HASH {
        check_topics(
            log,
            IGPv2SettlementEvents::Interaction::SIGNATURE_HASH,
            2,
            "Interaction",
        )?;
        let event = IGPv2SettlementEvents::Interaction::decode_raw_log_validate(
            log.topics().iter().copied(),
            log.data.as_ref(),
        )?;
        Ok(SettlementEvent::Interaction {
            target: Address::from_bytes(event.target.into_array()),
            value: Amount::from_u256(event.value),
            selector: event.selector.0,
        })
    } else if topic0 == IGPv2SettlementEvents::Settlement::SIGNATURE_HASH {
        check_topics(
            log,
            IGPv2SettlementEvents::Settlement::SIGNATURE_HASH,
            2,
            "Settlement",
        )?;
        let event = IGPv2SettlementEvents::Settlement::decode_raw_log_validate(
            log.topics().iter().copied(),
            log.data.as_ref(),
        )?;
        Ok(SettlementEvent::Settlement {
            solver: Address::from_bytes(event.solver.into_array()),
        })
    } else if topic0 == IGPv2SettlementEvents::OrderInvalidated::SIGNATURE_HASH {
        check_topics(
            log,
            IGPv2SettlementEvents::OrderInvalidated::SIGNATURE_HASH,
            2,
            "OrderInvalidated",
        )?;
        let event = IGPv2SettlementEvents::OrderInvalidated::decode_raw_log_validate(
            log.topics().iter().copied(),
            log.data.as_ref(),
        )?;
        Ok(SettlementEvent::OrderInvalidated {
            owner: Address::from_bytes(event.owner.into_array()),
            order_uid: order_uid_from_bytes(&event.orderUid)?,
        })
    } else if topic0 == IGPv2SettlementEvents::PreSignature::SIGNATURE_HASH {
        check_topics(
            log,
            IGPv2SettlementEvents::PreSignature::SIGNATURE_HASH,
            2,
            "PreSignature",
        )?;
        let event = IGPv2SettlementEvents::PreSignature::decode_raw_log_validate(
            log.topics().iter().copied(),
            log.data.as_ref(),
        )?;
        Ok(SettlementEvent::PreSignature {
            owner: Address::from_bytes(event.owner.into_array()),
            order_uid: order_uid_from_bytes(&event.orderUid)?,
            signed: event.signed,
        })
    } else {
        Err(ContractsError::UnexpectedEventTopics {
            event: "settlement",
        })
    }
}

impl TryFrom<&LogData> for SettlementEvent {
    type Error = ContractsError;

    /// Decodes a `GPv2Settlement` event log; see [`decode_settlement_log`].
    fn try_from(log: &LogData) -> Result<Self, Self::Error> {
        decode_settlement_log(log)
    }
}

#[cfg(test)]
mod call_tests {
    use super::{encode_invalidate_order, encode_set_pre_signature};
    use cow_sdk_core::OrderUid;

    fn sample_uid() -> OrderUid {
        // A 56-byte UID is 32-byte order digest || 20-byte owner || 4-byte validTo.
        OrderUid::new(format!("0x{}", "11".repeat(56))).expect("56-byte uid is valid")
    }

    fn canonical_selector(signature: &str) -> [u8; 4] {
        alloy_primitives::keccak256(signature.as_bytes()).0[..4]
            .try_into()
            .expect("keccak256 output is always 32 bytes, slicing [..4] is infallible")
    }

    #[test]
    fn set_pre_signature_starts_with_the_canonical_upstream_selector() {
        let encoded = encode_set_pre_signature(&sample_uid(), true);
        assert_eq!(
            &encoded[..4],
            canonical_selector("setPreSignature(bytes,bool)"),
            "setPreSignature selector must match the upstream GPv2Settlement signature",
        );
    }

    #[test]
    fn invalidate_order_starts_with_the_canonical_upstream_selector() {
        let encoded = encode_invalidate_order(&sample_uid());
        assert_eq!(
            &encoded[..4],
            canonical_selector("invalidateOrder(bytes)"),
            "invalidateOrder selector must match the upstream GPv2Settlement signature",
        );
    }

    #[test]
    fn set_pre_signature_encodes_the_signed_flag_in_the_second_head_word() {
        // Head layout: word 0 is the dynamic-bytes offset, word 1 the right-aligned
        // bool. The flag is the last byte of word 1 (selector + 64 - 1).
        let on = encode_set_pre_signature(&sample_uid(), true);
        let off = encode_set_pre_signature(&sample_uid(), false);
        assert_eq!(on[4 + 63], 1, "signed = true encodes a trailing 1 byte");
        assert_eq!(off[4 + 63], 0, "signed = false encodes a trailing 0 byte");
        assert_ne!(on, off, "the signed flag changes the encoded call-data");
    }
}
