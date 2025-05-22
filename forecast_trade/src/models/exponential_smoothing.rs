//! Exponential smoothing models for time series forecasting

use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::{ForecastModel, ForecastResult, TrainedForecastModel};

/// Simple exponential smoothing model
#[derive(Debug, Clone)]
pub struct ExponentialSmoothing {
    /// Name of the model
    name: String,
    /// Smoothing parameter
    alpha: f64,
}

/// Trained exponential smoothing model
#[derive(Debug, Clone)]
pub struct TrainedExponentialSmoothing {
    /// Name of the model
    name: String,
    /// Smoothing parameter
    alpha: f64,
    /// Current level
    level: f64,
    /// Last observed value
    last_value: f64,
}

impl ExponentialSmoothing {
    /// Create a new exponential smoothing model
    pub fn new(alpha: f64) -> Result<Self> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(ForecastError::InvalidParameter(
                "Alpha must be between 0 and 1".to_string(),
            ));
        }

        Ok(Self {
            name: format!("Exponential Smoothing (alpha={})", alpha),
            alpha,
        })
    }
}

impl ForecastModel for ExponentialSmoothing {
    type Trained = TrainedExponentialSmoothing;

    fn train(&self, data: &TimeSeriesData) -> Result<Self::Trained> {
        let prices = data.close_prices();
        if prices.is_empty() {
            return Err(ForecastError::DataError(
                "Empty time series data".to_string(),
            ));
        }

        // Initialize level with first observation
        let mut level = prices[0];

        // Update level using exponential smoothing formula
        for &value in &prices[1..] {
            level = self.alpha * value + (1.0 - self.alpha) * level;
        }

        // Return trained model
        Ok(TrainedExponentialSmoothing {
            name: self.name.clone(),
            alpha: self.alpha,
            level,
            last_value: *prices.last().unwrap(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl TrainedForecastModel for TrainedExponentialSmoothing {
    fn forecast(&self, horizon: usize) -> Result<ForecastResult> {
        // In simple exponential smoothing, the forecast is constant at the last level
        let values = vec![self.level; horizon];

        Ok(ForecastResult::new(values, horizon)?)
    }

    fn predict(&self, data: &TimeSeriesData) -> Result<ForecastResult> {
        let prices = data.close_prices();
        if prices.is_empty() {
            return Err(ForecastError::DataError(
                "Empty time series data".to_string(),
            ));
        }

        let mut predictions = Vec::with_capacity(prices.len());
        let mut current_level = prices[0];

        // First prediction is just the first observation
        predictions.push(current_level);

        // Generate one-step-ahead predictions
        for i in 1..prices.len() {
            // Update level using the current observation
            current_level = self.alpha * prices[i - 1] + (1.0 - self.alpha) * current_level;

            // Prediction for next step is the current level
            predictions.push(current_level);
        }

        Ok(ForecastResult::new(predictions.clone(), predictions.len())?)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
