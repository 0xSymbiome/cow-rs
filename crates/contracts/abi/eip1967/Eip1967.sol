// SPDX-License-Identifier: LGPL-3.0-or-later
pragma solidity >=0.7.6 <0.9.0;

// Provenance
// ----------
// Storage-slot layout sourced from EIP-1967 (Standard Proxy Storage Slots,
// https://eips.ethereum.org/EIPS/eip-1967). The admin-slot excerpt also
// matches the upstream cowprotocol/contracts library definition at
// https://github.com/cowprotocol/contracts — src/contracts/libraries/
// GPv2EIP1967.sol.
//
// The proxy ownership interface (`IEip173Proxy`) follows EIP-173 (Contract
// Ownership Standard, https://eips.ethereum.org/EIPS/eip-173).
//
// Both slots are stored under the "keccak256(name) - 1" derivation so that
// `sload` from any other offset cannot accidentally collide. The Rust
// bindings derived from this excerpt live in `crates/contracts/src/proxy.rs`.

library Eip1967Slots {
    /// @dev EIP-1967 proxy admin storage slot:
    ///      keccak256("eip1967.proxy.admin") - 1.
    bytes32 internal constant ADMIN_SLOT =
        hex"b53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103";

    /// @dev EIP-1967 proxy implementation storage slot:
    ///      keccak256("eip1967.proxy.implementation") - 1.
    bytes32 internal constant IMPLEMENTATION_SLOT =
        hex"360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";
}

interface IEip173Proxy {
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    function owner() external view returns (address);

    function transferOwnership(address newOwner) external;

    function supportsInterface(bytes4 interfaceID) external view returns (bool);
}
