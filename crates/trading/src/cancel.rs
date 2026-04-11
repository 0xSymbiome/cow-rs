use cow_sdk_core::{AsyncSigner, ProtocolOptions, Signer};
use cow_sdk_orderbook::{EcdsaSigningScheme, OrderCancellations};
use cow_sdk_signing::{SigningScheme as SigningSchemeContract, sign_order_cancellations_async};

use crate::{OrderTraderParameters, OrderbookClient, TraderParameters, TradingError};

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
    let options = ProtocolOptions {
        env: params.env.or(trader.env).or(Some(orderbook.context().env)),
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
        trader.chain_id,
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
