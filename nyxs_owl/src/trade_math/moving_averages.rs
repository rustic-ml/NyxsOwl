use polars::prelude::{
    DataFrame, DataType, NamedFrom, PolarsResult, RollingOptionsFixedWindow, Series, SeriesOpsTime,
};

use polars::error::PolarsError;

/// Calculates the Simple Moving Average (SMA) for a series.
///
/// # Arguments
/// * `series` - The input Series of data (expected to be Float64).
/// * `period` - The lookback period for the SMA.
///
/// # Returns
/// A `PolarsResult<Series>` containing the SMA values.
pub fn calculate_sma(series: &Series, period: usize) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::ComputeError(
            "SMA period must be greater than 0".into(),
        ));
    }
    if series.dtype() != &DataType::Float64 {
        return Err(PolarsError::ComputeError(
            "Input series for SMA must be of type Float64.".into(),
        ));
    }
    if series.len() < period {
        // Not enough data for a full window, return series of nulls
        let nulls: Vec<Option<f64>> = vec![None; series.len()];
        return Ok(Series::new(series.name().clone(), nulls));
    }

    let sma = series.rolling_mean(RollingOptionsFixedWindow {
        window_size: period,
        min_periods: period, // Calculate only when the window is full
        weights: None,
        center: false,
        fn_params: None,
    })?;
    Ok(sma)
}

/// Calculates the Exponential Moving Average (EMA) for a series.
///
/// EMA gives more weight to recent prices, making it more responsive than SMA.
/// The first EMA value is typically the SMA of the first 'period' prices.
/// Subsequent EMA values are calculated as:
/// EMA = (Current Price - Previous EMA) * Multiplier + Previous EMA
/// where Multiplier = Smoothing / (1 + Period). A common Smoothing factor is 2.
///
/// # Arguments
/// * `series` - The input Series of data (expected to be Float64).
/// * `period` - The lookback period for the EMA.
/// * `smoothing` - The smoothing factor, typically 2.0.
///
/// # Returns
/// A `PolarsResult<Series>` containing the EMA values.
pub fn calculate_ema(series: &Series, period: usize, smoothing: f64) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::ComputeError(
            "EMA period must be greater than 0".into(),
        ));
    }
    if smoothing <= 0.0 {
        return Err(PolarsError::ComputeError(
            "EMA smoothing factor must be greater than 0".into(),
        ));
    }
    if series.dtype() != &DataType::Float64 {
        return Err(PolarsError::ComputeError(
            "Input series for EMA must be of type Float64.".into(),
        ));
    }

    let ca = series.f64()?;
    let len = ca.len();

    if len == 0 {
        return Ok(Series::new_empty(series.name().clone(), &DataType::Float64));
    }

    if len < period {
        // Not enough data for even the initial SMA, return series of nulls
        let nulls: Vec<Option<f64>> = vec![None; len];
        return Ok(Series::new(series.name().clone(), nulls));
    }

    let multiplier = smoothing / (1.0 + period as f64);
    let mut ema_values: Vec<Option<f64>> = vec![None; len];

    // Calculate initial SMA for the first EMA value
    // The first 'period' elements of SMA will be null if min_periods = period
    // The first valid SMA is at index period - 1
    let initial_sma_series = calculate_sma(series, period)?;

    if let Some(initial_value) = initial_sma_series.f64()?.get(period - 1) {
        ema_values[period - 1] = Some(initial_value);

        // Calculate subsequent EMA values
        for i in period..len {
            if let Some(current_price) = ca.get(i) {
                if let Some(prev_ema) = ema_values[i - 1] {
                    // previous EMA must be valid
                    ema_values[i] = Some((current_price - prev_ema) * multiplier + prev_ema);
                } else {
                    // This case should ideally not be hit if logic is correct and period >=1
                    // If prev_ema is None but we are past the initial SMA period, it's an issue
                    // For robustness, could attempt to re-seed SMA if a large gap of Nones occurred
                    // but for now, if prev_ema is None, current EMA also becomes None.
                    ema_values[i] = None;
                }
            } else {
                ema_values[i] = None; // If current price is None, EMA is None
            }
        }
    }
    // If initial_sma_series.f64()?.get(period - 1) was None, ema_values remains all None which is correct.

    Ok(Series::new(series.name().clone(), ema_values))
}

/// Calculates the Weighted Moving Average (WMA) for a series.
///
/// WMA assigns more weight to recent data points.
/// For a period `n`, the weights are `n, n-1, ..., 1` for the most recent to oldest data point.
/// WMA = sum(Price[i] * Weight[i]) / sum(Weights)
///
/// # Arguments
/// * `series` - The input Series of data (expected to be Float64).
/// * `period` - The lookback period for the WMA.
///
/// # Returns
/// A `PolarsResult<Series>` containing the WMA values.
pub fn calculate_wma(series: &Series, period: usize) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::ComputeError(
            "WMA period must be greater than 0".into(),
        ));
    }
    if series.dtype() != &DataType::Float64 {
        return Err(PolarsError::ComputeError(
            "Input series for WMA must be of type Float64.".into(),
        ));
    }

    let ca = series.f64()?;
    let len = ca.len();

    if len == 0 {
        return Ok(Series::new_empty(series.name().clone(), &DataType::Float64));
    }

    if len < period {
        let nulls: Vec<Option<f64>> = vec![None; len];
        return Ok(Series::new(series.name().clone(), nulls));
    }

    let sum_of_weights = (period * (period + 1)) as f64 / 2.0;
    let mut wma_values: Vec<Option<f64>> = vec![None; len];

    for i in (period - 1)..len {
        let window = ca.slice(i as i64 - (period as i64 - 1), period);
        let mut weighted_sum = 0.0;
        let mut current_weight = period as f64;
        let mut all_some = true;

        for k in 0..period {
            if let Some(val) = window.get(k) {
                weighted_sum += val * current_weight;
            } else {
                all_some = false;
                break;
            }
            current_weight -= 1.0;
        }

        if all_some {
            wma_values[i] = Some(weighted_sum / sum_of_weights);
        } else {
            wma_values[i] = None; // If any value in window is None, WMA is None
        }
    }

    Ok(Series::new(series.name().clone(), wma_values))
}

/// Calculates the Volume Weighted Average Price (VWAP).
///
/// VWAP is calculated as Cumulative (Typical Price * Volume) / Cumulative Volume.
/// Typical Price = (High + Low + Close) / 3.
/// This implementation calculates VWAP cumulatively over the provided DataFrame.
/// For daily VWAP with resets, this function should be applied to single-day DataFrames.
///
/// # Arguments
/// * `df` - A Polars DataFrame containing "high", "low", "close", and "volume" columns.
///
/// # Returns
/// A `PolarsResult<Series>` containing the VWAP values, named "vwap".
pub fn calculate_vwap(df: &DataFrame) -> PolarsResult<Series> {
    let high = df.column("high")?.f64()?;
    let low = df.column("low")?.f64()?;
    let close = df.column("close")?.f64()?;
    let volume = df.column("volume")?.f64()?;

    if !(high.len() == low.len() && low.len() == close.len() && close.len() == volume.len()) {
        return Err(PolarsError::ShapeMismatch(
            "Input columns for VWAP must have the same length".into(),
        ));
    }

    let high_series = Series::from(high.clone());
    let low_series = Series::from(low.clone());
    let close_series = Series::from(close.clone());
    let temp_sum = (&high_series + &low_series)?;
    let sum_series = (&temp_sum + &close_series)?;
    let mut typical_price_series = &sum_series / 3.0;
    typical_price_series.rename("typical_price".into());

    let volume_series_for_mult = Series::from(volume.clone());
    let tp_times_volume_series = (&typical_price_series * &volume_series_for_mult)?;

    // Manual cumulative sum since cumsum method might not be available
    let tp_vol_ca = tp_times_volume_series.f64()?;
    let mut cumulative_tp_vol_values = Vec::with_capacity(tp_vol_ca.len());
    let mut cum_sum = 0.0;
    for i in 0..tp_vol_ca.len() {
        if let Some(val) = tp_vol_ca.get(i) {
            cum_sum += val;
            cumulative_tp_vol_values.push(Some(cum_sum));
        } else {
            cumulative_tp_vol_values.push(None);
        }
    }
    let cumulative_tp_volume = Series::new("cumulative_tp_volume".into(), cumulative_tp_vol_values);

    let volume_series = Series::from(volume.clone());
    let vol_ca = volume_series.f64()?;
    let mut cumulative_vol_values = Vec::with_capacity(vol_ca.len());
    let mut cum_vol = 0.0;
    for i in 0..vol_ca.len() {
        if let Some(val) = vol_ca.get(i) {
            cum_vol += val;
            cumulative_vol_values.push(Some(cum_vol));
        } else {
            cumulative_vol_values.push(None);
        }
    }
    let cumulative_volume = Series::new("cumulative_volume".into(), cumulative_vol_values);

    // Handle potential division by zero if cumulative_volume is 0 at any point
    // This can happen if volume is 0 for initial rows.
    let vwap_values: Vec<Option<f64>> = cumulative_tp_volume
        .f64()?
        .into_iter()
        .zip(cumulative_volume.f64()?)
        .map(|(ctpv_opt, cv_opt)| match (ctpv_opt, cv_opt) {
            (Some(ctpv), Some(cv)) if cv != 0.0 => Some(ctpv / cv),
            _ => None, // If cumulative volume is 0 or any component is None
        })
        .collect();

    let vwap_series = Series::new("vwap".into(), vwap_values);
    Ok(vwap_series)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::AnyValue;
    use polars::prelude::*;

    fn create_test_series(name: &str, data: Vec<Option<f64>>) -> Series {
        Series::new(name.into(), data)
    }

    #[test]
    fn test_sma_calculation() -> PolarsResult<()> {
        let s = create_test_series(
            "price",
            vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)],
        );
        let sma2 = calculate_sma(&s, 2)?;
        assert_eq!(sma2.get(0).unwrap(), AnyValue::Null);
        assert_eq!(sma2.get(1).unwrap(), AnyValue::Float64(1.5));
        assert_eq!(sma2.get(2).unwrap(), AnyValue::Float64(2.5));
        assert_eq!(sma2.get(3).unwrap(), AnyValue::Float64(3.5));
        assert_eq!(sma2.get(4).unwrap(), AnyValue::Float64(4.5));

        let sma3 = calculate_sma(&s, 3)?;
        assert_eq!(sma3.get(0).unwrap(), AnyValue::Null);
        assert_eq!(sma3.get(1).unwrap(), AnyValue::Null);
        assert_eq!(sma3.get(2).unwrap(), AnyValue::Float64(2.0));
        assert_eq!(sma3.get(3).unwrap(), AnyValue::Float64(3.0));
        assert_eq!(sma3.get(4).unwrap(), AnyValue::Float64(4.0));
        Ok(())
    }

    #[test]
    fn test_sma_empty_series() -> PolarsResult<()> {
        let s = create_test_series("price", vec![]);
        let sma3 = calculate_sma(&s, 3)?;
        assert_eq!(sma3.len(), 0);
        Ok(())
    }

    #[test]
    fn test_sma_insufficient_data() {
        let s = create_test_series("price", vec![Some(1.0), Some(2.0)]);
        let sma3 = calculate_sma(&s, 3).unwrap(); // Expect Ok with nulls
        assert_eq!(sma3.len(), 2);
        assert!(sma3.is_null().all());
    }

    #[test]
    fn test_sma_zero_period() {
        let s = create_test_series("price", vec![Some(1.0), Some(2.0), Some(3.0)]);
        assert!(calculate_sma(&s, 0).is_err());
    }

    #[test]
    fn test_sma_wrong_dtype() {
        let s = Series::new("price_int".into(), vec![1i32, 2, 3]);
        assert!(calculate_sma(&s, 2).is_err());
    }

    #[test]
    fn test_ema_calculation() -> PolarsResult<()> {
        let s = create_test_series(
            "price",
            vec![
                Some(10.0),
                Some(11.0),
                Some(12.0),
                Some(13.0),
                Some(14.0),
                Some(15.0),
            ],
        );
        let period = 3;
        let smoothing = 2.0;

        // Expected values for EMA(3) with smoothing=2
        // SMA(3) for first value: (10+11+12)/3 = 11.0. This is ema_values[2]
        // Multiplier = 2 / (1 + 3) = 0.5
        // EMA[3] = (Price[3] - EMA[2]) * 0.5 + EMA[2] = (13 - 11.0) * 0.5 + 11.0 = 1.0 + 11.0 = 12.0
        // EMA[4] = (Price[4] - EMA[3]) * 0.5 + EMA[3] = (14 - 12.0) * 0.5 + 12.0 = 1.0 + 12.0 = 13.0
        // EMA[5] = (Price[5] - EMA[4]) * 0.5 + EMA[4] = (15 - 13.0) * 0.5 + 13.0 = 1.0 + 13.0 = 14.0

        let ema_series = calculate_ema(&s, period, smoothing)?;

        assert_eq!(ema_series.get(0).unwrap(), AnyValue::Null);
        assert_eq!(ema_series.get(1).unwrap(), AnyValue::Null);
        assert_eq!(
            ema_series.get(2).unwrap().try_extract::<f64>().unwrap(),
            11.0
        );
        assert_eq!(
            ema_series.get(3).unwrap().try_extract::<f64>().unwrap(),
            12.0
        );
        assert_eq!(
            ema_series.get(4).unwrap().try_extract::<f64>().unwrap(),
            13.0
        );
        assert_eq!(
            ema_series.get(5).unwrap().try_extract::<f64>().unwrap(),
            14.0
        );
        Ok(())
    }

    #[test]
    fn test_ema_with_nones_in_data() -> PolarsResult<()> {
        let s = create_test_series(
            "price",
            vec![
                Some(10.0),
                Some(11.0),
                Some(12.0),
                None,
                Some(14.0),
                Some(15.0),
            ],
        );
        let period = 3;
        let smoothing = 2.0;
        // SMA(3) for first value: (10+11+12)/3 = 11.0. EMA[2]
        // Price[3] is None, so EMA[3] should be None
        // Price[4] is 14.0. Prev EMA (EMA[3]) is None, so EMA[4] should be None.
        // Price[5] is 15.0. Prev EMA (EMA[4]) is None, so EMA[5] should be None.
        // This tests cascading Nones.

        let ema_series = calculate_ema(&s, period, smoothing)?;
        assert_eq!(ema_series.get(0).unwrap(), AnyValue::Null);
        assert_eq!(ema_series.get(1).unwrap(), AnyValue::Null);
        assert_eq!(
            ema_series.get(2).unwrap().try_extract::<f64>().unwrap(),
            11.0
        );
        assert_eq!(ema_series.get(3).unwrap(), AnyValue::Null); // Due to None price
        assert_eq!(ema_series.get(4).unwrap(), AnyValue::Null); // Due to previous EMA being None
        assert_eq!(ema_series.get(5).unwrap(), AnyValue::Null); // Due to previous EMA being None
        Ok(())
    }

    #[test]
    fn test_ema_insufficient_data() -> PolarsResult<()> {
        let s = create_test_series("price", vec![Some(1.0), Some(2.0)]);
        let ema3 = calculate_ema(&s, 3, 2.0)?;
        assert_eq!(ema3.len(), 2);
        assert!(ema3.is_null().all());
        Ok(())
    }

    #[test]
    fn test_ema_zero_period() {
        let s = create_test_series("price", vec![Some(1.0)]);
        assert!(calculate_ema(&s, 0, 2.0).is_err());
    }

    #[test]
    fn test_ema_zero_smoothing() {
        let s = create_test_series("price", vec![Some(1.0)]);
        assert!(calculate_ema(&s, 3, 0.0).is_err());
    }

    #[test]
    fn test_ema_wrong_dtype() {
        let s_int = Series::new("price_int".into(), vec![1i32, 2, 3, 4, 5]);
        assert!(calculate_ema(&s_int, 3, 2.0).is_err());
    }

    #[test]
    fn test_ema_len_zero_series() -> PolarsResult<()> {
        let s_empty = create_test_series("price", vec![]);
        let ema_empty = calculate_ema(&s_empty, 3, 2.0)?;
        assert_eq!(ema_empty.len(), 0);
        Ok(())
    }

    #[test]
    fn test_wma_calculation() -> PolarsResult<()> {
        let s = create_test_series(
            "price",
            vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)],
        );
        let period = 3;
        // Weights for period 3: 3, 2, 1. Sum of weights = 6.
        // WMA[2]: (1*1 + 2*2 + 3*3) / 6 = (1+4+9)/6 = 14/6 = 2.333333
        // WMA[3]: (2*1 + 3*2 + 4*3) / 6 = (2+6+12)/6 = 20/6 = 3.333333
        // WMA[4]: (3*1 + 4*2 + 5*3) / 6 = (3+8+15)/6 = 26/6 = 4.333333
        // Polars calculation is from left to right, so weights are applied to oldest to newest if using simple iteration on slice.
        // Correct weights: Price[oldest]*1 + Price[middle]*2 + Price[newest]*3
        // Window for WMA[2] is [1,2,3]. (1*1 + 2*2 + 3*3)/6 = 14/6
        // Window for WMA[3] is [2,3,4]. (2*1 + 3*2 + 4*3)/6 = 20/6
        // Window for WMA[4] is [3,4,5]. (3*1 + 4*2 + 5*3)/6 = 26/6
        // My loop is: current_weight starts at 'period' and applies to window.get(k) where k=0 is oldest in slice.
        // So my code is Price[oldest]*Weight[period] + ... Price[newest]*Weight[1]. That's correct.

        let wma_series = calculate_wma(&s, period)?;
        assert_eq!(wma_series.get(0).unwrap(), AnyValue::Null);
        assert_eq!(wma_series.get(1).unwrap(), AnyValue::Null);
        assert!(
            (wma_series.get(2).unwrap().try_extract::<f64>().unwrap()
                - (1.0 * 3.0 + 2.0 * 2.0 + 3.0 * 1.0) / 6.0)
                .abs()
                < 1e-6,
            "WMA[2] incorrect. Expected {}, got {}",
            (1.0 * 3.0 + 2.0 * 2.0 + 3.0 * 1.0) / 6.0,
            wma_series.get(2).unwrap().try_extract::<f64>().unwrap()
        );
        assert!(
            (wma_series.get(3).unwrap().try_extract::<f64>().unwrap()
                - (2.0 * 3.0 + 3.0 * 2.0 + 4.0 * 1.0) / 6.0)
                .abs()
                < 1e-6
        );
        assert!(
            (wma_series.get(4).unwrap().try_extract::<f64>().unwrap()
                - (3.0 * 3.0 + 4.0 * 2.0 + 5.0 * 1.0) / 6.0)
                .abs()
                < 1e-6
        );
        Ok(())
    }

    #[test]
    fn test_wma_with_nones() -> PolarsResult<()> {
        let s = create_test_series(
            "price",
            vec![Some(1.0), Some(2.0), None, Some(4.0), Some(5.0)],
        );
        let period = 3;
        let wma_series = calculate_wma(&s, period)?;
        assert_eq!(wma_series.get(0).unwrap(), AnyValue::Null);
        assert_eq!(wma_series.get(1).unwrap(), AnyValue::Null);
        assert_eq!(wma_series.get(2).unwrap(), AnyValue::Null); // Window [1,2,None]
        assert_eq!(wma_series.get(3).unwrap(), AnyValue::Null); // Window [2,None,4]
        assert_eq!(wma_series.get(4).unwrap(), AnyValue::Null); // Window [None,4,5]
        Ok(())
    }

    #[test]
    fn test_wma_insufficient_data() -> PolarsResult<()> {
        let s = create_test_series("price", vec![Some(1.0), Some(2.0)]);
        let wma3 = calculate_wma(&s, 3)?;
        assert_eq!(wma3.len(), 2);
        assert!(wma3.is_null().all());
        Ok(())
    }

    #[test]
    fn test_wma_zero_period() {
        let s = create_test_series("price", vec![Some(1.0)]);
        assert!(calculate_wma(&s, 0).is_err());
    }

    #[test]
    fn test_wma_wrong_dtype() {
        let s_int = Series::new("price_int".into(), vec![1i32, 2, 3, 4, 5]);
        assert!(calculate_wma(&s_int, 3).is_err());
    }

    #[test]
    fn test_wma_len_zero_series() -> PolarsResult<()> {
        let s_empty = create_test_series("price", vec![]);
        let wma_empty = calculate_wma(&s_empty, 3)?;
        assert_eq!(wma_empty.len(), 0);
        Ok(())
    }

    #[test]
    fn test_vwap_calculation() -> PolarsResult<()> {
        let df = polars::prelude::df! {
            "high" => &[Some(10.1), Some(10.3), Some(10.2), Some(10.5), Some(10.6)],
            "low" => &[Some(9.9), Some(10.0), Some(10.0), Some(10.2), Some(10.3)],
            "close" => &[Some(10.0), Some(10.2), Some(10.1), Some(10.4), Some(10.5)],
            "volume" => &[Some(100.0), Some(150.0), Some(120.0), Some(200.0), Some(180.0)]
        }?;

        // Expected calculations:
        // Period 0: TP = (10.1+9.9+10.0)/3 = 10.0. TP_Vol = 1000. CumTPVol=1000. CumVol=100. VWAP=10.0
        // Period 1: TP = (10.3+10.0+10.2)/3 = 10.166667. TP_Vol = 1525. CumTPVol=2525. CumVol=250. VWAP=10.1
        // Period 2: TP = (10.2+10.0+10.1)/3 = 10.1. TP_Vol = 1212. CumTPVol=3737. CumVol=370. VWAP=10.1
        // Period 3: TP = (10.5+10.2+10.4)/3 = 10.366667. TP_Vol = 2073.3334. CumTPVol=5810.3334. CumVol=570. VWAP=10.193567
        // Period 4: TP = (10.6+10.3+10.5)/3 = 10.466667. TP_Vol = 1884.00006. CumTPVol=7694.33346. CumVol=750. VWAP=10.259111

        let vwap_series = calculate_vwap(&df)?;
        assert_eq!(vwap_series.name(), "vwap");
        let vwap_ca = vwap_series.f64()?;

        assert!((vwap_ca.get(0).unwrap() - 10.0).abs() < 1e-6);
        assert!((vwap_ca.get(1).unwrap() - 10.1).abs() < 1e-6);
        assert!((vwap_ca.get(2).unwrap() - 10.1).abs() < 1e-6);
        assert!((vwap_ca.get(3).unwrap() - 10.193567).abs() < 1e-6);
        assert!((vwap_ca.get(4).unwrap() - 10.259111).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_vwap_with_zero_volume() -> PolarsResult<()> {
        let df = polars::prelude::df! {
            "high" => &[Some(10.1), Some(10.3), Some(10.2)],
            "low" => &[Some(9.9), Some(10.0), Some(10.0)],
            "close" => &[Some(10.0), Some(10.2), Some(10.1)],
            "volume" => &[Some(0.0), Some(150.0), Some(120.0)] // First volume is zero
        }?;
        let vwap_series = calculate_vwap(&df)?;
        assert_eq!(vwap_series.get(0).unwrap(), AnyValue::Null); // VWAP is None if CumVol is 0
                                                                 // P1: TP1=(10.3+10.0+10.2)/3 = 10.166667. TPVol1=1525. CumVol=150. CumTPVol=1525(since P0 had 0 vol). VWAP = 10.166667
        assert!(
            (vwap_series.get(1).unwrap().try_extract::<f64>().unwrap() - 10.166666666666666).abs()
                < 1e-6
        );
        Ok(())
    }

    #[test]
    fn test_vwap_with_all_zero_volume() -> PolarsResult<()> {
        let df = polars::prelude::df! {
            "high" => &[Some(10.1), Some(10.3)],
            "low" => &[Some(9.9), Some(10.0)],
            "close" => &[Some(10.0), Some(10.2)],
            "volume" => &[Some(0.0), Some(0.0)]
        }?;
        let vwap_series = calculate_vwap(&df)?;
        assert!(vwap_series.is_null().all());
        Ok(())
    }

    #[test]
    fn test_vwap_missing_columns() {
        let df_no_high = polars::prelude::df! {
            "low" => &[Some(9.9)], "close" => &[Some(10.0)], "volume" => &[Some(100.0)]
        }
        .unwrap();
        assert!(calculate_vwap(&df_no_high).is_err());
    }

    #[test]
    fn test_vwap_empty_df() -> PolarsResult<()> {
        let df_empty = polars::prelude::df! {
            "high" => Vec::<Option<f64>>::new(),
            "low" => Vec::<Option<f64>>::new(),
            "close" => Vec::<Option<f64>>::new(),
            "volume" => Vec::<Option<f64>>::new()
        }
        .unwrap();
        let vwap_series = calculate_vwap(&df_empty)?;
        assert_eq!(vwap_series.len(), 0);
        Ok(())
    }

    #[test]
    fn test_vwap_shape_mismatch() {
        // Test that polars itself catches the shape mismatch during DataFrame creation
        let df_result = polars::prelude::df! {
            "high" => &[Some(10.1), Some(10.3)],
            "low" => &[Some(9.9)], // Different length
            "close" => &[Some(10.0), Some(10.2)],
            "volume" => &[Some(100.0), Some(150.0)]
        };
        assert!(df_result.is_err()); // DataFrame creation should fail with mismatched column heights
    }
}
