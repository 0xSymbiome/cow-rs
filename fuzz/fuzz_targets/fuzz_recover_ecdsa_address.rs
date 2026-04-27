#![no_main]

use arbitrary::Arbitrary;
use cow_sdk_contracts::{ContractsError, Signature, SigningScheme};
use cow_sdk_core::Hash32;
use libfuzzer_sys::fuzz_target;

#[derive(Debug, Arbitrary)]
struct Input {
    digest: [u8; 32],
    signature: [u8; 65],
    scheme: u8,
}

fuzz_target!(|input: Input| {
    let digest = Hash32::new(format!("0x{}", hex::encode(input.digest)))
        .expect("fixed-width digest bytes must form a valid Hash32");
    let scheme = if input.scheme % 2 == 0 {
        SigningScheme::Eip712
    } else {
        SigningScheme::EthSign
    };
    let signature = Signature::Ecdsa {
        scheme,
        data: format!("0x{}", hex::encode(input.signature)),
    };

    match signature.recover_ecdsa_address(&digest) {
        Ok(address) => {
            assert_eq!(address.byte_length(), 20);
            assert!(address.as_str().starts_with("0x"));
        }
        Err(
            ContractsError::InvalidSignatureRecoveryByte { .. }
            | ContractsError::SignatureRecovery { .. }
            | ContractsError::InvalidSignatureLength { .. }
            | ContractsError::DecodeHex { .. }
            | ContractsError::InvalidHexPrefix { .. }
            | ContractsError::InvalidDecodedLength { .. },
        ) => {}
        Err(error) => panic!("unexpected recovery error: {error:?}"),
    }
});
