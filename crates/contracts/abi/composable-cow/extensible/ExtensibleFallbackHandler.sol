// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0 <0.9.0;

/**
 * @title ExtensibleFallbackHandler
 * @author CoW Protocol developers (composable-cow upstream — pinned
 *         at composable-cow SHA `471ca59aa95da1bbf3b03e002de96449bc78e6f0`)
 * @dev Vendored excerpt of the Safe ExtensibleFallbackHandler that
 *      ComposableCoW dispatches signature verification through. The
 *      excerpt carries the canonical public surface that the
 *      alloy::sol! bindings consume.
 */

interface IERC165 {
    function supportsInterface(bytes4 interfaceId) external view returns (bool);
}

interface ERC1271 {
    function isValidSignature(bytes32 _hash, bytes memory _signature)
        external
        view
        returns (bytes4 magicValue);
}

interface Safe {
    function getMessageHash(bytes memory message) external view returns (bytes32);
    function domainSeparator() external view returns (bytes32);
}

interface ISafeSignatureVerifier {
    function isValidSafeSignature(
        Safe safe,
        address sender,
        bytes32 _hash,
        bytes32 domainSeparator,
        bytes32 typeHash,
        bytes calldata encodedData,
        bytes calldata payload
    ) external view returns (bytes4 magic);
}

interface ISignatureVerifierMuxer {
    function defaultVerifier(Safe safe) external view returns (ISafeSignatureVerifier);
    function domainVerifiers(Safe safe, bytes32 domainSeparator)
        external
        view
        returns (ISafeSignatureVerifier);
    function setDomainVerifier(bytes32 domainSeparator, ISafeSignatureVerifier verifier)
        external;
}

contract ExtensibleFallbackHandler is ISignatureVerifierMuxer, ERC1271, IERC165 {
    /// @notice The `safeSignature(bytes32,bytes32,bytes,bytes)` selector
    ///         used by the muxer to dispatch to a registered verifier.
    bytes32 public constant SIGNATURE_VERIFIER_MUXER_INTERFACE_ID =
        0x62af8dc2;

    function defaultVerifier(Safe safe) external view returns (ISafeSignatureVerifier) {}

    function domainVerifiers(Safe safe, bytes32 domainSeparator)
        external
        view
        returns (ISafeSignatureVerifier)
    {}

    function setDomainVerifier(bytes32 domainSeparator, ISafeSignatureVerifier verifier)
        external
    {}

    function isValidSignature(bytes32 _hash, bytes memory _signature)
        external
        view
        returns (bytes4 magicValue)
    {}

    function supportsInterface(bytes4 interfaceId) external pure returns (bool) {
        return interfaceId == 0x62af8dc2
            || interfaceId == type(IERC165).interfaceId;
    }
}
