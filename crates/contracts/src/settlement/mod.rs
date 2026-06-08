//! `GPv2Settlement` ABI binding and fail-closed event decoding.

/// Typed `GPv2Settlement` event bindings and a fail-closed log decoder.
pub mod events;

use alloy_sol_types::sol;

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
