// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0 <0.9.0;

import {GPv2Order} from "../interfaces/IConditionalOrder.sol";

/**
 * @title TWAPOrder
 * @author CoW Protocol developers (composable-cow upstream — pinned
 *         at composable-cow SHA `471ca59aa95da1bbf3b03e002de96449bc78e6f0`)
 * @dev Internal TWAP staticInput struct + validation surface. The
 *      vendored excerpt carries the canonical staticInput shape that
 *      the `TWAP` handler decodes from `ConditionalOrderParams.staticInput`
 *      and the 8 invariants enforced at validate time.
 */
library TWAPOrder {
    /// @dev TWAP staticInput packed via `abi.encode`.
    struct Data {
        IERC20 sellToken;
        IERC20 buyToken;
        address receiver;
        uint256 partSellAmount;
        uint256 minPartLimit;
        uint256 t0;
        uint256 n;
        uint256 t;
        uint256 span;
        bytes32 appData;
    }

    /// @notice TWAP validation errors (canonical reason strings).
    error InvalidSameToken();
    error InvalidToken();
    error InvalidPartSellAmount();
    error InvalidMinPartLimit();
    error InvalidStartTime();
    error InvalidNumParts();
    error InvalidFrequency();
    error InvalidSpan();

    /**
     * @dev Validate the 8 TWAP invariants. Mirrors the upstream
     *      `TWAPOrder.validate` revert sites byte-for-byte.
     */
    function validate(Data memory data) internal pure {
        if (data.sellToken == data.buyToken) revert InvalidSameToken();
        if (address(data.sellToken) == address(0) || address(data.buyToken) == address(0)) {
            revert InvalidToken();
        }
        if (data.partSellAmount == 0) revert InvalidPartSellAmount();
        if (data.minPartLimit == 0) revert InvalidMinPartLimit();
        if (data.t0 >= type(uint32).max) revert InvalidStartTime();
        if (data.n < 2 || data.n > type(uint32).max) revert InvalidNumParts();
        if (data.t == 0 || data.t > 365 days) revert InvalidFrequency();
        if (data.span > data.t) revert InvalidSpan();
    }
}

interface IERC20 {
    function balanceOf(address account) external view returns (uint256);
}
