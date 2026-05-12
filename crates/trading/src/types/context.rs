#![allow(
    clippy::redundant_pub_crate,
    reason = "these validators intentionally stay pub(crate) and are re-exported through types::mod for unchanged crate-local call sites"
)]

use cow_sdk_core::{CowEnv, SupportedChainId};
use cow_sdk_orderbook::{OrderbookClient, OrderbookRuntimeBinding};

use crate::{OrderbookContextValue, TradingError};

pub(crate) fn validate_orderbook_chain_context<O>(
    orderbook_client: &O,
    requested_chain: Option<SupportedChainId>,
) -> Result<(), TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let context = orderbook_client.context();

    if let Some(chain_id) = requested_chain
        && chain_id != context.chain_id
    {
        return Err(TradingError::InjectedOrderbookContextConflict {
            field: "chainId",
            requested: OrderbookContextValue::ChainId(u64::from(chain_id)),
            configured: OrderbookContextValue::ChainId(u64::from(context.chain_id)),
        });
    }

    Ok(())
}

pub(crate) fn validate_orderbook_env_context<O>(
    orderbook_client: &O,
    requested_env: Option<CowEnv>,
) -> Result<(), TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let context = orderbook_client.context();

    if let Some(env) = requested_env
        && env != context.env
    {
        return Err(TradingError::InjectedOrderbookContextConflict {
            field: "env",
            requested: OrderbookContextValue::Env(env),
            configured: OrderbookContextValue::Env(context.env),
        });
    }

    Ok(())
}

pub(crate) fn validate_orderbook_context<O>(
    orderbook_client: &O,
    requested_chain: Option<SupportedChainId>,
    requested_env: Option<CowEnv>,
) -> Result<(), TradingError>
where
    O: OrderbookClient + ?Sized,
{
    validate_orderbook_chain_context(orderbook_client, requested_chain)?;
    validate_orderbook_env_context(orderbook_client, requested_env)
}

pub(crate) fn validate_quote_orderbook_binding<O>(
    orderbook_client: &O,
    quoted_binding: Option<&OrderbookRuntimeBinding>,
) -> Result<(), TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let Some(quoted_binding) = quoted_binding else {
        return Err(TradingError::MissingQuoteOrderbookBinding);
    };
    let submission_binding = orderbook_client.runtime_binding();

    if quoted_binding.chain_id != submission_binding.chain_id {
        return Err(TradingError::QuoteOrderbookBindingConflict {
            field: "chainId",
            quoted: OrderbookContextValue::ChainId(u64::from(quoted_binding.chain_id)),
            submitted: OrderbookContextValue::ChainId(u64::from(submission_binding.chain_id)),
        });
    }
    if quoted_binding.env != submission_binding.env {
        return Err(TradingError::QuoteOrderbookBindingConflict {
            field: "env",
            quoted: OrderbookContextValue::Env(quoted_binding.env),
            submitted: OrderbookContextValue::Env(submission_binding.env),
        });
    }
    if let (Some(quoted_base_url), Some(submission_base_url)) = (
        quoted_binding.resolved_base_url.as_ref(),
        submission_binding.resolved_base_url.as_ref(),
    ) && quoted_base_url != submission_base_url
    {
        return Err(TradingError::QuoteOrderbookBindingConflict {
            field: "baseUrl",
            quoted: OrderbookContextValue::BaseUrl(quoted_base_url.clone().into()),
            submitted: OrderbookContextValue::BaseUrl(submission_base_url.clone().into()),
        });
    }

    Ok(())
}
