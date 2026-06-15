use cow_sdk_contracts::{Signature, SigningScheme};
use cow_sdk_orderbook::SigningScheme as OrderbookSigningScheme;

fn contract_scheme_label(scheme: SigningScheme) -> &'static str {
    match scheme {
        SigningScheme::Eip712 => "eip712",
        SigningScheme::EthSign => "ethsign",
        SigningScheme::Eip1271 => "eip1271",
        SigningScheme::PreSign => "presign",
    }
}

fn signature_label(signature: Signature) -> &'static str {
    match signature {
        Signature::Ecdsa { .. } => "ecdsa",
        Signature::Eip1271 { .. } => "eip1271",
        Signature::PreSign { .. } => "presign",
    }
}

fn orderbook_scheme_label(scheme: OrderbookSigningScheme) -> &'static str {
    match scheme {
        OrderbookSigningScheme::Eip712 => "eip712",
        OrderbookSigningScheme::EthSign => "ethsign",
        OrderbookSigningScheme::Eip1271 => "eip1271",
        OrderbookSigningScheme::PreSign => "presign",
    }
}

fn main() {}
