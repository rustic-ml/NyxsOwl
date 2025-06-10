// nyxs_owl/src/technical_strategies/oscillators/rsi_strategy.rs
//! Relative Strength Index (RSI) Strategy using ta-lib-in-rust.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::{DataFrame, Series, NamedFrom, PolarsResult, DataType, Float64Type};
use polars::chunked_array::ChunkedArray;
use ta_lib_in_rust::indicators::oscillators::calculate_rsi;

/// Generates trading signals based on the Relative Strength Index (RSI).
///
/// A buy signal is generated when RSI crosses above the oversold threshold from below.
/// A sell signal is generated when RSI crosses below the overbought threshold from above.
///
/// # Arguments
/// * `df` - A Polars DataFrame containing the price data.
/// * `price_column` - The name of the column in `df` that contains the price data (e.g., "close").
/// * `period` - The period for RSI calculation. Must be greater than 0.
/// * `oversold_threshold` - The RSI level below which is considered oversold (e.g., 30.0).
/// * `overbought_threshold` - The RSI level above which is considered overbought (e.g., 70.0).
///
/// # Returns
/// A `Result` containing a `Vec<Signal>` or a `NyxsOwlError`.
pub fn rsi_signals(
    df: &DataFrame,
    price_column: &str,
    period: usize,
    oversold_threshold: f64,
    overbought_threshold: f64,
) -> Result<Vec<Signal>> {
    if period == 0 {
        return Err(NyxsOwlError::InvalidParameter(
            "RSI period must be greater than 0.".to_string(),
        ));
    }
    if oversold_threshold >= overbought_threshold {
        return Err(NyxsOwlError::InvalidParameter(
            "Oversold threshold must be less than overbought threshold.".to_string(),
        ));
    }

    let prices_series = df.column(price_column).map_err(|e| {
        NyxsOwlError::DataError(format!("Price column '{}' not found: {}", price_column, e))
    })?;

    // RSI in ta-lib-in-rust likely requires at least `period` values to produce the first non-null output.
    // For crossover detection (current and previous RSI), we need at least `period + 1` data points in the series.
    if prices_series.len() <= period {
        return Err(NyxsOwlError::MissingData(format!(
            "Price data length ({}) must be greater than the RSI period ({}).",
            prices_series.len(),
            period
        )));
    }

    let rsi_series = calculate_rsi(df, period, price_column).map_err(|e| {
        NyxsOwlError::StrategyError(format!("Failed to calculate RSI: {:?}", e))
    })?;
    
    let rsi_ca: &ChunkedArray<Float64Type> = rsi_series.f64().map_err(|_| NyxsOwlError::StrategyError("RSI Series is not Float64".to_string()))?;

    let data_len = prices_series.len();
    let mut signals = vec![Signal::Hold; data_len];

    // Iteration should start from an index where both current and previous RSI values can be non-null.
    // If `calculate_rsi` with period `p` produces first value at index `p` (0-indexed after initial nulls),
    // then `rsi_ca.get(period)` would be the first value, and `rsi_ca.get(period -1)` would be null.
    // So, to compare `rsi[i]` and `rsi[i-1]`, `i` must start from `period + 1` if the first value is at index `period`.
    // However, ta-lib-in-rust might return a series that is shorter or has a different null padding scheme.
    // Assuming `calculate_rsi` returns a series of the same length as input `df` rows,
    // padded with nulls. The first non-null RSI value is typically at index `period`.
    // So, for `prev_rsi` to be valid, `i-1` must be at least `period`. So `i` must be at least `period + 1`.
    // Let's start loop from `period + 1` assuming first RSI value at index `period`.
    for i in (period + 1)..data_len {
        let current_rsi_opt = rsi_ca.get(i);
        let prev_rsi_opt = rsi_ca.get(i - 1);

        if let (Some(current_rsi), Some(prev_rsi)) = (current_rsi_opt, prev_rsi_opt) {
            // Buy signal: RSI crosses above oversold threshold
            if prev_rsi <= oversold_threshold && current_rsi > oversold_threshold {
                signals[i] = Signal::Buy;
            }
            // Sell signal: RSI crosses below overbought threshold
            else if prev_rsi >= overbought_threshold && current_rsi < overbought_threshold {
                signals[i] = Signal::Sell;
            }
        }
    }
    Ok(signals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::{df, PolarsError};

    fn create_test_df(prices: Vec<f64>) -> PolarsResult<DataFrame> {
        df! {
            "close" => prices
        }
    }

    #[test]
    fn test_rsi_invalid_parameters_df() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0];
        let df = create_test_df(prices).unwrap();
        assert!(rsi_signals(&df, "close", 0, 30.0, 70.0).is_err());
        assert!(rsi_signals(&df, "close", 14, 70.0, 30.0).is_err());
    }

    #[test]
    fn test_rsi_insufficient_data_df() {
        let prices_short = vec![10.0, 11.0, 12.0]; // len 3
        let df_short = create_test_df(prices_short).unwrap();
        // period is 14, df_short.len() (3) <= period (14) -> error
        assert!(rsi_signals(&df_short, "close", 14, 30.0, 70.0).is_err());

        let prices_equal = vec![10.0; 14]; // len 14
        let df_equal = create_test_df(prices_equal).unwrap();
        // period is 14, df_equal.len() (14) <= period (14) -> error
        assert!(rsi_signals(&df_equal, "close", 14, 30.0, 70.0).is_err());
    }
    
    #[test]
    fn test_rsi_column_not_found_df() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0];
        let df = create_test_df(prices).unwrap();
        assert!(rsi_signals(&df, "non_existent", 14, 30.0, 70.0).is_err());
    }

    // To test actual RSI signals, we need a known sequence of prices that produces predictable RSI values
    // and crosses. This is more complex due to RSI calculation details.
    // Example from: https://github.com/TA-Lib/ta-lib-python/blob/master/tests/test_abstract.py 
    // (though this is for TA-Lib, ta-lib-in-rust should be similar for standard cases)
    // RSI(14) for a series of closing prices
    // Prices: [44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28]
    // RSI[13] (first value) should be around 66.91 (approx)
    // RSI[14] should be around 66.91 (approx)

    #[test]
    fn test_rsi_signals_generation_simple_case() {
        // This test case requires `ta-lib-in-rust` to produce predictable RSI values.
        // The exact RSI values depend on the specific smoothing method used (Wilder's, etc.)
        // Let's construct a very simple case that should clearly cross thresholds.
        // prices_up:  strong rise, should push RSI to overbought
        // prices_down: strong fall, should push RSI to oversold

        let prices_for_rsi_calc = (
            (1..=30).map(|i| 50.0 + i as f64 * 0.5). // Rising prices: 50.5, 51.0 ... 65.0
            chain((1..=30).map(|i| 65.0 - i as f64 * 0.5)) // Falling prices: 64.5, 64.0 ... 50.0
        ).collect::<Vec<f64>>();
        
        let df = create_test_df(prices_for_rsi_calc.clone()).unwrap();
        let period = 14;
        let oversold = 30.0;
        let overbought = 70.0;

        match rsi_signals(&df, "close", period, oversold, overbought) {
            Ok(signals) => {
                assert_eq!(signals.len(), prices_for_rsi_calc.len());
                
                let rsi_series = calculate_rsi(&df, period, "close").unwrap();
                // println!("Test RSI Series: {:?}", rsi_series);

                let mut buy_signal_found = false;
                let mut sell_signal_found = false;

                for i in (period + 1)..signals.len() {
                    if signals[i] == Signal::Buy {
                        buy_signal_found = true;
                        // Check RSI value at this point if needed for more specific assertions
                        // let rsi_val = rsi_series.f64().unwrap().get(i).unwrap_or_default();
                        // println!("Buy signal at index {} with RSI {}", i, rsi_val);
                    }
                    if signals[i] == Signal::Sell {
                        sell_signal_found = true;
                        // let rsi_val = rsi_series.f64().unwrap().get(i).unwrap_or_default();
                        // println!("Sell signal at index {} with RSI {}", i, rsi_val);
                    }
                }
                // With 30 rising then 30 falling periods, we expect RSI to cross 70 down, and 30 up.
                assert!(buy_signal_found, "Expected at least one buy signal.");
                assert!(sell_signal_found, "Expected at least one sell signal.");
            },
            Err(e) => panic!("RSI signal generation failed: {:?}", e),
        }
    }
} 