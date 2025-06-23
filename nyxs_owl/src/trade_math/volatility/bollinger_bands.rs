use polars::prelude::*;

/// Calculate Bollinger Bands
///
/// Bollinger Bands consist of:
/// - A middle band (simple moving average)
/// - An upper band (middle band + N standard deviations)
/// - A lower band (middle band - N standard deviations)
///
/// # Arguments
/// * `series` - Series of price data
/// * `period` - The period for the moving average (typically 20)
/// * `std_dev` - Number of standard deviations (typically 2.0)
///
/// # Returns
/// * `PolarsResult<(Series, Series, Series)>` - (upper band, middle band, lower band)
pub fn calculate_bollinger_bands(
    series: &Series,
    period: usize,
    std_dev: f64,
) -> PolarsResult<(Series, Series, Series)> {
    if period == 0 {
        return Err(PolarsError::ComputeError(
            "Bollinger Bands period must be greater than 0".into(),
        ));
    }

    if std_dev <= 0.0 {
        return Err(PolarsError::ComputeError(
            "Standard deviation multiplier must be greater than 0".into(),
        ));
    }

    let options = RollingOptionsFixedWindow {
        window_size: period,
        min_periods: period,
        center: false,
        weights: None,
        fn_params: None,
    };

    let middle = series.rolling_mean(options.clone())?;
    let std = series.rolling_std(options)?;

    let upper = (&middle + &(&std * std_dev))?;
    let lower = (&middle - &(&std * std_dev))?;

    Ok((
        upper.with_name("upper_band".into()),
        middle.with_name("middle_band".into()),
        lower.with_name("lower_band".into()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_bands() {
        let prices = Series::new(
            "prices".into(),
            vec![
                10.0, 12.0, 11.0, 13.0, 14.0, 13.5, 15.0, 14.0, 13.0, 12.5, 11.5, 12.0, 13.0, 14.0,
                15.0, 14.5, 13.5, 12.5, 11.5, 12.0,
            ],
        );

        let period = 5;
        let std_dev = 2.0;

        let (upper, middle, lower) = calculate_bollinger_bands(&prices, period, std_dev).unwrap();

        // Test that bands are properly ordered
        for i in period..prices.len() {
            let u = upper.f64().unwrap().get(i).unwrap();
            let m = middle.f64().unwrap().get(i).unwrap();
            let l = lower.f64().unwrap().get(i).unwrap();

            assert!(u > m);
            assert!(m > l);
        }

        // Test invalid parameters
        assert!(calculate_bollinger_bands(&prices, 0, std_dev).is_err());
        assert!(calculate_bollinger_bands(&prices, period, 0.0).is_err());
        assert!(calculate_bollinger_bands(&prices, period, -1.0).is_err());
    }
}
