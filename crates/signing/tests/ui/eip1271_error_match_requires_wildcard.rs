use cow_sdk_contracts::ContractsError;

fn classify(error: ContractsError) -> bool {
    match error {
        ContractsError::Core(_) => false,
        ContractsError::Cancelled => false,
        ContractsError::UnsupportedChain(_) => false,
        ContractsError::InvalidOrderUidLength { .. } => false,
        ContractsError::InvalidNumeric { .. } => false,
        ContractsError::NumericOverflow { .. } => false,
        ContractsError::InvalidFlags(_) => false,
        ContractsError::UnsupportedSigningScheme(_) => false,
        ContractsError::InvalidEip1271SignatureData => false,
        ContractsError::UnsupportedEip1271Verifier { .. } => false,
        ContractsError::Eip1271Provider { .. } => false,
        ContractsError::MalformedEip1271Response { .. } => false,
        ContractsError::Eip1271MagicValueMismatch { .. } => true,
        ContractsError::MissingClearingPrice { .. } => false,
        ContractsError::MissingExecutedAmount => false,
        ContractsError::MissingTrade => false,
        ContractsError::ZeroReceiver => false,
        ContractsError::InvalidTokenIndex { .. } => false,
        ContractsError::ForbiddenInteractionTarget { .. } => false,
        ContractsError::Provider { .. } => false,
        ContractsError::Abi(_) => false,
        ContractsError::DecodeHex { .. } => false,
        ContractsError::InvalidHexPrefix { .. } => false,
        ContractsError::InvalidDecodedLength { .. } => false,
        ContractsError::Serialization { .. } => false,
        ContractsError::InvalidSignatureLength { .. } => false,
        ContractsError::InvalidSignatureRecoveryByte { .. } => false,
        ContractsError::SignatureSchemeNotEcdsa => false,
        ContractsError::SignatureRecovery { .. } => false,
    }
}

fn main() {
    let _ = classify;
}
