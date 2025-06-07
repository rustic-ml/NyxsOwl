// nyxs_owl/src/technical_strategies/moving_averages/sma_crossover_strategy.rs
//! Simple Moving Average (SMA) Crossover Strategy using local trade_math module.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::{DataFrame, Series, NamedFrom};
use crate::trade_math::moving_averages::calculate_sma;

/// Generates trading signals based on the crossover of two SMAs.
///
/// A buy signal is generated when the short-term SMA crosses above the long-term SMA.
/// A sell signal is generated when the short-term SMA crosses below the long-term SMA.
///
/// # Arguments
/// * `df` - A Polars DataFrame containing the price data.
/// * `price_column` - The name of the column in `df` that contains the price data (e.g., "close").
/// * `short_period` - The period for the short-term SMA. Must be > 0.
/// * `long_period` - The period for the long-term SMA. Must be > short_period.
///
/// # Returns
/// A `Result` containing a `Vec<Signal>` or a `NyxsOwlError`.
pub fn sma_crossover_signals(
    df: &DataFrame,
    price_column: &str,
    short_period: usize,
    long_period: usize,
) -> Result<Vec<Signal>> {
    if short_period == 0 || long_period == 0 {
        return Err(NyxsOwlError::InvalidParameter("SMA periods must be greater than 0.".into()));
    }
    if short_period >= long_period {
        return Err(NyxsOwlError::InvalidParameter(
            "Short-term SMA period must be less than long-term SMA period.".into(),
        ));
    }

    let prices = df.column(price_column).map_err(|e| {
        NyxsOwlError::DataError(format!("Price column '{}' not found: {}", price_column, e))
    })?;
    
    let data_len = prices.len();
    if data_len < long_period {
        return Err(NyxsOwlError::MissingData(format!(
            "Price data length ({}) insufficient for longest SMA period ({}).",
            data_len, long_period
        )));
    }

    // Use the calculate_sma from trade_math module
    let sma_short = calculate_sma(prices, short_period).map_err(|e| {
        NyxsOwlError::StrategyError(format!(
            "Failed to calculate short SMA (period {}): {}", short_period, e
        ))
    })?;
    let sma_long = calculate_sma(prices, long_period).map_err(|e| {
        NyxsOwlError::StrategyError(format!(
            "Failed to calculate long SMA (period {}): {}", long_period, e
        ))
    })?;

    let sma_short_ca = sma_short.f64()
        .map_err(|_| NyxsOwlError::StrategyError("Short SMA Series is not Float64".into()))?;
    let sma_long_ca = sma_long.f64()
        .map_err(|_| NyxsOwlError::StrategyError("Long SMA Series is not Float64".into()))?;

    let mut signals = vec![Signal::Hold; data_len];

    // SMA produces values after its period. Crossover needs previous and current values for both SMAs.
    // First possible signal is at index `long_period` (if 0-indexed data), as we need `sma_long[long_period-1]` and `sma_long[long_period]`.
    let first_signal_idx = long_period; 

    for i in first_signal_idx..data_len {
        if i == 0 { continue; } // Should be covered by first_signal_idx check

        let current_short_opt = sma_short_ca.get(i);
        let current_long_opt = sma_long_ca.get(i);
        let prev_short_opt = sma_short_ca.get(i - 1);
        let prev_long_opt = sma_long_ca.get(i - 1);

        if let (Some(current_short), Some(current_long), Some(prev_short), Some(prev_long)) =
            (current_short_opt, current_long_opt, prev_short_opt, prev_long_opt)
        {
            // Buy signal: Short SMA crosses above Long SMA
            if prev_short <= prev_long && current_short > current_long {
                signals[i] = Signal::Buy;
            }
            // Sell signal: Short SMA crosses below Long SMA
            else if prev_short >= prev_long && current_short < current_long {
                signals[i] = Signal::Sell;
            }
        }
    }
    Ok(signals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::{df, PolarsError, AnyValue}; // Added AnyValue

    fn create_test_df_sma(len: usize, start_price: f64, trend: f64) -> PolarsResult<DataFrame> {
        let prices: Vec<f64> = (0..len)
            .map(|i| start_price + (i as f64 * trend) + (i as f64 * 0.2).sin() * 2.0)
            .collect();
        df! {
            "close" => prices
        }
    }

    #[test]
    fn test_sma_crossover_invalid_periods() {
        let df = create_test_df_sma(50, 100.0, 0.1).unwrap();
        assert!(matches!(sma_crossover_signals(&df, "close", 0, 10), Err(NyxsOwlError::InvalidParameter(_))));
        assert!(matches!(sma_crossover_signals(&df, "close", 10, 0), Err(NyxsOwlError::InvalidParameter(_))));
        assert!(matches!(sma_crossover_signals(&df, "close", 10, 10), Err(NyxsOwlError::InvalidParameter(_))));
        assert!(matches!(sma_crossover_signals(&df, "close", 20, 10), Err(NyxsOwlError::InvalidParameter(_))));
    }

    #[test]
    fn test_sma_crossover_insufficient_data() {
        let short_p = 5;
        let long_p = 10;
        let df_too_short = create_test_df_sma(long_p - 1, 100.0, 0.0).unwrap();
        assert!(matches!(sma_crossover_signals(&df_too_short, "close", short_p, long_p), Err(NyxsOwlError::MissingData(_))));
        
        let df_just_enough = create_test_df_sma(long_p, 100.0, 0.0).unwrap();
        match sma_crossover_signals(&df_just_enough, "close", short_p, long_p) {
            Ok(signals) => {
                assert_eq!(signals.len(), long_p);
                assert!(signals.iter().all(|&s| s == Signal::Hold)); // Loop starts at long_p, no signals generated
            },
            Err(e) => panic!("Expected Ok for data length == long_period, got {:?}", e)
        }
    }

    #[test]
    fn test_sma_crossover_column_not_found() {
        let df = create_test_df_sma(50, 100.0, 0.1).unwrap();
        assert!(matches!(sma_crossover_signals(&df, "non_existent", 5, 10), Err(NyxsOwlError::DataError(_))));
    }

    #[test]
    fn test_sma_crossover_upward_trend_buy_signal() {
        // Data that creates a clear upward trend causing short SMA to cross long SMA
        let prices: Vec<f64> = (0..30)
            .map(|i| 10.0 + i as f64 * 0.5) // Steady rise
            .collect();
        let df = df!{"close" => prices}.unwrap();
        let short_p = 5;
        let long_p = 10;

        match sma_crossover_signals(&df, "close", short_p, long_p) {
            Ok(signals) => {
                // signals[long_p-1] is the first point where both SMAs are non-null
                // signals[long_p] is the first point where a crossover *could* be detected
                let buy_signal_found = signals.iter().skip(long_p).any(|&s| s == Signal::Buy);
                if !buy_signal_found {
                    let price_series = df.column("close").unwrap();
                    let sma_s = calculate_sma(price_series, short_p).unwrap();
                    let sma_l = calculate_sma(price_series, long_p).unwrap();
                    println!("Prices: {:?}", price_series.f64().unwrap().to_vec());
                    println!("SMA Short ({}): {:?}", short_p, sma_s.f64().unwrap().to_vec());
                    println!("SMA Long ({}): {:?}", long_p, sma_l.f64().unwrap().to_vec());
                    println!("Signals: {:?}", signals);
                }
                assert!(buy_signal_found, "Expected a Buy signal in an upward trend.");
            }
            Err(e) => panic!("SMA crossover signal generation failed: {:?}", e),
        }
    }

    #[test]
    fn test_sma_crossover_downward_trend_sell_signal() {
        // Data that creates a clear downward trend
        let prices: Vec<f64> = (0..30)
            .map(|i| 30.0 - i as f64 * 0.5) // Steady fall
            .collect();
        let df = df!{"close" => prices}.unwrap();
        let short_p = 5;
        let long_p = 10;

        match sma_crossover_signals(&df, "close", short_p, long_p) {
            Ok(signals) => {
                let sell_signal_found = signals.iter().skip(long_p).any(|&s| s == Signal::Sell);
                 if !sell_signal_found {
                    let price_series = df.column("close").unwrap();
                    let sma_s = calculate_sma(price_series, short_p).unwrap();
                    let sma_l = calculate_sma(price_series, long_p).unwrap();
                    println!("Prices: {:?}", price_series.f64().unwrap().to_vec());
                    println!("SMA Short ({}): {:?}", short_p, sma_s.f64().unwrap().to_vec());
                    println!("SMA Long ({}): {:?}", long_p, sma_l.f64().unwrap().to_vec());
                    println!("Signals: {:?}", signals);
                }
                assert!(sell_signal_found, "Expected a Sell signal in a downward trend.");
            }
            Err(e) => panic!("SMA crossover signal generation failed: {:?}", e),
        }
    }
} 