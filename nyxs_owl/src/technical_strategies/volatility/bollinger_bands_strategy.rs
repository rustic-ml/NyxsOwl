// nyxs_owl/src/technical_strategies/volatility/bollinger_bands_strategy.rs
//! Bollinger Bands Strategy using local trade_math module.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use crate::trade_math::volatility::calculate_bollinger_bands;
use polars::chunked_array::ChunkedArray;
use polars::prelude::{DataFrame, DataType, Float64Type, NamedFrom, PolarsResult, Series};

/// Generates trading signals based on Bollinger Bands, typically for mean reversion.
///
/// A buy signal is generated when the price closes back inside the lower band after being below it.
/// A sell signal is generated when the price closes back inside the upper band after being above it.
///
/// # Arguments
/// * `df` - A Polars DataFrame containing the price data.
/// * `price_column` - The name of the column in `df` that contains the close price data (e.g., "close").
/// * `period` - The period for the moving average (middle band) (e.g., 20). Must be > 0.
/// * `std_dev` - The number of standard deviations for the upper and lower bands (e.g., 2.0). Must be > 0.
///
/// # Returns
/// A `Result` containing a `Vec<Signal>` or a `NyxsOwlError`.
pub fn bollinger_bands_signals(
    df: &DataFrame,
    price_column: &str,
    period: usize,
    std_dev: f64,
) -> Result<Vec<Signal>> {
    if period == 0 {
        return Err(NyxsOwlError::InvalidParameter(
            "Bollinger Bands period must be greater than 0.".to_string(),
        ));
    }
    if std_dev <= 0.0 {
        return Err(NyxsOwlError::InvalidParameter(
            "Bollinger Bands standard deviation must be greater than 0.".to_string(),
        ));
    }

    let close_prices_column = df.column(price_column).map_err(|e| {
        NyxsOwlError::DataError(format!("Price column '{}' not found: {}", price_column, e))
    })?;

    // Convert Column to Series for calculation - fix borrowing issue
    let close_prices_column_ref = close_prices_column.clone();
    let close_prices_series_opt = close_prices_column_ref.as_series();
    let close_prices_series = close_prices_series_opt.ok_or_else(|| {
        NyxsOwlError::DataError("Failed to convert Column to Series".to_string())
    })?;

    let data_len = df.height();
    if data_len < period + 1 {
        return Err(NyxsOwlError::MissingData(format!(
            "Price data length ({}) must be at least period + 1 ({}) for Bollinger Bands signal generation.",
            data_len,
            period + 1
        )));
    }

    let (upper_band_series, _middle_band_series, lower_band_series) =
        calculate_bollinger_bands(&close_prices_series, period, std_dev).map_err(|e| {
            NyxsOwlError::StrategyError(format!(
                "Failed to calculate Bollinger Bands using trade_math: {}",
                e
            ))
        })?;

    let close_prices_ca: &ChunkedArray<Float64Type> = close_prices_column
        .f64()
        .map_err(|_| NyxsOwlError::DataError("Close price Series is not Float64".to_string()))?;
    let upper_band_ca: &ChunkedArray<Float64Type> = upper_band_series
        .f64()
        .map_err(|_| NyxsOwlError::StrategyError("Upper_Band Series is not Float64".to_string()))?;
    let lower_band_ca: &ChunkedArray<Float64Type> = lower_band_series
        .f64()
        .map_err(|_| NyxsOwlError::StrategyError("Lower_Band Series is not Float64".to_string()))?;

    let mut signals = vec![Signal::Hold; data_len];

    let first_valid_signal_idx = period;

    for i in first_valid_signal_idx..data_len {
        let current_close_opt = close_prices_ca.get(i);
        let prev_close_opt = close_prices_ca.get(i - 1);
        let current_upper_opt = upper_band_ca.get(i);
        let current_lower_opt = lower_band_ca.get(i);
        let prev_upper_opt = upper_band_ca.get(i - 1);
        let prev_lower_opt = lower_band_ca.get(i - 1);

        if let (
            Some(current_close),
            Some(prev_close),
            Some(current_upper),
            Some(current_lower),
            Some(prev_upper),
            Some(prev_lower),
        ) = (
            current_close_opt,
            prev_close_opt,
            current_upper_opt,
            current_lower_opt,
            prev_upper_opt,
            prev_lower_opt,
        ) {
            if prev_close <= prev_lower && current_close > current_lower {
                signals[i] = Signal::Buy;
            } else if prev_close >= prev_upper && current_close < current_upper {
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

    fn create_bb_test_df(len: usize) -> PolarsResult<DataFrame> {
        let prices: Vec<f64> = (0..len)
            .map(|i| {
                50.0 + 10.0 * ((i as f64 * 0.1).sin()) // Sinusoidal wave for prices
            })
            .collect();
        df! {
            "close" => prices
        }
    }

    #[test]
    fn test_bollinger_bands_invalid_params() {
        let df = create_bb_test_df(50).unwrap();
        assert!(bollinger_bands_signals(&df, "close", 0, 2.0).is_err()); // zero period
        assert!(bollinger_bands_signals(&df, "close", 20, 0.0).is_err()); // zero std_dev
        assert!(bollinger_bands_signals(&df, "close", 20, -1.0).is_err()); // negative std_dev
    }

    #[test]
    fn test_bollinger_bands_insufficient_data() {
        let period = 20;
        let df_too_short = create_bb_test_df(period).unwrap();
        assert!(bollinger_bands_signals(&df_too_short, "close", period, 2.0).is_err());

        let df_just_enough = create_bb_test_df(period + 1).unwrap();
        assert!(bollinger_bands_signals(&df_just_enough, "close", period, 2.0).is_ok());
    }

    #[test]
    fn test_bollinger_bands_missing_column() {
        let df_no_close = df! {"open" => vec![50.0; 30]}.unwrap();
        assert!(bollinger_bands_signals(&df_no_close, "close", 20, 2.0).is_err());
    }

    #[test]
    fn test_bollinger_bands_signals_conceptual() {
        let df = create_bb_test_df(100).unwrap(); // Sufficient data with varying prices
        let period = 20;
        let std_dev = 2.0;

        match bollinger_bands_signals(&df, "close", period, std_dev) {
            Ok(signals) => {
                assert_eq!(signals.len(), df.height());

                let has_buy_signal = signals.iter().any(|&s| s == Signal::Buy);
                let has_sell_signal = signals.iter().any(|&s| s == Signal::Sell);

                if df.height() > period + 5 {
                    // Ensure enough data for signals to form
                    assert!(has_buy_signal || has_sell_signal, "Expected Bollinger Bands to generate signals with this data. Check logic or test data. Signals: {:?}", signals.iter().enumerate().filter(|&(_,s)| *s != Signal::Hold).collect::<Vec<_>>());
                }
            }
            Err(e) => panic!("Bollinger Bands signal generation failed: {:?}", e),
        }
    }
}
