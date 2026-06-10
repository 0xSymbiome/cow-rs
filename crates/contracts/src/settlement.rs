//! `GPv2Settlement` ABI binding and fail-closed event decoding.
//!
//! This module owns the typed `GPv2Settlement` call binding (`IGPv2Settlement`,
//! whose `setPreSignature` and `invalidateOrder` calls the SDK encodes) and a
//! fail-closed decoder for the settlement event surface.
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

use alloy_primitives::LogData;
use alloy_sol_types::{SolEvent, sol};

use cow_sdk_core::{Address, Amount, OrderUid};

use crate::errors::ContractsError;
use crate::order::ORDER_UID_LENGTH;
use crate::primitives::check_topics;

sol! {
    // Canonical GPv2Settlement ABI surface. Signatures mirror the
    // mainnet-deployed GPv2Settlement contract at
    // 0x9008D19f58AAbD9eD0D60971565AA8510560ab41, whose source is
    // cowprotocol/contracts `src/contracts/GPv2Settlement.sol` plus
    // `libraries/GPv2Trade.sol` and `libraries/GPv2Interaction.sol`, pinned by
    // commit in `parity/source-lock.yaml`. Consumers encode the
    // `setPreSignature` and `invalidateOrder` calls from this binding; the call
    // selectors are proven against the fixtures under `parity/fixtures/` and the
    // crate parity tests.
    #[sol(rename_all = "camelcase")]
    interface IGPv2Settlement {
        struct TradeData {
            uint256 sellTokenIndex;
            uint256 buyTokenIndex;
            address receiver;
            uint256 sellAmount;
            uint256 buyAmount;
            uint32 validTo;
            bytes32 appData;
            uint256 feeAmount;
            uint256 flags;
            uint256 executedAmount;
            bytes signature;
        }

        struct InteractionData {
            address target;
            uint256 value;
            bytes callData;
        }

        function settle(
            address[] calldata tokens,
            uint256[] calldata clearingPrices,
            TradeData[] calldata trades,
            InteractionData[][3] calldata interactions
        ) external;

        function invalidateOrder(bytes calldata orderUid) external;

        function setPreSignature(bytes calldata orderUid, bool signed) external;

        function freeFilledAmountStorage(bytes[] calldata orderUids) external;

        function freePreSignatureStorage(bytes[] calldata orderUids) external;
    }
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

/// Length-checks a decoded `bytes orderUid` field and wraps it as an [`OrderUid`].
fn order_uid_from_bytes(bytes: &[u8]) -> Result<OrderUid, ContractsError> {
    let uid: [u8; ORDER_UID_LENGTH] =
        bytes
            .try_into()
            .map_err(|_| ContractsError::InvalidOrderUidLength {
                actual: bytes.len(),
            })?;
    Ok(OrderUid::from_bytes(uid))
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
