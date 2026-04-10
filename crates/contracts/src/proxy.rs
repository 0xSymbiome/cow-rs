use cow_sdk_core::{Address, ContractHandle, Provider};

use crate::{ContractsError, primitives::parse_hex_exact};

pub const IMPLEMENTATION_STORAGE_SLOT: &str =
    "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";
pub const OWNER_STORAGE_SLOT: &str =
    "0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103";

pub const EIP173_PROXY_ABI: [&str; 4] = [
    "event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)",
    "function owner() view external returns(address)",
    "function transferOwnership(address newOwner) external",
    "function supportsInterface(bytes4 interfaceID) external view returns (bool)",
];

pub fn implementation_address<P>(provider: &P, proxy: &Address) -> Result<Address, ContractsError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    decode_storage_address(
        provider
            .get_storage_at(proxy, IMPLEMENTATION_STORAGE_SLOT)
            .map_err(|error| ContractsError::Provider(error.to_string()))?
            .as_str(),
    )
}

pub fn owner_address<P>(provider: &P, proxy: &Address) -> Result<Address, ContractsError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    decode_storage_address(
        provider
            .get_storage_at(proxy, OWNER_STORAGE_SLOT)
            .map_err(|error| ContractsError::Provider(error.to_string()))?
            .as_str(),
    )
}

pub fn proxy_interface(address: &Address) -> Result<ContractHandle, ContractsError> {
    let abi_json = serde_json::to_string(&EIP173_PROXY_ABI)
        .map_err(|error| ContractsError::Serialization(error.to_string()))?;
    Ok(ContractHandle {
        address: address.clone(),
        abi_json,
    })
}

fn decode_storage_address(value: &str) -> Result<Address, ContractsError> {
    let bytes = parse_hex_exact(value, "storageSlot", 32)?;
    Address::new(format!("0x{}", hex::encode(&bytes[12..]))).map_err(Into::into)
}
