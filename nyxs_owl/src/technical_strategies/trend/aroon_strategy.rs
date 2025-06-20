// nyxs_owl/src/technical_strategies/trend/aroon_strategy.rs
//! Aroon Indicator Strategy using local trade_math module.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use crate::trade_math::trend::calculate_aroon;
use polars::prelude::*;

/// Generates trading signals based on Aroon Up and Aroon Down crossovers.
///
/// A buy signal is generated when Aroon Up crosses above Aroon Down.
/// A sell signal is generated when Aroon Up crosses below Aroon Down.
/// Often, a threshold (e.g., 70 for strong trend, 30 for weak) is also considered,
/// but this basic version focuses on the crossover.
///
/// # Arguments
/// * `df` - A Polars DataFrame containing high and low price data.
/// * `high_column` - The name of the column for high prices.
/// * `low_column` - The name of the column for low prices.
/// * `period` - The lookback period for Aroon calculation (e.g., 14 or 25). Must be > 0.
///
/// # Returns
/// A `Result` containing a `Vec<Signal>` or a `NyxsOwlError`.
pub fn aroon_signals(
    df: &DataFrame,
    high_column: &str,
    low_column: &str,
    period: usize,
) -> Result<Vec<Signal>> {
    if period == 0 {
        return Err(NyxsOwlError::InvalidParameter(
            "Aroon period must be greater than 0.".into(),
        ));
    }

    let high_prices_series = df.column(high_column).map_err(|e| {
        NyxsOwlError::DataError(format!("High column '{}' not found: {}", high_column, e))
    })?;
    let low_prices_series = df.column(low_column).map_err(|e| {
        NyxsOwlError::DataError(format!("Low column '{}' not found: {}", low_column, e))
    })?;

    if high_prices_series.len() != low_prices_series.len() {
        return Err(NyxsOwlError::DataError(
            "High and Low price series must have the same length for Aroon strategy.".into(),
        ));
    }
    let data_len = high_prices_series.len();
    if data_len < period {
        // Not enough data to calculate Aroon for even one value, return all Hold.
        // Or error, consistent with other strategies. calculate_aroon handles this.
        return Err(NyxsOwlError::MissingData(format!(
            "Price data length ({}) insufficient for Aroon calculation (period {}).",
            data_len, period
        )));
    }

    // Use the new calculate_aroon function from trade_math
    let (aroon_up_series, aroon_down_series) = calculate_aroon(
        high_prices_series.as_series().ok_or_else(|| {
            NyxsOwlError::DataError("Failed to convert high column to series".into())
        })?,
        low_prices_series.as_series().ok_or_else(|| {
            NyxsOwlError::DataError("Failed to convert low column to series".into())
        })?,
        period,
    )
    .map_err(|e| {
        NyxsOwlError::StrategyError(format!(
            "Failed to calculate Aroon using trade_math: {}. Period: {}",
            e, period
        ))
    })?;

    let aroon_up_ca = aroon_up_series
        .f64()
        .map_err(|_| NyxsOwlError::StrategyError("Aroon Up series is not Float64".into()))?;
    let aroon_down_ca = aroon_down_series
        .f64()
        .map_err(|_| NyxsOwlError::StrategyError("Aroon Down series is not Float64".into()))?;

    let mut signals = vec![Signal::Hold; data_len];

    // Aroon produces values after `period` lookback. First signal can be at index `period` (if 0-indexed data)
    // or `period-1` if the value at that index is the first non-null. `map_windows` behavior.
    // For crossover, we need current and previous, so start iteration from `period` (or an index where prev is valid).
    // `calculate_aroon` pads with nulls for first `period-1` elements. So first valid data is at index `period-1`.
    // For crossover `i-1`, `i`, first `i` can be `period`.
    let first_signal_idx = period;

    for (i, signal) in signals
        .iter_mut()
        .enumerate()
        .take(data_len)
        .skip(first_signal_idx)
    {
        let aroon_up_val_opt = aroon_up_ca.get(i);
        let aroon_down_val_opt = aroon_down_ca.get(i);

        if let (Some(aroon_up), Some(aroon_down)) = (aroon_up_val_opt, aroon_down_val_opt) {
            // Buy signal: Aroon Up > 70 and Aroon Down < 30
            if aroon_up > 70.0 && aroon_down < 30.0 {
                *signal = Signal::Buy;
            }
            // Sell signal: Aroon Down > 70 and Aroon Up < 30
            else if aroon_down > 70.0 && aroon_up < 30.0 {
                *signal = Signal::Sell;
            }
        }
    }
    Ok(signals)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_df_aroon(len: usize) -> std::result::Result<DataFrame, NyxsOwlError> {
        let highs: Vec<f64> = (0..len)
            .map(|i| 50.0 + (i % 10) as f64 + (i as f64 * 0.1).sin() * 5.0)
            .collect();
        let lows: Vec<f64> = (0..len)
            .map(|i| 40.0 - (i % 5) as f64 + (i as f64 * 0.1).cos() * 5.0)
            .collect();
        df! {
            "high" => highs,
            "low" => lows
        }
        .map_err(NyxsOwlError::PolarsError)
    }

    #[test]
    fn test_aroon_strategy_invalid_period() -> std::result::Result<(), NyxsOwlError> {
        let df = create_test_df_aroon(50)?;
        assert!(matches!(
            aroon_signals(&df, "high", "low", 0),
            Err(NyxsOwlError::InvalidParameter(_))
        ));
        Ok(())
    }

    #[test]
    fn test_aroon_strategy_insufficient_data() -> std::result::Result<(), NyxsOwlError> {
        let period = 14;
        let df_too_short = create_test_df_aroon(period - 1)?;
        assert!(matches!(
            aroon_signals(&df_too_short, "high", "low", period),
            Err(NyxsOwlError::MissingData(_))
        ));

        let df_just_enough_calc = create_test_df_aroon(period)?;
        match aroon_signals(&df_just_enough_calc, "high", "low", period) {
            Ok(signals) => {
                assert_eq!(signals.len(), period);
                assert!(signals.iter().all(|&s| s == Signal::Hold));
            }
            Err(e) => panic!("Expected Ok for data length == period, got {:?}", e),
        }
        Ok(())
    }

    #[test]
    fn test_aroon_strategy_columns_not_found() -> std::result::Result<(), NyxsOwlError> {
        let df = create_test_df_aroon(50)?;
        assert!(matches!(
            aroon_signals(&df, "non_existent", "low", 14),
            Err(NyxsOwlError::DataError(_))
        ));
        assert!(matches!(
            aroon_signals(&df, "high", "non_existent", 14),
            Err(NyxsOwlError::DataError(_))
        ));
        Ok(())
    }

    #[test]
    fn test_aroon_strategy_signals_conceptual() -> std::result::Result<(), NyxsOwlError> {
        let df = create_test_df_aroon(100)?;
        let period = 14;

        match aroon_signals(&df, "high", "low", period) {
            Ok(signals) => {
                assert_eq!(signals.len(), df.height());
                let has_buy_signal = signals.iter().any(|&s| s == Signal::Buy);
                let has_sell_signal = signals.iter().any(|&s| s == Signal::Sell);

                if !(has_buy_signal || has_sell_signal) {
                    let high_s = df.column("high").unwrap();
                    let low_s = df.column("low").unwrap();
                    if let Ok((up, down)) = calculate_aroon(
                        high_s.as_series().unwrap(),
                        low_s.as_series().unwrap(),
                        period,
                    ) {
                        println!("Aroon Up: {:?}", up.f64().unwrap().to_vec());
                        println!("Aroon Down: {:?}", down.f64().unwrap().to_vec());
                        println!("Signals: {:?}", signals);
                    }
                }
                assert!(
                    has_buy_signal || has_sell_signal,
                    "Expected Aroon strategy to generate signals."
                );
            }
            Err(e) => {
                panic!("Aroon strategy signal generation failed: {:?}", e);
            }
        }
        Ok(())
    }
}
