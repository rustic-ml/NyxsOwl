use polars::prelude::*;
use super::common_signals::{ActionSignal, CrossoverSignal, TrendSignal};

// --- SMA Trend Identification ---
/// Identifies the trend based on the two most recent points of the SMA.
///
/// # Arguments
/// * `sma_series`: A Polars Series representing the Simple Moving Average values.
///
/// # Returns
/// A `Result` containing a `TrendSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Uptrend`: If the latest SMA value is greater than the previous SMA value.
/// - `Downtrend`: If the latest SMA value is less than the previous SMA value.
/// - `NoClearTrend`: If SMA values are equal or there's not enough data (less than 2 points).
pub fn trend_from_sma(sma_series: &Series) -> Result<TrendSignal, PolarsError> {
    if sma_series.len() < 2 {
        return Ok(TrendSignal::NoClearTrend);
    }
    let sma_f64 = sma_series.f64()?;
    let last_sma = sma_f64.get(sma_series.len() - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get last SMA value".into()))?;
    let prev_sma = sma_f64.get(sma_series.len() - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous SMA value".into()))?;

    if last_sma > prev_sma {
        Ok(TrendSignal::Uptrend)
    } else if last_sma < prev_sma {
        Ok(TrendSignal::Downtrend)
    } else {
        Ok(TrendSignal::NoClearTrend)
    }
}

// --- Price-SMA Crossover Strategy ---
/// Generates a signal when the price crosses over or under the SMA.
///
/// # Arguments
/// * `close_prices`: A Polars Series representing the closing prices.
/// * `sma_series`: A Polars Series representing the Simple Moving Average values.
///
/// # Returns
/// A `Result` containing a `CrossoverSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Bullish`: If the previous price was below or equal to the previous SMA, and the current price is above the current SMA.
/// - `Bearish`: If the previous price was above or equal to the previous SMA, and the current price is below the current SMA.
/// - `NoSignal`: Otherwise, or if there's not enough data (less than 2 points per series) or series lengths mismatch.
pub fn price_sma_crossover_signal(
    close_prices: &Series,
    sma_series: &Series,
) -> Result<CrossoverSignal, PolarsError> {
    let len = close_prices.len();
    if len < 2 || sma_series.len() < 2 || len != sma_series.len() {
        return Ok(CrossoverSignal::NoSignal);
    }

    let prices_f64 = close_prices.f64()?;
    let sma_f64 = sma_series.f64()?;

    let current_price = prices_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current price".into()))?;
    let previous_price = prices_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous price".into()))?;
    let current_sma = sma_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current SMA".into()))?;
    let previous_sma = sma_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous SMA".into()))?;

    if previous_price <= previous_sma && current_price > current_sma {
        Ok(CrossoverSignal::Bullish)
    } else if previous_price >= previous_sma && current_price < current_sma {
        Ok(CrossoverSignal::Bearish)
    } else {
        Ok(CrossoverSignal::NoSignal)
    }
}

// --- SMA-SMA Crossover Strategy (Golden Cross / Death Cross) ---
/// Generates a signal when a shorter-term SMA crosses over or under a longer-term SMA.
///
/// # Arguments
/// * `short_sma`: A Polars Series for the shorter-term SMA.
/// * `long_sma`: A Polars Series for the longer-term SMA.
///
/// # Returns
/// A `Result` containing a `CrossoverSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Bullish` (Golden Cross): If the previous short SMA was below or equal to the previous long SMA, and the current short SMA is above the current long SMA.
/// - `Bearish` (Death Cross): If the previous short SMA was above or equal to the previous long SMA, and the current short SMA is below the current long SMA.
/// - `NoSignal`: Otherwise, or if not enough data or series lengths mismatch.
pub fn sma_sma_crossover_signal(
    short_sma: &Series,
    long_sma: &Series,
) -> Result<CrossoverSignal, PolarsError> {
    let len = short_sma.len();
    if len < 2 || long_sma.len() < 2 || len != long_sma.len() {
        return Ok(CrossoverSignal::NoSignal);
    }

    let short_sma_f64 = short_sma.f64()?;
    let long_sma_f64 = long_sma.f64()?;

    let current_short_sma = short_sma_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current short SMA".into()))?;
    let previous_short_sma = short_sma_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous short SMA".into()))?;
    let current_long_sma = long_sma_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current long SMA".into()))?;
    let previous_long_sma = long_sma_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous long SMA".into()))?;

    if previous_short_sma <= previous_long_sma && current_short_sma > current_long_sma {
        Ok(CrossoverSignal::Bullish)
    } else if previous_short_sma >= previous_long_sma && current_short_sma < current_long_sma {
        Ok(CrossoverSignal::Bearish)
    } else {
        Ok(CrossoverSignal::NoSignal)
    }
}

/// Translates a Price-SMA crossover signal into a Buy/Sell/Hold action.
pub fn price_sma_crossover_action(
    close_prices: &Series,
    sma_series: &Series,
) -> Result<ActionSignal, PolarsError> {
    match price_sma_crossover_signal(close_prices, sma_series)? {
        CrossoverSignal::Bullish => Ok(ActionSignal::Buy),
        CrossoverSignal::Bearish => Ok(ActionSignal::Sell),
        CrossoverSignal::NoSignal => Ok(ActionSignal::Hold),
    }
}

/// Translates an SMA-SMA crossover signal into a Buy/Sell/Hold action.
pub fn sma_sma_crossover_action(
    short_sma: &Series,
    long_sma: &Series,
) -> Result<ActionSignal, PolarsError> {
    match sma_sma_crossover_signal(short_sma, long_sma)? {
        CrossoverSignal::Bullish => Ok(ActionSignal::Buy),
        CrossoverSignal::Bearish => Ok(ActionSignal::Sell),
        CrossoverSignal::NoSignal => Ok(ActionSignal::Hold),
    }
}

// TODO: Add unit tests for these strategy functions. 