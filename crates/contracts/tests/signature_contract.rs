mod common;

use cow_sdk_contracts::{
    ContractsError, EIP1271_MAGICVALUE, Eip1271SignatureData, Eip1271VerificationRequest,
    Signature, SigningScheme, decode_eip1271_signature_data, decode_signing_scheme,
    encode_eip1271_signature_data, encode_signing_scheme, function_magic_value,
    normalized_ecdsa_signature, verify_eip1271_signature,
};
use cow_sdk_core::{Address, Hash32, HexData};

use common::{MockProvider, fixture_case};

fn expected_u8(value: &serde_json::Value) -> u8 {
    u8::try_from(value.as_u64().unwrap()).expect("fixture discriminant must fit in u8")
}

#[test]
fn signing_scheme_and_magic_value_match_fixture_contract() {
    let schemes = fixture_case("contracts-signing-scheme-discriminants");
    let expected = &schemes["expected"];
    assert_eq!(
        encode_signing_scheme(SigningScheme::Eip712),
        expected_u8(&expected["EIP712"])
    );
    assert_eq!(
        encode_signing_scheme(SigningScheme::EthSign),
        expected_u8(&expected["ETHSIGN"])
    );
    assert_eq!(
        encode_signing_scheme(SigningScheme::Eip1271),
        expected_u8(&expected["EIP1271"])
    );
    assert_eq!(
        encode_signing_scheme(SigningScheme::PreSign),
        expected_u8(&expected["PRESIGN"])
    );

    assert_eq!(decode_signing_scheme(0).unwrap(), SigningScheme::Eip712);
    assert_eq!(decode_signing_scheme(1).unwrap(), SigningScheme::EthSign);
    assert_eq!(decode_signing_scheme(2).unwrap(), SigningScheme::Eip1271);
    assert_eq!(decode_signing_scheme(3).unwrap(), SigningScheme::PreSign);
    assert!(decode_signing_scheme(4).is_err());

    let magic = fixture_case("contracts-eip1271-magic-value");
    assert_eq!(
        EIP1271_MAGICVALUE,
        magic["expected"]["magic_value"].as_str().unwrap()
    );
    assert_eq!(
        function_magic_value("isValidSignature(bytes32,bytes)"),
        EIP1271_MAGICVALUE
    );
}

#[test]
fn eip1271_signature_payloads_roundtrip_with_variable_lengths() {
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();

    for signature in [
        "0x",
        "0x1234",
        "0x29a674dfc87f8c78fc2bfbcbe8ffdd435091a6a84bc7761db72a45da453d73ac41c5ce28eceb34be73fddc12a5d04af6e736405e41b613aeefeed3db8122420c1b",
    ] {
        let data = Eip1271SignatureData {
            verifier: verifier.clone(),
            signature: signature.to_owned(),
        };

        let encoded = encode_eip1271_signature_data(&data).unwrap();
        let decoded = decode_eip1271_signature_data(&encoded).unwrap();
        assert_eq!(
            decoded.verifier.normalized_key(),
            data.verifier.normalized_key()
        );
        assert_eq!(decoded.signature, data.signature);
    }

    assert!(decode_eip1271_signature_data("0x1234").is_err());
}

#[test]
fn signature_helpers_preserve_public_contract_surface() {
    let signer = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let ecdsa = Signature::Ecdsa {
        scheme: SigningScheme::Eip712,
        data: normalized_ecdsa_signature(
            "0x29A674DFC87F8C78FC2BFBCBE8FFDD435091A6A84BC7761DB72A45DA453D73AC41C5CE28ECEB34BE73FDDC12A5D04AF6E736405E41B613AEEFEED3DB8122420C1B",
        )
        .unwrap(),
    };
    let pre_sign = Signature::PreSign {
        owner: signer.clone(),
    };
    let eip1271 = Signature::Eip1271 {
        data: Eip1271SignatureData {
            verifier: signer,
            signature: "0x1234".to_owned(),
        },
    };

    assert_eq!(ecdsa.scheme(), SigningScheme::Eip712);
    assert_eq!(pre_sign.scheme(), SigningScheme::PreSign);
    assert_eq!(eip1271.scheme(), SigningScheme::Eip1271);
    assert!(SigningScheme::Eip712.is_ecdsa());
    assert!(SigningScheme::EthSign.is_ecdsa());
    assert!(!SigningScheme::Eip1271.is_ecdsa());
}

#[test]
fn eip1271_verification_reads_contract_code_and_magic_value() {
    let provider = MockProvider::new();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    provider.set_code(Some("0x6001600055"));
    provider.set_response("\"0x1626ba7e\"");

    verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest {
            verifier: verifier.clone(),
            digest: Hash32::new(format!("0x{}", "11".repeat(32))).unwrap(),
            signature: HexData::new("0x1234").unwrap(),
        },
    )
    .unwrap();

    let call = provider.calls.borrow().last().cloned().unwrap();
    assert_eq!(call.address, verifier);
    assert_eq!(call.method, "isValidSignature");
    assert!(call.abi_json.contains("\"bytes4\""));
    let args: Vec<String> = serde_json::from_str(&call.args_json).unwrap();
    assert_eq!(args[0], format!("0x{}", "11".repeat(32)));
    assert_eq!(args[1], "0x1234");
}

#[test]
fn eip1271_verification_fails_closed_for_missing_code_and_transport_errors() {
    let provider = MockProvider::new();
    let verifier = Address::new("0x1111111111111111111111111111111111111111").unwrap();

    let missing = verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest {
            verifier: verifier.clone(),
            digest: Hash32::new(format!("0x{}", "22".repeat(32))).unwrap(),
            signature: HexData::new("0x").unwrap(),
        },
    )
    .unwrap_err();
    assert_eq!(
        missing,
        ContractsError::UnsupportedEip1271Verifier {
            verifier: verifier.clone()
        }
    );

    provider.set_code(Some("0x6001600055"));
    provider.set_response_error(Some("rpc unavailable"));
    let transport = verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest {
            verifier,
            digest: Hash32::new(format!("0x{}", "33".repeat(32))).unwrap(),
            signature: HexData::new("0x1234").unwrap(),
        },
    )
    .unwrap_err();
    assert_eq!(
        transport,
        ContractsError::Eip1271Provider {
            operation: "read_contract",
            message: "rpc unavailable".to_owned()
        }
    );
}

#[test]
fn eip1271_verification_rejects_malformed_and_wrong_magic_responses() {
    let provider = MockProvider::new();
    let verifier = Address::new("0x2222222222222222222222222222222222222222").unwrap();
    provider.set_code(Some("0x6001600055"));

    provider.set_response("{\"unexpected\":true}");
    let malformed = verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest {
            verifier: verifier.clone(),
            digest: Hash32::new(format!("0x{}", "44".repeat(32))).unwrap(),
            signature: HexData::new("0x1234").unwrap(),
        },
    )
    .unwrap_err();
    assert_eq!(
        malformed,
        ContractsError::MalformedEip1271Response {
            response: "{\"unexpected\":true}".to_owned()
        }
    );

    provider.set_response("\"0xffffffff\"");
    let mismatch = verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest {
            verifier,
            digest: Hash32::new(format!("0x{}", "55".repeat(32))).unwrap(),
            signature: HexData::new("0x1234").unwrap(),
        },
    )
    .unwrap_err();
    assert_eq!(
        mismatch,
        ContractsError::Eip1271MagicValueMismatch {
            expected: EIP1271_MAGICVALUE.to_owned(),
            actual: "0xffffffff".to_owned()
        }
    );
}
