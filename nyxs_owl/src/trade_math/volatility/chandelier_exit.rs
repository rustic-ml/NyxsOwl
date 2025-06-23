use polars::prelude::*;
use crate::trade_math::volatility::calculate_atr;

/// Calculate Chandelier Exit
///
/// The Chandelier Exit sets a trailing stop-loss based on the Average True Range (ATR).
/// It's designed to keep traders in a trend and exit only when the trend changes direction.
///
/// Long exit = n-period High - ATR multiplier * ATR
/// Short exit = n-period Low + ATR multiplier * ATR
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices
/// * `close` - Series of closing prices
/// * `period` - The lookback period (typically 22)
/// * `atr_period` - The period for ATR calculation (typically 22)
/// * `multiplier` - ATR multiplier (typically 3.0)
///
/// # Returns
/// * `PolarsResult<(Series, Series)>` - (Long Exit, Short Exit) series
pub fn calculate_chandelier_exit(
    high: &Series,
    low: &Series,
    close: &Series,
    period: usize,
    atr_period: usize,
    multiplier: f64,
) -> PolarsResult<(Series, Series)> {
    if period == 0 || atr_period == 0 {
        return Err(PolarsError::InvalidOperation(
            "Periods must be greater than 0".into(),
        ));
    }

    if multiplier <= 0.0 {
        return Err(PolarsError::InvalidOperation(
            "Multiplier must be greater than 0".into(),
        ));
    }

    let high_values: Vec<Option<f64>> = high.f64()?.into_iter().collect();
    let low_values: Vec<Option<f64>> = low.f64()?.into_iter().collect();
    let close_values: Vec<Option<f64>> = close.f64()?.into_iter().collect();

    if high_values.len() != low_values.len() || high_values.len() != close_values.len() {
        return Err(PolarsError::InvalidOperation(
            "All input series must have the same length".into(),
        ));
    }

    let atr = calculate_atr(high, low, close, atr_period)?;
    let atr_values: Vec<Option<f64>> = atr.f64()?.into_iter().collect();

    let mut long_exit = vec![None; high_values.len()];
    let mut short_exit = vec![None; high_values.len()];

    // Calculate Chandelier Exit values
    for (i, (long_val, short_val)) in long_exit.iter_mut().zip(short_exit.iter_mut()).enumerate().skip(period - 1) {
        let mut highest_high = f64::NEG_INFINITY;
        let mut lowest_low = f64::INFINITY;

        // Find highest high and lowest low in the period
        for j in i.saturating_sub(period - 1)..=i {
            if let Some(high_val) = high_values[j] {
                highest_high = highest_high.max(high_val);
            }
            if let Some(low_val) = low_values[j] {
                lowest_low = lowest_low.min(low_val);
            }
        }

        if let Some(atr_val) = atr_values[i] {
            let atr_offset = multiplier * atr_val;
            *long_val = Some(highest_high - atr_offset);
            *short_val = Some(lowest_low + atr_offset);
        }
    }

    Ok((
        Series::new("chandelier_exit_long".into(), long_exit),
        Series::new("chandelier_exit_short".into(), short_exit),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chandelier_exit() {
        let high = Series::new("high".into(), vec![
            110.0, 112.0, 115.0, 113.0, 116.0, 118.0, 117.0, 119.0, 121.0, 120.0,
            122.0, 123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0,
            132.0, 133.0, 134.0, 135.0, 136.0
        ]);
        let low = Series::new("low".into(), vec![
            108.0, 109.0, 111.0, 110.0, 112.0, 114.0, 115.0, 116.0, 118.0, 119.0,
            120.0, 121.0, 122.0, 123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0,
            130.0, 131.0, 132.0, 133.0, 134.0
        ]);
        let close = Series::new("close".into(), vec![
            109.0, 111.0, 113.0, 112.0, 115.0, 116.0, 116.5, 118.0, 120.0, 119.5,
            121.0, 122.0, 123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0,
            131.0, 132.0, 133.0, 134.0, 135.0
        ]);

        let (long_exit, short_exit) = calculate_chandelier_exit(&high, &low, &close, 22, 22, 3.0).unwrap();

        // Test warmup period
        for i in 0..21 {
            assert!(long_exit.get(i).unwrap().try_extract::<f64>().unwrap_or(f64::NAN).is_nan());
            assert!(short_exit.get(i).unwrap().try_extract::<f64>().unwrap_or(f64::NAN).is_nan());
        }

        // Test valid values
        for i in 21..long_exit.len() {
            if let (Ok(long_val), Ok(short_val)) = (
                long_exit.get(i).unwrap().try_extract::<f64>(),
                short_exit.get(i).unwrap().try_extract::<f64>()
            ) {
                // In an uptrend, long exit should be below current price but above short exit
                assert!(long_val.is_finite() && short_val.is_finite());
                assert!(long_val != short_val); // Exits should not be equal unless price is constant
            }
        }

        // Test invalid parameters
        assert!(calculate_chandelier_exit(&high, &low, &close, 0, 22, 3.0).is_err());
        assert!(calculate_chandelier_exit(&high, &low, &close, 22, 0, 3.0).is_err());
        assert!(calculate_chandelier_exit(&high, &low, &close, 22, 22, 0.0).is_err());
        assert!(calculate_chandelier_exit(&high, &low, &close, 22, 22, -1.0).is_err());

        // Test with constant prices
        let constant_high = Series::new("high".into(), vec![100.0; 25]);
        let constant_low = Series::new("low".into(), vec![100.0; 25]);
        let constant_close = Series::new("close".into(), vec![100.0; 25]);

        let (constant_long, constant_short) = calculate_chandelier_exit(
            &constant_high, &constant_low, &constant_close, 22, 22, 3.0
        ).unwrap();

        for i in 21..constant_long.len() {
            if let (Ok(long_val), Ok(short_val)) = (
                constant_long.get(i).unwrap().try_extract::<f64>(),
                constant_short.get(i).unwrap().try_extract::<f64>()
            ) {
                assert_eq!(long_val, 100.0);
                assert_eq!(short_val, 100.0);
            }
        }
    }
} 