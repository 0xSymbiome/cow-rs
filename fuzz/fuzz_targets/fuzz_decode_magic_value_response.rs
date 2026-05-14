#![no_main]

//! Fuzz target for EIP-1271 magic-value response decoding.
//!
//! **Surface:** `cow_sdk_contracts::verify_eip1271_signature` driving the
//! crate-private `decode_magic_value_response` decoder through the
//! provider's `read_contract` response. The decoder itself is
//! `pub(crate)`, so this target exercises the next public wrapper that
//! consumes its result.
//! **Property:** `PROP-CON-009`.
//! **Seed contract:** corpus inputs cover the canonical magic-value
//! string `"0x1626ba7e"`, the JSON-quoted form `"\"0x1626ba7e\""`,
//! the canonical magic value with an uppercase hex body, a boundary
//! payload that is JSON-typed but not a string, a 4-byte hex blob with
//! flipped trailing bits, an empty body, and a non-hex adversarial blob.
//! **Corpus README:** `../corpus/fuzz_decode_magic_value_response/README.md`.
//!
//! The fuzzer fixture provider returns the raw `&[u8]` from the fuzzer
//! as the `read_contract` response, with `0xfe` deployed code so the
//! `UnsupportedEip1271Verifier` early-exit never fires.
//!
//! Invariants:
//!
//! * `verify_eip1271_signature` never panics for any response body.
//! * `Ok(())` from the wrapper implies the response decoded to the
//!   canonical EIP-1271 magic value `0x1626ba7e` (`[0x16, 0x26, 0xba,
//!   0x7e]`), and so the underlying input was either the JSON string
//!   literal `"0x1626ba7e"` or a raw `0x1626ba7e` hex blob, case-
//!   insensitive.
//! * `Err(ContractsError::Eip1271MagicValueMismatch { actual, .. })`
//!   surfaces a 4-byte hex value that is not the canonical magic value.
//! * `Err(ContractsError::MalformedEip1271Response { .. })` is returned
//!   for every payload that fails to decode as a 4-byte hex string.
//! * Determinism: the same response body produces the same typed
//!   outcome across two consecutive calls.

use std::{cell::RefCell, fmt, rc::Rc};

use cow_sdk_contracts::{
    ContractsError, EIP1271_MAGICVALUE, Eip1271VerificationRequest, verify_eip1271_signature,
};
use cow_sdk_core::{
    Address, BlockInfo, ContractCall, ContractHandle, Hash32, HexData, Provider,
    TransactionHash, TransactionReceipt, TransactionRequest,
};
use libfuzzer_sys::fuzz_target;

const CANONICAL_MAGIC_VALUE_BYTES: [u8; 4] = [0x16, 0x26, 0xba, 0x7e];

fuzz_target!(|data: &[u8]| {
    let response = String::from_utf8_lossy(data).into_owned();

    let provider = StubProvider::new(response.clone());
    let request = Eip1271VerificationRequest::new(
        Address::from_bytes([0x11; 20]),
        Hash32::from_bytes([0xAB; 32]),
        HexData::new("0xdeadbeef").expect("static fixture signature must remain valid"),
    );

    let first = verify_eip1271_signature(&provider, &request);
    let second = verify_eip1271_signature(&provider, &request);

    match (&first, &second) {
        (Ok(()), Ok(())) => {
            // Ok must imply the response was the canonical EIP-1271
            // magic value after JSON-string or raw-hex decoding.
            assert!(
                response_matches_canonical_magic_value(&response),
                "Ok(()) must imply the response decoded to the canonical magic value, \
                 but the raw response was {response:?}",
            );
        }
        (
            Err(ContractsError::Eip1271MagicValueMismatch { expected, actual: actual_a }),
            Err(ContractsError::Eip1271MagicValueMismatch { actual: actual_b, .. }),
        ) => {
            assert_eq!(
                *expected, CANONICAL_MAGIC_VALUE_BYTES,
                "mismatch error must surface the canonical expected magic value",
            );
            assert_ne!(
                *actual_a, CANONICAL_MAGIC_VALUE_BYTES,
                "mismatch error must surface a non-canonical 4-byte actual value",
            );
            assert_eq!(
                actual_a, actual_b,
                "decode_magic_value_response must be deterministic on the same response",
            );
        }
        (
            Err(ContractsError::MalformedEip1271Response { .. }),
            Err(ContractsError::MalformedEip1271Response { .. }),
        ) => {
            // Malformed responses are the documented failure class for
            // every payload that does not decode as 4 bytes of hex.
        }
        (Err(ContractsError::Eip1271Provider { .. }), Err(ContractsError::Eip1271Provider { .. }))
        | (
            Err(ContractsError::UnsupportedEip1271Verifier { .. }),
            Err(ContractsError::UnsupportedEip1271Verifier { .. }),
        ) => {
            // Unreachable for the stub provider, but documented as
            // legal verifier-side outcomes.
        }
        (Ok(()), _) | (_, Ok(())) => {
            panic!(
                "verify_eip1271_signature must be deterministic, got first={first:?}, second={second:?}",
            );
        }
        (Err(first_err), Err(second_err)) => {
            // Two error classes are acceptable as long as the matched
            // variants on both calls share the same discriminant.
            assert_eq!(
                std::mem::discriminant(first_err),
                std::mem::discriminant(second_err),
                "verify_eip1271_signature must surface the same error class on repeat calls",
            );
        }
    }

    // Cross-check the documented constant surface — the public hex form
    // must match the canonical 4-byte magic value used by the wrapper.
    let constant_bytes =
        hex::decode(EIP1271_MAGICVALUE.trim_start_matches("0x")).expect("EIP-1271 constant must decode");
    assert_eq!(
        constant_bytes, CANONICAL_MAGIC_VALUE_BYTES,
        "EIP1271_MAGICVALUE constant must equal the canonical 0x1626ba7e magic value",
    );
});

fn response_matches_canonical_magic_value(response: &str) -> bool {
    let candidate = match serde_json::from_str::<serde_json::Value>(response) {
        Ok(serde_json::Value::String(value)) => value,
        Ok(_) => return false,
        Err(_) => response.to_owned(),
    };
    let Some(stripped) = candidate.strip_prefix("0x").or_else(|| candidate.strip_prefix("0X")) else {
        return false;
    };
    let Ok(bytes) = hex::decode(stripped) else {
        return false;
    };
    bytes == CANONICAL_MAGIC_VALUE_BYTES
}

#[derive(Debug, Clone)]
struct StubProviderError(String);

impl fmt::Display for StubProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Default)]
struct DummySigner;

struct StubProvider {
    response: Rc<RefCell<String>>,
}

impl StubProvider {
    fn new(response: String) -> Self {
        Self {
            response: Rc::new(RefCell::new(response)),
        }
    }
}

impl Provider for StubProvider {
    type Signer = DummySigner;
    type Error = StubProviderError;

    fn signer_or_null(&self) -> Option<&Self::Signer> {
        None
    }

    fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(Some(
            HexData::new("0xfe").expect("static fixture deployed code must remain valid"),
        ))
    }

    fn get_transaction_receipt(
        &self,
        _transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(DummySigner)
    }

    fn get_storage_at(&self, _address: &Address, _slot: &str) -> Result<HexData, Self::Error> {
        Err(StubProviderError(
            "stub provider does not implement get_storage_at".to_owned(),
        ))
    }

    fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Err(StubProviderError(
            "stub provider does not implement call".to_owned(),
        ))
    }

    fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
        Ok(self.response.borrow().clone())
    }

    fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }

    fn set_signer(&mut self, _signer: Self::Signer) {}

    fn set_provider(&mut self, _provider_hint: String) {}

    fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
    }
}
