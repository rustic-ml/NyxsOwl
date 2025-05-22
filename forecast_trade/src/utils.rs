//! Utility functions for the forecast_trade crate

use crate::error::{ForecastError, Result};
use crate::data::TimeSeriesData;
use chrono::{DateTime, Duration, Utc};

/// Split time series data into training and test sets
pub fn train_test_split(data: &[f64], test_ratio: f64) -> (Vec<f64>, Vec<f64>) {
    if data.is_empty() || test_ratio <= 0.0 || test_ratio >= 1.0 {
        return (data.to_vec(), Vec::new());
    }
    
    let test_size = (data.len() as f64 * test_ratio).round() as usize;
    let train_size = data.len() - test_size;
    
    let train = data[..train_size].to_vec();
    let test = data[train_size..].to_vec();
    
    (train, test)
}

/// Create future timestamps for forecasting
pub fn future_timestamps(last_timestamp: DateTime<Utc>, horizon: usize, frequency: &str) -> Result<Vec<DateTime<Utc>>> {
    let mut timestamps = Vec::with_capacity(horizon);
    let mut current = last_timestamp;
    
    let duration = match frequency {
        "daily" | "d" | "1d" => Duration::days(1),
        "weekly" | "w" | "1w" => Duration::weeks(1),
        "monthly" | "m" | "1m" => Duration::days(30),
        "hourly" | "h" | "1h" => Duration::hours(1),
        "minute" | "min" | "1min" => Duration::minutes(1),
        _ => return Err(ForecastError::ValidationError(format!("Unsupported frequency: {}", frequency))),
    };
    
    for _ in 0..horizon {
        current = current + duration;
        timestamps.push(current);
    }
    
    Ok(timestamps)
}

/// Calculate accuracy metrics for a forecast vs actual values
pub fn forecast_accuracy(forecast: &[f64], actual: &[f64]) -> Result<ForecastAccuracy> {
    if forecast.len() != actual.len() || forecast.is_empty() {
        return Err(ForecastError::ValidationError(
            "Forecast and actual values must have the same non-zero length".to_string(),
        ));
    }
    
    let n = forecast.len() as f64;
    
    // Calculate errors
    let errors: Vec<f64> = forecast.iter()
        .zip(actual.iter())
        .map(|(&f, &a)| a - f)
        .collect();
    
    // Mean Absolute Error
    let mae = errors.iter().map(|e| e.abs()).sum::<f64>() / n;
    
    // Mean Squared Error
    let mse = errors.iter().map(|e| e.powi(2)).sum::<f64>() / n;
    
    // Root Mean Squared Error
    let rmse = mse.sqrt();
    
    // Mean Absolute Percentage Error
    let mape = actual.iter()
        .zip(errors.iter())
        .filter(|(&a, _)| a != 0.0)
        .map(|(&a, &e)| (e.abs() / a.abs()) * 100.0)
        .sum::<f64>() / n;
    
    // Symmetric Mean Absolute Percentage Error
    let smape = actual.iter()
        .zip(forecast.iter())
        .map(|(&a, &f)| {
            let abs_a = a.abs();
            let abs_f = f.abs();
            if abs_a + abs_f == 0.0 {
                0.0
            } else {
                200.0 * (a - f).abs() / (abs_a + abs_f)
            }
        })
        .sum::<f64>() / n;
    
    Ok(ForecastAccuracy {
        mae,
        mse,
        rmse,
        mape,
        smape,
    })
}

/// Forecast accuracy metrics
#[derive(Debug, Clone)]
pub struct ForecastAccuracy {
    /// Mean Absolute Error
    pub mae: f64,
    /// Mean Squared Error
    pub mse: f64,
    /// Root Mean Squared Error
    pub rmse: f64,
    /// Mean Absolute Percentage Error
    pub mape: f64,
    /// Symmetric Mean Absolute Percentage Error
    pub smape: f64,
}

impl std::fmt::Display for ForecastAccuracy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Forecast Accuracy Metrics:")?;
        writeln!(f, "  MAE:   {:.4}", self.mae)?;
        writeln!(f, "  MSE:   {:.4}", self.mse)?;
        writeln!(f, "  RMSE:  {:.4}", self.rmse)?;
        writeln!(f, "  MAPE:  {:.4}%", self.mape)?;
        writeln!(f, "  SMAPE: {:.4}%", self.smape)?;
        Ok(())
    }
} 