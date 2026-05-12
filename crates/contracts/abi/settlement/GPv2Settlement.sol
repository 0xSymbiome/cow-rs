// SPDX-License-Identifier: LGPL-3.0-or-later
pragma solidity >=0.7.6 <0.9.0;
pragma abicoder v2;

// Provenance
// ----------
// Upstream repository: https://github.com/cowprotocol/contracts
// Source files folded into this excerpt:
//   * src/contracts/GPv2Settlement.sol        — settle, freeFilledAmountStorage,
//                                                freePreSignatureStorage, invalidateOrder,
//                                                setPreSignature function signatures
//   * src/contracts/libraries/GPv2Trade.sol    — Data struct (line 65-131)
//   * src/contracts/libraries/GPv2Interaction.sol
//                                              — Data struct
//
// The signatures reproduced here are the on-chain ABI of the canonical
// GPv2Settlement contract deployed at 0x9008D19f58AAbD9eD0D60971565AA8510560ab41
// on every supported chain. They are stable since the initial mainnet
// deployment and are used by this crate to generate Rust call-type and
// struct bindings through the alloy::sol! macro.
//
// This file is documentation-only: it preserves upstream provenance for
// reviewers and IDEs. The Rust bindings derived from the same signatures
// live in `crates/contracts/src/settlement/mod.rs`.

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
