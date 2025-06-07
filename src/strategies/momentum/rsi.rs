use polars::prelude::*;
// Assuming common_signals is general enough. If not, might need a momentum-specific common signals module.
use crate::strategies::moving_averages::common_signals::{ActionSignal, CrossoverSignal};

const RSI_DEFAULT_OVERBOUGHT: f64 = 70.0;
const RSI_DEFAULT_OVERSOLD: f64 = 30.0;
const RSI_DEFAULT_CENTERLINE: f64 = 50.0;

// --- RSI Overbought/Oversold Crossover Strategy ---
/// Generates a signal when RSI crosses back from overbought or oversold zones.
///
/// # Arguments
/// * `rsi_series`: A Polars Series representing the RSI values.
/// * `overbought_level`: The RSI level considered overbought (e.g., 70.0).
/// * `oversold_level`: The RSI level considered oversold (e.g., 30.0).
///
/// # Returns
/// A `Result` containing a `CrossoverSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Bullish`: RSI was at or below `oversold_level` and now crossed above it.
/// - `Bearish`: RSI was at or above `overbought_level` and now crossed below it.
/// - `NoSignal`: Otherwise, or if not enough data (less than 2 points for basic crossover, ideally 3 for clear zone exit).
pub fn rsi_overbought_oversold_crossover_signal(
    rsi_series: &Series,
    overbought_level: f64,
    oversold_level: f64,
) -> Result<CrossoverSignal, PolarsError> {
    let len = rsi_series.len();
    if len < 2 { // Need at least two points to detect a crossover
        return Ok(CrossoverSignal::NoSignal);
    }

    let rsi_f64 = rsi_series.f64()?;
    let current_rsi = rsi_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current RSI value".into()))?;
    let previous_rsi = rsi_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous RSI value".into()))?;

    // Bullish: Was oversold (or at boundary) and crossed up
    if previous_rsi <= oversold_level && current_rsi > oversold_level {
        return Ok(CrossoverSignal::Bullish);
    }

    // Bearish: Was overbought (or at boundary) and crossed down
    if previous_rsi >= overbought_level && current_rsi < overbought_level {
        return Ok(CrossoverSignal::Bearish);
    }
    
    Ok(CrossoverSignal::NoSignal)
}

// --- RSI Centerline Crossover Strategy ---
/// Generates a signal when the RSI crosses over or under the centerline (typically 50).
///
/// # Arguments
/// * `rsi_series`: A Polars Series representing the RSI values.
/// * `centerline`: The RSI centerline value (e.g., 50.0).
///
/// # Returns
/// A `Result` containing a `CrossoverSignal` or a `PolarsError`.
///
/// # Strategy
/// - `Bullish`: RSI crosses above the `centerline` from below or equal.
/// - `Bearish`: RSI crosses below the `centerline` from above or equal.
/// - `NoSignal`: Otherwise, or if not enough data (less than 2 points).
pub fn rsi_centerline_crossover_signal(
    rsi_series: &Series,
    centerline: f64,
) -> Result<CrossoverSignal, PolarsError> {
    let len = rsi_series.len();
    if len < 2 {
        return Ok(CrossoverSignal::NoSignal);
    }

    let rsi_f64 = rsi_series.f64()?;
    let current_rsi = rsi_f64.get(len - 1).ok_or_else(|| PolarsError::ComputeError("Failed to get current RSI value".into()))?;
    let previous_rsi = rsi_f64.get(len - 2).ok_or_else(|| PolarsError::ComputeError("Failed to get previous RSI value".into()))?;

    if previous_rsi <= centerline && current_rsi > centerline {
        Ok(CrossoverSignal::Bullish)
    } else if previous_rsi >= centerline && current_rsi < centerline {
        Ok(CrossoverSignal::Bearish)
    } else {
        Ok(CrossoverSignal::NoSignal)
    }
}

/// Translates an RSI Overbought/Oversold Crossover signal into a Buy/Sell/Hold action.
/// Uses default overbought (70) and oversold (30) levels.
pub fn rsi_overbought_oversold_action(rsi_series: &Series) -> Result<ActionSignal, PolarsError> {
    match rsi_overbought_oversold_crossover_signal(rsi_series, RSI_DEFAULT_OVERBOUGHT, RSI_DEFAULT_OVERSOLD)? {
        CrossoverSignal::Bullish => Ok(ActionSignal::Buy),
        CrossoverSignal::Bearish => Ok(ActionSignal::Sell),
        CrossoverSignal::NoSignal => Ok(ActionSignal::Hold),
    }
}

/// Translates an RSI Centerline Crossover signal into a Buy/Sell/Hold action.
/// Uses default centerline (50).
pub fn rsi_centerline_crossover_action(rsi_series: &Series) -> Result<ActionSignal, PolarsError> {
    match rsi_centerline_crossover_signal(rsi_series, RSI_DEFAULT_CENTERLINE)? {
        CrossoverSignal::Bullish => Ok(ActionSignal::Buy),
        CrossoverSignal::Bearish => Ok(ActionSignal::Sell),
        CrossoverSignal::NoSignal => Ok(ActionSignal::Hold),
    }
}

// TODO: Add unit tests.
// TODO: Implement RSI Divergence strategies (more complex, requires tracking peaks/troughs on price and RSI).
// TODO: Implement RSI Failure Swing strategies (more complex). 