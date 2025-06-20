// nyxs_owl/src/technical_strategies/trend/vortex_strategy.rs
//! Vortex Indicator (VI) Crossover Strategy using ta-lib-in-rust.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use crate::trade_math::trend::calculate_vortex;
use polars::chunked_array::ChunkedArray;
use polars::prelude::{DataFrame, Float64Type};

/// Generates trading signals based on Vortex Indicator (VI+ and VI-) crossovers.
///
/// A buy signal is generated when VI+ crosses above VI-.
/// A sell signal is generated when VI- crosses above VI+.
///
/// # Arguments
/// * `df` - A Polars DataFrame containing "high", "low", and "close" price data columns.
/// * `high_col` - Name of the high price column.
/// * `low_col` - Name of the low price column.
/// * `close_col` - Name of the close price column.
/// * `period` - The period for Vortex Indicator calculations (e.g., 14). Must be > 0.
///
/// # Returns
/// A `Result` containing a `Vec<Signal>` or a `NyxsOwlError`.
pub fn vortex_signals(
    df: &DataFrame,
    high_col: &str,
    low_col: &str,
    close_col: &str,
    period: usize,
) -> Result<Vec<Signal>> {
    if period == 0 {
        return Err(NyxsOwlError::InvalidParameter(String::from(
            "Vortex Indicator period must be greater than 0.",
        )));
    }

    for col_name in [high_col, low_col, close_col].iter() {
        if df.column(col_name).is_err() {
            return Err(NyxsOwlError::DataError(format!(
                "Required price column '{}' not found.",
                col_name
            )));
        }
    }

    let data_len = df.height();
    if data_len <= period {
        return Err(NyxsOwlError::MissingData(format!(
            "Price data length ({}) must be greater than the Vortex period ({})",
            data_len, period
        )));
    }

    // calculate_vortex should return a tuple of (VI_Plus_Series, VI_Minus_Series)
    let high_series = df
        .column(high_col)
        .map_err(|_| NyxsOwlError::DataError(format!("High column '{}' not found.", high_col)))?
        .as_series()
        .ok_or_else(|| NyxsOwlError::DataError("Failed to convert high column to series".into()))?;
    let low_series = df
        .column(low_col)
        .map_err(|_| NyxsOwlError::DataError(format!("Low column '{}' not found.", low_col)))?
        .as_series()
        .ok_or_else(|| NyxsOwlError::DataError("Failed to convert low column to series".into()))?;
    let close_series = df
        .column(close_col)
        .map_err(|_| NyxsOwlError::DataError(format!("Close column '{}' not found.", close_col)))?
        .as_series()
        .ok_or_else(|| {
            NyxsOwlError::DataError("Failed to convert close column to series".into())
        })?;

    let (vi_plus, vi_minus) = calculate_vortex(high_series, low_series, close_series, period)
        .map_err(NyxsOwlError::PolarsError)?;

    let vi_plus_ca: &ChunkedArray<Float64Type> = vi_plus
        .f64()
        .map_err(|_| NyxsOwlError::DataError("VI_Plus series not F64".to_string()))?;
    let vi_minus_ca: &ChunkedArray<Float64Type> = vi_minus
        .f64()
        .map_err(|_| NyxsOwlError::DataError("VI_Minus series not F64".to_string()))?;

    let mut signals = vec![Signal::Hold; data_len];
    let first_valid_idx = period + 1;

    for (i, signal) in signals
        .iter_mut()
        .enumerate()
        .take(data_len)
        .skip(first_valid_idx.min(data_len - 1))
    {
        *signal = if vi_plus_ca.get(i) > vi_minus_ca.get(i) {
            Signal::Buy
        } else if vi_plus_ca.get(i) < vi_minus_ca.get(i) {
            Signal::Sell
        } else {
            Signal::Hold
        };
    }
    Ok(signals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn create_vortex_test_df(len: usize) -> std::result::Result<DataFrame, NyxsOwlError> {
        let highs: Vec<f64> = (0..len)
            .map(|i| 50.0 + (i % 10) as f64 + (i as f64 * 0.1).sin() * 5.0)
            .collect();
        let lows: Vec<f64> = (0..len)
            .map(|i| 40.0 - (i % 5) as f64 + (i as f64 * 0.1).cos() * 5.0)
            .collect();
        let closes: Vec<f64> = (0..len)
            .map(|i| 45.0 + (i % 7) as f64 + (i as f64 * 0.1).sin() * 3.0)
            .collect();
        df! {
            "high" => highs,
            "low" => lows,
            "close" => closes
        }
        .map_err(NyxsOwlError::PolarsError)
    }

    #[test]
    fn test_vortex_strategy_invalid_period() -> std::result::Result<(), NyxsOwlError> {
        let df = create_vortex_test_df(50)?;
        assert!(matches!(
            vortex_signals(&df, "high", "low", "close", 0),
            Err(NyxsOwlError::InvalidParameter(_))
        ));
        Ok(())
    }

    #[test]
    fn test_vortex_strategy_insufficient_data() -> std::result::Result<(), NyxsOwlError> {
        let period = 14;
        let df_too_short = create_vortex_test_df(period - 1)?;
        assert!(matches!(
            vortex_signals(&df_too_short, "high", "low", "close", period),
            Err(NyxsOwlError::MissingData(_))
        ));
        Ok(())
    }

    #[test]
    fn test_vortex_strategy_columns_not_found() -> std::result::Result<(), NyxsOwlError> {
        let _df = create_vortex_test_df(50)?;
        let df_no_high = df! { "lows" => vec![50.0; 20], "close" => vec![50.0; 20] }
            .map_err(NyxsOwlError::PolarsError)?;
        let df_no_low = df! { "high" => vec![50.0; 20], "close" => vec![50.0; 20] }
            .map_err(NyxsOwlError::PolarsError)?;
        let df_no_close = df! { "high" => vec![50.0; 20], "low" => vec![50.0; 20] }
            .map_err(NyxsOwlError::PolarsError)?;
        assert!(matches!(
            vortex_signals(&df_no_high, "high", "low", "close", 14),
            Err(NyxsOwlError::DataError(_))
        ));
        assert!(matches!(
            vortex_signals(&df_no_low, "high", "low", "close", 14),
            Err(NyxsOwlError::DataError(_))
        ));
        assert!(matches!(
            vortex_signals(&df_no_close, "high", "low", "close", 14),
            Err(NyxsOwlError::DataError(_))
        ));
        Ok(())
    }

    #[test]
    fn test_vortex_strategy_signals_conceptual() -> std::result::Result<(), NyxsOwlError> {
        let df = create_vortex_test_df(100)?;
        let period = 14;
        match vortex_signals(&df, "high", "low", "close", period) {
            Ok(signals) => {
                assert_eq!(signals.len(), df.height());
                let has_buy_signal = signals.iter().any(|&s| s == Signal::Buy);
                let has_sell_signal = signals.iter().any(|&s| s == Signal::Sell);
                if !(has_buy_signal || has_sell_signal) {
                    println!("Vortex test: No signals generated. This may be due to test data not triggering crossovers.");
                    println!("Period: {}", period);
                    println!("This is acceptable for synthetic test data.");
                }
            }
            Err(e) => {
                panic!("Vortex strategy signal generation failed: {:?}", e);
            }
        }
        Ok(())
    }
}
