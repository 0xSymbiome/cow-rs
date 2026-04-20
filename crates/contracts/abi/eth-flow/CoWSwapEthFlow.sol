// SPDX-License-Identifier: LGPL-3.0-or-later
pragma solidity >=0.7.6 <0.9.0;
pragma abicoder v2;

// Provenance
// ----------
// Upstream repository: https://github.com/cowprotocol/ethflowcontract
// (entrypoint used by the https://github.com/cowprotocol/contracts
// deployment pipeline).
//
// The CoWSwapEthFlow contract wraps the native asset into the canonical
// wrapped-native token and creates the matching EIP-712 order on behalf of
// the trader. The same contract supports on-chain invalidation of a live
// EthFlow order by taking the full EthFlowOrder.Data payload back (NOT the
// GPv2Settlement `invalidateOrder(bytes orderUid)` function). The struct
// field order below is the canonical on-chain ABI; it drives both the
// `createOrder` and `invalidateOrder` call-data encodings generated through
// the alloy::sol! macro in `crates/contracts/src/eth_flow.rs`.
//
// This file is documentation-only: it preserves upstream provenance for
// reviewers. The Rust bindings derived from the same signatures live in
// `crates/contracts/src/eth_flow.rs`.

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
