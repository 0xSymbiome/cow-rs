// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0 <0.9.0;

import {GPv2Order, IERC165} from "./interfaces/IConditionalOrder.sol";
import {IConditionalOrder, IConditionalOrderGenerator} from "./interfaces/IConditionalOrder.sol";

/**
 * @title BaseConditionalOrder
 * @author CoW Protocol developers (composable-cow upstream — pinned
 *         at composable-cow SHA `471ca59aa95da1bbf3b03e002de96449bc78e6f0`)
 * @dev Abstract base contract that every conditional-order handler
 *      (TWAP, GoodAfterTime, StopLoss, TradeAboveThreshold,
 *      PerpetualStableSwap, and custom handlers) inherits from. The
 *      vendored excerpt carries the load-bearing public surface that
 *      the alloy::sol! bindings consume: the `verify(...)` and
 *      `getTradeableOrder(...)` entry points, the `OrderNotValid`
 *      custom error, and the `supportsInterface` ERC-165 surface.
 */
abstract contract BaseConditionalOrder is IConditionalOrderGenerator {
    /// @notice Reason-string custom error fired when the handler
    ///         rejects the input. Selector: `0xc8fc2725`.
    error OrderNotValid(string reason);

    /**
     * @inheritdoc IConditionalOrder
     * @dev The handler verifies the order by re-deriving the
     *      tradeable order from the static input and offchain input
     *      and asserting byte-equality with the supplied `_order`.
     *      Subclasses override `getTradeableOrder` rather than this
     *      method.
     */
    function verify(
        address owner,
        address sender,
        bytes32 _hash,
        bytes32 domainSeparator,
        bytes32 ctx,
        bytes calldata staticInput,
        bytes calldata offchainInput,
        GPv2Order.Data calldata _order
    ) external view virtual override {
        GPv2Order.Data memory tradeableOrder =
            getTradeableOrder(owner, sender, ctx, staticInput, offchainInput);
        if (
            GPv2Order.hash(tradeableOrder, domainSeparator) !=
            GPv2Order.hash(_order, domainSeparator)
        ) {
            revert OrderNotValid("order mismatch");
        }
    }

    /**
     * @inheritdoc IConditionalOrderGenerator
     */
    function getTradeableOrder(
        address owner,
        address sender,
        bytes32 ctx,
        bytes calldata staticInput,
        bytes calldata offchainInput
    ) public view virtual override returns (GPv2Order.Data memory);

    /**
     * @inheritdoc IERC165
     */
    function supportsInterface(bytes4 interfaceId)
        external
        pure
        virtual
        override
        returns (bool)
    {
        return interfaceId == type(IConditionalOrderGenerator).interfaceId
            || interfaceId == type(IConditionalOrder).interfaceId
            || interfaceId == type(IERC165).interfaceId;
    }
}
