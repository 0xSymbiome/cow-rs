// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0 <0.9.0;

/**
 * @title IPreSignStorage
 * @author CoW Protocol developers (cow-shed upstream — pinned at
 *         cow-shed SHA `9e01a88e0010314ee1e4c1a822105897a87d3bda`)
 * @dev Interface for the COW Shed pre-sign storage that backs
 *      `executePreSignedHooks`. The proxy indexes pre-signed hook
 *      batches by struct hash (no domain prefix; the proxy
 *      deduplicates inside the implementation, not at the EIP-712
 *      domain layer).
 */
interface IPreSignStorage {
    /**
     * @notice Returns whether `executePreSignedHooks` has been
     *         pre-authorized for `structHash` by the proxy owner.
     */
    function preSigned(bytes32 structHash) external view returns (bool);

    /**
     * @notice Pre-authorize a hook batch for later execution by any
     *         caller (including a relayer). Idempotent: re-signing
     *         the same struct hash is a no-op.
     */
    function setPreSigned(bytes32 structHash, bool approved) external;
}
