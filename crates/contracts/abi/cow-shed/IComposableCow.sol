// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0 <0.9.0;

/**
 * @title IComposableCow
 * @author CoW Protocol developers (cow-shed upstream — pinned at
 *         cow-shed SHA `9e01a88e0010314ee1e4c1a822105897a87d3bda`)
 * @dev Interface that `COWShedForComposableCoW` calls into when
 *      bridging the composable framework to the COW Shed account-
 *      abstraction proxy on Gnosis Chain (chain id 100 only). The
 *      vendored excerpt carries the load-bearing entry points that
 *      the bridge forwarder uses.
 */
interface IComposableCow {
    /**
     * @notice Returns whether `(owner, root, proof)` authorizes the
     *         conditional-order params on the composable framework.
     */
    function singleOrders(address owner, bytes32 hash) external view returns (bool);

    /**
     * @notice Returns the merkle root authorized by `owner`.
     */
    function roots(address owner) external view returns (bytes32);

    /**
     * @notice Returns the registered swap guard for `owner`, or
     *         `address(0)` if none.
     */
    function swapGuards(address owner) external view returns (address);
}
