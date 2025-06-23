use crate::trade_math::volatility::calculate_atr;
use polars::prelude::*;

/// Calculate SuperTrend indicator
///
/// SuperTrend is a trend-following indicator that combines ATR with price action
/// to identify trend direction and potential reversal points.
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices
/// * `close` - Series of closing prices
/// * `period` - The period for ATR calculation (typically 10)
/// * `multiplier` - ATR multiplier (typically 3.0)
///
/// # Returns
/// * `PolarsResult<(Series, Series)>` - (SuperTrend line, Trend direction)
pub fn calculate_supertrend(
    high: &Series,
    low: &Series,
    close: &Series,
    period: usize,
    multiplier: f64,
) -> PolarsResult<(Series, Series)> {
    if period == 0 {
        return Err(PolarsError::InvalidOperation(
            "SuperTrend period must be greater than 0".into(),
        ));
    }

    if multiplier <= 0.0 {
        return Err(PolarsError::InvalidOperation(
            "SuperTrend multiplier must be greater than 0".into(),
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

    let atr = calculate_atr(high, low, close, period)?;
    let atr_values: Vec<Option<f64>> = atr.f64()?.into_iter().collect();

    let mut supertrend_values = vec![None; high_values.len()];
    let mut trend_direction = vec![None; high_values.len()];

    let mut prev_supertrend = None;
    let mut prev_trend = 1; // 1 for uptrend, -1 for downtrend

    // Calculate SuperTrend for each period
    for i in (period - 1)..high_values.len() {
        if let (Some(high_val), Some(low_val), Some(close_val), Some(atr_val)) = (
            high_values[i],
            low_values[i],
            close_values[i],
            atr_values[i],
        ) {
            let basic_upper = (high_val + low_val) / 2.0 + multiplier * atr_val;
            let basic_lower = (high_val + low_val) / 2.0 - multiplier * atr_val;

            let (supertrend_val, trend) = if let Some(prev_st) = prev_supertrend {
                if prev_trend == 1 {
                    // Uptrend
                    let new_supertrend = basic_lower.min(prev_st);
                    if close_val <= new_supertrend {
                        (basic_upper, -1) // Switch to downtrend
                    } else {
                        (new_supertrend, 1) // Continue uptrend
                    }
                } else {
                    // Downtrend
                    let new_supertrend = basic_upper.max(prev_st);
                    if close_val >= new_supertrend {
                        (basic_lower, 1) // Switch to uptrend
                    } else {
                        (new_supertrend, -1) // Continue downtrend
                    }
                }
            } else {
                // First value
                (basic_lower, 1)
            };

            supertrend_values[i] = Some(supertrend_val);
            trend_direction[i] = Some(trend as f64);

            prev_supertrend = Some(supertrend_val);
            prev_trend = trend;
        }
    }

    Ok((
        Series::new("supertrend".into(), supertrend_values),
        Series::new("trend_direction".into(), trend_direction),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supertrend() {
        let high = Series::new(
            "high".into(),
            vec![10.0, 12.0, 11.0, 13.0, 14.0, 13.5, 15.0, 14.0, 13.0, 12.5],
        );
        let low = Series::new(
            "low".into(),
            vec![9.0, 10.0, 9.5, 11.0, 12.0, 11.5, 13.0, 12.0, 11.0, 10.5],
        );
        let close = Series::new(
            "close".into(),
            vec![9.5, 11.0, 10.0, 12.0, 13.0, 12.5, 14.0, 13.0, 12.0, 11.5],
        );

        let period = 3;
        let multiplier = 2.0;

        let (supertrend, trend_direction) =
            calculate_supertrend(&high, &low, &close, period, multiplier).unwrap();

        // Test that SuperTrend values are finite
        for i in period..supertrend.len() {
            if let Ok(st_val) = supertrend.get(i).unwrap().try_extract::<f64>() {
                assert!(st_val.is_finite());
            }
        }

        // Test that trend direction is valid
        for i in period..trend_direction.len() {
            if let Ok(trend) = trend_direction.get(i).unwrap().try_extract::<f64>() {
                assert!(trend == 1.0 || trend == -1.0);
            }
        }

        // Test invalid parameters
        assert!(calculate_supertrend(&high, &low, &close, 0, multiplier).is_err());
        assert!(calculate_supertrend(&high, &low, &close, period, 0.0).is_err());
        assert!(calculate_supertrend(&high, &low, &close, period, -1.0).is_err());
    }
}
