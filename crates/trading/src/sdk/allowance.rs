use cow_sdk_core::{Amount, AsyncProvider, AsyncSigner, CowEnv, Provider, Signer, TransactionHash};

use super::TradingSdk;
use crate::{
    AllowanceParameters, ApprovalParameters, TradingError, get_cow_protocol_allowance,
    get_cow_protocol_allowance_async,
};

impl TradingSdk {
    /// Reads the `CoW` Protocol allowance using a sync provider.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or provider
    /// reads fail.
    pub fn get_cow_protocol_allowance<P>(
        &self,
        provider: &P,
        params: &AllowanceParameters,
    ) -> Result<Amount, TradingError>
    where
        P: Provider,
        P::Error: std::fmt::Display,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let env = trader.env.unwrap_or(CowEnv::Prod);

        get_cow_protocol_allowance(
            provider,
            &params.token_address,
            &params.owner,
            chain_id,
            env,
            params.vault_relayer_override.as_ref(),
        )
    }

    /// Reads the `CoW` Protocol allowance using an async provider.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or provider
    /// reads fail.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.get_cow_protocol_allowance_async",
            ),
        ),
    )]
    pub async fn get_cow_protocol_allowance_async<P>(
        &self,
        provider: &P,
        params: &AllowanceParameters,
    ) -> Result<Amount, TradingError>
    where
        P: AsyncProvider,
        P::Error: std::fmt::Display,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let env = trader.env.unwrap_or(CowEnv::Prod);

        get_cow_protocol_allowance_async(
            provider,
            &params.token_address,
            &params.owner,
            chain_id,
            env,
            params.vault_relayer_override.as_ref(),
        )
        .await
    }

    /// Sends an approval transaction using a sync signer.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or
    /// transaction submission fails.
    pub fn approve_cow_protocol<S>(
        &self,
        signer: &S,
        params: &ApprovalParameters,
    ) -> Result<TransactionHash, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let env = trader.env.unwrap_or(CowEnv::Prod);

        crate::approve_cow_protocol(signer, params, chain_id, env)
    }

    /// Sends an approval transaction using an async signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once the approval transaction
    /// has been broadcast, it cannot be withdrawn.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or
    /// transaction submission fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.approve_cow_protocol_async",
            ),
        ),
    )]
    pub async fn approve_cow_protocol_async<S>(
        &self,
        signer: &S,
        params: &ApprovalParameters,
    ) -> Result<TransactionHash, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let env = trader.env.unwrap_or(CowEnv::Prod);

        crate::approve_cow_protocol_async(signer, params, chain_id, env).await
    }
}
