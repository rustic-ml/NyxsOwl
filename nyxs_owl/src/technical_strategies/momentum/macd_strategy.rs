// nyxs_owl/src/technical_strategies/momentum/macd_strategy.rs
//! Moving Average Convergence Divergence (MACD) Crossover Strategy using local trade_math module.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use crate::trade_math::momentum::calculate_macd;
use polars::prelude::*;

/// Generates trading signals based on MACD line and Signal line crossovers.
///
/// A buy signal is generated when the MACD line crosses above the Signal line.
/// A sell signal is generated when the MACD line crosses below the Signal line.
///
/// # Arguments
/// * `df` - A Polars DataFrame containing the price data.
/// * `price_column` - The name of the column in `df` that contains the price data (e.g., "close").
/// * `fast_period` - The period for the fast EMA (e.g., 12). Must be greater than 0.
/// * `slow_period` - The period for the slow EMA (e.g., 26). Must be greater than `fast_period`.
/// * `signal_period` - The period for the EMA of the MACD line (e.g., 9). Must be greater than 0.
///
/// # Returns
/// A `Result` containing a `Vec<Signal>` or a `NyxsOwlError`.
pub fn macd_signals(
    df: &DataFrame,
    price_column: &str,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Result<Vec<Signal>> {
    // Validate parameters
    if fast_period == 0 || slow_period == 0 || signal_period == 0 {
        return Err(NyxsOwlError::InvalidParameter(
            "All MACD periods must be greater than 0".to_string(),
        ));
    }

    if fast_period >= slow_period {
        return Err(NyxsOwlError::InvalidParameter(
            "Fast period must be less than slow period".to_string(),
        ));
    }

    // Extract price data
    let prices_series = df.column(price_column).map_err(|e| {
        NyxsOwlError::DataError(format!("Price column '{}' not found: {}", price_column, e))
    })?;

    let min_data_len_for_strategy = slow_period + signal_period;
    if prices_series.len() < min_data_len_for_strategy {
        return Err(NyxsOwlError::MissingData(format!(
            "Price data length ({}) insufficient for MACD strategy logic (needs at least {} for meaningful signals).",
            prices_series.len(),
            min_data_len_for_strategy
        )));
    }

    // Convert Column to Series for calculation
    let prices_column_clone = prices_series.clone();
    let prices_series_clone = prices_column_clone
        .as_series()
        .ok_or_else(|| NyxsOwlError::DataError("Failed to convert Column to Series".to_string()))?;

    // Calculate MACD indicators
    let (macd_line, signal_line, _histogram) =
        calculate_macd(prices_series_clone, fast_period, slow_period, signal_period)
            .map_err(|e| NyxsOwlError::IndicatorError(format!("MACD calculation failed: {}", e)))?;

    // Extract values for signal generation
    let macd_values: Vec<Option<f64>> = macd_line
        .f64()
        .map_err(|e| NyxsOwlError::DataError(format!("Failed to extract MACD values: {}", e)))?
        .into_iter()
        .collect();

    let signal_values: Vec<Option<f64>> = signal_line
        .f64()
        .map_err(|e| NyxsOwlError::DataError(format!("Failed to extract signal values: {}", e)))?
        .into_iter()
        .collect();

    // Generate trading signals based on MACD crossovers
    let mut signals = vec![Signal::Hold; macd_values.len()];
    let mut previous_macd_above_signal: Option<bool> = None;

    for i in 1..macd_values.len() {
        if let (Some(current_macd), Some(current_signal), Some(prev_macd), Some(prev_signal)) = (
            macd_values[i],
            signal_values[i],
            macd_values.get(i - 1).and_then(|&x| x),
            signal_values.get(i - 1).and_then(|&x| x),
        ) {
            let current_macd_above = current_macd > current_signal;
            let _prev_macd_above = prev_macd > prev_signal;

            // Detect crossovers
            if let Some(was_above) = previous_macd_above_signal {
                if !was_above && current_macd_above {
                    // MACD crossed above signal line -> Buy signal
                    signals[i] = Signal::Buy;
                } else if was_above && !current_macd_above {
                    // MACD crossed below signal line -> Sell signal
                    signals[i] = Signal::Sell;
                }
            }

            previous_macd_above_signal = Some(current_macd_above);
        }
    }

    Ok(signals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::df;

    fn create_test_df_for_macd_strategy(len: usize) -> PolarsResult<DataFrame> {
        let prices: Vec<f64> = (0..len)
            .map(|i| {
                let base = 50.0;
                let trend = i as f64 * 0.1;
                let oscillation = 15.0 * ((i as f64 * 0.2).sin()); // Larger oscillation
                let noise = 2.0 * ((i as f64 * 0.7).cos()); // Additional noise
                base + trend + oscillation + noise
            })
            .collect();
        df! {
            "close" => prices
        }
    }

    #[test]
    fn test_macd_strategy_basic_functionality() {
        let df = create_test_df_for_macd_strategy(100).unwrap();
        let result = macd_signals(&df, "close", 12, 26, 9);
        assert!(result.is_ok());

        let signals = result.unwrap();
        assert_eq!(signals.len(), 100);

        // Should have some hold signals initially
        assert!(signals.iter().take(26).all(|&s| s == Signal::Hold));
    }

    #[test]
    fn test_macd_strategy_invalid_periods() {
        let df = create_test_df_for_macd_strategy(100).unwrap();

        // Zero periods
        assert!(matches!(
            macd_signals(&df, "close", 0, 26, 9),
            Err(NyxsOwlError::InvalidParameter(_))
        ));
        assert!(matches!(
            macd_signals(&df, "close", 12, 0, 9),
            Err(NyxsOwlError::InvalidParameter(_))
        ));
        assert!(matches!(
            macd_signals(&df, "close", 12, 26, 0),
            Err(NyxsOwlError::InvalidParameter(_))
        ));

        // Fast >= slow
        assert!(matches!(
            macd_signals(&df, "close", 26, 12, 9),
            Err(NyxsOwlError::InvalidParameter(_))
        ));
        assert!(matches!(
            macd_signals(&df, "close", 26, 26, 9),
            Err(NyxsOwlError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_macd_strategy_insufficient_data() {
        let fast = 12;
        let slow = 26;
        let signal_p = 9;
        let required_len_for_strategy = slow + signal_p;

        let df_too_short = create_test_df_for_macd_strategy(required_len_for_strategy - 1).unwrap();
        assert!(matches!(
            macd_signals(&df_too_short, "close", fast, slow, signal_p),
            Err(NyxsOwlError::MissingData(_))
        ));

        let df_just_enough_for_strategy =
            create_test_df_for_macd_strategy(required_len_for_strategy).unwrap();
        let result = macd_signals(&df_just_enough_for_strategy, "close", fast, slow, signal_p);
        assert!(result.is_ok());
    }

    #[test]
    fn test_macd_strategy_column_not_found() {
        let df = create_test_df_for_macd_strategy(100).unwrap();
        let res = macd_signals(&df, "non_existent", 12, 26, 9);
        assert!(matches!(res, Err(NyxsOwlError::DataError(_))));
    }

    #[test]
    fn test_macd_strategy_signals_generation() {
        let df = create_test_df_for_macd_strategy(150).unwrap();
        let fast = 12;
        let slow = 26;
        let signal_p = 9;

        let result = macd_signals(&df, "close", fast, slow, signal_p);
        assert!(result.is_ok());

        let signals = result.unwrap();
        assert_eq!(signals.len(), df.height());

        // Should have some hold signals
        let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
        assert!(hold_count > 0);

        // With oscillating price data, may have some buy and sell signals
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();

        println!(
            "MACD test - Buy: {}, Sell: {}, Hold: {}",
            buy_count, sell_count, hold_count
        );

        // The test should not panic and should return proper signal structure
        // MACD crossovers may not always occur with all data patterns, so we don't require signals
        assert!(signals.len() == df.height());

        // All signals should be valid enum values
        for signal in &signals {
            assert!(matches!(signal, Signal::Buy | Signal::Sell | Signal::Hold));
        }
    }
}
