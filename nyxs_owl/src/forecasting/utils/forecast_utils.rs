//! Forecasting utility functions
//!
//! This module provides common utility functions used by forecasting strategies.

use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::*;

/// Extract numeric data from a DataFrame column
pub fn extract_numeric_series(df: &DataFrame, column_name: &str) -> Result<Vec<f64>> {
    let series = df.column(column_name).map_err(|e| {
        NyxsOwlError::DataError(format!("Column '{}' not found: {}", column_name, e))
    })?;

    match series.dtype() {
        DataType::Float64 => {
            let values: Vec<f64> = series
                .f64()
                .map_err(|e| NyxsOwlError::DataError(format!("Failed to cast to f64: {}", e)))?
                .into_no_null_iter()
                .collect();
            Ok(values)
        }
        DataType::Float32 => {
            let values: Vec<f64> = series
                .f32()
                .map_err(|e| NyxsOwlError::DataError(format!("Failed to cast to f32: {}", e)))?
                .into_no_null_iter()
                .map(|x| x as f64)
                .collect();
            Ok(values)
        }
        DataType::Int64 => {
            let values: Vec<f64> = series
                .i64()
                .map_err(|e| NyxsOwlError::DataError(format!("Failed to cast to i64: {}", e)))?
                .into_no_null_iter()
                .map(|x| x as f64)
                .collect();
            Ok(values)
        }
        DataType::Int32 => {
            let values: Vec<f64> = series
                .i32()
                .map_err(|e| NyxsOwlError::DataError(format!("Failed to cast to i32: {}", e)))?
                .into_no_null_iter()
                .map(|x| x as f64)
                .collect();
            Ok(values)
        }
        _ => Err(NyxsOwlError::DataError(format!(
            "Column '{}' must be numeric, found: {:?}",
            column_name,
            series.dtype()
        ))),
    }
}

/// Calculate percentage change between current and forecasted value
pub fn calculate_percentage_change(current: f64, forecast: f64) -> f64 {
    if current == 0.0 {
        0.0
    } else {
        (forecast - current) / current
    }
}

/// Convert percentage change to trading signal based on threshold
pub fn percentage_change_to_signal(pct_change: f64, threshold: f64) -> Signal {
    if pct_change > threshold {
        Signal::Buy
    } else if pct_change < -threshold {
        Signal::Sell
    } else {
        Signal::Hold
    }
}

/// Validate that a DataFrame has minimum required rows and specified columns
pub fn validate_dataframe(
    df: &DataFrame,
    required_columns: &[&str],
    min_rows: usize,
    strategy_name: &str,
) -> Result<()> {
    // Check minimum rows
    if df.height() < min_rows {
        return Err(NyxsOwlError::MissingData(format!(
            "{} strategy requires at least {} rows, got {}",
            strategy_name,
            min_rows,
            df.height()
        )));
    }

    // Check required columns exist
    for col_name in required_columns {
        if df.column(col_name).is_err() {
            return Err(NyxsOwlError::DataError(format!(
                "{} strategy requires column '{}', but it was not found. Available columns: {:?}",
                strategy_name,
                col_name,
                df.get_column_names()
            )));
        }
    }

    Ok(())
}

/// Create a rolling window iterator over a slice
pub fn rolling_windows<T>(data: &[T], window_size: usize) -> impl Iterator<Item = &[T]> {
    (window_size..=data.len()).map(move |i| &data[i - window_size..i])
}

/// Calculate simple statistics for a slice of f64 values
#[derive(Debug, Clone)]
pub struct SimpleStats {
    /// Mean (average) of the data values
    pub mean: f64,
    /// Standard deviation of the data values
    pub std: f64,
    /// Minimum value in the data
    pub min: f64,
    /// Maximum value in the data
    pub max: f64,
    /// Number of data points
    pub count: usize,
}

impl SimpleStats {
    /// Calculate statistics for a slice of f64 values
    ///
    /// # Arguments
    /// * `data` - Slice of f64 values to calculate statistics for
    ///
    /// # Returns
    /// A `SimpleStats` instance with calculated statistics or an error if data is empty
    pub fn calculate(data: &[f64]) -> Result<Self> {
        if data.is_empty() {
            return Err(NyxsOwlError::DataError(
                "Cannot calculate stats for empty data".to_string(),
            ));
        }

        let count = data.len();
        let sum: f64 = data.iter().sum();
        let mean = sum / count as f64;

        let variance: f64 = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / count as f64;
        let std = variance.sqrt();

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Ok(SimpleStats {
            mean,
            std,
            min,
            max,
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn create_test_dataframe() -> PolarsResult<DataFrame> {
        df! {
            "timestamp" => vec!["2023-01-01", "2023-01-02", "2023-01-03"],
            "close" => vec![100.0, 102.0, 98.0],
            "volume" => vec![1000i64, 1200i64, 800i64],
        }
    }

    #[test]
    fn test_extract_numeric_series() {
        let df = create_test_dataframe().unwrap();

        // Test f64 column
        let close_values = extract_numeric_series(&df, "close").unwrap();
        assert_eq!(close_values, vec![100.0, 102.0, 98.0]);

        // Test i64 column
        let volume_values = extract_numeric_series(&df, "volume").unwrap();
        assert_eq!(volume_values, vec![1000.0, 1200.0, 800.0]);

        // Test missing column
        let result = extract_numeric_series(&df, "missing");
        assert!(matches!(result, Err(NyxsOwlError::DataError(_))));
    }

    #[test]
    fn test_calculate_percentage_change() {
        assert_relative_eq!(calculate_percentage_change(100.0, 105.0), 0.05);
        assert_relative_eq!(calculate_percentage_change(100.0, 95.0), -0.05);
        assert_relative_eq!(calculate_percentage_change(0.0, 100.0), 0.0); // Edge case
        assert_relative_eq!(calculate_percentage_change(100.0, 100.0), 0.0);
    }

    #[test]
    fn test_percentage_change_to_signal() {
        let threshold = 0.02; // 2%

        assert_eq!(percentage_change_to_signal(0.03, threshold), Signal::Buy);
        assert_eq!(percentage_change_to_signal(-0.03, threshold), Signal::Sell);
        assert_eq!(percentage_change_to_signal(0.01, threshold), Signal::Hold);
        assert_eq!(percentage_change_to_signal(-0.01, threshold), Signal::Hold);
        assert_eq!(percentage_change_to_signal(0.02, threshold), Signal::Hold); // Exactly at threshold
    }

    #[test]
    fn test_validate_dataframe() {
        let df = create_test_dataframe().unwrap();

        // Valid case
        let result = validate_dataframe(&df, &["close", "volume"], 2, "TestStrategy");
        assert!(result.is_ok());

        // Missing column
        let result = validate_dataframe(&df, &["close", "missing"], 2, "TestStrategy");
        assert!(matches!(result, Err(NyxsOwlError::DataError(_))));

        // Insufficient rows
        let result = validate_dataframe(&df, &["close"], 10, "TestStrategy");
        assert!(matches!(result, Err(NyxsOwlError::MissingData(_))));
    }

    #[test]
    fn test_rolling_windows() {
        let data = vec![1, 2, 3, 4, 5];
        let windows: Vec<&[i32]> = rolling_windows(&data, 3).collect();

        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0], &[1, 2, 3]);
        assert_eq!(windows[1], &[2, 3, 4]);
        assert_eq!(windows[2], &[3, 4, 5]);
    }

    #[test]
    fn test_simple_stats() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = SimpleStats::calculate(&data).unwrap();

        assert_relative_eq!(stats.mean, 3.0);
        assert_relative_eq!(stats.min, 1.0);
        assert_relative_eq!(stats.max, 5.0);
        assert_eq!(stats.count, 5);
        assert!(stats.std > 0.0);

        // Test empty data
        let result = SimpleStats::calculate(&[]);
        assert!(matches!(result, Err(NyxsOwlError::DataError(_))));
    }
}
