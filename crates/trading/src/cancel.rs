use cow_sdk_core::Signer;
use cow_sdk_orderbook::{EcdsaSigningScheme, OrderCancellations};
use cow_sdk_signing::{SigningScheme as SigningSchemeContract, sign_order_cancellations};

use crate::types::{validate_orderbook_chain_context, validate_orderbook_env_context};
use crate::{OrderTraderParams, OrderbookClient, TraderParams, TradingError};

/// Signs and submits an off-chain cancellation.
///
/// Any explicit chain or environment must agree with the injected orderbook
/// client, which remains the canonical runtime authority for signing and
/// submission.
///
/// # Errors
///
/// Returns [`TradingError`] when signing fails, unsupported local signing
/// schemes are produced, or the orderbook rejects the cancellation.
pub async fn offchain_cancel_order<O, S>(
    orderbook: &O,
    params: &OrderTraderParams,
    trader: &TraderParams,
    signer: &S,
) -> Result<bool, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
{
    validate_orderbook_chain_context(orderbook, Some(trader.chain_id))?;
    validate_orderbook_chain_context(orderbook, params.chain_id)?;
    validate_orderbook_env_context(orderbook, trader.env)?;
    validate_orderbook_env_context(orderbook, params.env)?;

    let orderbook_context = orderbook.context();
    let canonical_chain_id = orderbook_context.chain_id;
    let canonical_env = orderbook_context.env;
    let options = crate::onchain::protocol_options(
        Some(canonical_env),
        params.settlement_contract_override.as_ref(),
        trader.settlement_contract_override.as_ref(),
        params.eth_flow_contract_override.as_ref(),
        trader.eth_flow_contract_override.as_ref(),
    );
    let signing = sign_order_cancellations(
        std::slice::from_ref(&params.order_uid),
        canonical_chain_id,
        signer,
        Some(&options),
    )
    .await?;
    let scheme = match signing.signing_scheme {
        SigningSchemeContract::Eip712 => EcdsaSigningScheme::Eip712,
        SigningSchemeContract::EthSign => EcdsaSigningScheme::EthSign,
        _ => {
            return Err(TradingError::UnsupportedLocalSigningScheme {
                scheme: signing.signing_scheme,
            });
        }
    };
    let body = OrderCancellations::new(vec![params.order_uid], signing.signature)
        .with_signing_scheme(scheme);

    orderbook.send_cancellations(&body).await?;
    Ok(true)
}
