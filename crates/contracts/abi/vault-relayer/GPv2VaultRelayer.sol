// SPDX-License-Identifier: LGPL-3.0-or-later
pragma solidity >=0.7.6 <0.9.0;
pragma abicoder v2;

// Provenance
// ----------
// Upstream repository: https://github.com/cowprotocol/contracts
// Source files folded into this excerpt:
//   * src/contracts/GPv2VaultRelayer.sol   — external ABI surface of the
//                                             GPv2 Vault Relayer contract
//   * src/ts/vault.ts                       — partial Balancer V2 Vault
//                                             interface used for role grants
//
// The GPv2VaultRelayer is the contract that moves trader balances into the
// GPv2Settlement contract and proxies Balancer Vault batch swaps on behalf of
// the solver. Before solvers can call it, the Balancer Authorizer must grant
// the vault relayer explicit permission to invoke the two Vault methods
// included below (`manageUserBalance` and `batchSwap`). This crate derives the
// authorization role IDs from the `IVault` selectors; the `IGPv2VaultRelayer`
// surface is carried forward for completeness and future consumers.
//
// This file is documentation-only: it preserves upstream provenance for
// reviewers. The Rust bindings derived from the same signatures live in
// `crates/contracts/src/vault.rs`.

interface IGPv2VaultRelayer {
    struct Transfer {
        address account;
        address token;
        uint256 amount;
        uint8 balance;
    }

    struct BatchSwapStep {
        bytes32 poolId;
        uint256 assetInIndex;
        uint256 assetOutIndex;
        uint256 amount;
        bytes userData;
    }

    struct FundManagement {
        address sender;
        bool fromInternalBalance;
        address recipient;
        bool toInternalBalance;
    }

    function transferFromAccounts(Transfer[] calldata transfers) external;

    function batchSwapWithFee(
        uint8 kind,
        BatchSwapStep[] calldata swaps,
        address[] memory tokens,
        FundManagement memory funds,
        int256[] memory limits,
        uint256 deadline,
        Transfer calldata feeTransfer
    ) external returns (int256[] memory tokenDeltas);
}

// Partial Balancer V2 Vault ABI — the two methods the GPv2VaultRelayer calls
// on behalf of GPv2Settlement and whose selectors drive the role-grant flow.
interface IVault {
    struct UserBalanceOp {
        uint8 kind;
        address asset;
        uint256 amount;
        address sender;
        address recipient;
    }

    struct BatchSwapStep {
        bytes32 poolId;
        uint256 assetInIndex;
        uint256 assetOutIndex;
        uint256 amount;
        bytes userData;
    }

    struct FundManagement {
        address sender;
        bool fromInternalBalance;
        address recipient;
        bool toInternalBalance;
    }

    function manageUserBalance(UserBalanceOp[] calldata ops) external payable;

    function batchSwap(
        uint8 kind,
        BatchSwapStep[] calldata swaps,
        address[] memory assets,
        FundManagement memory funds,
        int256[] memory limits,
        uint256 deadline
    ) external payable returns (int256[] memory assetDeltas);
}
