//! Forecasting models for time series data

use crate::data::TimeSeriesData;
use crate::error::Result;
use std::fmt::Debug;

/// Forecast result containing predicted values
#[derive(Debug, Clone)]
pub struct ForecastResult {
    /// Forecasted values
    pub(crate) values: Vec<f64>,
    /// Number of periods forecasted
    horizons: usize,
    /// Confidence intervals (optional)
    pub(crate) intervals: Option<Vec<(f64, f64)>>,
    /// Timestamps (optional)
    pub(crate) timestamps: Option<Vec<chrono::DateTime<chrono::Utc>>>,
}

impl ForecastResult {
    /// Create a new forecast result
    pub fn new(values: Vec<f64>, horizons: usize) -> Result<Self> {
        if values.len() != horizons {
            return Err(crate::error::ForecastError::ValidationError(format!(
                "Values length ({}) doesn't match horizons ({})",
                values.len(),
                horizons
            )));
        }

        Ok(Self {
            values,
            horizons,
            intervals: None,
            timestamps: None,
        })
    }

    /// Create a new forecast result with confidence intervals
    pub fn new_with_intervals(
        values: Vec<f64>,
        horizons: usize,
        intervals: Vec<(f64, f64)>,
    ) -> Result<Self> {
        if values.len() != horizons {
            return Err(crate::error::ForecastError::ValidationError(format!(
                "Values length ({}) doesn't match horizons ({})",
                values.len(),
                horizons
            )));
        }

        if values.len() != intervals.len() {
            return Err(crate::error::ForecastError::ValidationError(format!(
                "Values length ({}) doesn't match intervals length ({})",
                values.len(),
                intervals.len()
            )));
        }

        Ok(Self {
            values,
            horizons,
            intervals: Some(intervals),
            timestamps: None,
        })
    }

    /// Get the forecasted values
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Get the number of periods forecasted
    pub fn horizons(&self) -> usize {
        self.horizons
    }

    /// Get the confidence intervals, if available
    pub fn intervals(&self) -> Option<&[(f64, f64)]> {
        self.intervals.as_deref()
    }

    /// Get the timestamps, if available
    pub fn timestamps(&self) -> Option<&[chrono::DateTime<chrono::Utc>]> {
        self.timestamps.as_deref()
    }

    /// Generate confidence intervals for the forecast
    pub fn confidence_intervals(&self, confidence_level: f64) -> Result<Vec<(f64, f64)>> {
        if confidence_level <= 0.0 || confidence_level >= 1.0 {
            return Err(crate::error::ForecastError::ValidationError(
                "Confidence level must be between 0 and 1".to_string(),
            ));
        }

        // For simplicity, use a fixed standard deviation multiplier based on confidence level
        // In a real implementation, this would be based on the t-distribution or normal distribution
        let z_score = match confidence_level {
            c if c >= 0.99 => 2.576, // 99% confidence
            c if c >= 0.95 => 1.96,  // 95% confidence
            c if c >= 0.90 => 1.645, // 90% confidence
            _ => 1.0,                // Default fallback
        };

        // Assume a simple model with constant variance for the forecast
        // In a real implementation, this would be model-specific
        let std_dev = 0.05; // 5% standard deviation

        let intervals: Vec<(f64, f64)> = self
            .values
            .iter()
            .map(|v| {
                let margin = z_score * std_dev * v.abs();
                (*v - margin, *v + margin)
            })
            .collect();

        Ok(intervals)
    }

    /// Calculate mean absolute error between forecast and actual values
    pub fn mean_absolute_error(&self, actual: &[f64]) -> Result<f64> {
        if self.values.len() != actual.len() {
            return Err(crate::error::ForecastError::ValidationError(format!(
                "Forecast length ({}) doesn't match actual length ({})",
                self.values.len(),
                actual.len()
            )));
        }

        let sum: f64 = self
            .values
            .iter()
            .zip(actual.iter())
            .map(|(f, a)| (f - a).abs())
            .sum();

        Ok(sum / self.values.len() as f64)
    }

    /// Calculate mean squared error between forecast and actual values
    pub fn mean_squared_error(&self, actual: &[f64]) -> Result<f64> {
        if self.values.len() != actual.len() {
            return Err(crate::error::ForecastError::ValidationError(format!(
                "Forecast length ({}) doesn't match actual length ({})",
                self.values.len(),
                actual.len()
            )));
        }

        let sum: f64 = self
            .values
            .iter()
            .zip(actual.iter())
            .map(|(f, a)| (f - a).powi(2))
            .sum();

        Ok(sum / self.values.len() as f64)
    }
}

/// Trained forecast model
pub trait TrainedForecastModel: Debug {
    /// Generate forecast for future periods
    fn forecast(&self, horizons: usize) -> Result<ForecastResult>;

    /// Predict values for the training data
    fn predict(&self, data: &TimeSeriesData) -> Result<ForecastResult>;

    /// Name of the model
    fn name(&self) -> &str;
}

/// Forecast model that can be trained on time series data
pub trait ForecastModel: Debug + Clone {
    /// The type of trained model produced
    type Trained: TrainedForecastModel;

    /// Train the model on time series data
    fn train(&self, data: &TimeSeriesData) -> Result<Self::Trained>;

    /// Get the name of the model
    fn name(&self) -> &str;
}

pub mod arima;
pub mod exponential_smoothing;
pub mod moving_average;
