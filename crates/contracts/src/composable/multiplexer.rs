//! Merkle tree over conditional orders for the `ComposableCoW.setRoot` path.
//!
//! An owner can authorize many conditional orders under one root through
//! `setRoot` rather than a `create` call for each. A leaf is the order's
//! parameters keccak-hashed twice over their ABI encoding,
//! `keccak256(keccak256(abi.encode(params)))`, the form `ComposableCoW._auth`
//! checks (`ComposableCoW.sol:297`). `abi.encode(params)` encodes the
//! `ConditionalOrderParams` struct, so Solidity prepends the dynamic-struct
//! offset; the leaf is therefore not the bare `(address, bytes32, bytes)` tuple a
//! generic `StandardMerkleTree` hashes, and a tree built that way would not
//! authenticate on-chain. The contract verifies inclusion with `OpenZeppelin`'s
//! `MerkleProof.verify` (`ComposableCoW.sol:4`, called at `:300`); its internal
//! nodes are the `keccak256` of the two child hashes ordered ascending. This tree
//! reproduces both rules, so its roots and proofs verify on-chain and round trip
//! through [`verify_merkle_proof`].

use alloy_primitives::{Keccak256, keccak256};

use cow_sdk_core::Hash32;

use super::{ConditionalOrderParams, conditional_order_id};

/// Returns the merkle leaf the contract checks for one conditional order:
/// `keccak256(keccak256(abi.encode(params)))` (`ComposableCoW.sol:297`).
#[must_use]
pub fn merkle_leaf(params: &ConditionalOrderParams) -> Hash32 {
    let id = conditional_order_id(params);
    Hash32::from_bytes(keccak256(id.as_alloy().as_slice()).0)
}

/// Combines two child nodes the way `MerkleProof.verify` does: the `keccak256`
/// of the pair ordered ascending.
fn hash_pair(a: Hash32, b: Hash32) -> Hash32 {
    let (lo, hi) = if a.as_alloy() <= b.as_alloy() {
        (a, b)
    } else {
        (b, a)
    };
    let mut hasher = Keccak256::new();
    hasher.update(lo.as_alloy().as_slice());
    hasher.update(hi.as_alloy().as_slice());
    Hash32::from_bytes(hasher.finalize().0)
}

/// A merkle tree of conditional-order leaves in input order.
///
/// `levels[0]` holds the leaves and the final level holds the root. Input order
/// is preserved, so the index a caller passes to [`Multiplexer::proof`] is the
/// position of the order it added. The contract authenticates a proof by folding
/// it to the owner's root, not by a fixed tree shape, so this self-consistent
/// tree is all the `setRoot` path needs.
#[derive(Debug, Clone)]
pub struct Multiplexer {
    levels: Vec<Vec<Hash32>>,
    root: Hash32,
}

/// Failure modes for [`Multiplexer`] construction and proof generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum MultiplexerError {
    /// Construction was attempted with no conditional orders.
    #[error("a multiplexer needs at least one conditional order")]
    NoOrders,
    /// A proof was requested for a leaf index past the last order.
    #[error("conditional-order leaf index is out of range")]
    LeafIndexOutOfRange,
}

impl Multiplexer {
    /// Builds the tree from conditional-order parameters, hashing each into its
    /// leaf.
    ///
    /// # Errors
    ///
    /// Returns [`MultiplexerError::NoOrders`] when `orders` is empty.
    pub fn from_params(orders: &[ConditionalOrderParams]) -> Result<Self, MultiplexerError> {
        if orders.is_empty() {
            return Err(MultiplexerError::NoOrders);
        }
        let leaves: Vec<Hash32> = orders.iter().map(merkle_leaf).collect();
        let mut levels = vec![leaves];
        // `levels` always holds at least the leaf level, so indexing the last
        // level is in bounds on every iteration and after the loop.
        while levels[levels.len() - 1].len() > 1 {
            let prev = &levels[levels.len() - 1];
            let mut next = Vec::with_capacity(prev.len().div_ceil(2));
            let mut i = 0;
            while i < prev.len() {
                if i + 1 < prev.len() {
                    next.push(hash_pair(prev[i], prev[i + 1]));
                } else {
                    next.push(prev[i]);
                }
                i += 2;
            }
            levels.push(next);
        }
        let last = &levels[levels.len() - 1];
        let root = last[0];
        Ok(Self { levels, root })
    }

    /// Returns the merkle root passed to `ComposableCoW.setRoot`.
    #[must_use]
    pub const fn root(&self) -> Hash32 {
        self.root
    }

    /// Returns the leaves in input order.
    #[must_use]
    pub fn leaves(&self) -> &[Hash32] {
        &self.levels[0]
    }

    /// Returns the inclusion proof for the leaf at `index`.
    ///
    /// A single-leaf tree has an empty proof, which the on-chain verifier
    /// accepts.
    ///
    /// # Errors
    ///
    /// Returns [`MultiplexerError::LeafIndexOutOfRange`] when `index` is past the
    /// last leaf.
    pub fn proof(&self, index: usize) -> Result<Vec<Hash32>, MultiplexerError> {
        if index >= self.leaves().len() {
            return Err(MultiplexerError::LeafIndexOutOfRange);
        }
        let mut proof = Vec::new();
        let mut idx = index;
        for level in &self.levels[..self.levels.len() - 1] {
            let sibling = idx ^ 1;
            if sibling < level.len() {
                proof.push(level[sibling]);
            }
            idx /= 2;
        }
        Ok(proof)
    }

    /// Returns the inclusion proof for the order with these `params`, or `None`
    /// when it is not in the tree.
    ///
    /// The order is located by its [`merkle_leaf`], so a consumer asks for a proof
    /// by the order it holds.
    #[must_use]
    pub fn proof_for(&self, params: &ConditionalOrderParams) -> Option<Vec<Hash32>> {
        let leaf = merkle_leaf(params);
        let index = self
            .leaves()
            .iter()
            .position(|candidate| *candidate == leaf)?;
        self.proof(index).ok()
    }
}

/// Checks that the order with these `params` is included under `root` by its
/// `proof`, using the same ascending node rule as the on-chain
/// `MerkleProof.verify`.
///
/// The leaf is recomputed from the params here, so the proof is checked against
/// the order it belongs to, and a consumer can reject a bad proof before it
/// reaches the chain.
#[must_use]
pub fn verify_merkle_proof(
    root: Hash32,
    proof: &[Hash32],
    params: &ConditionalOrderParams,
) -> bool {
    let mut current = merkle_leaf(params);
    for sibling in proof {
        current = hash_pair(current, *sibling);
    }
    current.as_alloy() == root.as_alloy()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::B256;
    use cow_sdk_core::{Address, HexData};

    fn params(byte: u8) -> ConditionalOrderParams {
        ConditionalOrderParams {
            handler: Address::from_bytes([byte; 20]),
            salt: Hash32::from_bytes([byte; 32]),
            static_input: HexData::new("0xabcd").unwrap(),
        }
    }

    fn h32(hex: &str) -> Hash32 {
        Hash32::from_bytes(hex.parse::<B256>().unwrap().0)
    }

    #[test]
    fn building_from_no_orders_errors() {
        assert_eq!(
            Multiplexer::from_params(&[]).unwrap_err(),
            MultiplexerError::NoOrders
        );
    }

    #[test]
    fn single_leaf_root_is_the_leaf_with_an_empty_proof() {
        let orders = [params(1)];
        let tree = Multiplexer::from_params(&orders).unwrap();
        assert_eq!(tree.root(), merkle_leaf(&orders[0]));
        let proof = tree.proof(0).unwrap();
        assert!(proof.is_empty());
        assert!(verify_merkle_proof(tree.root(), &proof, &orders[0]));
    }

    #[test]
    fn every_leaf_proof_verifies_across_sizes() {
        let orders: Vec<_> = (1_u8..=10).map(params).collect();
        let tree = Multiplexer::from_params(&orders).unwrap();
        for (i, order) in orders.iter().enumerate() {
            let proof = tree.proof(i).unwrap();
            assert!(
                verify_merkle_proof(tree.root(), &proof, order),
                "order {i} failed"
            );
            // proof_for finds the same proof by order, with no index in hand.
            assert_eq!(tree.proof_for(order), Some(proof));
        }
        // An order outside the tree neither verifies against a member's proof nor
        // yields a proof of its own.
        let proof = tree.proof(0).unwrap();
        assert!(!verify_merkle_proof(tree.root(), &proof, &params(0xff)));
        assert_eq!(tree.proof_for(&params(0xff)), None);
    }

    #[test]
    fn a_proof_past_the_last_leaf_errors() {
        let orders = [params(1), params(2)];
        let tree = Multiplexer::from_params(&orders).unwrap();
        assert_eq!(
            tree.proof(2).unwrap_err(),
            MultiplexerError::LeafIndexOutOfRange
        );
    }

    #[test]
    fn merkle_leaf_double_hashes_the_id() {
        // ComposableCoW._auth merkle leaf = keccak256(bytes.concat(hash(params))).
        let order = params(1);
        let id = conditional_order_id(&order);
        let expected = Hash32::from_bytes(keccak256(id.as_alloy().as_slice()).0);
        assert_eq!(merkle_leaf(&order), expected);
    }

    #[test]
    fn leaf_is_the_contract_struct_encoding_not_the_bare_tuple() {
        // `ComposableCoW.hash` keccak-hashes `abi.encode(params)` of the struct,
        // which prepends Solidity's dynamic-struct `0x20` offset. The bare
        // `(address, bytes32, bytes)` tuple a `StandardMerkleTree` would hash gives
        // a different leaf that fails on-chain. This golden vector, cross-checked
        // against an independent hand-built ABI encoding, pins the leaf to the
        // contract's struct encoding so the offset can never be lost.
        assert_eq!(
            merkle_leaf(&params(1)),
            h32("0xfb4e6219e85240b65ecf7d8326254596ea837b66d3df77a26b926259c26ab529"),
        );
    }
}
