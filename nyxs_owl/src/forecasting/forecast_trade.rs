#![cfg(feature = "forecasting")]

use chrono::{DateTime, Utc};
// thiserror::Error is not used by InternalForecastError directly, but NyxsOwlError uses it.
// No need to import it here if not deriving Error on InternalForecastError.
use log::{debug, error, warn};

// Import the correct NyxsOwlError
use crate::simple_types::{NyxsOwlError, Result as NyxsOwlResult};

#[derive(Debug)] // Simpler derive for InternalForecastError as it's just for mapping
enum InternalForecastError {
    ArimaError(String),
    InputError(String),
    ModelError(String),
}

// Convert InternalForecastError to the crate-wide NyxsOwlError
impl From<InternalForecastError> for NyxsOwlError {
    fn from(err: InternalForecastError) -> Self {
        match err {
            InternalForecastError::ArimaError(s) => {
                NyxsOwlError::StrategyError(format!("ARIMA forecasting error: {}", s))
            }
            InternalForecastError::InputError(s) => {
                NyxsOwlError::DataError(format!("Input data error for forecast: {}", s))
            }
            InternalForecastError::ModelError(s) => {
                NyxsOwlError::StrategyError(format!("Underlying model error: {}", s))
            }
        }
    }
}

impl From<oxidiviner::core::OxiError> for InternalForecastError {
    fn from(err: oxidiviner::core::OxiError) -> Self {
        InternalForecastError::ModelError(format!("OxiDiviner core error: {}", err))
    }
}

impl From<polars::error::PolarsError> for InternalForecastError {
    fn from(err: polars::error::PolarsError) -> Self {
        InternalForecastError::InputError(format!("Polars data processing error: {}", err))
    }
}

/// Performs ARIMA forecasting using OxiDiviner (v1.1.0) via `quick::arima_forecast_custom`.
///
/// # Arguments
/// * `timestamps`: A `Vec<DateTime<Utc>>` representing the timestamps of the historical data.
/// * `data`: A `Vec<f64>` representing the historical time series data, corresponding to the timestamps.
/// * `order`: A tuple `(usize, usize, usize)` for (p, d, q) of the ARIMA model.
/// * `periods_to_forecast`: The number of future periods to forecast.
///
/// # Returns
/// A `Result` containing a `Vec<f64>` of forecasted values, or a `NyxsOwlError`.
///
/// # Enhanced Features
/// - Outlier detection and robust preprocessing
/// - Numerical stability validation
/// - Enhanced data quality checks
/// - Adaptive model validation
pub fn forecast_arima(
    timestamps: Vec<DateTime<Utc>>,
    data: Vec<f64>,
    order: (usize, usize, usize),
    periods_to_forecast: usize,
) -> NyxsOwlResult<Vec<f64>> {
    // Changed to NyxsOwlResult (which implies NyxsOwlError)
    if data.is_empty() || timestamps.is_empty() {
        warn!("forecast_arima called with empty data or timestamps.");
        return Err(InternalForecastError::InputError(
            "Input data or timestamps slice is empty.".to_string(),
        )
        .into());
    }
    if data.len() != timestamps.len() {
        warn!("forecast_arima called with mismatched data and timestamps lengths.");
        return Err(InternalForecastError::InputError(
            "Data and timestamps lengths do not match.".to_string(),
        )
        .into());
    }
    if periods_to_forecast == 0 {
        debug!("forecast_arima called with periods_to_forecast = 0. Returning empty vec.");
        return Ok(Vec::new());
    }

    let (p, d, q) = order;

    // Enhanced minimum data requirement calculation
    let min_theoretical = p + d + q + 1;
    let min_practical = (p.max(1) + q.max(1) + d + 10).max(20);
    let suggested_min_len = min_practical.max(min_theoretical * 3); // Conservative estimate

    if data.len() < suggested_min_len {
        warn!(
            "forecast_arima: Data length {} is less than suggested minimum of {} for order {:?}. Model may be unstable.",
            data.len(), suggested_min_len, order
        );
    }

    debug!(
        "Calling OxiDiviner arima_forecast_custom with data len: {}, timestamps len: {}, order: (p={}, d={}, q={}), periods: {}",
        data.len(), timestamps.len(), p, d, q, periods_to_forecast
    );

    // Enhanced data quality validation
    if data.iter().any(|&val| !val.is_finite()) {
        warn!("forecast_arima: Input data contains non-finite (NaN or Infinity) values.");
        return Err(InternalForecastError::InputError(
            "Input data contains non-finite values.".to_string(),
        )
        .into());
    }

    // Enhanced constant data detection with tighter tolerance
    if data.len() > 1 {
        let first_val = data[0];
        let tolerance = 1e-14; // Increased precision
        if data.iter().all(|&val| (val - first_val).abs() < tolerance) {
            warn!("forecast_arima: Constant data detected (tolerance: {}). Returning constant forecasts.", tolerance);
            return Ok(vec![first_val; periods_to_forecast]);
        }
    }

    // Outlier detection and handling
    let cleaned_data = detect_and_handle_outliers(&data)?;
    let use_cleaned = !cleaned_data.is_empty() && cleaned_data.len() == data.len();
    let final_data = if use_cleaned { &cleaned_data } else { &data };

    // Data stability check
    if let Err(stability_error) = validate_data_stability(&final_data) {
        warn!(
            "forecast_arima: Data stability check failed: {}",
            stability_error
        );
        // Continue with warning but monitor results more carefully
    }

    match oxidiviner::quick::arima_forecast_custom(
        timestamps,
        final_data.clone(),
        periods_to_forecast,
        p,
        d,
        q,
    ) {
        Ok(forecasts) => {
            // Enhanced forecast validation
            if forecasts.iter().any(|&f| !f.is_finite()) {
                warn!("ARIMA forecast from OxiDiviner produced NaN or Infinite values. Order: {:?}. Replacing with error.", order);
                return Err(InternalForecastError::ArimaError(
                    "Forecast resulted in NaN or Infinite values.".to_string(),
                )
                .into());
            }

            // Forecast reasonableness check
            if let Err(reasonableness_error) = validate_forecast_reasonableness(&data, &forecasts) {
                warn!(
                    "forecast_arima: Forecast reasonableness check failed: {}",
                    reasonableness_error
                );
                // Continue with warning - may still be valid in volatile markets
            }

            debug!(
                "OxiDiviner arima_forecast_custom successful. Forecast len: {}",
                forecasts.len()
            );
            Ok(forecasts)
        }
        Err(e) => {
            error!(
                "OxiDiviner arima_forecast_custom failed. Order: {:?}. Error: {}",
                order, e
            );
            Err(InternalForecastError::from(e).into())
        }
    }
}

/// Detect and handle outliers in time series data using IQR method
/// Returns cleaned data or empty vector if no outliers detected
fn detect_and_handle_outliers(data: &[f64]) -> NyxsOwlResult<Vec<f64>> {
    if data.len() < 10 {
        return Ok(Vec::new()); // Not enough data for meaningful outlier detection
    }

    // Calculate rolling statistics for outlier detection
    let window_size = (data.len() / 10).max(5).min(20);
    let mut outlier_indices = Vec::new();

    for i in window_size..data.len() - window_size {
        let start = i - window_size;
        let end = i + window_size + 1;
        let window = &data[start..end];

        // Calculate IQR for the window
        let mut sorted_window = window.to_vec();
        sorted_window.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_idx = sorted_window.len() / 4;
        let q3_idx = 3 * sorted_window.len() / 4;
        let q1 = sorted_window[q1_idx];
        let q3 = sorted_window[q3_idx];
        let iqr = q3 - q1;

        // Outlier detection with 2.0 IQR threshold (more conservative)
        let lower_bound = q1 - 2.0 * iqr;
        let upper_bound = q3 + 2.0 * iqr;

        if data[i] < lower_bound || data[i] > upper_bound {
            outlier_indices.push(i);
        }
    }

    if outlier_indices.is_empty() {
        return Ok(Vec::new()); // No outliers detected
    }

    // Create cleaned data by replacing outliers with median of surrounding values
    let mut cleaned = data.to_vec();
    for &idx in &outlier_indices {
        let start = idx.saturating_sub(3);
        let end = (idx + 4).min(data.len());
        let mut neighbors: Vec<f64> = data[start..end]
            .iter()
            .enumerate()
            .filter(|(i, _)| start + i != idx) // Exclude the outlier itself
            .map(|(_, &val)| val)
            .collect();

        if !neighbors.is_empty() {
            neighbors.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = neighbors[neighbors.len() / 2];
            cleaned[idx] = median;
            debug!(
                "Replaced outlier at index {}: {} -> {}",
                idx, data[idx], median
            );
        }
    }

    debug!(
        "Detected and corrected {} outliers in time series data",
        outlier_indices.len()
    );
    Ok(cleaned)
}

/// Validate data stability for ARIMA modeling
fn validate_data_stability(data: &[f64]) -> Result<(), String> {
    if data.len() < 10 {
        return Err("Insufficient data for stability analysis".to_string());
    }

    // Check for extreme volatility that might indicate non-stationarity
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
    let std_dev = variance.sqrt();

    // Check coefficient of variation (relative volatility)
    if mean != 0.0 {
        let cv = std_dev / mean.abs();
        if cv > 2.0 {
            // Very high relative volatility
            return Err(format!("High relative volatility detected (CV: {:.3})", cv));
        }
    }

    // Check for trend in variance (heteroskedasticity)
    let mid_point = data.len() / 2;
    let first_half = &data[..mid_point];
    let second_half = &data[mid_point..];

    let var1 = calculate_variance(first_half);
    let var2 = calculate_variance(second_half);

    if var1 > 0.0 && var2 > 0.0 {
        let variance_ratio = var2 / var1;
        if variance_ratio > 4.0 || variance_ratio < 0.25 {
            return Err(format!(
                "Significant variance change detected (ratio: {:.3})",
                variance_ratio
            ));
        }
    }

    Ok(())
}

/// Calculate variance of a data slice
fn calculate_variance(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }

    let mean = data.iter().sum::<f64>() / data.len() as f64;
    data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64
}

/// Validate forecast reasonableness compared to historical data
fn validate_forecast_reasonableness(historical: &[f64], forecasts: &[f64]) -> Result<(), String> {
    if historical.is_empty() || forecasts.is_empty() {
        return Ok(());
    }

    // Calculate historical statistics
    let hist_mean = historical.iter().sum::<f64>() / historical.len() as f64;
    let hist_std = calculate_variance(historical).sqrt();

    // Check if forecasts are within reasonable bounds
    for (i, &forecast) in forecasts.iter().enumerate() {
        // Allow forecasts within 5 standard deviations of historical mean
        let reasonable_bound = 5.0 * hist_std;
        if (forecast - hist_mean).abs() > reasonable_bound {
            return Err(format!(
                "Forecast {} at step {} outside reasonable bounds ({}σ from mean)",
                forecast,
                i,
                (forecast - hist_mean).abs() / hist_std
            ));
        }

        // Check for extreme jumps between consecutive forecasts
        if i > 0 {
            let forecast_change = (forecast - forecasts[i - 1]).abs();
            let max_reasonable_change = 3.0 * hist_std; // Allow 3σ change between periods
            if forecast_change > max_reasonable_change {
                return Err(format!(
                    "Extreme forecast jump detected between periods {} and {}: {}",
                    i - 1,
                    i,
                    forecast_change
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // Use the crate-wide NyxsOwlError for tests
    use crate::simple_types::NyxsOwlError as CrateNyxsOwlError;
    use chrono::{Duration, TimeZone};

    // Helper to check for errors that originated from forecast logic (DataError or StrategyError)
    fn is_forecast_related_error(err: &CrateNyxsOwlError) -> bool {
        matches!(
            err,
            CrateNyxsOwlError::DataError(_) | CrateNyxsOwlError::StrategyError(_)
        )
    }

    fn create_sample_timestamps(n: usize) -> Vec<DateTime<Utc>> {
        let start_time = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        (0..n)
            .map(|i| start_time + Duration::days(i as i64))
            .collect()
    }

    #[test]
    fn test_forecast_arima_simple_success() {
        let n = 30;
        let timestamps = create_sample_timestamps(n);
        let data: Vec<f64> = (1..=n)
            .map(|x| x as f64 * 1.5 + ((x % 5) as f64 * 0.5))
            .collect();
        let order = (1, 1, 0);
        let periods = 3;
        match forecast_arima(timestamps, data, order, periods) {
            Ok(forecasts) => {
                assert_eq!(forecasts.len(), periods);
                assert!(!forecasts.iter().any(|f| !f.is_finite()));
                println!("Forecasts {:?} for simple series: {:?}", order, forecasts);
            }
            Err(e) => panic!(
                "forecast_arima_custom failed for simple success case: {:?}",
                e
            ),
        }
    }

    #[test]
    fn test_forecast_arima_pdq_success() {
        let n = 40;
        let timestamps = create_sample_timestamps(n);
        let data: Vec<f64> = vec![
            10.0, 10.2, 10.4, 10.3, 10.5, 10.6, 10.8, 10.7, 10.9, 11.0, 11.2, 11.3, 11.5, 11.4,
            11.6, 11.7, 11.9, 11.8, 12.0, 12.2, 12.1, 12.3, 12.4, 12.6, 12.5, 12.7, 12.8, 13.0,
            12.9, 13.1, 13.3, 13.2, 13.4, 13.5, 13.7, 13.6, 13.8, 13.9, 14.1, 14.0,
        ];
        let order = (2, 1, 1);
        let periods = 5;
        match forecast_arima(timestamps, data, order, periods) {
            Ok(forecasts) => {
                assert_eq!(forecasts.len(), periods);
                assert!(!forecasts.iter().any(|f| !f.is_finite()));
                println!("Forecasts {:?} for sample series: {:?}", order, forecasts);
            }
            Err(e) => panic!(
                "forecast_arima_custom failed for ARIMA(2,1,1) case: {:?}",
                e
            ),
        }
    }

    #[test]
    fn test_forecast_arima_empty_data_or_timestamps() {
        let timestamps = create_sample_timestamps(10);
        let data: Vec<f64> = (1..=10).map(|x| x as f64).collect();

        match forecast_arima(vec![], data.clone(), (1, 0, 0), 1) {
            Err(ref e) if is_forecast_related_error(e) => {
                assert!(e
                    .to_string()
                    .contains("Input data or timestamps slice is empty"));
            }
            res => panic!("Expected DataError for empty timestamps, got {:?}", res),
        }
        match forecast_arima(timestamps.clone(), vec![], (1, 0, 0), 1) {
            Err(ref e) if is_forecast_related_error(e) => {
                assert!(e
                    .to_string()
                    .contains("Input data or timestamps slice is empty"));
            }
            res => panic!("Expected DataError for empty data, got {:?}", res),
        }
    }

    #[test]
    fn test_forecast_arima_mismatched_lengths() {
        let timestamps = create_sample_timestamps(10);
        let data: Vec<f64> = (1..=5).map(|x| x as f64).collect();
        match forecast_arima(timestamps, data, (1, 0, 0), 1) {
            Err(ref e) if is_forecast_related_error(e) => {
                assert!(e
                    .to_string()
                    .contains("Data and timestamps lengths do not match"));
            }
            res => panic!("Expected DataError for mismatched lengths, got {:?}", res),
        }
    }

    #[test]
    fn test_forecast_arima_zero_periods() {
        let timestamps = create_sample_timestamps(10);
        let data: Vec<f64> = (1..=10).map(|x| x as f64).collect();
        match forecast_arima(timestamps, data, (1, 0, 0), 0) {
            Ok(forecasts) => assert!(forecasts.is_empty()),
            Err(e) => panic!("Expected Ok with empty vec for zero periods, got {:?}", e),
        }
    }

    #[test]
    fn test_forecast_arima_insufficient_data_for_order() {
        let timestamps = create_sample_timestamps(2);
        let data: Vec<f64> = vec![1.0, 2.0];
        let order = (2, 1, 1);
        match forecast_arima(timestamps, data, order, 1) {
            Err(ref e) if is_forecast_related_error(e) => {
                println!(
                    "Received expected forecast error for insufficient data: {}",
                    e
                );
                let err_string = e.to_string().to_lowercase();
                assert!(
                    err_string.contains("oxidiviner")
                        || err_string.contains("model")
                        || err_string.contains("arima")
                        || err_string.contains("series length")
                        || err_string.contains("underlying model error")
                );
            }
            res => panic!(
                "Expected StrategyError or DataError for insufficient data, got {:?}",
                res
            ),
        }
    }

    #[test]
    fn test_forecast_arima_non_finite_input_data() {
        let timestamps = create_sample_timestamps(5);
        let data: Vec<f64> = vec![1.0, 2.0, f64::NAN, 4.0, 5.0];
        match forecast_arima(timestamps, data, (1, 0, 0), 1) {
            Err(ref e) if is_forecast_related_error(e) => {
                assert!(e
                    .to_string()
                    .contains("Input data contains non-finite values"));
            }
            res => panic!(
                "Expected DataError for non-finite input data, got {:?}",
                res
            ),
        }
    }

    #[test]
    fn test_constant_data_leading_to_potential_fit_issues() {
        let n = 30;
        let timestamps = create_sample_timestamps(n);
        let data: Vec<f64> = vec![5.0; n];
        let order = (1, 0, 0);
        let periods = 3;

        match forecast_arima(timestamps.clone(), data.clone(), order, periods) {
            Ok(forecasts) => {
                assert_eq!(forecasts.len(), periods);
                assert!(!forecasts.iter().any(|f| !f.is_finite()));
                println!(
                    "Forecasts for constant data (order {:?}): {:?}",
                    order, forecasts
                );
                // OxiDiviner with AR(1,0,0) on constant data might forecast the constant value.
                // Or could error depending on matrix inversions etc. if variance is zero.
                // Current OxiDiviner (1.1.0) seems to handle constant data for AR(1) by forecasting the constant.
                assert!(forecasts.iter().all(|&f| (f - 5.0).abs() < 1e-9));
            }
            Err(e) => {
                // Some ARIMA implementations might error on constant data due to zero variance.
                // If OxiDiviner errors, this test might need to expect an error.
                warn!("ARIMA on constant data resulted in error (order {:?}): {:?}. This might be backend specific.", order, e);
                // Allow specific errors if library changes, but prefer Ok for robust libraries.
                // For now, panic if it's not Ok, as OxiDiviner seems to handle it.
                panic!(
                    "forecast_arima_custom failed for constant data case: {:?}",
                    e
                )
            }
        }

        // Test with d=1 which should make it non-constant for fitting
        let order_d1 = (1, 1, 0);
        match forecast_arima(timestamps, data, order_d1, periods) {
            Ok(forecasts) => {
                assert_eq!(forecasts.len(), periods);
                assert!(!forecasts.iter().any(|f| !f.is_finite()));
                // After differencing constant data, it becomes zeros (except first NA). ARIMA on zeros should forecast zero.
                // Original data was 5.0. If forecast is for diff, it's 0. If for original scale, it's 5.0.
                // oxidiviner::quick::arima_forecast_custom returns forecasts on the original scale.
                assert!(
                    forecasts.iter().all(|&f| (f - 5.0).abs() < 1e-9),
                    "Forecasts: {:?}",
                    forecasts
                );
                println!(
                    "Forecasts for constant data (order {:?}): {:?}",
                    order_d1, forecasts
                );
            }
            Err(e) => panic!(
                "forecast_arima_custom failed for constant data with d=1 (order {:?}): {:?}",
                order_d1, e
            ),
        }
    }
}
