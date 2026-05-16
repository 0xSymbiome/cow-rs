// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0 <0.9.0;

/**
 * @title TWAPOrderMathLib
 * @author CoW Protocol developers (composable-cow upstream — pinned
 *         at composable-cow SHA `471ca59aa95da1bbf3b03e002de96449bc78e6f0`)
 * @dev TWAP part-index + validity-window math. The vendored excerpt
 *      carries the canonical functions that the `TWAP` handler uses
 *      to compute the current part index and the corresponding
 *      validTo deadline.
 */
library TWAPOrderMathLib {
    /// @notice The TWAP schedule has not yet started.
    error InvalidStartTime();
    /// @notice The TWAP schedule has fully elapsed.
    error InvalidNumParts();

    /**
     * @notice Compute the current part index given `t0`, `t`, `n`,
     *         and the current block timestamp. Reverts with
     *         `InvalidStartTime()` before `t0`. Reverts with
     *         `InvalidNumParts()` after the final part window
     *         closes.
     */
    function calculateValidTo(
        uint256 currentTime,
        uint256 t0,
        uint256 n,
        uint256 t,
        uint256 span
    ) internal pure returns (uint256 partIndex, uint256 validTo) {
        if (currentTime < t0) revert InvalidStartTime();
        uint256 elapsed = currentTime - t0;
        partIndex = elapsed / t;
        if (partIndex >= n) revert InvalidNumParts();
        uint256 partStart = t0 + partIndex * t;
        validTo = span == 0 ? partStart + t - 1 : partStart + span - 1;
    }
}
