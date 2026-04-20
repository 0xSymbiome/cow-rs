// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6 <0.9.0;

// Provenance
// ----------
// The minimal ERC-20 surface reproduced below is the standard defined by
// EIP-20 (https://eips.ethereum.org/EIPS/eip-20). The canonical Solidity
// reference implementation lives in the OpenZeppelin contracts library at
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/
// contracts/token/ERC20/IERC20.sol. This excerpt covers the public surface
// every downstream consumer in the cow-rs workspace needs (`balanceOf`,
// `approve`, `allowance`, `transfer`, `transferFrom`) plus the two standard
// events.
//
// This file is documentation-only: it preserves upstream provenance for
// reviewers. The Rust bindings derived from the same signatures live in
// `crates/contracts/src/erc20.rs`.

interface IERC20 {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    function balanceOf(address account) external view returns (uint256);

    function transfer(address to, uint256 value) external returns (bool);

    function allowance(address owner, address spender) external view returns (uint256);

    function approve(address spender, uint256 value) external returns (bool);

    function transferFrom(address from, address to, uint256 value) external returns (bool);
}
