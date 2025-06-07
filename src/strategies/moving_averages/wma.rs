use polars::prelude::*;
use super::common_signals::{ActionSignal, CrossoverSignal, TrendSignal};

// --- WMA Trend Identification ---
/// Identifies the trend based on the two most recent points of the WMA.
///
/// # Arguments
/// * `wma_series`: A Polars Series representing the Weighted Moving Average values.
///
/// # Returns
/// A `Result` containing a `TrendSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Uptrend`: If the latest WMA value is greater than the previous WMA value.
/// - `Downtrend`: If the latest WMA value is less than the previous WMA value.
/// - `NoClearTrend`: If WMA values are equal or there's not enough data.
pub fn trend_from_wma(wma_series: &Series) -> Result<TrendSignal, PolarsError> {
    if wma_series.len() < 2 {
        return Ok(TrendSignal::NoClearTrend);
    }
    let wma_f64 = wma_series.f64()?;
    let last_wma = wma_f64.get(wma_series.len() - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get last WMA value".into()))?;
    let prev_wma = wma_f64.get(wma_series.len() - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous WMA value".into()))?;

    if last_wma > prev_wma {
        Ok(TrendSignal::Uptrend)
    } else if last_wma < prev_wma {
        Ok(TrendSignal::Downtrend)
    } else {
        Ok(TrendSignal::NoClearTrend)
    }
}

// --- Price-WMA Crossover Strategy ---
/// Generates a signal when the price crosses over or under the WMA.
///
/// # Arguments
/// * `close_prices`: A Polars Series representing the closing prices.
/// * `wma_series`: A Polars Series representing the Weighted Moving Average values.
///
/// # Returns
/// A `Result` containing a `CrossoverSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Bullish`: Previous price <= previous WMA, and current price > current WMA.
/// - `Bearish`: Previous price >= previous WMA, and current price < current WMA.
/// - `NoSignal`: Otherwise, or if not enough data or series lengths mismatch.
pub fn price_wma_crossover_signal(
    close_prices: &Series,
    wma_series: &Series,
) -> Result<CrossoverSignal, PolarsError> {
    let len = close_prices.len();
    if len < 2 || wma_series.len() < 2 || len != wma_series.len() {
        return Ok(CrossoverSignal::NoSignal);
    }

    let prices_f64 = close_prices.f64()?;
    let wma_f64 = wma_series.f64()?;

    let current_price = prices_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current price".into()))?;
    let previous_price = prices_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous price".into()))?;
    let current_wma = wma_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current WMA".into()))?;
    let previous_wma = wma_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous WMA".into()))?;

    if previous_price <= previous_wma && current_price > current_wma {
        Ok(CrossoverSignal::Bullish)
    } else if previous_price >= previous_wma && current_price < current_wma {
        Ok(CrossoverSignal::Bearish)
    } else {
        Ok(CrossoverSignal::NoSignal)
    }
}

// --- WMA-WMA Crossover Strategy ---
/// Generates a signal when a shorter-term WMA crosses over or under a longer-term WMA.
///
/// # Arguments
/// * `short_wma`: A Polars Series for the shorter-term WMA.
/// * `long_wma`: A Polars Series for the longer-term WMA.
///
/// # Returns
/// A `Result` containing a `CrossoverSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Bullish`: Previous short WMA <= previous long WMA, and current short WMA > current long WMA.
/// - `Bearish`: Previous short WMA >= previous long WMA, and current short WMA < current long WMA.
/// - `NoSignal`: Otherwise, or if not enough data or series lengths mismatch.
pub fn wma_wma_crossover_signal(
    short_wma: &Series,
    long_wma: &Series,
) -> Result<CrossoverSignal, PolarsError> {
    let len = short_wma.len();
    if len < 2 || long_wma.len() < 2 || len != long_wma.len() {
        return Ok(CrossoverSignal::NoSignal);
    }

    let short_wma_f64 = short_wma.f64()?;
    let long_wma_f64 = long_wma.f64()?;

    let current_short_wma = short_wma_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current short WMA".into()))?;
    let previous_short_wma = short_wma_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous short WMA".into()))?;
    let current_long_wma = long_wma_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current long WMA".into()))?;
    let previous_long_wma = long_wma_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous long WMA".into()))?;

    if previous_short_wma <= previous_long_wma && current_short_wma > current_long_wma {
        Ok(CrossoverSignal::Bullish)
    } else if previous_short_wma >= previous_long_wma && current_short_wma < current_long_wma {
        Ok(CrossoverSignal::Bearish)
    } else {
        Ok(CrossoverSignal::NoSignal)
    }
}

/// Translates a Price-WMA crossover signal into a Buy/Sell/Hold action.
pub fn price_wma_crossover_action(
    close_prices: &Series,
    wma_series: &Series,
) -> Result<ActionSignal, PolarsError> {
    match price_wma_crossover_signal(close_prices, wma_series)? {
        CrossoverSignal::Bullish => Ok(ActionSignal::Buy),
        CrossoverSignal::Bearish => Ok(ActionSignal::Sell),
        CrossoverSignal::NoSignal => Ok(ActionSignal::Hold),
    }
}

/// Translates a WMA-WMA crossover signal into a Buy/Sell/Hold action.
pub fn wma_wma_crossover_action(
    short_wma: &Series,
    long_wma: &Series,
) -> Result<ActionSignal, PolarsError> {
    match wma_wma_crossover_signal(short_wma, long_wma)? {
        CrossoverSignal::Bullish => Ok(ActionSignal::Buy),
        CrossoverSignal::Bearish => Ok(ActionSignal::Sell),
        CrossoverSignal::NoSignal => Ok(ActionSignal::Hold),
    }
}

// TODO: Add unit tests for these strategy functions. 