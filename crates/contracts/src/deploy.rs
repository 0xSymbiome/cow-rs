use alloy_primitives::{B256, address, fixed_bytes, keccak256};
use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, CowEnv, SupportedChainId};

use crate::{
    ContractsError,
    deployments::{ContractId, Registry},
};

/// Deterministic deployment salt used by `CoW` deployments.
///
/// Pinned to the 32-byte payload `0x4d61...0000` and emitted as a typed
/// [`alloy_primitives::B256`] compile-time literal via [`fixed_bytes!`];
/// see ADR 0052 for the canonical-primitive-layer doctrine.
pub const SALT: B256 =
    fixed_bytes!("0x4d61747472657373657320696e204265726c696e210000000000000000000000");
/// Deployer contract address used for deterministic deployment derivation.
///
/// Pinned to the 20-byte payload `0x4e59b448...956c` (the Arachnid
/// deterministic-deployment proxy) and emitted as a typed
/// [`alloy_primitives::Address`] compile-time literal via [`address!`].
pub const DEPLOYER_CONTRACT: alloy_primitives::Address =
    address!("0x4e59b44847b379578588920ca78fbf26c0b4956c");

/// Supported named `CoW` deployment artifacts.
///
/// The enum is `#[non_exhaustive]` so additional deployment artifacts can
/// extend the public surface without breaking existing consumers. Internal
/// matches remain exhaustive; downstream matches must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContractName {
    /// Authenticator contract.
    Authenticator,
    /// Settlement contract.
    Settlement,
    /// Trade-simulation helper contract.
    TradeSimulator,
}

/// Core `CoW` deployment addresses for a supported chain.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractAddresses {
    /// Settlement contract address.
    pub settlement: Address,
    /// Vault relayer address.
    pub vault_relayer: Address,
    /// `EthFlow` contract address.
    pub eth_flow: Address,
}

impl ContractAddresses {
    /// Creates a set of canonical deployment addresses.
    #[must_use]
    pub const fn new(settlement: Address, vault_relayer: Address, eth_flow: Address) -> Self {
        Self {
            settlement,
            vault_relayer,
            eth_flow,
        }
    }
}

/// Computes a deterministic deployment address from bytecode and constructor arguments.
///
/// # Errors
///
/// Returns [`ContractsError`] when bytecode or constructor arguments are not
/// valid hex, or when address validation fails during `CREATE2` derivation.
pub fn deterministic_deployment_address(
    bytecode: &str,
    deployment_arguments: &[String],
) -> Result<Address, ContractsError> {
    let mut init_code = decode_hex_field(bytecode, "bytecode")?;
    for arg in deployment_arguments {
        init_code.extend_from_slice(&decode_hex_field(arg, "deploymentArgument")?);
    }

    // Delegate the EIP-1014 byte assembly (`0xff || deployer || salt ||
    // keccak256(init_code)`) and the final keccak256 to the maintained
    // primitive. `Address::create2_from_code` computes the init-code hash
    // internally and slices the trailing 20 bytes to form the derived
    // address. `DEPLOYER_CONTRACT` and `SALT` are typed compile-time
    // literals, so the CREATE2 inputs reach the alloy primitive without
    // any runtime hex decoding. The inline regression test in this
    // module reconstructs the canonical EIP-1014 formula from first
    // principles and asserts the helper output matches at the byte
    // level.
    let derived = DEPLOYER_CONTRACT.create2_from_code(SALT, &init_code);
    Ok(Address::from_bytes(derived.into()))
}

/// Decodes a `0x`-prefixed hex string into raw bytes, mapping prefix and
/// character errors onto the contracts-side typed error surface.
fn decode_hex_field(value: &str, field: &'static str) -> Result<Vec<u8>, ContractsError> {
    let stripped = value
        .strip_prefix("0x")
        .ok_or(ContractsError::InvalidHexPrefix { field })?;
    alloy_primitives::hex::decode(stripped).map_err(|source| ContractsError::DecodeHex { field, source })
}

/// Returns the canonical production deployment addresses for a supported chain.
///
/// # Errors
///
/// Returns [`ContractsError::UnsupportedChain`] when `chain_id` is not part of
/// the supported `CoW` deployment set.
///
/// # Panics
///
/// Panics if the embedded deployment registry is missing an entry for any of
/// the three canonical contracts on the resolved chain. The shipped registry
/// manifest is validated at compile time, so this panic cannot be reached
/// from an unmodified binary.
pub fn deployment_for_chain(chain_id: u64) -> Result<ContractAddresses, ContractsError> {
    let chain = SupportedChainId::try_from(chain_id)
        .map_err(|_| ContractsError::UnsupportedChain(chain_id))?;
    let registry = Registry::default();
    Ok(ContractAddresses::new(
        // SAFETY: Registry::default parses the build-validated embedded
        // manifest, which must include canonical production contracts for each
        // supported chain.
        registry
            .address(ContractId::Settlement, chain, CowEnv::Prod)
            .expect("canonical settlement address is registered for every supported chain"),
        registry
            .address(ContractId::VaultRelayer, chain, CowEnv::Prod)
            .expect("canonical vault-relayer address is registered for every supported chain"),
        registry
            .address(ContractId::EthFlow, chain, CowEnv::Prod)
            .expect("canonical EthFlow address is registered for every supported chain"),
    ))
}

/// Returns the keccak256 hash of the deployment init code.
///
/// # Errors
///
/// Returns [`ContractsError`] when bytecode or constructor arguments are not
/// valid hex.
pub fn deployment_address_hash_input(
    bytecode: &str,
    deployment_arguments: &[String],
) -> Result<[u8; 32], ContractsError> {
    let mut init_code = decode_hex_field(bytecode, "bytecode")?;
    for arg in deployment_arguments {
        init_code.extend_from_slice(&decode_hex_field(arg, "deploymentArgument")?);
    }
    Ok(keccak256(&init_code).0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::{Digest, Keccak256};

    /// Hand-rolled keccak256 over `bytes` returning the raw `[u8; 32]`
    /// digest. Crate code routes through `alloy_primitives::keccak256`
    /// per ADR 0052; this helper deliberately runs `sha3::Keccak256`
    /// directly so the parity check compares the crate output against an
    /// independent keccak implementation.
    fn keccak256(bytes: impl AsRef<[u8]>) -> [u8; 32] {
        let digest = Keccak256::digest(bytes.as_ref());
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    }

    fn sample_init_code_parts() -> (&'static str, Vec<String>) {
        (
            "0x6001600055",
            vec!["0x1234".to_owned(), "0xabcd".to_owned()],
        )
    }

    #[test]
    fn deployment_hash_input_matches_the_keccak_of_bytecode_and_arguments() {
        let (bytecode, deployment_arguments) = sample_init_code_parts();
        let mut init_code = alloy_primitives::hex::decode(bytecode.trim_start_matches("0x")).unwrap();
        init_code.extend_from_slice(&alloy_primitives::hex::decode("1234").unwrap());
        init_code.extend_from_slice(&alloy_primitives::hex::decode("abcd").unwrap());

        assert_eq!(
            deployment_address_hash_input(bytecode, &deployment_arguments).unwrap(),
            keccak256(init_code)
        );
    }

    #[test]
    fn deterministic_deployment_address_matches_the_create2_formula() {
        let (bytecode, deployment_arguments) = sample_init_code_parts();
        let hash = deployment_address_hash_input(bytecode, &deployment_arguments).unwrap();

        let mut payload = Vec::with_capacity(85);
        payload.push(0xff);
        payload.extend_from_slice(DEPLOYER_CONTRACT.as_slice());
        payload.extend_from_slice(SALT.as_slice());
        payload.extend_from_slice(&hash);
        let expected = keccak256(payload);

        assert_eq!(
            deterministic_deployment_address(bytecode, &deployment_arguments)
                .unwrap()
                .to_hex_string(),
            format!("0x{}", alloy_primitives::hex::encode(&expected[12..]))
        );
    }

    #[test]
    fn salt_bytes_match_pinned_literal() {
        // Pinned 32-byte form of "Mattresses in Berlin!" followed by 11
        // zero bytes. The independent literal here proves the
        // `fixed_bytes!` macro emits the canonical bytes; if a future
        // contributor inadvertently shifts the SALT declaration, this
        // assertion catches the drift before the parity oracle does.
        let pinned: [u8; 32] = [
            0x4d, 0x61, 0x74, 0x74, 0x72, 0x65, 0x73, 0x73, 0x65, 0x73, 0x20, 0x69, 0x6e, 0x20,
            0x42, 0x65, 0x72, 0x6c, 0x69, 0x6e, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(SALT.0, pinned);
    }

    #[test]
    fn deployer_contract_bytes_match_pinned_arachnid_proxy() {
        // Pinned 20-byte form of the Arachnid deterministic-deployment
        // proxy address. The independent literal here proves the
        // `address!` macro emits the canonical bytes.
        let pinned: [u8; 20] = [
            0x4e, 0x59, 0xb4, 0x48, 0x47, 0xb3, 0x79, 0x57, 0x85, 0x88, 0x92, 0x0c, 0xa7, 0x8f,
            0xbf, 0x26, 0xc0, 0xb4, 0x95, 0x6c,
        ];
        assert_eq!(DEPLOYER_CONTRACT.into_array(), pinned);
    }
}
