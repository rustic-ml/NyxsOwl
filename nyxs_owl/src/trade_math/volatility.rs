use crate::trade_math::moving_averages::calculate_sma;
use polars::error::PolarsError;
use polars::prelude::{
    DataFrame, DataType, NamedFrom, PolarsResult, RollingOptionsFixedWindow, Series, SeriesOpsTime,
};

/// Calculates the standard deviation for a series over a rolling window.
fn calculate_rolling_std_dev(series: &Series, period: usize) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::ComputeError(
            "Rolling std dev period must be greater than 0".into(),
        ));
    }
    if series.dtype() != &DataType::Float64 {
        return Err(PolarsError::ComputeError(
            "Input series for rolling_std_dev must be of type Float64.".into(),
        ));
    }
    if series.len() < period {
        let nulls: Vec<Option<f64>> = vec![None; series.len()];
        return Ok(Series::new(series.name().clone(), nulls));
    }

    // Use the Series rolling_std method with RollingOptionsFixedWindow
    let std_dev = series.rolling_std(RollingOptionsFixedWindow {
        window_size: period,
        min_periods: period,
        weights: None,
        center: false,
        fn_params: None,
    })?;
    Ok(std_dev)
}

/// Calculates Bollinger Bands.
///
/// # Arguments
/// * `prices` - A Series of price data.
/// * `period` - The period for the moving average (middle band).
/// * `std_dev_mult` - The number of standard deviations for the upper and lower bands.
///
/// # Returns
/// A `PolarsResult` containing a tuple of three `Series`: (Upper Band, Middle Band, Lower Band).
pub fn calculate_bollinger_bands(
    prices: &Series,
    period: usize,
    std_dev_mult: f64,
) -> PolarsResult<(Series, Series, Series)> {
    if period == 0 {
        return Err(PolarsError::ComputeError(
            "Bollinger Bands period must be greater than 0.".into(),
        ));
    }
    if std_dev_mult <= 0.0 {
        return Err(PolarsError::ComputeError(
            "Bollinger Bands standard deviation multiplier must be greater than 0.".into(),
        ));
    }
    if prices.dtype() != &DataType::Float64 {
        return Err(PolarsError::ComputeError(
            "Price series for Bollinger Bands must be of type Float64.".into(),
        ));
    }
    if prices.len() < period {
        let s_name = prices.name().clone();
        let null_series = Series::new_null(s_name, prices.len());
        return Ok((null_series.clone(), null_series.clone(), null_series));
    }

    let middle_band_series = calculate_sma(prices, period)?;

    let rolling_std_series = calculate_rolling_std_dev(prices, period)?;

    let std_dev_scaled_series = &rolling_std_series * std_dev_mult;

    let mut upper_band = (&middle_band_series + &std_dev_scaled_series)?;
    upper_band.rename("upper_band".into());

    let mut lower_band = (&middle_band_series - &std_dev_scaled_series)?;
    lower_band.rename("lower_band".into());

    let mut middle_band_named = middle_band_series.clone();
    middle_band_named.rename("middle_band".into());

    Ok((upper_band, middle_band_named, lower_band))
}

/// Calculates the Average True Range (ATR).
///
/// ATR is a measure of market volatility.
/// True Range (TR) for each period is the greatest of:
///   - Current High - Current Low
///   - abs(Current High - Previous Close)
///   - abs(Current Low - Previous Close)
///     ATR is typically a smoothed average (Wilder's smoothing) of TR values.
///
/// # Arguments
/// * `df` - DataFrame with "high", "low", "close" columns (Float64).
/// * `period` - The lookback period for ATR (commonly 14).
///
/// # Returns
/// A `PolarsResult<Series>` containing the ATR values.
pub fn calculate_atr(df: &DataFrame, period: usize) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::ComputeError(
            "ATR period must be greater than 0".into(),
        ));
    }

    let high_ca = df.column("high")?.f64()?;
    let low_ca = df.column("low")?.f64()?;
    let close_ca = df.column("close")?.f64()?;

    let len = close_ca.len();
    if len == 0 {
        return Ok(Series::new_empty("atr".into(), &DataType::Float64));
    }
    // ATR needs at least `period` TR values for the initial SMA, and TR needs `PreviousClose`.
    // So, `period + 1` original data points are needed for the first ATR value.
    if len <= period {
        let nulls: Vec<Option<f64>> = vec![None; len];
        return Ok(Series::new("atr".into(), nulls));
    }

    let mut prev_close_vec: Vec<Option<f64>> = vec![None];
    prev_close_vec.extend(close_ca.into_iter().take(len - 1));
    let prev_close_ca = Series::new("prev_close".into(), prev_close_vec)
        .f64()?
        .clone();

    let mut tr_values: Vec<Option<f64>> = vec![None; len]; // TR for index 0 is None

    for (i, tr_value) in tr_values.iter_mut().enumerate().skip(1) {
        // Start from 1 because TR uses previous close
        let h_opt = high_ca.get(i);
        let l_opt = low_ca.get(i);
        let pc_opt = prev_close_ca.get(i); // This corresponds to original close_ca.get(i-1)

        if let (Some(h), Some(l), Some(pc)) = (h_opt, l_opt, pc_opt) {
            let tr1 = h - l;
            let tr2 = (h - pc).abs();
            let tr3 = (l - pc).abs();
            *tr_value = Some(tr1.max(tr2).max(tr3));
        } else {
            *tr_value = None; // If any component is None, TR is None
        }
    }

    let _tr_series = Series::new("tr".into(), tr_values.clone()); // Keep tr_values for direct access if needed

    // Calculate ATR using Wilder's smoothing
    // Initial ATR is the SMA of the first 'period' TR values.
    // TR values start from index 1 of tr_values (corresponds to price index 1)
    // So, the first 'period' TRs are tr_values[1]...tr_values[period]

    let mut atr_result_values: Vec<Option<f64>> = vec![None; len];

    // Calculate initial SMA of TR for the first ATR value
    // This first ATR value will be at index `period` of the final ATR series
    let initial_tr_slice_for_sma: Vec<Option<f64>> =
        tr_values.iter().skip(1).take(period).cloned().collect();
    if initial_tr_slice_for_sma.iter().any(|opt_v| opt_v.is_none())
        || initial_tr_slice_for_sma.len() < period
    {
        // Not enough valid TRs for initial SMA, ATR remains None for this and subsequent points unless re-seeded
        // This is already covered by len <= period for the entire output.
        // If there are Nones within the first `period` TRs, SMA will be None.
    } else {
        let initial_sum_tr: f64 = initial_tr_slice_for_sma
            .iter()
            .map(|opt_v| opt_v.unwrap_or(0.0))
            .sum();
        let mut current_atr = initial_sum_tr / period as f64;
        atr_result_values[period] = Some(current_atr);

        // Subsequent ATR values using Wilder's smoothing
        // Starts from TR value at index `period + 1` (tr_values[period+1])
        // This TR value corresponds to original price data index `period + 1`
        // The ATR calculated will be for index `period + 1`
        for i in (period + 1)..len {
            if let Some(current_tr_val) = tr_values[i] {
                current_atr = (current_atr * (period - 1) as f64 + current_tr_val) / period as f64;
                atr_result_values[i] = Some(current_atr);
            } else {
                // If current TR is None, ATR propagation stops or becomes None.
                // For simplicity, if TR is None, current ATR becomes None.
                atr_result_values[i] = None;
            }
        }
    }

    Ok(Series::new("atr".into(), atr_result_values))
}

/// Calculates the Ease of Movement (EOM / EMV).
///
/// EOM highlights the relationship between price change and volume.
/// High EOM values occur when prices move easily on low volume.
/// Low EOM values occur when prices struggle to move on high volume.
///
/// Calculation:
/// 1. Midpoint = (High + Low) / 2
/// 2. MidpointMove = Current Midpoint - Previous Midpoint
/// 3. BoxRatio = (Volume / ScalingFactor) / (High - Low)
/// 4. OnePeriodEOM = MidpointMove / BoxRatio  (If High == Low or Volume == 0, OnePeriodEOM is 0)
/// 5. EOM = SMA(OnePeriodEOM, period)
///
/// # Arguments
/// * `df` - DataFrame with "high", "low", "volume" columns (Float64).
/// * `period` - The lookback period for the final SMA of EOM (e.g., 14).
/// * `scaling_factor` - A factor to scale volume (e.g., 10000, 1000000).
///
/// # Returns
/// A `PolarsResult<Series>` containing the Ease of Movement values.
pub fn calculate_ease_of_movement(
    df: &DataFrame,
    period: usize,
    scaling_factor: f64,
) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::ComputeError(
            "EOM period must be greater than 0".into(),
        ));
    }
    if scaling_factor == 0.0 {
        return Err(PolarsError::ComputeError(
            "EOM scaling_factor cannot be zero".into(),
        ));
    }

    let high_ca = df.column("high")?.f64()?;
    let low_ca = df.column("low")?.f64()?;
    let volume_ca = df.column("volume")?.f64()?;

    let len = high_ca.len();
    if len == 0 {
        return Ok(Series::new_empty("eom".into(), &DataType::Float64));
    }
    if len < period + 1 {
        // Need 1 for midpoint move, then `period` for SMA
        let nulls: Vec<Option<f64>> = vec![None; len];
        return Ok(Series::new("eom".into(), nulls));
    }

    // Calculate Midpoint
    let midpoint_ca = (high_ca + low_ca) / 2.0;

    let mut one_period_eom_values: Vec<Option<f64>> = vec![None; len]; // EOM[0] is None due to prev_midpoint

    for (i, eom_value) in one_period_eom_values.iter_mut().enumerate().skip(1) {
        // Start from 1 for MidpointMove
        let current_high = high_ca.get(i);
        let current_low = low_ca.get(i);
        let current_volume = volume_ca.get(i);
        let current_midpoint = midpoint_ca.get(i);
        let prev_midpoint = midpoint_ca.get(i - 1);

        if let (Some(ch), Some(cl), Some(vol), Some(cm), Some(pm)) = (
            current_high,
            current_low,
            current_volume,
            current_midpoint,
            prev_midpoint,
        ) {
            if ch == cl || vol == 0.0 {
                // If high equals low, or volume is zero, EOM is 0
                *eom_value = Some(0.0);
                continue;
            }

            let midpoint_move = cm - pm;
            let box_ratio = (vol / scaling_factor) / (ch - cl);

            if box_ratio != 0.0 {
                *eom_value = Some(midpoint_move / box_ratio);
            } else {
                // This case should ideally be rare if ch != cl and vol != 0.
                // If box_ratio is zero (e.g. scaled volume is zero but H-L is not),
                // EOM could be infinite. Setting to 0 or a large number might be options.
                // Stockcharts typically shows 0 if H=L or Vol=0, which we handle above.
                // For extremely small volume that rounds to 0 after scaling, BoxRatio could be 0.
                // Let's treat it as effectively zero movement or infinite resistance.
                *eom_value = Some(0.0);
            }
        } else {
            *eom_value = None; // If any required data is None
        }
    }

    let one_period_eom_series = Series::new("one_period_eom".into(), one_period_eom_values);

    // Calculate SMA of OnePeriodEOM
    // Note: calculate_sma expects the series passed to it to have enough non-null values.
    // The first element of one_period_eom_series is None.
    // The sma needs `period` values. So the sma output will have Nones for first `period` (from its input perspective).
    // effectively, final EOM will have `period` Nones from one_period_eom_series perspective, and one_period_eom_series[0] is already None.
    let eom_sma =
        crate::trade_math::moving_averages::calculate_sma(&one_period_eom_series, period)?;

    Ok(eom_sma.with_name("eom".into()))
}

/// Calculates the Volume Price Trend (VPT), also known as Price Volume Trend (PVT).
///
/// VPT is a momentum-based indicator used to measure money flow.
/// It is a cumulative indicator:
/// PVT_current = Previous_PVT + (Volume_current * ((CurrentClose - PreviousClose) / PreviousClose))
/// The first PVT value is typically 0 or the first calculated term.
///
/// # Arguments
/// * `df` - DataFrame with "close" and "volume" columns (Float64).
///
/// # Returns
/// A `PolarsResult<Series>` containing the VPT values.
pub fn calculate_volume_price_trend(df: &DataFrame) -> PolarsResult<Series> {
    let close_ca = df.column("close")?.f64()?;
    let volume_ca = df.column("volume")?.f64()?;

    let len = close_ca.len();
    if len == 0 {
        return Ok(Series::new_empty("vpt".into(), &DataType::Float64));
    }

    let mut vpt_values: Vec<Option<f64>> = vec![None; len];

    if len > 0 {
        // First VPT value can be considered 0, or based on first day if no prior data.
        // For simplicity if we use the formula, first value will be None due to PreviousClose.
        // Let's initialize vpt_values[0] to 0.0 as a common starting point for cumulative indicators.
        // Alternatively, if we want the first calculated term: if close_ca.get(0) is Some and vol_ca.get(0) is Some, vpt_values[0] would be calculated.
        // But since it depends on prev_close, it would be None based on strict formula for index 0.
        // Starting with 0.0 for vpt_values[0] is a common approach.

        // To strictly follow the formula where PVT_current uses Previous_PVT:
        // PVT[0] = Volume[0] * (Close[0] - Close[-1]) / Close[-1] -> undefined if Close[-1] doesn't exist.
        // So vpt_values[0] will remain None if we purely rely on formula with prev_close.
        // If we set vpt_values[0] = 0, then vpt_values[1] = vpt_values[0] + Vol[1]*( (C[1]-C[0])/C[0] )

        let mut current_vpt = 0.0; // Initialize first PVT (previous PVT for the first calculation pass)
        vpt_values[0] = Some(0.0); // Set the first actual VPT value to 0.0.

        for i in 1..len {
            let current_close_opt = close_ca.get(i);
            let prev_close_opt = close_ca.get(i - 1);
            let current_volume_opt = volume_ca.get(i);

            if let (Some(cc), Some(pc), Some(vol)) =
                (current_close_opt, prev_close_opt, current_volume_opt)
            {
                if pc != 0.0 {
                    let price_change_pct = (cc - pc) / pc;
                    current_vpt += vol * price_change_pct;
                    vpt_values[i] = Some(current_vpt);
                } else {
                    // Price percentage change cannot be calculated if previous close is 0.
                    // Propagate None or carry forward previous VPT.
                    // Carrying forward might be misleading. Let's use None for this point.
                    current_vpt = vpt_values[i - 1].unwrap_or(0.0); // Reset/get last valid VPT.
                    vpt_values[i] = None;
                }
            } else {
                // If data is missing, result for this point is None. Carry previous valid vpt for next calc if possible.
                current_vpt = vpt_values[i - 1].unwrap_or(0.0);
                vpt_values[i] = None;
            }
        }
    }

    Ok(Series::new("vpt".into(), vpt_values))
}

/// Calculates the On-Balance Volume (OBV).
///
/// OBV is a momentum indicator that uses volume flow to predict changes in stock price.
/// It is a cumulative indicator:
/// - If CurrentClose > PreviousClose, OBV_current = Previous_OBV + CurrentVolume
/// - If CurrentClose < PreviousClose, OBV_current = Previous_OBV - CurrentVolume
/// - If CurrentClose == PreviousClose, OBV_current = Previous_OBV
///   The first OBV value is typically 0.
///
/// # Arguments
/// * `df` - DataFrame with "close" and "volume" columns (Float64).
///
/// # Returns
/// A `PolarsResult<Series>` containing the OBV values.
pub fn calculate_obv(df: &DataFrame) -> PolarsResult<Series> {
    let close_ca = df.column("close")?.f64()?;
    let volume_ca = df.column("volume")?.f64()?;

    let len = close_ca.len();
    if len == 0 {
        return Ok(Series::new_empty("obv".into(), &DataType::Float64));
    }

    let mut obv_values: Vec<Option<f64>> = vec![None; len];
    let mut current_obv = 0.0; // Initialize first OBV (previous OBV for the first calculation pass)

    if len > 0 {
        // First OBV actual value set to 0, if data exists for it.
        // If close_ca.get(0) or volume_ca.get(0) is None, obv_values[0] will be None.
        // Else it is 0.0. current_obv state tracks the numeric value for accumulation.
        if close_ca.get(0).is_some() && volume_ca.get(0).is_some() {
            obv_values[0] = Some(0.0);
            // current_obv is already 0.0, representing the OBV *before* considering day 0's volume.
            // So for day 0, Previous_OBV effectively is 0.
            // The actual OBV for day 0 is set to 0, and subsequent calculations use this.
        } else {
            obv_values[0] = None;
            // If day 0 data is incomplete, cannot start OBV. `current_obv` remains 0 for a potential valid start later.
        }

        for i in 1..len {
            let current_close_opt = close_ca.get(i);
            let prev_close_opt = close_ca.get(i - 1);
            let current_volume_opt = volume_ca.get(i);

            // Carry forward previous OBV value if the current one cannot be calculated
            let prev_obv_for_calc = obv_values[i - 1].unwrap_or(current_obv);
            // If obv_values[i-1] was None, use the last known good current_obv state.

            if let (Some(cc), Some(pc), Some(vol)) =
                (current_close_opt, prev_close_opt, current_volume_opt)
            {
                current_obv = if cc > pc {
                    prev_obv_for_calc + vol
                } else if cc < pc {
                    prev_obv_for_calc - vol
                } else {
                    prev_obv_for_calc
                };
                obv_values[i] = Some(current_obv);
            } else {
                // If data is missing for current calculation, current iteration's OBV is None.
                // current_obv state (accumulator) remains the value from the last successful calculation (or obv_values[i-1]).
                current_obv = prev_obv_for_calc; // Ensure current_obv state doesn't advance if calc fails
                obv_values[i] = None;
            }
        }
    }
    Ok(Series::new("obv".into(), obv_values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::{AnyValue, IntoLazy};
    // calculate_sma is now imported, so its specific tests are in moving_averages.rs
    // We keep tests for calculate_bollinger_bands and calculate_rolling_std_dev here.

    fn create_test_series(name: &str, data: Vec<Option<f64>>) -> Series {
        Series::new(name.into(), data)
    }

    // Tests for calculate_rolling_std_dev can be added here if desired,
    // or kept minimal if it's considered a well-tested polars internal.
    #[test]
    fn test_rolling_std_dev_basic() -> PolarsResult<()> {
        let s = create_test_series(
            "price",
            vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)],
        );
        let std2 = calculate_rolling_std_dev(&s, 2)?;
        assert_eq!(std2.get(0).unwrap(), AnyValue::Null);
        // For [1,2], std is sqrt(((1-1.5)^2 + (2-1.5)^2)/2) without ddof, or /1 with ddof=1 (polars default for sample std)
        // Polars rolling_std by default has ddof=1.
        // std of (1,2) = sqrt( ((1-1.5)^2 + (2-1.5)^2) / (2-1) ) = sqrt(0.25 + 0.25) = sqrt(0.5) approx 0.7071
        assert!((std2.get(1).unwrap().try_extract::<f64>().unwrap() - 0.70710678).abs() < 0.0001);
        Ok(())
    }

    #[test]
    fn test_bollinger_bands_basic() -> PolarsResult<()> {
        let prices = Series::new(
            "close".into(),
            vec![
                Some(10.0),
                Some(12.0),
                Some(11.0),
                Some(13.0),
                Some(15.0),
                Some(14.0),
                Some(16.0),
                Some(18.0),
                Some(17.0),
                Some(19.0),
            ],
        );
        let period = 5;
        let std_dev_mult = 2.0;

        let (upper, middle, lower) = calculate_bollinger_bands(&prices, period, std_dev_mult)?;

        assert_eq!(upper.len(), prices.len());
        assert_eq!(middle.len(), prices.len());
        assert_eq!(lower.len(), prices.len());

        assert_eq!(middle.get(3).unwrap(), AnyValue::Null);
        assert_eq!(middle.get(4).unwrap(), AnyValue::Float64(12.2));
        assert_eq!(middle.get(5).unwrap(), AnyValue::Float64(13.0));

        // Verify one set of band values (e.g. for index 4 where middle is 12.2)
        // prices for window: [10,12,11,13,15]. std_dev for this is approx 1.92353 (ddof=1)
        // upper = 12.2 + 2 * 1.92353 = 12.2 + 3.84706 = 16.04706
        // lower = 12.2 - 2 * 1.92353 = 12.2 - 3.84706 = 8.35294
        assert!((upper.get(4).unwrap().try_extract::<f64>().unwrap() - 16.04706).abs() < 0.0001);
        assert!((lower.get(4).unwrap().try_extract::<f64>().unwrap() - 8.35294).abs() < 0.0001);

        assert_eq!(upper.name(), "upper_band");
        assert_eq!(middle.name(), "middle_band");
        assert_eq!(lower.name(), "lower_band");

        Ok(())
    }

    #[test]
    fn test_bollinger_bands_invalid_inputs() {
        let prices = Series::new("close".into(), vec![Some(10.0), Some(12.0)]);
        assert!(calculate_bollinger_bands(&prices, 0, 2.0).is_err());
        assert!(calculate_bollinger_bands(&prices, 5, 0.0).is_err());
        assert!(calculate_bollinger_bands(&prices, 5, -1.0).is_err());

        let prices_short = Series::new("close".into(), vec![Some(10.0), Some(11.0)]);
        let (up, mid, low) = calculate_bollinger_bands(&prices_short, 3, 2.0).unwrap();
        assert!(up.is_null().all());
        assert!(mid.is_null().all());
        assert!(low.is_null().all());

        let prices_int = Series::new("close".into(), vec![10i32, 12, 11, 13, 15]);
        assert!(calculate_bollinger_bands(&prices_int, 3, 2.0).is_err());
    }

    #[test]
    fn test_atr_calculation() -> PolarsResult<()> {
        // Example from https://school.stockcharts.com/doku.php?id=technical_indicators:average_true_range_atr
        // Simplified to match Wilder's smoothing after initial SMA.
        // Day | H   | L   | C   | TR  | ATR14 (SMA for first, then Wilder)
        // 1   |30.10|29.66|30.01| -   | -
        // 2   |30.13|29.80|29.88|.44  | - (TR for day 1 using day0 close not given, using H-L: 0.44. Prev C for day 1 = 30.01)
        // ... (need 14 TR values for first ATR)
        // For this test, let's use a shorter period and verify Wilder's smoothing part mainly.
        // Prices from TA-Lib test data for ATR(5)
        // High:  [..., 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0 ]
        // Low:   [...,  2.0,  2.0,  2.0,  2.0,  2.0,  2.0,  2.0,  2.0 ]
        // Close: [...,  6.0,  7.0,  6.0,  7.0,  6.0,  7.0,  6.0,  7.0 ] (len 8 for this slice)
        // PrevC: [...,  X,    6.0,  7.0,  6.0,  7.0,  6.0,  7.0,  6.0 ]
        // TR:    [...,  TR0,  4.0,  3.0,  4.0,  3.0,  4.0,  3.0,  4.0 ]
        // ATR(5) (TA-Lib): ..., 3.6, 3.6, 3.6, 3.6  (Stable TR leads to stable ATR)

        let high_vec = vec![Some(10.0); 10];
        let low_vec = vec![Some(2.0); 10];
        let close_data = vec![
            Some(6.0),
            Some(7.0),
            Some(6.0),
            Some(7.0),
            Some(6.0),
            Some(7.0),
            Some(6.0),
            Some(7.0),
            Some(6.0),
            Some(7.0),
        ];

        let df = polars::prelude::df! {
            "high" => high_vec,
            "low" => low_vec,
            "close" => close_data
        }?;
        let period = 5;
        let atr_series = calculate_atr(&df, period)?;

        // Expected TR values (starting from index 1 of tr_values, which is index 1 of price data):
        // C0=6.0
        // H1=10, L1=2, C1=7.0, PC0=6.0. TR1 = max(8, 10-6, 2-6) = max(8,4,4) = 8.0  <- ERROR in manual TR logic previously, was 4.
        // H2=10, L2=2, C2=6.0, PC1=7.0. TR2 = max(8, 10-7, 2-7) = max(8,3,5) = 8.0
        // H3=10, L3=2, C3=7.0, PC2=6.0. TR3 = max(8, 10-6, 2-6) = max(8,4,4) = 8.0
        // H4=10, L4=2, C4=6.0, PC3=7.0. TR4 = max(8, 10-7, 2-7) = max(8,3,5) = 8.0
        // H5=10, L5=2, C5=7.0, PC4=6.0. TR5 = max(8, 10-6, 2-6) = max(8,4,4) = 8.0
        // Initial 5 TRs are all 8.0.
        // Initial ATR (SMA of these 5 TR values) = 8.0. This is atr_result_values[period] = atr_result_values[5]

        // Next TR (index 6 for prices): H6=10, L6=2, C6=6.0, PC5=7.0. TR6 = 8.0
        // ATR6 = (ATR5 * 4 + TR6) / 5 = (8.0 * 4 + 8.0) / 5 = (32+8)/5 = 40/5 = 8.0

        // So, for this data, ATR should be 8.0 after the initial period.
        // First ATR at index `period` (5).
        for i in 0..period {
            assert_eq!(
                atr_series.get(i).unwrap(),
                AnyValue::Null,
                "ATR at index {} should be None",
                i
            );
        }

        for i in period..atr_series.len() {
            if let Ok(val) = atr_series.get(i).unwrap().try_extract::<f64>() {
                assert!(
                    (val - 8.0).abs() < 1e-6,
                    "ATR at index {} expected 8.0, got {}",
                    i,
                    val
                );
            } else {
                panic!("ATR at index {} was unexpectedly None", i);
            }
        }
        Ok(())
    }

    #[test]
    fn test_atr_with_nones() -> PolarsResult<()> {
        let high_vec = vec![
            Some(10.0),
            Some(10.0),
            None,
            Some(10.0),
            Some(10.0),
            Some(10.0),
        ];
        let low_vec = vec![Some(2.0), Some(2.0), Some(2.0), None, Some(2.0), Some(2.0)];
        let close_vec = vec![
            Some(6.0),
            Some(7.0),
            Some(6.0),
            Some(7.0),
            Some(6.0),
            Some(7.0),
        ];
        let df = polars::prelude::df! {
            "high" => high_vec, "low" => low_vec, "close" => close_vec
        }?;
        let period = 3;
        let atr_series = calculate_atr(&df, period)?;
        // TR[0] = None (by definition)
        // TR[1]: H=10, L=2, C=7, PC=6 -> TR = 8
        // TR[2]: H=None, L=2, C=6, PC=7 -> TR = None
        // TR[3]: H=10, L=None, C=7, PC=6 -> TR = None
        // Initial ATR at index 3 needs TR[1],TR[2],TR[3]. Since TR[2],TR[3] are None, initial ATR is None.
        // All subsequent ATRs should also be None.
        assert!(atr_series.is_null().all());
        Ok(())
    }

    #[test]
    fn test_atr_insufficient_data() -> PolarsResult<()> {
        let df = polars::prelude::df! {
            "high" => &[Some(10.0), Some(10.0)],
            "low" => &[Some(2.0), Some(2.0)],
            "close" => &[Some(6.0), Some(7.0)]
        }?;
        let atr_series = calculate_atr(&df, 3)?; // Needs period+1 = 4 rows for first ATR
        assert!(atr_series.is_null().all());
        assert_eq!(atr_series.len(), 2);
        Ok(())
    }

    #[test]
    fn test_atr_period_zero() {
        let df = polars::prelude::df!{"high" => &[Some(1.0)], "low" => &[Some(1.0)], "close" => &[Some(1.0)]}.unwrap();
        assert!(calculate_atr(&df, 0).is_err());
    }

    #[test]
    fn test_ease_of_movement_basic() -> PolarsResult<()> {
        // Data from: https://school.stockcharts.com/doku.php?id=technical_indicators:ease_of_movement_emv
        // H | L   | Vol     | MP   | MPM    | BoxRatio | 1-day EMV | 14-day EMV
        // Scaling factor chosen to match example if possible, or use common one like 10000 / 1000000
        // Stockcharts uses Volume in millions, so if Vol=1230000, they use 1.23
        // This implies a scaling factor of 1,000,000 if input volume is raw.
        // The example does not specify scaling factor in their Box Ratio formula directly, but implies Vol is already scaled.
        // Let's assume Vol is raw and use scaling_factor = 100_000.0 for test

        let high_vec = vec![
            Some(43.50),
            Some(43.54),
            Some(43.82),
            Some(43.81),
            Some(44.10),
            Some(43.85),
            Some(43.80),
            Some(43.80),
            Some(43.60),
            Some(43.95),
            Some(44.00),
            Some(44.05),
            Some(44.20),
            Some(44.20),
            Some(44.15), // 15 days
        ];
        let low_vec = vec![
            Some(43.01),
            Some(43.08),
            Some(43.39),
            Some(43.45),
            Some(43.70),
            Some(43.52),
            Some(43.51),
            Some(43.45),
            Some(43.30),
            Some(43.45),
            Some(43.75),
            Some(43.80),
            Some(43.82),
            Some(43.90),
            Some(43.85),
        ];
        let volume_vec: Vec<Option<f64>> = vec![
            Some(1236600.0),
            Some(1151500.0),
            Some(1748300.0),
            Some(1043800.0),
            Some(1140800.0),
            Some(707000.0),
            Some(671400.0),
            Some(955400.0),
            Some(1070200.0),
            Some(1123200.0),
            Some(800000.0),
            Some(750000.0),
            Some(920000.0),
            Some(600000.0),
            Some(750000.0),
        ];

        let df = polars::prelude::df! {
            "high" => high_vec,
            "low" => low_vec,
            "volume" => volume_vec
        }?;

        let period = 5; // Shorter period for testability
        let scaling_factor = 100_000.0;
        let eom_series = calculate_ease_of_movement(&df, period, scaling_factor)?;

        assert_eq!(eom_series.len(), df.height());

        // 1-day EOM values (manual calc for first few, with period=5 SMA)
        // Day 1 (idx 0): MP = 43.255, MPM = None, 1-EOM = None
        // Day 2 (idx 1): H=43.54, L=43.08, V=1151500. MP=43.31. PrevMP=43.255. MPM=0.055.
        //                H-L=0.46. BoxRatio=(1151500/100k)/0.46 = 11.515 / 0.46 = 25.0326
        //                1-EOM[1] = 0.055 / 25.0326 = 0.002197
        // Day 3 (idx 2): H=43.82, L=43.39, V=1748300. MP=43.605. PrevMP=43.31. MPM=0.295.
        //                H-L=0.43. BoxRatio=(1748300/100k)/0.43 = 17.483 / 0.43 = 40.658
        //                1-EOM[2] = 0.295 / 40.658 = 0.007255
        // ... and so on.
        // EOM (SMA of 1-EOM) will have first `period` (5) values as None from perspective of 1-EOM series.
        // And 1-EOM[0] is None. So final EOM[0]...EOM[4] are None.
        // EOM[5] is SMA(1-EOM[1]...1-EOM[5]).

        for i in 0..period {
            // First period value from SMA are None, plus 1-day EOM[0] is None.
            assert_eq!(
                eom_series.get(i).unwrap(),
                AnyValue::Null,
                "EOM at index {} should be None (initial)",
                i
            );
        }

        // Test a specific calculated value if available from a trusted source, or check for non-null after initial period.
        // For now, just check it's not all nulls after the initial period.
        let mut all_nulls_after_initial = true;
        for i in period..eom_series.len() {
            if !eom_series.get(i).unwrap().is_null() {
                all_nulls_after_initial = false;
                break;
            }
        }
        if eom_series.len() > period {
            assert!(
                !all_nulls_after_initial,
                "EOM series should have non-null values after initial period"
            );
        }
        Ok(())
    }

    #[test]
    fn test_eom_high_equals_low() -> PolarsResult<()> {
        let df = polars::prelude::df! {
            "high" =>   &[Some(10.0), Some(10.0), Some(12.0)],
            "low" =>    &[Some(9.0),  Some(10.0), Some(11.0)], // H=L at index 1
            "volume" => &[Some(1000.0), Some(1000.0), Some(1000.0)]
        }?;
        let eom = calculate_ease_of_movement(&df, 1, 10000.0)?;
        // 1-EOM[0] = None
        // 1-EOM[1]: H=10, L=10. Should be 0.  SMA(0) -> EOM[1]=0
        // 1-EOM[2]: MP0=9.5, MP1=10, MP2=11.5. MPM2=1.5. HL2=1. BoxRatio=(1000/10k)/1 = 0.1. 1-EOM[2]=1.5/0.1=15. SMA(15)->EOM[2]=15
        assert_eq!(eom.get(0).unwrap(), AnyValue::Null);
        assert_eq!(eom.get(1).unwrap().try_extract::<f64>().unwrap(), 0.0);
        assert!((eom.get(2).unwrap().try_extract::<f64>().unwrap() - 15.0).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_eom_zero_volume() -> PolarsResult<()> {
        let df = polars::prelude::df! {
            "high" =>   &[Some(10.0), Some(11.0), Some(12.0)],
            "low" =>    &[Some(9.0),  Some(10.0), Some(11.0)],
            "volume" => &[Some(1000.0), Some(0.0), Some(1000.0)] // Zero vol at index 1
        }?;
        let eom = calculate_ease_of_movement(&df, 1, 10000.0)?;
        // 1-EOM[1] for zero volume should be 0. SMA(0) -> EOM[1]=0
        assert_eq!(eom.get(0).unwrap(), AnyValue::Null);
        assert_eq!(eom.get(1).unwrap().try_extract::<f64>().unwrap(), 0.0);
        Ok(())
    }

    #[test]
    fn test_eom_invalid_inputs() -> PolarsResult<()> {
        let df = polars::prelude::df! {"h" => &[1.0], "l" => &[1.0], "v" => &[1.0]}?;
        assert!(calculate_ease_of_movement(
            &df.clone()
                .lazy()
                .rename(vec!["h", "l", "v"], vec!["high", "low", "volume"], true)
                .collect()?,
            0,
            10000.0
        )
        .is_err());
        assert!(calculate_ease_of_movement(
            &df.clone()
                .lazy()
                .rename(vec!["h", "l", "v"], vec!["high", "low", "volume"], true)
                .collect()?,
            1,
            0.0
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn test_eom_insufficient_data() -> PolarsResult<()> {
        let df = polars::prelude::df! {
            "high" => &[Some(10.0)], "low" => &[Some(9.0)], "volume" => &[Some(1000.0)]
        }?;
        let eom = calculate_ease_of_movement(&df, 2, 10000.0)?;
        // Needs period + 1 for SMA part. (2+1=3 rows for period 2 EOM)
        // Current implementation: len < period + 1 returns all nulls.
        assert!(eom.is_null().all());
        assert_eq!(eom.len(), 1);
        Ok(())
    }

    #[test]
    fn test_volume_price_trend_basic() -> PolarsResult<()> {
        // Close:  10, 10.5, 10.2, 10.7, 11
        // Volume:100, 110,  90,   120,  130
        let close_vec = vec![Some(10.0), Some(10.5), Some(10.2), Some(10.7), Some(11.0)];
        let volume_vec = vec![
            Some(100.0),
            Some(110.0),
            Some(90.0),
            Some(120.0),
            Some(130.0),
        ];
        let df = polars::prelude::df! {"close" => close_vec, "volume" => volume_vec}?;

        let vpt = calculate_volume_price_trend(&df)?;
        assert_eq!(vpt.len(), 5);

        // Expected values:
        // VPT[0] = 0.0 (initialization)
        // VPT[1] = VPT[0] + 110 * ((10.5 - 10.0) / 10.0) = 0 + 110 * 0.05 = 5.5
        // VPT[2] = VPT[1] + 90 * ((10.2 - 10.5) / 10.5) = 5.5 + 90 * (-0.0285714) = 5.5 - 2.571426 = 2.928574
        // VPT[3] = VPT[2] + 120 * ((10.7 - 10.2) / 10.2) = 2.928574 + 120 * (0.0490196) = 2.928574 + 5.882352 = 8.810926
        // VPT[4] = VPT[3] + 130 * ((11.0 - 10.7) / 10.7) = 8.810926 + 130 * (0.02803738) = 8.810926 + 3.6448594 = 12.4557854

        assert_eq!(vpt.get(0).unwrap().try_extract::<f64>().unwrap(), 0.0);
        assert!((vpt.get(1).unwrap().try_extract::<f64>().unwrap() - 5.5).abs() < 1e-6);
        assert!((vpt.get(2).unwrap().try_extract::<f64>().unwrap() - 2.9285714).abs() < 1e-6);
        assert!((vpt.get(3).unwrap().try_extract::<f64>().unwrap() - 8.810924).abs() < 1e-4); // Adjusted for precision
        assert!((vpt.get(4).unwrap().try_extract::<f64>().unwrap() - 12.455785).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_vpt_prev_close_zero() -> PolarsResult<()> {
        let df = polars::prelude::df! {
            "close" => vec![Some(10.0), Some(0.0), Some(5.0)],
            "volume" => vec![Some(100.0), Some(100.0), Some(100.0)]
        }?;
        let vpt = calculate_volume_price_trend(&df)?;
        // VPT[0] = 0.0
        // VPT[1] = 0.0 + 100 * ((0 - 10)/10) = 0 - 100 = -100.0
        // VPT[2]: prev_close is 0. Percentage change is undefined. VPT[2] should be None.
        //          current_vpt value for next step would be VPT[1]
        assert_eq!(vpt.get(0).unwrap().try_extract::<f64>().unwrap(), 0.0);
        assert!((vpt.get(1).unwrap().try_extract::<f64>().unwrap() - (-100.0)).abs() < 1e-6);
        assert_eq!(vpt.get(2).unwrap(), AnyValue::Null);
        Ok(())
    }

    #[test]
    fn test_vpt_empty_df() -> PolarsResult<()> {
        let schema = polars::prelude::Schema::from_iter(vec![
            ("close".into(), DataType::Float64),
            ("volume".into(), DataType::Float64),
        ]);
        let empty_df = DataFrame::from_rows_and_schema(&[], &schema)?;
        let vpt = calculate_volume_price_trend(&empty_df)?;
        assert_eq!(vpt.len(), 0);
        Ok(())
    }

    #[test]
    fn test_vpt_with_nones() -> PolarsResult<()> {
        let df = polars::prelude::df! {
            "close" => vec![Some(10.0), None, Some(5.0), Some(6.0)],
            "volume" => vec![Some(100.0), Some(100.0), Some(100.0), Some(100.0)]
        }?;
        let vpt = calculate_volume_price_trend(&df)?;
        // VPT[0] = 0.0
        // VPT[1] = None (due to None close)
        // VPT[2] = None (due to None prev_close for percentage change)
        // VPT[3] = previous_vpt (0.0 from index 0) + 100 * ((6.0-5.0)/5.0) = 0.0 + 100*0.2 = 20.0
        // The internal `current_vpt` state is important here.
        // In my impl, if data is missing, current_vpt carries over from last valid point, and current iteration result is None.

        assert_eq!(vpt.get(0).unwrap().try_extract::<f64>().unwrap(), 0.0);
        assert_eq!(vpt.get(1).unwrap(), AnyValue::Null);
        assert_eq!(vpt.get(2).unwrap(), AnyValue::Null);
        assert!(
            (vpt.get(3).unwrap().try_extract::<f64>().unwrap() - (0.0 + 100.0 * (6.0 - 5.0) / 5.0))
                .abs()
                < 1e-6
        );
        Ok(())
    }

    #[test]
    fn test_obv_calculation() -> PolarsResult<()> {
        // Close:  10, 10.5, 10.2, 10.2, 10.7
        // Volume:100, 110,  90,   120,  130
        let close_vec = vec![Some(10.0), Some(10.5), Some(10.2), Some(10.2), Some(10.7)];
        let volume_vec = vec![
            Some(100.0),
            Some(110.0),
            Some(90.0),
            Some(120.0),
            Some(130.0),
        ];
        let df = polars::prelude::df! {"close" => close_vec, "volume" => volume_vec}?;

        let obv = calculate_obv(&df)?;

        // Expected OBV:
        // OBV[0] = 0 (initialization)
        // OBV[1] = OBV[0] + 110 (since 10.5 > 10.0) = 0 + 110 = 110
        // OBV[2] = OBV[1] - 90  (since 10.2 < 10.5) = 110 - 90 = 20
        // OBV[3] = OBV[2] + 0   (since 10.2 == 10.2) = 20 + 0 = 20
        // OBV[4] = OBV[3] + 130 (since 10.7 > 10.2) = 20 + 130 = 150

        assert_eq!(obv.get(0).unwrap().try_extract::<f64>().unwrap(), 0.0);
        assert_eq!(obv.get(1).unwrap().try_extract::<f64>().unwrap(), 110.0);
        assert_eq!(obv.get(2).unwrap().try_extract::<f64>().unwrap(), 20.0);
        assert_eq!(obv.get(3).unwrap().try_extract::<f64>().unwrap(), 20.0);
        assert_eq!(obv.get(4).unwrap().try_extract::<f64>().unwrap(), 150.0);
        Ok(())
    }

    #[test]
    fn test_obv_with_nones_in_data() -> PolarsResult<()> {
        let close_vec = vec![Some(10.0), None, Some(10.2), Some(10.0), Some(10.5)];
        let volume_vec = vec![
            Some(100.0),
            Some(110.0),
            Some(90.0),
            Some(120.0),
            Some(130.0),
        ];
        let df = polars::prelude::df! {"close" => close_vec, "volume" => volume_vec}?;
        let obv = calculate_obv(&df)?;

        // OBV[0] = 0
        // OBV[1] = None (current close is None)
        // OBV[2] = None (previous close was None)
        // OBV[3] = OBV[2 based on current_obv state which would be from OBV[0]=0] - 120 = 0 - 120 = -120
        // OBV[4] = OBV[3] + 130 = -120 + 130 = 10

        assert_eq!(obv.get(0).unwrap().try_extract::<f64>().unwrap(), 0.0);
        assert_eq!(obv.get(1).unwrap(), AnyValue::Null);
        assert_eq!(obv.get(2).unwrap(), AnyValue::Null);
        assert_eq!(obv.get(3).unwrap().try_extract::<f64>().unwrap(), -120.0);
        assert_eq!(obv.get(4).unwrap().try_extract::<f64>().unwrap(), 10.0);
        Ok(())
    }

    #[test]
    fn test_obv_first_val_none() -> PolarsResult<()> {
        let close_vec = vec![None, Some(10.5)];
        let volume_vec = vec![Some(100.0), Some(110.0)];
        let df = polars::prelude::df! {"close" => close_vec, "volume" => volume_vec}?;
        let obv = calculate_obv(&df)?;
        assert_eq!(obv.get(0).unwrap(), AnyValue::Null);
        assert_eq!(obv.get(1).unwrap(), AnyValue::Null); // prev_close is None
        Ok(())
    }

    #[test]
    fn test_obv_empty_df() -> PolarsResult<()> {
        let schema = polars::prelude::Schema::from_iter(vec![
            ("close".into(), DataType::Float64),
            ("volume".into(), DataType::Float64),
        ]);
        let empty_df = DataFrame::from_rows_and_schema(&[], &schema)?;
        let obv = calculate_obv(&empty_df)?;
        assert_eq!(obv.len(), 0);
        Ok(())
    }
}
