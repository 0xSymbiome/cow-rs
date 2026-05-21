use std::collections::BTreeMap;

use alloy_primitives::Bytes;
use alloy_sol_types::SolCall;

use cow_sdk_core::{Address, Amount, SupportedChainId, TypedDataDomain};

use crate::{
    ContractsError,
    deployments::{ContractId, Registry},
    interaction::{Interaction, InteractionLike, normalize_interaction},
    order::{Order, extract_order_uid_params, normalize_order},
    signature::Signature,
};

use super::codec::{encode_settle_call, encode_trade as encode_settlement_trade};
#[allow(
    clippy::wildcard_imports,
    reason = "settlement encoder intentionally shares the parent sol! binding and DTO namespace"
)]
use super::*;
/// Stateful settlement encoder.
#[derive(Debug, Clone)]
pub struct SettlementEncoder {
    /// Typed-data domain used for the settlement.
    pub domain: TypedDataDomain,
    tokens: TokenRegistry,
    trades: Vec<Trade>,
    interactions: [Vec<Interaction>; 3],
    order_refunds: OrderRefunds,
}

impl SettlementEncoder {
    /// Creates a new settlement encoder.
    #[must_use]
    pub fn new(domain: TypedDataDomain) -> Self {
        Self {
            domain,
            tokens: TokenRegistry::new(),
            trades: Vec::new(),
            interactions: [Vec::new(), Vec::new(), Vec::new()],
            order_refunds: OrderRefunds::new(Vec::new(), Vec::new()),
        }
    }

    /// Returns the encoded token registry in index order.
    #[must_use]
    pub fn tokens(&self) -> Vec<Address> {
        self.tokens.addresses()
    }

    /// Returns the encoded trades.
    #[must_use]
    pub fn trades(&self) -> Vec<Trade> {
        self.trades.clone()
    }

    /// Returns the encoded interactions grouped by stage.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if post-stage order-refund interactions cannot
    /// be encoded.
    pub fn interactions(&self) -> Result<[Vec<Interaction>; 3], ContractsError> {
        Ok([
            self.interactions[InteractionStage::Pre as usize].clone(),
            self.interactions[InteractionStage::Intra as usize].clone(),
            {
                let mut post = self.interactions[InteractionStage::Post as usize].clone();
                post.extend(self.encoded_order_refunds()?);
                post
            },
        ])
    }

    /// Returns the encoded post-interactions used to clear refund storage.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if a stored order UID cannot be decoded.
    pub fn encoded_order_refunds(&self) -> Result<Vec<Interaction>, ContractsError> {
        let mut interactions = Vec::new();
        for (kind, order_uids) in [
            (
                OrderRefundKind::FilledAmount,
                &self.order_refunds.filled_amounts,
            ),
            (
                OrderRefundKind::PreSignature,
                &self.order_refunds.pre_signatures,
            ),
        ] {
            if order_uids.is_empty() {
                continue;
            }
            let encoded_uids: Vec<alloy_sol_types::private::Bytes> = order_uids
                .iter()
                .map(|uid| alloy_sol_types::private::Bytes::from(uid.as_slice().to_vec()))
                .collect();
            let call_data = match kind {
                OrderRefundKind::FilledAmount => IGPv2Settlement::freeFilledAmountStorageCall {
                    orderUids: encoded_uids,
                }
                .abi_encode(),
                OrderRefundKind::PreSignature => IGPv2Settlement::freePreSignatureStorageCall {
                    orderUids: encoded_uids,
                }
                .abi_encode(),
            };
            interactions.push(Interaction::new(
                self.domain.verifying_contract,
                Amount::zero(),
                Bytes::from(call_data),
            ));
        }
        Ok(interactions)
    }

    /// Returns clearing prices aligned to the encoder's token registry.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::MissingClearingPrice`] if any registered token
    /// is absent from `prices`.
    pub fn clearing_prices(&self, prices: &Prices) -> Result<Vec<Amount>, ContractsError> {
        let normalized: BTreeMap<String, Amount> = prices
            .iter()
            .map(|(token, price)| (token.normalized_key(), *price))
            .collect();

        self.tokens
            .addresses()
            .iter()
            .map(|token| {
                normalized
                    .get(&token.normalized_key())
                    .copied()
                    .ok_or_else(|| ContractsError::MissingClearingPrice { token: *token })
            })
            .collect()
    }

    /// Encodes and appends a trade.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if order normalization fails, if execution is
    /// missing for a partially fillable order, or if trade encoding fails.
    pub fn encode_trade(
        &mut self,
        order: &Order,
        signature: &Signature,
        execution: Option<TradeExecution>,
    ) -> Result<(), ContractsError> {
        let order = normalize_order(order)?;
        let execution = match execution {
            Some(execution) => execution,
            None if order.partially_fillable => return Err(ContractsError::MissingExecutedAmount),
            None => TradeExecution::new(Amount::zero()),
        };
        self.trades.push(encode_settlement_trade(
            &mut self.tokens,
            &order,
            signature,
            &execution,
        )?);
        Ok(())
    }

    /// Encodes and appends an interaction in the requested stage.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::ForbiddenInteractionTarget`] when this
    /// encoder's domain identifies a canonical registry settlement and the
    /// supplied interaction targets that settlement's paired vault relayer.
    pub fn encode_interaction(
        &mut self,
        interaction: &InteractionLike,
        stage: InteractionStage,
    ) -> Result<(), ContractsError> {
        if self
            .canonical_vault_relayer_target()
            .is_some_and(|target| interaction.target == target)
        {
            return Err(ContractsError::ForbiddenInteractionTarget {
                target: interaction.target,
            });
        }
        self.interactions[stage as usize].push(normalize_interaction(interaction));
        Ok(())
    }

    /// Appends order-refund storage-clearing requests.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if any supplied UID cannot be decoded.
    pub fn encode_order_refunds(&mut self, refunds: &OrderRefunds) -> Result<(), ContractsError> {
        for uid in refunds
            .filled_amounts
            .iter()
            .chain(refunds.pre_signatures.iter())
        {
            let _ = extract_order_uid_params(uid)?;
        }
        self.order_refunds
            .filled_amounts
            .extend(refunds.filled_amounts.clone());
        self.order_refunds
            .pre_signatures
            .extend(refunds.pre_signatures.clone());
        Ok(())
    }

    /// Returns the fully encoded settlement tuple.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if clearing prices or interactions cannot be encoded.
    pub fn encoded_settlement(&self, prices: &Prices) -> Result<EncodedSettlement, ContractsError> {
        Ok((
            self.tokens(),
            self.clearing_prices(prices)?,
            self.trades(),
            self.interactions()?,
        ))
    }

    /// Returns the ABI-encoded `settle(...)` call-data for the current encoder state.
    ///
    /// The encoded bytes match the canonical `GPv2Settlement` `settle` function
    /// selector and argument layout generated by the `alloy::sol!` binding,
    /// suitable for routing through a submission transport.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if clearing prices or interactions cannot be
    /// encoded, or if any typed domain value is not representable on the wire.
    pub fn encoded_settlement_calldata(&self, prices: &Prices) -> Result<Vec<u8>, ContractsError> {
        let (tokens, clearing_prices, trades, interactions) = self.encoded_settlement(prices)?;
        let call = encode_settle_call(&tokens, &clearing_prices, &trades, &interactions)?;
        Ok(call.abi_encode())
    }

    /// Returns an interaction-only settlement setup payload.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::encode_interaction`].
    pub fn encoded_setup(
        interactions: &[InteractionLike],
    ) -> Result<EncodedSettlement, ContractsError> {
        let mut encoder = Self::new(TypedDataDomain::new(
            "unused".to_owned(),
            "unused".to_owned(),
            0,
            Address::zero(),
        ));
        for interaction in interactions {
            encoder.encode_interaction(interaction, InteractionStage::Intra)?;
        }
        Ok((
            encoder.tokens(),
            Vec::new(),
            encoder.trades(),
            [
                encoder.interactions[InteractionStage::Pre as usize].clone(),
                encoder.interactions[InteractionStage::Intra as usize].clone(),
                encoder.interactions[InteractionStage::Post as usize].clone(),
            ],
        ))
    }

    fn canonical_vault_relayer_target(&self) -> Option<Address> {
        let chain_id = SupportedChainId::try_from(self.domain.chain_id).ok()?;
        let registry = Registry::default();
        let mut matches = registry
            .entries()
            .filter(|(contract_id, entry_chain_id, _, address)| {
                *contract_id == ContractId::Settlement
                    && *entry_chain_id == chain_id.into()
                    && *address == &self.domain.verifying_contract
            })
            .map(|(_, _, env, _)| env);

        let env = matches.next()?;
        if matches.next().is_some() {
            return None;
        }

        registry.address(ContractId::VaultRelayer, chain_id, env)
    }
}

#[cfg(test)]
mod tests {
    use cow_sdk_core::{
        Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource,
    };

    use crate::{order::Order, signature::Signature};

    use super::*;

    fn sample_order(partially_fillable: bool) -> Order {
        Order::new(
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            Some(Address::new("0x3333333333333333333333333333333333333333").unwrap()),
            Amount::new("10").unwrap(),
            Amount::new("20").unwrap(),
            123,
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
            Amount::new("1").unwrap(),
            OrderKind::Buy,
            partially_fillable,
            Some(SellTokenSource::Internal),
            Some(BuyTokenDestination::Internal),
        )
    }

    fn sample_signature() -> Signature {
        Signature::PreSign {
            owner: Address::new("0x4444444444444444444444444444444444444444").unwrap(),
        }
    }

    #[test]
    fn fill_or_kill_orders_default_the_executed_amount_to_zero() {
        let domain = TypedDataDomain::new(
            "Gnosis Protocol".to_owned(),
            "v2".to_owned(),
            1,
            Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
        );
        let mut encoder = SettlementEncoder::new(domain);

        encoder
            .encode_trade(&sample_order(false), &sample_signature(), None)
            .unwrap();

        assert_eq!(encoder.trades()[0].executed_amount, Amount::zero());
    }
}
