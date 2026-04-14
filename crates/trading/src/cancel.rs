use cow_sdk_core::{AsyncSigner, ProtocolOptions, Signer};
use cow_sdk_orderbook::{EcdsaSigningScheme, OrderCancellations};
use cow_sdk_signing::{SigningScheme as SigningSchemeContract, sign_order_cancellations_async};

use crate::types::{validate_orderbook_chain_context, validate_orderbook_env_context};
use crate::{OrderTraderParameters, OrderbookClient, TraderParameters, TradingError};

/// Signs and submits an off-chain cancellation using a sync signer.
///
/// # Errors
///
/// Returns [`TradingError`] when signing or orderbook submission fails.
pub async fn off_chain_cancel_order<O, S>(
    orderbook: &O,
    params: &OrderTraderParameters,
    trader: &TraderParameters,
    signer: &S,
) -> Result<bool, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
{
    off_chain_cancel_order_async(orderbook, params, trader, signer).await
}

/// Signs and submits an off-chain cancellation using an async signer.
///
/// Any explicit chain or environment must agree with the injected orderbook
/// client, which remains the canonical runtime authority for signing and
/// submission.
///
/// # Errors
///
/// Returns [`TradingError`] when signing fails, unsupported local signing
/// schemes are produced, or the orderbook rejects the cancellation.
pub async fn off_chain_cancel_order_async<O, S>(
    orderbook: &O,
    params: &OrderTraderParameters,
    trader: &TraderParameters,
    signer: &S,
) -> Result<bool, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    validate_orderbook_chain_context(orderbook, Some(trader.chain_id))?;
    validate_orderbook_chain_context(orderbook, params.chain_id)?;
    validate_orderbook_env_context(orderbook, trader.env)?;
    validate_orderbook_env_context(orderbook, params.env)?;

    let orderbook_context = orderbook.context();
    let canonical_chain_id = orderbook_context.chain_id;
    let canonical_env = orderbook_context.env;
    let options = ProtocolOptions {
        env: Some(canonical_env),
        settlement_contract_override: params
            .settlement_contract_override
            .clone()
            .or_else(|| trader.settlement_contract_override.clone()),
        eth_flow_contract_override: params
            .eth_flow_contract_override
            .clone()
            .or_else(|| trader.eth_flow_contract_override.clone()),
    };
    let signing = sign_order_cancellations_async(
        std::slice::from_ref(&params.order_uid),
        canonical_chain_id,
        signer,
        Some(&options),
    )
    .await?;
    let body = OrderCancellations {
        order_uids: vec![params.order_uid.clone()],
        signature: signing.signature,
        signing_scheme: match signing.signing_scheme {
            SigningSchemeContract::Eip712 => EcdsaSigningScheme::Eip712,
            SigningSchemeContract::EthSign => EcdsaSigningScheme::EthSign,
            SigningSchemeContract::Eip1271 | SigningSchemeContract::PreSign => {
                return Err(TradingError::UnsupportedLocalSigningScheme {
                    scheme: signing.signing_scheme,
                });
            }
        },
    };

    orderbook.send_signed_order_cancellations(&body).await?;
    Ok(true)
}
