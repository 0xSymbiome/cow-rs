use cow_sdk_trading::ClientRejection;

fn classify(error: ClientRejection) -> &'static str {
    match error {
        ClientRejection::MissingFrom => "missing-from",
        ClientRejection::ValidToInPast { .. } => "valid-to-in-past",
        ClientRejection::AppdataFromMismatch { .. } => "appdata-from-mismatch",
        ClientRejection::SameBuyAndSellToken { .. } => "same-token",
        ClientRejection::InvalidNativeSellToken => "invalid-native-sell-token",
        ClientRejection::ZeroAmount { .. } => "zero-amount",
        ClientRejection::OwnerMismatch { .. } => "owner-mismatch",
        ClientRejection::InvalidPartnerFee { .. } => "invalid-partner-fee",
    }
}

fn main() {
    let _ = classify;
}
