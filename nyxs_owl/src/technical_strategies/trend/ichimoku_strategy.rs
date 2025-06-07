// nyxs_owl/src/technical_strategies/trend/ichimoku_strategy.rs
//! Ichimoku Cloud Strategy using ta-lib-in-rust.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::{DataFrame, Series, NamedFrom, PolarsResult, DataType, Float64Type};
use polars::chunked_array::ChunkedArray;
use ta_lib_in_rust::indicators::trend::calculate_ichimoku_cloud;

/// Generates trading signals based on Ichimoku Cloud components:
/// Tenkan-sen/Kijun-sen crossover with Kumo (Cloud) confirmation.
///
/// A buy signal is generated if:
/// 1. Tenkan-sen crosses above Kijun-sen.
/// 2. The crossover happens above the Kumo.
/// 3. The price is currently above the Kumo.
///
/// A sell signal is generated if:
/// 1. Tenkan-sen crosses below Kijun-sen.
/// 2. The crossover happens below the Kumo.
/// 3. The price is currently below the Kumo.
///
/// # Arguments
/// * `df` - A Polars DataFrame with "high", "low", and "close" price data.
/// * `high_col` - Name of the high price column.
/// * `low_col` - Name of the low price column.
/// * `close_col` - Name of the close price column.
/// * `tenkan_period` - Period for Tenkan-sen (e.g., 9). Must be > 0.
/// * `kijun_period` - Period for Kijun-sen (e.g., 26). Must be > 0.
/// * `senkou_b_period` - Period for Senkou Span B (e.g., 52). Must be > 0.
///
/// # Returns
/// A `Result` containing a `Vec<Signal>` or a `NyxsOwlError`.
#[allow(clippy::too_many_arguments)]
pub fn ichimoku_kumo_breakout_signals(
    df: &DataFrame,
    high_col: &str,
    low_col: &str,
    close_col: &str,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
) -> Result<Vec<Signal>> {
    if tenkan_period == 0 || kijun_period == 0 || senkou_b_period == 0 {
        return Err(NyxsOwlError::InvalidParameter(
            "Ichimoku periods (tenkan, kijun, senkou_b) must be greater than 0.".to_string(),
        ));
    }

    // Ensure required columns exist
    for col_name in [high_col, low_col, close_col].iter() {
        if df.column(col_name).is_err() {
            return Err(NyxsOwlError::DataError(format!(
                "Required price column '{}' not found.", col_name
            )));
        }
    }
    let close_prices_series = df.column(close_col)?.clone(); // Used for price vs Kumo check

    let data_len = df.height();
    // Ichimoku needs data for the longest period (senkou_b_period) and displacement (kijun_period for Senkou Spans)
    let min_data_needed = senkou_b_period.max(kijun_period) + kijun_period; 
    if data_len <= min_data_needed {
        return Err(NyxsOwlError::MissingData(format!(
            "Price data length ({}) insufficient for Ichimoku Cloud ({}, {}, {}). Needs > ~{}.",
            data_len, tenkan_period, kijun_period, senkou_b_period, min_data_needed
        )));
    }

    // calculate_ichimoku_cloud returns a tuple of 5 Series: 
    // (Tenkan, Kijun, SenkouA, SenkouB, Chikou)
    let (tenkan_sen_series, kijun_sen_series, senkou_span_a_series, senkou_span_b_series, _chikou_span_series) = 
        calculate_ichimoku_cloud(df, high_col, low_col, close_col, tenkan_period, kijun_period, senkou_b_period)
        .map_err(|e| NyxsOwlError::StrategyError(format!("Failed to calculate Ichimoku Cloud: {:?}", e)))?;

    let tenkan_ca: &ChunkedArray<Float64Type> = tenkan_sen_series.f64().map_err(|_| NyxsOwlError::StrategyError("Tenkan-sen Series is not Float64".to_string()))?;
    let kijun_ca: &ChunkedArray<Float64Type> = kijun_sen_series.f64().map_err(|_| NyxsOwlError::StrategyError("Kijun-sen Series is not Float64".to_string()))?;
    let senkou_a_ca: &ChunkedArray<Float64Type> = senkou_span_a_series.f64().map_err(|_| NyxsOwlError::StrategyError("Senkou Span A Series is not Float64".to_string()))?;
    let senkou_b_ca: &ChunkedArray<Float64Type> = senkou_span_b_series.f64().map_err(|_| NyxsOwlError::StrategyError("Senkou Span B Series is not Float64".to_string()))?;
    let close_prices_ca: &ChunkedArray<Float64Type> = close_prices_series.f64().map_err(|_| NyxsOwlError::DataError("Close price Series for Ichimoku is not Float64".to_string()))?;

    let mut signals = vec![Signal::Hold; data_len];

    // Determine earliest valid index. Senkou spans are displaced by kijun_period.
    // Actual data for Senkou A/B starts effectively after `kijun_period` from calculation start.
    // Max of all periods involved in calculation + displacement (kijun_period for Senkou A/B)
    let first_valid_idx = senkou_b_period.max(kijun_period) + kijun_period -1; 
    // Ensure we have i-1 for prev values, and Senkou spans are valid (displaced)

    for i in first_valid_idx.min(data_len-1)..data_len {
        if i == 0 { continue; }

        let current_tenkan_opt = tenkan_ca.get(i);
        let prev_tenkan_opt = tenkan_ca.get(i - 1);
        let current_kijun_opt = kijun_ca.get(i);
        let prev_kijun_opt = kijun_ca.get(i - 1);
        
        // Senkou Spans A and B define the Kumo (Cloud).
        // These are plotted `kijun_period` ahead. So for current price at `i`,
        // we should compare with Senkou values at `i` (which were calculated based on past data and projected forward).
        let current_senkou_a_opt = senkou_a_ca.get(i); 
        let current_senkou_b_opt = senkou_b_ca.get(i);
        let current_close_opt = close_prices_ca.get(i);

        if let (Some(cur_tenkan), Some(prev_tenkan), Some(cur_kijun), Some(prev_kijun),
                Some(cur_senkou_a), Some(cur_senkou_b), Some(cur_close)) =
            (current_tenkan_opt, prev_tenkan_opt, current_kijun_opt, prev_kijun_opt, 
             current_senkou_a_opt, current_senkou_b_opt, current_close_opt)
        {
            let kumo_top = cur_senkou_a.max(cur_senkou_b);
            let kumo_bottom = cur_senkou_a.min(cur_senkou_b);

            // Bullish Crossover: Tenkan crosses above Kijun
            if prev_tenkan <= prev_kijun && cur_tenkan > cur_kijun {
                // Confirm crossover is above Kumo and price is above Kumo
                if cur_kijun > kumo_top && cur_close > kumo_top { // Crossover point (cur_kijun or cur_tenkan) is above kumo top
                    signals[i] = Signal::Buy;
                }
            }
            // Bearish Crossover: Tenkan crosses below Kijun
            else if prev_tenkan >= prev_kijun && cur_tenkan < cur_kijun {
                // Confirm crossover is below Kumo and price is below Kumo
                if cur_kijun < kumo_bottom && cur_close < kumo_bottom { // Crossover point is below kumo bottom
                    signals[i] = Signal::Sell;
                }
            }
        }
    }
    Ok(signals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::{df, PolarsError, AnyValue};

    // Helper to create a DataFrame with somewhat realistic HLC prices
    fn create_ichimoku_test_df(len: usize) -> PolarsResult<DataFrame> {
        let mut highs: Vec<f64> = Vec::with_capacity(len);
        let mut lows: Vec<f64> = Vec::with_capacity(len);
        let mut closes: Vec<f64> = Vec::with_capacity(len);
        for i in 0..len {
            let base = 100.0 + (i as f64 * 0.2).sin() * 10.0 + (i as f64 * 0.05) ; // Sinusoidal + slight uptrend
            highs.push(base + 2.0 + (i % 3) as f64);
            lows.push(base - 2.0 - (i % 3) as f64);
            closes.push(base + ((i % 5) - 2) as f64); // Add some noise to close
        }
        df! {
            "high" => highs,
            "low" => lows,
            "close" => closes
        }
    }

    #[test]
    fn test_ichimoku_invalid_periods() {
        let df = create_ichimoku_test_df(200).unwrap(); // Needs substantial data
        assert!(ichimoku_kumo_breakout_signals(&df, "high", "low", "close", 0, 26, 52).is_err());
        assert!(ichimoku_kumo_breakout_signals(&df, "high", "low", "close", 9, 0, 52).is_err());
        assert!(ichimoku_kumo_breakout_signals(&df, "high", "low", "close", 9, 26, 0).is_err());
    }

    #[test]
    fn test_ichimoku_insufficient_data() {
        let t = 9; let k = 26; let s_b = 52;
        let required_len = s_b.max(k) + k; // from function logic: senkou_b_period.max(kijun_period) + kijun_period;
        
        let df_too_short = create_ichimoku_test_df(required_len).unwrap(); // length == required
        assert!(ichimoku_kumo_breakout_signals(&df_too_short, "high", "low", "close", t, k, s_b).is_err());

        let df_ok = create_ichimoku_test_df(required_len + 1).unwrap();
        assert!(ichimoku_kumo_breakout_signals(&df_ok, "high", "low", "close", t, k, s_b).is_ok());
    }

    #[test]
    fn test_ichimoku_missing_columns() {
        let df_no_high = df! { "low" => vec![50.0; 100], "close" => vec![51.0; 100] }.unwrap();
        assert!(ichimoku_kumo_breakout_signals(&df_no_high, "high", "low", "close", 9, 26, 52).is_err());
    }

    #[test]
    fn test_ichimoku_signals_conceptual() {
        let df = create_ichimoku_test_df(250).unwrap(); // Ensure ample data
        let tenkan_p = 9;
        let kijun_p = 26;
        let senkou_b_p = 52;

        match ichimoku_kumo_breakout_signals(&df, "high", "low", "close", tenkan_p, kijun_p, senkou_b_p) {
            Ok(signals) => {
                assert_eq!(signals.len(), df.height());
                let has_buy_signal = signals.iter().any(|&s| s == Signal::Buy);
                let has_sell_signal = signals.iter().any(|&s| s == Signal::Sell);
                
                // Conceptual: with enough varied data, some signals should appear.
                // Exact signals depend on ta-lib-in-rust's Ichimoku calculation details (especially displacement)
                // and the strictness of the Kumo confirmation.
                // println!("Ichimoku Signals: {:?}", signals.iter().enumerate().filter(|&(_,s)| *s != Signal::Hold).collect::<Vec<_>>());
                // if let Ok((t, k, sa, sb, cs)) = calculate_ichimoku_cloud(&df, "high", "low", "close", tenkan_p, kijun_p, senkou_b_p) {
                //     let display_len = signals.len() - (senkou_b_p.max(kijun_p) + kijun_p - 5).min(signals.len());
                //     println!("Tenkan: {:?}", t.tail(Some(display_len)));
                //     println!("Kijun: {:?}", k.tail(Some(display_len)));
                //     println!("Senkou A: {:?}", sa.tail(Some(display_len)));
                //     println!("Senkou B: {:?}", sb.tail(Some(display_len)));
                //     println!("Close: {:?}", df.column("close").unwrap().tail(Some(display_len)));
                // }

                if df.height() > senkou_b_p.max(kijun_p) + kijun_p + 20 { // Check only if very ample data
                    assert!(has_buy_signal || has_sell_signal, 
                        "Expected Ichimoku to generate some signals with this dataset. Current Kumo confirmation is strict.");
                }
            },
            Err(e) => {
                // println!("Test DF for Ichimoku: {:?}", df.head(None));
                panic!("Ichimoku signal generation failed: {:?}", e);
            }
        }
    }
} 