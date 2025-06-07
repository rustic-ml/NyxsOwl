// nyxs_owl/src/technical_strategies/trend/vortex_strategy.rs
//! Vortex Indicator (VI) Crossover Strategy using ta-lib-in-rust.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::{DataFrame, Series, NamedFrom, PolarsResult, DataType, Float64Type};
use polars::chunked_array::ChunkedArray;
use ta_lib_in_rust::indicators::trend::calculate_vortex;

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
        return Err(NyxsOwlError::InvalidParameter(
            String::from("Vortex Indicator period must be greater than 0."),
        ));
    }

    for col_name in [high_col, low_col, close_col].iter() {
        if df.column(col_name).is_err() {
            return Err(NyxsOwlError::DataError(format!(
                "Required price column '{}' not found.", col_name
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
    let vortex_result = calculate_vortex(df, high_col, low_col, close_col, period);
    let (vi_plus_series, vi_minus_series) = vortex_result.map_err(|e| {
        NyxsOwlError::StrategyError(format!("Failed to calculate Vortex Indicator: {:?}", e))
    })?;

    let vi_plus_ca: &ChunkedArray<Float64Type> = vi_plus_series.f64()
        .map_err(|_| NyxsOwlError::DataError("VI_Plus series not F64".to_string()))?;
    let vi_minus_ca: &ChunkedArray<Float64Type> = vi_minus_series.f64()
        .map_err(|_| NyxsOwlError::DataError("VI_Minus series not F64".to_string()))?;

    let mut signals = vec![Signal::Hold; data_len];
    let first_valid_idx = period + 1; 

    for i in first_valid_idx.min(data_len-1)..data_len {
        let current_vi_plus_opt = vi_plus_ca.get(i);
        let prev_vi_plus_opt = vi_plus_ca.get(i - 1);
        let current_vi_minus_opt = vi_minus_ca.get(i);
        let prev_vi_minus_opt = vi_minus_ca.get(i - 1);

        if let (
            Some(current_plus), Some(prev_plus), 
            Some(current_minus), Some(prev_minus)
        ) = (
            current_vi_plus_opt, prev_vi_plus_opt, 
            current_vi_minus_opt, prev_vi_minus_opt
        ) {
            if prev_plus <= prev_minus && current_plus > current_minus {
                signals[i] = Signal::Buy;
            }
            else if prev_minus <= prev_plus && current_minus > current_plus {
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

    fn create_vortex_test_df(len: usize) -> PolarsResult<DataFrame> {
        let mut highs: Vec<f64> = Vec::with_capacity(len);
        let mut lows: Vec<f64> = Vec::with_capacity(len);
        let mut closes: Vec<f64> = Vec::with_capacity(len);
        for i in 0..len {
            let i_f64 = i as f64;
            let len_f64 = len as f64;
            let one_third_len = len_f64 / 3.0;
            let two_thirds_len = 2.0 * len_f64 / 3.0;

            let trend_val = if i_f64 < one_third_len {
                i_f64
            } else if i_f64 < two_thirds_len {
                one_third_len - (i_f64 - one_third_len) * 0.5
            } else {
                (one_third_len * 0.5) + (i_f64 - two_thirds_len)
            };
            let base = 50.0 + trend_val * 0.1 + 3.0 * (i_f64 * 0.05).cos();
            highs.push(base + 1.5);
            lows.push(base - 1.5);
            closes.push(base);
        }
        df! {
            "high" => highs,
            "low" => lows,
            "close" => closes
        }
    }

    #[test]
    fn test_vortex_invalid_params() {
        let df = create_vortex_test_df(50).unwrap();
        assert!(vortex_signals(&df, "high", "low", "close", 0).is_err());
    }

    #[test]
    fn test_vortex_insufficient_data() {
        let period = 14;
        let df_too_short = create_vortex_test_df(period).unwrap(); 
        assert!(vortex_signals(&df_too_short, "high", "low", "close", period).is_err());

        let df_just_enough = create_vortex_test_df(period + 1).unwrap();
        assert!(vortex_signals(&df_just_enough, "high", "low", "close", period).is_ok());
    }

    #[test]
    fn test_vortex_missing_columns() {
        let df_no_close = df! { "high" => vec![50.0; 30], "low" => vec![49.0; 30] }.unwrap();
        assert!(vortex_signals(&df_no_close, "high", "low", "close", 14).is_err());
    }

    #[test]
    fn test_vortex_signals_conceptual() {
        let df = create_vortex_test_df(100).unwrap(); 
        let period = 14;

        match vortex_signals(&df, "high", "low", "close", period) {
            Ok(signals) => {
                assert_eq!(signals.len(), df.height());
                
                let has_buy_signal = signals.iter().any(|&s| s == Signal::Buy);
                let has_sell_signal = signals.iter().any(|&s| s == Signal::Sell);
                
                if df.height() > period + 10 { 
                    assert!(has_buy_signal || has_sell_signal, 
                        "Expected Vortex strategy to generate signals with this data. Check calculations or test data.");
                }
            },
            Err(e) => panic!("Vortex signal generation failed: {:?}", e),
        }
    }
} 