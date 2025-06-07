use polars::prelude::*;
use super::common_signals::ActionSignal;

#[derive(Debug, PartialEq, Clone)]
pub enum VwapPricePosition {
    AboveVwap,   // Considered bullish for the session
    BelowVwap,   // Considered bearish for the session
    AtVwap,      // Price is at or very close to VWAP
    NoSignal,    // Not enough data or series lengths mismatch
}

/// Assesses the current price position relative to the VWAP.
/// VWAP is typically an intraday indicator.
///
/// # Arguments
/// * `close_prices`: A Polars Series representing the closing prices.
/// * `vwap_series`: A Polars Series representing the VWAP values.
///
/// # Returns
/// A `Result` containing a `VwapPricePosition` or a `PolarsError`.
///
/// # Strategy
/// - `AboveVwap`: If the latest price is above the latest VWAP value.
/// - `BelowVwap`: If the latest price is below the latest VWAP value.
/// - `AtVwap`: If the latest price is equal to the latest VWAP value.
/// - `NoSignal`: If there's not enough data (less than 1 point) or series lengths mismatch.
pub fn get_price_vwap_position(
    close_prices: &Series,
    vwap_series: &Series,
) -> Result<VwapPricePosition, PolarsError> {
    let len = close_prices.len();
    if len == 0 || vwap_series.len() == 0 || len != vwap_series.len() {
        return Ok(VwapPricePosition::NoSignal);
    }

    let prices_f64 = close_prices.f64()?;
    let vwap_f64 = vwap_series.f64()?;

    let current_price = prices_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current price".into()))?;
    let current_vwap = vwap_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current VWAP".into()))?;

    // Using a small epsilon for floating point comparison might be considered for AtVwap,
    // but for simplicity, direct comparison is used here.
    if current_price > current_vwap {
        Ok(VwapPricePosition::AboveVwap)
    } else if current_price < current_vwap {
        Ok(VwapPricePosition::BelowVwap)
    } else {
        Ok(VwapPricePosition::AtVwap)
    }
}

/// Translates a VwapPricePosition signal into a Buy/Sell/Hold action.
///
/// # Arguments
/// * `close_prices`: A Polars Series representing the closing prices.
/// * `vwap_series`: A Polars Series representing the VWAP values.
///
/// # Returns
/// A `Result` containing an `ActionSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Buy`: If price is AboveVwap (bullish intraday).
/// - `Sell`: If price is BelowVwap (bearish intraday).
/// - `Hold`: If price is AtVwap or NoSignal.
pub fn vwap_price_position_action(
    close_prices: &Series,
    vwap_series: &Series,
) -> Result<ActionSignal, PolarsError> {
    match get_price_vwap_position(close_prices, vwap_series)? {
        VwapPricePosition::AboveVwap => Ok(ActionSignal::Buy),
        VwapPricePosition::BelowVwap => Ok(ActionSignal::Sell),
        VwapPricePosition::AtVwap | VwapPricePosition::NoSignal => Ok(ActionSignal::Hold),
    }
}

// TODO: Add unit tests for these strategy functions.
// TODO: Consider strategies for VWAP crossovers if VWAP is treated more like a MA,
//       or VWAP band strategies (requires std dev bands calculation). 