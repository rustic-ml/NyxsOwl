use polars::prelude::*;

/// Calculate Average True Range (ATR)
///
/// ATR measures market volatility by decomposing the entire range of an asset price
/// for a specific period. It is used as a component in many volatility-based indicators.
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices
/// * `close` - Series of closing prices
/// * `period` - The period for ATR calculation (typically 14 or 20)
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing ATR values
pub fn calculate_atr(
    high: &Series,
    low: &Series,
    close: &Series,
    period: usize,
) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::InvalidOperation(
            "ATR period must be greater than 0".into(),
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

    let mut atr_values = vec![None; high_values.len()];
    let mut prev_close = None;
    let mut prev_atr = None;

    // Calculate True Range and ATR
    for (i, atr_value) in atr_values.iter_mut().enumerate() {
        if let (Some(high_val), Some(low_val), Some(close_val)) =
            (high_values[i], low_values[i], close_values[i])
        {
            // Calculate True Range
            let tr = if let Some(prev) = prev_close {
                let high_low = high_val - low_val;
                let high_prev_close: f64 = high_val - prev;
                let low_prev_close: f64 = low_val - prev;
                high_low
                    .max(high_prev_close.abs())
                    .max(low_prev_close.abs())
            } else {
                high_val - low_val
            };

            // Calculate ATR
            *atr_value = Some(if let Some(prev_atr_val) = prev_atr {
                ((prev_atr_val * (period - 1) as f64) + tr) / period as f64
            } else {
                tr
            });

            prev_close = Some(close_val);
            prev_atr = *atr_value;
        }
    }

    Ok(Series::new("atr".into(), atr_values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atr() {
        let high = Series::new(
            "high".into(),
            vec![10.0, 12.0, 11.0, 13.0, 14.0, 13.5, 15.0],
        );
        let low = Series::new("low".into(), vec![9.0, 10.0, 9.5, 11.0, 12.0, 11.5, 13.0]);
        let close = Series::new(
            "close".into(),
            vec![9.5, 11.0, 10.0, 12.0, 13.0, 12.5, 14.0],
        );

        let period = 5;
        let atr = calculate_atr(&high, &low, &close, period).unwrap();
        let atr_values = atr.f64().unwrap();

        // Test that ATR values are positive
        for i in period..atr_values.len() {
            assert!(atr_values.get(i).unwrap() > 0.0);
        }

        // Test invalid parameters
        assert!(calculate_atr(&high, &low, &close, 0).is_err());

        // Test mismatched lengths
        let short_high = Series::new("high".into(), vec![10.0, 12.0]);
        assert!(calculate_atr(&short_high, &low, &close, period).is_err());
    }
}
