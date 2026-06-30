//! Authorization-as-a-value order placement (ADR 0073).
//!
//! Order placement takes its authorization mode as one [`Authorization`] value
//! and returns the typed [`OrderPlacement`] sum, so a smart-contract-wallet
//! order is the same call shape as an EOA order. The scheme statically selects
//! the result arm: [`Authorization::Ecdsa`] and [`Authorization::Eip1271`]
//! resolve to [`OrderPlacement::Live`] (the order is valid once posted), and
//! [`Authorization::PreSign`] resolves to [`OrderPlacement::PendingActivation`],
//! whose [`SafeActivation`] bundles the on-chain approve-then-set-pre-signature
//! pair the owner still owes.
//!
//! Every method here is a thin front-end over the existing scheme dispatch
//! ([`post_swap_order_from_quote`](crate::post_swap_order_from_quote),
//! [`post_limit_order`](crate::post_limit_order),
//! [`post_swap_order_presign`](crate::post_swap_order_presign),
//! [`post_limit_order_presign`](crate::post_limit_order_presign)) and reshapes
//! the [`OrderPostingResult`] into the [`OrderPlacement`] sum. The placement
//! path enforces no new invariants: `from == owner`, the zero fee amount, the
//! empty pre-sign signature, app-data hashing, and order-UID derivation are all
//! owned by the dispatch it reuses.

use std::sync::Arc;

use cow_sdk_contracts::UnsignedTransaction;
use cow_sdk_core::{
    Address, Amount, OrderUid, ProtocolOptions, Signer, SupportedChainId, TransactionBroadcast,
    TransactionRequest, TypedDataPayload,
};
use cow_sdk_orderbook::{OrderbookClient, SigningScheme};
use cow_sdk_signing::eip1271::Eip1271Signer;

use crate::{LimitTradeParams, QuoteResults, TradeAdvancedSettings, TraderParams, TradingError};

/// Public no-signer stand-in for the smart-account placement arms.
///
/// [`Authorization::Eip1271`] and [`Authorization::PreSign`] consult no ECDSA
/// signer: the owner is explicit and the contract signature or on-chain pre-sign
/// authorizes the order. This type is the default `S` of [`Authorization`] so
/// those arms need no turbofish, and it is the signer threaded through the
/// EIP-1271 placement arm — where the owner is injected and the EIP-1271
/// provider produces the signature, so none of its operations are reached. Every
/// operation fails with a description of the unexpected call, so a pipeline
/// change that starts consulting the signer on this path surfaces loudly instead
/// of fabricating a signature.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoSigner;

impl NoSigner {
    fn unreachable_operation(operation: &str) -> String {
        format!("smart-account placement must not consult a signer ({operation})")
    }
}

impl Signer for NoSigner {
    type Error = String;

    async fn address(&self) -> Result<Address, Self::Error> {
        Err(Self::unreachable_operation("address"))
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Err(Self::unreachable_operation("sign_message"))
    }

    async fn sign_typed_data_payload(
        &self,
        _payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        Err(Self::unreachable_operation("sign_typed_data_payload"))
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err(Self::unreachable_operation("send_transaction"))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Err(Self::unreachable_operation("estimate_gas"))
    }
}

/// On-chain activation a smart-contract wallet must run to authorize a posted
/// pre-sign order.
///
/// Carries the ordered approve-then-set-pre-signature pair as
/// [`UnsignedTransaction`] values (ADR 0070) for one smart-account batch. The
/// bundle is transport-neutral: a single-owner Safe can send the calls
/// directly, while a higher-threshold Safe proposes them to its transaction
/// service for the owners to co-sign. The first call grants the vault relayer
/// the sell-token allowance the order needs at fill time; the second flips the
/// settlement `setPreSignature` flag that makes the order fillable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SafeActivation {
    /// Ordered `[approve, setPreSignature]` calls for one smart-account batch.
    ///
    /// Convert an element to the [`TransactionRequest`] wire shape with
    /// `.into()` to send a leg individually.
    pub calls: Vec<UnsignedTransaction>,
}

impl SafeActivation {
    /// Creates a Safe activation from its ordered calls.
    #[must_use]
    pub const fn new(calls: Vec<UnsignedTransaction>) -> Self {
        Self { calls }
    }
}

/// Typed placement result returned by [`place_swap`] and [`place_limit`].
///
/// The authorization mode statically selects the arm, so the on-chain
/// obligation of a pre-sign order cannot be dropped: the `order_uid` of a
/// [`OrderPlacement::PendingActivation`] is reachable only by matching the arm
/// that also yields its [`SafeActivation`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OrderPlacement {
    /// The order is live at post — produced by [`Authorization::Ecdsa`] and
    /// [`Authorization::Eip1271`].
    Live {
        /// Final order UID.
        order_uid: OrderUid,
    },
    /// The order is posted but not yet authorized on-chain — produced by
    /// [`Authorization::PreSign`]. The owner must send or propose `activation`
    /// from the smart account to make the order fillable.
    PendingActivation {
        /// Final order UID.
        order_uid: OrderUid,
        /// On-chain approve-then-set-pre-signature bundle.
        activation: SafeActivation,
    },
}

impl OrderPlacement {
    /// Returns the final order UID regardless of arm.
    #[must_use]
    pub const fn order_uid(&self) -> &OrderUid {
        match self {
            Self::Live { order_uid } | Self::PendingActivation { order_uid, .. } => order_uid,
        }
    }
}

/// Order authorization mode carried as a value (ADR 0073).
///
/// `S` is the ECDSA signer type carried by [`Authorization::Ecdsa`]; the
/// [`Authorization::Eip1271`] and [`Authorization::PreSign`] arms do not consult
/// it. The default `S` lets the smart-account arms be written without a
/// turbofish: `Authorization::PreSign` and `Authorization::Eip1271(provider)`
/// infer `S` from this default, while the [`Authorization::Ecdsa`] arm pins `S`
/// to the supplied signer.
pub enum Authorization<'a, S: Signer = NoSigner> {
    /// EOA / EIP-712 signing — gasless. Resolves to [`OrderPlacement::Live`].
    Ecdsa(&'a S),
    /// Safe off-chain EIP-1271 contract signature — gasless. Resolves to
    /// [`OrderPlacement::Live`].
    Eip1271(Arc<dyn Eip1271Signer>),
    /// Safe on-chain pre-sign — no signing. Resolves to
    /// [`OrderPlacement::PendingActivation`].
    PreSign,
}

impl<S: Signer> core::fmt::Debug for Authorization<'_, S> {
    /// Prints the authorization arm by name only, never the carried signer or
    /// contract-signature provider, so a signer's internals stay out of any
    /// debug rendering.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let arm = match self {
            Self::Ecdsa(_) => "Ecdsa(..)",
            Self::Eip1271(_) => "Eip1271(..)",
            Self::PreSign => "PreSign",
        };
        formatter.write_str(arm)
    }
}

impl<'a, S: Signer> Authorization<'a, S> {
    /// Constructs the ECDSA authorization carrying `signer`.
    #[must_use]
    pub const fn ecdsa(signer: &'a S) -> Self {
        Self::Ecdsa(signer)
    }
}

impl Authorization<'_, NoSigner> {
    /// Constructs the EIP-1271 authorization carrying a contract-signature
    /// provider, pinning the unused ECDSA signer type so the call site needs no
    /// turbofish.
    #[must_use]
    pub fn eip1271(provider: Arc<dyn Eip1271Signer>) -> Self {
        Self::Eip1271(provider)
    }

    /// Constructs the pre-sign authorization, pinning the unused ECDSA signer
    /// type so the call site needs no turbofish.
    #[must_use]
    pub const fn pre_sign() -> Self {
        Self::PreSign
    }
}

/// Builds the on-chain activation bundle for an already-posted pre-sign order.
///
/// Composes the two calls a smart-contract wallet runs to authorize the order:
/// the ERC-20 `approve` of the sell-token allowance for the vault relayer
/// ([`approve_transaction`](cow_sdk_contracts::approve_transaction), spender
/// resolved through the embedded registry / `options` override) and the
/// settlement `setPreSignature(uid, true)`
/// ([`pre_sign_transaction`](cow_sdk_contracts::pre_sign_transaction)). Both
/// legs carry zero native value. The builder is pure: it reads no on-chain
/// allowance and always emits both calls, so a caller whose vault-relayer
/// allowance already covers the sell amount may drop the approve leg.
///
/// A pre-sign order needs no balance or allowance at creation, so the approve is
/// part of the activation rather than the placement, matching the orderbook's
/// acceptance rules.
///
/// # Errors
///
/// Returns [`TradingError::Contracts`] when no settlement deployment is
/// registered for the chain/environment.
pub fn build_presign_activation(
    order_uid: &OrderUid,
    sell_token: Address,
    amount: Amount,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<SafeActivation, TradingError> {
    let spender = cow_sdk_contracts::resolve_contract_address(
        cow_sdk_contracts::ContractId::VaultRelayer,
        None,
        chain_id,
        options
            .and_then(|opts| opts.env)
            .unwrap_or(cow_sdk_core::CowEnv::Prod),
    )
    .ok_or(cow_sdk_contracts::ContractsError::DeploymentNotFound {
        contract: "vault-relayer",
        chain_id: u64::from(chain_id),
    })?;
    let approve = cow_sdk_contracts::approve_transaction(sell_token, spender, amount);
    let set_pre_signature = cow_sdk_contracts::pre_sign_transaction(order_uid, chain_id, options)?;

    Ok(SafeActivation::new(vec![approve, set_pre_signature]))
}

/// Posts a swap order from quote results, selecting the signing path from
/// `auth` and reshaping the result into the [`OrderPlacement`] sum (ADR 0073).
///
/// Thin front-end over the existing dispatch:
/// [`Authorization::Ecdsa`] / [`Authorization::Eip1271`] post through
/// [`post_swap_order_from_quote`](crate::post_swap_order_from_quote) and resolve
/// to [`OrderPlacement::Live`]; [`Authorization::PreSign`] posts through
/// [`post_swap_order_presign`](crate::post_swap_order_presign) and resolves to
/// [`OrderPlacement::PendingActivation`] whose activation is built by
/// [`build_presign_activation`] for `owner`'s sell token and amount.
///
/// # Errors
///
/// Returns [`TradingError`] when the quote binding no longer matches, when
/// signing fails, when the orderbook rejects the submission, or when the
/// pre-sign activation cannot be built.
pub async fn place_swap<O, S>(
    quote_results: &QuoteResults,
    owner: Address,
    auth: Authorization<'_, S>,
    trader: &TraderParams,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPlacement, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
{
    let settings = settings_with_owner(advanced_settings, owner);
    match auth {
        Authorization::Ecdsa(signer) => {
            let result = crate::post::post_swap_order_from_quote(
                quote_results,
                trader,
                signer,
                Some(&settings),
                orderbook,
            )
            .await?;
            Ok(OrderPlacement::Live {
                order_uid: result.order_id,
            })
        }
        Authorization::Eip1271(provider) => {
            let settings = settings_with_eip1271(settings, provider);
            let result = crate::post::post_swap_order_from_quote(
                quote_results,
                trader,
                &NoSigner,
                Some(&settings),
                orderbook,
            )
            .await?;
            Ok(OrderPlacement::Live {
                order_uid: result.order_id,
            })
        }
        Authorization::PreSign => {
            let result = crate::post::post_swap_order_presign(
                quote_results,
                trader,
                Some(&settings),
                orderbook,
            )
            .await?;
            into_pending_activation(
                result.order_id,
                result.order_to_sign.sell_amount,
                quote_results.trade_parameters.sell_token,
                trader,
            )
        }
    }
}

/// Posts a limit order, selecting the signing path from `auth` and reshaping the
/// result into the [`OrderPlacement`] sum (ADR 0073).
///
/// Thin front-end over the existing dispatch:
/// [`Authorization::Ecdsa`] / [`Authorization::Eip1271`] post through
/// [`post_limit_order`](crate::post_limit_order) and resolve to
/// [`OrderPlacement::Live`]; [`Authorization::PreSign`] posts through
/// [`post_limit_order_presign`](crate::post_limit_order_presign) and resolves to
/// [`OrderPlacement::PendingActivation`] whose activation is built by
/// [`build_presign_activation`] for the order's sell token and amount.
///
/// # Errors
///
/// Returns [`TradingError`] when required defaults are missing, when app-data
/// generation or signing fails, when the orderbook rejects the submission, or
/// when the pre-sign activation cannot be built.
pub async fn place_limit<O, S>(
    params: LimitTradeParams,
    owner: Address,
    auth: Authorization<'_, S>,
    trader: &TraderParams,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPlacement, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
{
    let mut params = params;
    params.owner = Some(owner);
    let sell_token = params.sell_token;
    let sell_amount = params.sell_amount;
    let settings = advanced_settings.cloned();
    match auth {
        Authorization::Ecdsa(signer) => {
            let result = crate::post::post_limit_order(
                &params,
                trader,
                signer,
                settings.as_ref(),
                orderbook,
            )
            .await?;
            Ok(OrderPlacement::Live {
                order_uid: result.order_id,
            })
        }
        Authorization::Eip1271(provider) => {
            let settings = settings_with_eip1271(settings.unwrap_or_default(), provider);
            let result = crate::post::post_limit_order(
                &params,
                trader,
                &NoSigner,
                Some(&settings),
                orderbook,
            )
            .await?;
            Ok(OrderPlacement::Live {
                order_uid: result.order_id,
            })
        }
        Authorization::PreSign => {
            let result = crate::post::post_limit_order_presign(
                &params,
                trader,
                settings.as_ref(),
                orderbook,
            )
            .await?;
            into_pending_activation(result.order_id, sell_amount, sell_token, trader)
        }
    }
}

/// Reshapes a pre-sign posting result into the pending-activation arm, building
/// the activation for `sell_token` and the order's signed sell amount.
fn into_pending_activation(
    order_uid: OrderUid,
    sell_amount: Amount,
    sell_token: Address,
    trader: &TraderParams,
) -> Result<OrderPlacement, TradingError> {
    let activation = build_presign_activation(
        &order_uid,
        sell_token,
        sell_amount,
        trader.chain_id,
        Some(&presign_options(trader)),
    )?;
    Ok(OrderPlacement::PendingActivation {
        order_uid,
        activation,
    })
}

/// Builds the protocol options the pre-sign activation needs from the trader's
/// environment and settlement override.
fn presign_options(trader: &TraderParams) -> ProtocolOptions {
    let mut options =
        ProtocolOptions::new().with_env(trader.env.unwrap_or(cow_sdk_core::CowEnv::Prod));
    if let Some(overrides) = trader.settlement_contract_override.as_ref() {
        options = options.with_settlement_contract_override(overrides.clone());
    }
    options
}

/// Clones the advanced settings and injects `owner` as the quote-request `from`
/// so the swap-from-quote dispatch posts with `from == owner`.
fn settings_with_owner(
    advanced_settings: Option<&TradeAdvancedSettings>,
    owner: Address,
) -> TradeAdvancedSettings {
    let mut settings = advanced_settings.cloned().unwrap_or_default();
    settings.quote_request = Some(
        settings
            .quote_request
            .take()
            .unwrap_or_default()
            .with_from(owner),
    );
    settings
}

/// Injects the EIP-1271 provider and the EIP-1271 signing scheme into the
/// submission settings, reusing the existing additional-params seam.
fn settings_with_eip1271(
    mut settings: TradeAdvancedSettings,
    provider: Arc<dyn Eip1271Signer>,
) -> TradeAdvancedSettings {
    settings.additional_params = Some(
        settings
            .additional_params
            .take()
            .unwrap_or_default()
            .with_signing_scheme(SigningScheme::Eip1271)
            .with_custom_eip1271_signature_shared(provider),
    );
    settings
}
