// nyxs_owl/src/technical_strategies/momentum/stochastic_strategy.rs
//! Stochastic Oscillator Strategy using local trade_math module.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use crate::trade_math::momentum::calculate_stochastic;
use polars::prelude::*;

/// Configuration for Stochastic strategy parameters
#[derive(Debug, Clone)]
pub struct StochasticConfig {
    /// The column name for high prices.
    pub high_column: String,
    /// The column name for low prices.
    pub low_column: String,
    /// The column name for close prices.
    pub close_column: String,
    /// The period for %K calculation (typically 14).
    pub k_period: usize,
    /// The period for %D smoothing (typically 3).
    pub d_period: usize,
    /// The oversold threshold (typically 20).
    pub oversold_threshold: f64,
    /// The overbought threshold (typically 80).
    pub overbought_threshold: f64,
}

impl Default for StochasticConfig {
    fn default() -> Self {
        Self {
            high_column: "high".to_string(),
            low_column: "low".to_string(),
            close_column: "close".to_string(),
            k_period: 14,
            d_period: 3,
            oversold_threshold: 20.0,
            overbought_threshold: 80.0,
        }
    }
}

/// Generate trading signals based on Stochastic Oscillator crossovers.
///
/// This strategy looks for %K and %D line crossovers in oversold and overbought zones.
/// Buy signals are generated when %K crosses above %D in the oversold zone.
/// Sell signals are generated when %K crosses below %D in the overbought zone.
///
/// # Arguments
/// * `df` - DataFrame containing OHLC data.
/// * `config` - Configuration containing all strategy parameters.
///
/// # Returns
/// A `Result` containing a `Vec<Signal>` or a `NyxsOwlError`.
pub fn stochastic_signals(df: &DataFrame, config: &StochasticConfig) -> Result<Vec<Signal>> {
    // Validate parameters
    if config.k_period == 0 || config.d_period == 0 {
        return Err(NyxsOwlError::InvalidParameter(
            "Stochastic periods must be greater than 0".to_string(),
        ));
    }

    if config.oversold_threshold >= config.overbought_threshold {
        return Err(NyxsOwlError::InvalidParameter(
            "Oversold threshold must be less than overbought threshold".to_string(),
        ));
    }

    if config.oversold_threshold < 0.0 || config.overbought_threshold > 100.0 {
        return Err(NyxsOwlError::InvalidParameter(
            "Stochastic thresholds must be between 0 and 100".to_string(),
        ));
    }

    // Extract OHLC data
    let high_series = df.column(&config.high_column).map_err(|e| {
        NyxsOwlError::DataError(format!(
            "High column '{}' not found: {}",
            config.high_column, e
        ))
    })?;

    let low_series = df.column(&config.low_column).map_err(|e| {
        NyxsOwlError::DataError(format!(
            "Low column '{}' not found: {}",
            config.low_column, e
        ))
    })?;

    let close_series = df.column(&config.close_column).map_err(|e| {
        NyxsOwlError::DataError(format!(
            "Close column '{}' not found: {}",
            config.close_column, e
        ))
    })?;

    let min_data_len = config.k_period + config.d_period;
    if high_series.len() < min_data_len {
        return Err(NyxsOwlError::MissingData(format!(
            "Insufficient data: {} rows, need at least {} for Stochastic strategy",
            high_series.len(),
            min_data_len
        )));
    }

    // Convert Columns to Series for calculation
    let high_column_clone = high_series.clone();
    let high_series_clone = high_column_clone.as_series().ok_or_else(|| {
        NyxsOwlError::DataError("Failed to convert high Column to Series".to_string())
    })?;
    let low_column_clone = low_series.clone();
    let low_series_clone = low_column_clone.as_series().ok_or_else(|| {
        NyxsOwlError::DataError("Failed to convert low Column to Series".to_string())
    })?;
    let close_column_clone = close_series.clone();
    let close_series_clone = close_column_clone.as_series().ok_or_else(|| {
        NyxsOwlError::DataError("Failed to convert close Column to Series".to_string())
    })?;

    // Calculate Stochastic indicators
    let (k_line, d_line) = calculate_stochastic(
        high_series_clone,
        low_series_clone,
        close_series_clone,
        config.k_period,
        config.d_period,
    )
    .map_err(|e| NyxsOwlError::IndicatorError(format!("Stochastic calculation failed: {}", e)))?;

    // Extract values for signal generation
    let k_values: Vec<Option<f64>> = k_line
        .f64()
        .map_err(|e| NyxsOwlError::DataError(format!("Failed to extract %K values: {}", e)))?
        .into_iter()
        .collect();

    let d_values: Vec<Option<f64>> = d_line
        .f64()
        .map_err(|e| NyxsOwlError::DataError(format!("Failed to extract %D values: {}", e)))?
        .into_iter()
        .collect();

    // Generate trading signals
    let mut signals = vec![Signal::Hold; k_values.len()];
    let mut previous_k_above_d: Option<bool> = None;

    for i in 1..k_values.len() {
        if let (Some(current_k), Some(current_d), Some(prev_k), Some(prev_d)) = (
            k_values[i],
            d_values[i],
            k_values.get(i - 1).and_then(|&x| x),
            d_values.get(i - 1).and_then(|&x| x),
        ) {
            let current_k_above = current_k > current_d;
            let _prev_k_above = prev_k > prev_d;

            // Detect crossovers in specific zones
            if let Some(was_above) = previous_k_above_d {
                if !was_above && current_k_above && current_k < config.oversold_threshold {
                    // %K crossed above %D in oversold zone -> Buy signal
                    signals[i] = Signal::Buy;
                } else if was_above && !current_k_above && current_k > config.overbought_threshold {
                    // %K crossed below %D in overbought zone -> Sell signal
                    signals[i] = Signal::Sell;
                }
            }

            previous_k_above_d = Some(current_k_above);
        }
    }

    Ok(signals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::error::PolarsResult;
    use polars::prelude::*;

    fn create_test_df_for_stochastic_strategy(len: usize) -> PolarsResult<DataFrame> {
        let base_price = 50.0;
        let mut high_prices = Vec::new();
        let mut low_prices = Vec::new();
        let mut close_prices = Vec::new();

        for i in 0..len {
            let base = base_price + (i as f64 * 0.1);
            let oscillation = 5.0 * (i as f64 * 0.2).sin();

            let close = base + oscillation;
            let high = close + 1.0 + (i as f64 * 0.05).cos().abs();
            let low = close - 1.0 - (i as f64 * 0.05).sin().abs();

            high_prices.push(high);
            low_prices.push(low);
            close_prices.push(close);
        }

        df! {
            "high" => high_prices,
            "low" => low_prices,
            "close" => close_prices
        }
    }

    #[test]
    fn test_stochastic_strategy_basic_functionality() {
        let df = create_test_df_for_stochastic_strategy(50).unwrap();
        let config = StochasticConfig {
            k_period: 14,
            d_period: 3,
            oversold_threshold: 20.0,
            overbought_threshold: 80.0,
            ..Default::default()
        };
        let result = stochastic_signals(&df, &config);
        assert!(result.is_ok());

        let signals = result.unwrap();
        assert_eq!(signals.len(), 50);

        // Should have some hold signals initially
        assert!(signals.iter().take(17).all(|&s| s == Signal::Hold));
    }

    #[test]
    fn test_stochastic_strategy_invalid_periods() {
        let df = create_test_df_for_stochastic_strategy(50).unwrap();

        // Zero periods
        let config_zero_k = StochasticConfig {
            k_period: 0,
            d_period: 3,
            ..Default::default()
        };
        assert!(matches!(
            stochastic_signals(&df, &config_zero_k),
            Err(NyxsOwlError::InvalidParameter(_))
        ));

        let config_zero_d = StochasticConfig {
            k_period: 14,
            d_period: 0,
            ..Default::default()
        };
        assert!(matches!(
            stochastic_signals(&df, &config_zero_d),
            Err(NyxsOwlError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_stochastic_strategy_invalid_thresholds() {
        let df = create_test_df_for_stochastic_strategy(50).unwrap();

        // Invalid threshold ranges
        let config_invalid_order = StochasticConfig {
            oversold_threshold: 80.0,
            overbought_threshold: 20.0,
            ..Default::default()
        };
        assert!(matches!(
            stochastic_signals(&df, &config_invalid_order),
            Err(NyxsOwlError::InvalidParameter(_))
        ));

        let config_negative = StochasticConfig {
            oversold_threshold: -10.0,
            overbought_threshold: 80.0,
            ..Default::default()
        };
        assert!(matches!(
            stochastic_signals(&df, &config_negative),
            Err(NyxsOwlError::InvalidParameter(_))
        ));

        let config_too_high = StochasticConfig {
            oversold_threshold: 20.0,
            overbought_threshold: 110.0,
            ..Default::default()
        };
        assert!(matches!(
            stochastic_signals(&df, &config_too_high),
            Err(NyxsOwlError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_stochastic_strategy_insufficient_data() {
        let k_period = 14;
        let d_period = 3;
        let required_len = k_period + d_period;

        let df_too_short = create_test_df_for_stochastic_strategy(required_len - 1).unwrap();
        let config = StochasticConfig {
            k_period,
            d_period,
            ..Default::default()
        };
        assert!(matches!(
            stochastic_signals(&df_too_short, &config),
            Err(NyxsOwlError::MissingData(_))
        ));

        let df_just_enough = create_test_df_for_stochastic_strategy(required_len).unwrap();
        let result = stochastic_signals(&df_just_enough, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stochastic_strategy_column_not_found() {
        let df = create_test_df_for_stochastic_strategy(50).unwrap();

        let config_invalid_high = StochasticConfig {
            high_column: "non_existent".to_string(),
            ..Default::default()
        };
        assert!(matches!(
            stochastic_signals(&df, &config_invalid_high),
            Err(NyxsOwlError::DataError(_))
        ));

        let config_invalid_low = StochasticConfig {
            low_column: "non_existent".to_string(),
            ..Default::default()
        };
        assert!(matches!(
            stochastic_signals(&df, &config_invalid_low),
            Err(NyxsOwlError::DataError(_))
        ));

        let config_invalid_close = StochasticConfig {
            close_column: "non_existent".to_string(),
            ..Default::default()
        };
        assert!(matches!(
            stochastic_signals(&df, &config_invalid_close),
            Err(NyxsOwlError::DataError(_))
        ));
    }

    #[test]
    fn test_stochastic_strategy_signals_generation() {
        let df = create_test_df_for_stochastic_strategy(100).unwrap();
        let config = StochasticConfig {
            k_period: 14,
            d_period: 3,
            oversold_threshold: 20.0,
            overbought_threshold: 80.0,
            ..Default::default()
        };
        let result = stochastic_signals(&df, &config);
        assert!(result.is_ok());

        let signals = result.unwrap();
        assert_eq!(signals.len(), df.height());

        // Should have some hold signals
        let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
        assert!(hold_count > 0);

        // Count signal types
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();

        // With oscillating price data, might have some trading signals
        // Note: Stochastic signals are more specific (only in extreme zones), so might have fewer signals
        println!(
            "Buy signals: {}, Sell signals: {}, Hold signals: {}",
            buy_count, sell_count, hold_count
        );
    }
}
