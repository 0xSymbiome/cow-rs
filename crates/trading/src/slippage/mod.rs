//! Slippage and fee calculation helpers.

pub use self::amounts::calculate_quote_amounts_and_costs;
pub(crate) use self::amounts::{gas_with_margin, parse_integer};
pub use self::breakdown::partner_fee_bps;
pub use self::policy::{
    resolve_slippage_suggestion, sanitize_protocol_fee_bps, suggest_slippage_bps,
    suggest_slippage_from_fee, suggest_slippage_from_volume,
};

mod amounts;
mod breakdown;
mod policy;

use cow_sdk_core::SupportedChainId;

/// Default quote validity, in seconds, when no explicit validity window is supplied.
pub const DEFAULT_QUOTE_VALIDITY: u32 = 60 * 30;
/// Default slippage suggestion, in basis points, for flows that do not require a higher floor.
pub const DEFAULT_SLIPPAGE_BPS: u32 = 50;
/// Maximum supported slippage, in basis points.
pub const MAX_SLIPPAGE_BPS: u32 = 10_000;
/// Extra gas margin, in percent, added to derived on-chain transaction estimates.
pub const GAS_MARGIN_PERCENT: u32 = 20;
/// Fallback gas limit used when no explicit verification gas limit is available.
pub const GAS_LIMIT_DEFAULT: u32 = 150_000;

pub(super) const ONE_HUNDRED_BPS: i64 = 10_000;

/// Returns the default slippage floor for the given chain and trade style.
#[must_use]
pub const fn default_slippage_bps(_chain_id: SupportedChainId, _is_ethflow: bool) -> u32 {
    DEFAULT_SLIPPAGE_BPS
}
