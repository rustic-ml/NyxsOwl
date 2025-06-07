use polars::prelude::*;
use super::common_signals::{ActionSignal, CrossoverSignal, TrendSignal};

// --- EMA Trend Identification ---
/// Identifies the trend based on the two most recent points of the EMA.
///
/// # Arguments
/// * `ema_series`: A Polars Series representing the Exponential Moving Average values.
///
/// # Returns
/// A `Result` containing a `TrendSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Uptrend`: If the latest EMA value is greater than the previous EMA value.
/// - `Downtrend`: If the latest EMA value is less than the previous EMA value.
/// - `NoClearTrend`: If EMA values are equal or there's not enough data (less than 2 points).
pub fn trend_from_ema(ema_series: &Series) -> Result<TrendSignal, PolarsError> {
    if ema_series.len() < 2 {
        return Ok(TrendSignal::NoClearTrend);
    }
    let ema_f64 = ema_series.f64()?;
    let last_ema = ema_f64.get(ema_series.len() - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get last EMA value".into()))?;
    let prev_ema = ema_f64.get(ema_series.len() - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous EMA value".into()))?;

    if last_ema > prev_ema {
        Ok(TrendSignal::Uptrend)
    } else if last_ema < prev_ema {
        Ok(TrendSignal::Downtrend)
    } else {
        Ok(TrendSignal::NoClearTrend)
    }
}

// --- Price-EMA Crossover Strategy ---
/// Generates a signal when the price crosses over or under the EMA.
///
/// # Arguments
/// * `close_prices`: A Polars Series representing the closing prices.
/// * `ema_series`: A Polars Series representing the Exponential Moving Average values.
///
/// # Returns
/// A `Result` containing a `CrossoverSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Bullish`: If the previous price was below or equal to the previous EMA, and the current price is above the current EMA.
/// - `Bearish`: If the previous price was above or equal to the previous EMA, and the current price is below the current EMA.
/// - `NoSignal`: Otherwise, or if not enough data or series lengths mismatch.
pub fn price_ema_crossover_signal(
    close_prices: &Series,
    ema_series: &Series,
) -> Result<CrossoverSignal, PolarsError> {
    let len = close_prices.len();
    if len < 2 || ema_series.len() < 2 || len != ema_series.len() {
        return Ok(CrossoverSignal::NoSignal);
    }

    let prices_f64 = close_prices.f64()?;
    let ema_f64 = ema_series.f64()?;

    let current_price = prices_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current price".into()))?;
    let previous_price = prices_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous price".into()))?;
    let current_ema = ema_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current EMA".into()))?;
    let previous_ema = ema_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous EMA".into()))?;

    if previous_price <= previous_ema && current_price > current_ema {
        Ok(CrossoverSignal::Bullish)
    } else if previous_price >= previous_ema && current_price < current_ema {
        Ok(CrossoverSignal::Bearish)
    } else {
        Ok(CrossoverSignal::NoSignal)
    }
}

// --- EMA-EMA Crossover Strategy ---
/// Generates a signal when a shorter-term EMA crosses over or under a longer-term EMA.
///
/// # Arguments
/// * `short_ema`: A Polars Series for the shorter-term EMA.
/// * `long_ema`: A Polars Series for the longer-term EMA.
///
/// # Returns
/// A `Result` containing a `CrossoverSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Bullish`: If the previous short EMA was below or equal to the previous long EMA, and the current short EMA is above the current long EMA.
/// - `Bearish`: If the previous short EMA was above or equal to the previous long EMA, and the current short EMA is below the current long EMA.
/// - `NoSignal`: Otherwise, or if not enough data or series lengths mismatch.
pub fn ema_ema_crossover_signal(
    short_ema: &Series,
    long_ema: &Series,
) -> Result<CrossoverSignal, PolarsError> {
    let len = short_ema.len();
    if len < 2 || long_ema.len() < 2 || len != long_ema.len() {
        return Ok(CrossoverSignal::NoSignal);
    }

    let short_ema_f64 = short_ema.f64()?;
    let long_ema_f64 = long_ema.f64()?;

    let current_short_ema = short_ema_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current short EMA".into()))?;
    let previous_short_ema = short_ema_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous short EMA".into()))?;
    let current_long_ema = long_ema_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current long EMA".into()))?;
    let previous_long_ema = long_ema_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous long EMA".into()))?;

    if previous_short_ema <= previous_long_ema && current_short_ema > current_long_ema {
        Ok(CrossoverSignal::Bullish)
    } else if previous_short_ema >= previous_long_ema && current_short_ema < current_long_ema {
        Ok(CrossoverSignal::Bearish)
    } else {
        Ok(CrossoverSignal::NoSignal)
    }
}

/// Translates a Price-EMA crossover signal into a Buy/Sell/Hold action.
pub fn price_ema_crossover_action(
    close_prices: &Series,
    ema_series: &Series,
) -> Result<ActionSignal, PolarsError> {
    match price_ema_crossover_signal(close_prices, ema_series)? {
        CrossoverSignal::Bullish => Ok(ActionSignal::Buy),
        CrossoverSignal::Bearish => Ok(ActionSignal::Sell),
        CrossoverSignal::NoSignal => Ok(ActionSignal::Hold),
    }
}

/// Translates an EMA-EMA crossover signal into a Buy/Sell/Hold action.
pub fn ema_ema_crossover_action(
    short_ema: &Series,
    long_ema: &Series,
) -> Result<ActionSignal, PolarsError> {
    match ema_ema_crossover_signal(short_ema, long_ema)? {
        CrossoverSignal::Bullish => Ok(ActionSignal::Buy),
        CrossoverSignal::Bearish => Ok(ActionSignal::Sell),
        CrossoverSignal::NoSignal => Ok(ActionSignal::Hold),
    }
}

// TODO: Add unit tests for these strategy functions. 