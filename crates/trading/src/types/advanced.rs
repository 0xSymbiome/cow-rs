use std::{fmt, sync::Arc};

use cow_sdk_app_data::AppDataParams;
use cow_sdk_core::Amount;
use cow_sdk_orderbook::SigningScheme;
use cow_sdk_signing::eip1271::Eip1271SignatureProvider;

use super::{EthFlowOrderExistsChecker, QuoteRequestOverride, SlippageSuggestionProvider};

/// Optional knobs applied after quoting and before final submission.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct PostTradeAdditionalParams {
    /// Optional existence checker used by `EthFlow` unique-order-id generation.
    pub check_eth_flow_order_exists: Option<Arc<dyn EthFlowOrderExistsChecker>>,
    /// Optional network cost amount folded into amount calculations.
    pub network_costs_amount: Option<Amount>,
    /// Explicit signing scheme override for submission.
    pub signing_scheme: Option<SigningScheme>,
    /// Optional custom EIP-1271 signer for smart-account signatures.
    pub custom_eip1271_signature: Option<Arc<dyn Eip1271SignatureProvider>>,
    /// Whether costs, slippage, and fees should be applied when building the order payload.
    pub apply_costs_slippage_and_fees: Option<bool>,
}

impl PostTradeAdditionalParams {
    /// Creates an empty post-trade additional-parameter bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy with an explicit `EthFlow` existence checker.
    #[must_use]
    pub fn with_check_eth_flow_order_exists(
        mut self,
        checker: Arc<dyn EthFlowOrderExistsChecker>,
    ) -> Self {
        self.check_eth_flow_order_exists = Some(checker);
        self
    }

    /// Returns a copy with an explicit network-costs amount.
    #[must_use]
    pub const fn with_network_costs_amount(mut self, amount: Amount) -> Self {
        self.network_costs_amount = Some(amount);
        self
    }

    /// Returns a copy with an explicit signing-scheme override.
    #[must_use]
    pub const fn with_signing_scheme(mut self, scheme: SigningScheme) -> Self {
        self.signing_scheme = Some(scheme);
        self
    }

    /// Returns a copy with a custom EIP-1271 signature provider.
    #[must_use]
    pub fn with_custom_eip1271_signature(
        mut self,
        provider: Arc<dyn Eip1271SignatureProvider>,
    ) -> Self {
        self.custom_eip1271_signature = Some(provider);
        self
    }

    /// Returns a copy with an explicit cost/slippage/fee application flag.
    #[must_use]
    pub const fn with_apply_costs_slippage_and_fees(mut self, apply: bool) -> Self {
        self.apply_costs_slippage_and_fees = Some(apply);
        self
    }
}

impl fmt::Debug for PostTradeAdditionalParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostTradeAdditionalParams")
            .field(
                "check_eth_flow_order_exists",
                &self.check_eth_flow_order_exists.is_some(),
            )
            .field("network_costs_amount", &self.network_costs_amount)
            .field("signing_scheme", &self.signing_scheme)
            .field(
                "custom_eip1271_signature",
                &self.custom_eip1271_signature.is_some(),
            )
            .field(
                "apply_costs_slippage_and_fees",
                &self.apply_costs_slippage_and_fees,
            )
            .finish()
    }
}

/// Advanced settings shared by swap and limit-order quote and post workflows.
///
/// Limit-order flows leave `slippage_suggester` as `None` because the
/// limit submission path does not apply slippage in the same shape as
/// swaps; the field is documented but unused on that flow.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct TradeAdvancedSettings {
    /// Optional direct orderbook quote-request overrides.
    pub quote_request: Option<QuoteRequestOverride>,
    /// Optional app-data overrides merged into generated app-data documents.
    pub app_data: Option<AppDataParams>,
    /// Optional submission-time behavior overrides.
    pub additional_params: Option<PostTradeAdditionalParams>,
    /// Optional custom slippage-suggestion provider.
    ///
    /// Ignored on limit-order flows; limit orders do not apply
    /// slippage in the same shape as swaps.
    pub slippage_suggester: Option<Arc<dyn SlippageSuggestionProvider>>,
}

impl TradeAdvancedSettings {
    /// Creates an empty advanced-settings bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy with explicit quote-request overrides attached.
    #[must_use]
    pub const fn with_quote_request(mut self, overrides: QuoteRequestOverride) -> Self {
        self.quote_request = Some(overrides);
        self
    }

    /// Returns a copy with explicit app-data overrides attached.
    #[must_use]
    pub fn with_app_data(mut self, app_data: AppDataParams) -> Self {
        self.app_data = Some(app_data);
        self
    }

    /// Returns a copy with explicit submission-time additional parameters attached.
    #[must_use]
    pub fn with_additional_params(mut self, params: PostTradeAdditionalParams) -> Self {
        self.additional_params = Some(params);
        self
    }

    /// Returns a copy with a custom slippage-suggestion provider attached.
    ///
    /// Limit-order flows ignore this provider; only swap quote and
    /// post flows read it.
    #[must_use]
    pub fn with_slippage_suggester(
        mut self,
        suggester: Arc<dyn SlippageSuggestionProvider>,
    ) -> Self {
        self.slippage_suggester = Some(suggester);
        self
    }
}

impl fmt::Debug for TradeAdvancedSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradeAdvancedSettings")
            .field("quote_request", &self.quote_request)
            .field("app_data", &self.app_data)
            .field("additional_params", &self.additional_params)
            .field("slippage_suggester", &self.slippage_suggester.is_some())
            .finish()
    }
}
