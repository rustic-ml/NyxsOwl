//! Moving average models for time series forecasting

use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::{ForecastModel, ForecastResult, TrainedForecastModel};

/// Simple Moving Average model
#[derive(Debug, Clone)]
pub struct SimpleMA {
    /// Name of the model
    name: String,
    /// Window size
    window: usize,
}

/// Trained Simple Moving Average model
#[derive(Debug, Clone)]
pub struct TrainedSimpleMA {
    /// Name of the model
    name: String,
    /// Window size
    window: usize,
    /// Historical data
    historical_data: Vec<f64>,
    /// Last calculated average
    last_average: f64,
}

/// Exponential Moving Average model
#[derive(Debug, Clone)]
pub struct ExponentialMA {
    /// Name of the model
    name: String,
    /// Smoothing factor
    alpha: f64,
}

/// Trained Exponential Moving Average model
#[derive(Debug, Clone)]
pub struct TrainedExponentialMA {
    /// Name of the model
    name: String,
    /// Smoothing factor
    alpha: f64,
    /// Historical data
    historical_data: Vec<f64>,
    /// Current value
    current_value: f64,
}

impl SimpleMA {
    /// Create a new Simple Moving Average model
    pub fn new(window: usize) -> Result<Self> {
        if window == 0 {
            return Err(ForecastError::InvalidParameter(
                "Window size must be positive".to_string(),
            ));
        }

        Ok(Self {
            name: format!("Simple Moving Average (window={})", window),
            window,
        })
    }
}

impl ForecastModel for SimpleMA {
    type Trained = TrainedSimpleMA;

    fn train(&self, data: &TimeSeriesData) -> Result<Self::Trained> {
        let prices = data.close_prices();
        if prices.len() < self.window {
            return Err(ForecastError::ValidationError(format!(
                "Insufficient data for SMA. Need at least {} observations.",
                self.window
            )));
        }

        // Calculate the last average
        let last_average =
            prices[prices.len() - self.window..].iter().sum::<f64>() / self.window as f64;

        Ok(TrainedSimpleMA {
            name: self.name.clone(),
            window: self.window,
            historical_data: prices.to_vec(),
            last_average,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl TrainedForecastModel for TrainedSimpleMA {
    fn forecast(&self, horizon: usize) -> Result<ForecastResult> {
        // For simple MA, the forecast is constant at the last average
        let values = vec![self.last_average; horizon];

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

        // For the first window-1 points, we can't make predictions yet
        for i in 0..self.window.min(prices.len()) {
            if i < self.window - 1 {
                predictions.push(prices[i]); // Just use the actual value
            } else {
                // Calculate moving average for points from 0 to i
                let ma = prices[i + 1 - self.window..=i].iter().sum::<f64>() / self.window as f64;
                predictions.push(ma);
            }
        }

        // For the rest, calculate moving averages
        for i in self.window..prices.len() {
            let ma = prices[i - self.window..i].iter().sum::<f64>() / self.window as f64;
            predictions.push(ma);
        }

        Ok(ForecastResult::new(predictions.clone(), predictions.len())?)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl ExponentialMA {
    /// Create a new Exponential Moving Average model
    pub fn new(alpha: f64) -> Result<Self> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(ForecastError::InvalidParameter(
                "Alpha must be between 0 and 1".to_string(),
            ));
        }

        Ok(Self {
            name: format!("Exponential Moving Average (alpha={})", alpha),
            alpha,
        })
    }
}

impl ForecastModel for ExponentialMA {
    type Trained = TrainedExponentialMA;

    fn train(&self, data: &TimeSeriesData) -> Result<Self::Trained> {
        let prices = data.close_prices();
        if prices.is_empty() {
            return Err(ForecastError::DataError(
                "Empty time series data".to_string(),
            ));
        }

        // Initialize with first value
        let mut current_value = prices[0];

        // Apply EMA formula
        for &price in &prices[1..] {
            current_value = self.alpha * price + (1.0 - self.alpha) * current_value;
        }

        Ok(TrainedExponentialMA {
            name: self.name.clone(),
            alpha: self.alpha,
            historical_data: prices.to_vec(),
            current_value,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl TrainedForecastModel for TrainedExponentialMA {
    fn forecast(&self, horizon: usize) -> Result<ForecastResult> {
        // For EMA, the forecast is constant at the last smoothed value
        let values = vec![self.current_value; horizon];

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
        let mut current_value = prices[0];

        // First prediction is the first value
        predictions.push(current_value);

        // Generate one-step-ahead predictions
        for i in 1..prices.len() {
            // Update current value with previous price
            current_value = self.alpha * prices[i - 1] + (1.0 - self.alpha) * current_value;

            // Prediction for next step
            predictions.push(current_value);
        }

        Ok(ForecastResult::new(predictions.clone(), predictions.len())?)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
