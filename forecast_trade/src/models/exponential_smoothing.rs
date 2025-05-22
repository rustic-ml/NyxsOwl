//! Exponential smoothing model for time series forecasting

use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::{ErrorMetrics, ForecastModel, ForecastResult};
use crate::strategies::TimeGranularity;

/// Simple Exponential Smoothing model
#[derive(Debug, Clone)]
pub struct ExponentialSmoothing {
    /// Smoothing factor alpha (0 < alpha < 1)
    alpha: f64,
    /// Time granularity
    time_granularity: TimeGranularity,
    /// Current smoothed value
    level: Option<f64>,
}

impl ExponentialSmoothing {
    /// Create a new exponential smoothing model with the specified alpha
    pub fn new(alpha: f64) -> Result<Self> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(ForecastError::InvalidParameter(
                "Alpha must be between 0 and 1".to_string(),
            ));
        }

        Ok(Self {
            alpha,
            time_granularity: TimeGranularity::Daily,
            level: None,
        })
    }

    /// Create a new exponential smoothing model for minute data
    pub fn new_minute(alpha: f64) -> Result<Self> {
        let mut model = Self::new(alpha)?;
        model.time_granularity = TimeGranularity::Minute;
        Ok(model)
    }

    /// Create a new exponential smoothing model with default parameters for the given granularity
    pub fn with_default_params(granularity: TimeGranularity) -> Result<Self> {
        match granularity {
            TimeGranularity::Daily => Self::new(0.2), // Lower alpha for daily data (less responsive)
            TimeGranularity::Minute => Self::new(0.4), // Higher alpha for minute data (more responsive)
        }
    }
}

impl ForecastModel for ExponentialSmoothing {
    fn train(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>> {
        if data.is_empty() {
            return Err(ForecastError::DataError("Empty data".to_string()));
        }

        let prices = data.close_prices();
        let mut model = self.clone();
        
        // Initialize level with first observation
        let mut level = prices[0];
        
        // Apply exponential smoothing
        for &price in &prices[1..] {
            level = self.alpha * price + (1.0 - self.alpha) * level;
        }
        
        model.level = Some(level);
        
        Ok(Box::new(model))
    }
    
    fn forecast(&self, data: &TimeSeriesData, periods: usize) -> Result<ForecastResult> {
        if data.is_empty() {
            return Err(ForecastError::DataError("Empty data".to_string()));
        }
        
        // If not trained, use the last value as the level
        let level = self.level.unwrap_or_else(|| {
            let prices = data.close_prices();
            prices[prices.len() - 1]
        });
        
        // For simple exponential smoothing, forecast is constant
        let values = vec![level; periods];
        
        // Calculate in-sample error metrics for the provided data
        let prices = data.close_prices();
        let mut forecasts = Vec::with_capacity(prices.len());
        let mut current_level = prices[0];
        forecasts.push(current_level);
        
        for i in 1..prices.len() {
            current_level = self.alpha * prices[i] + (1.0 - self.alpha) * current_level;
            forecasts.push(current_level);
        }
        
        // Calculate error metrics
        let mut sum_abs_error = 0.0;
        let mut sum_squared_error = 0.0;
        let mut sum_abs_pct_error = 0.0;
        
        for i in 1..prices.len() {
            let error = prices[i] - forecasts[i - 1];
            sum_abs_error += error.abs();
            sum_squared_error += error * error;
            
            if prices[i] != 0.0 {
                sum_abs_pct_error += (error / prices[i]).abs();
            }
        }
        
        let n = prices.len() - 1;
        let mae = sum_abs_error / n as f64;
        let mse = sum_squared_error / n as f64;
        let rmse = mse.sqrt();
        let mape = sum_abs_pct_error / n as f64 * 100.0;
        
        let error_metrics = ErrorMetrics {
            mae,
            mse,
            rmse,
            mape,
        };
        
        Ok(ForecastResult {
            values,
            confidence_intervals: None,
            error_metrics: Some(error_metrics),
        })
    }
    
    fn validate(&self, train_data: &TimeSeriesData, test_data: &TimeSeriesData) -> Result<ErrorMetrics> {
        if train_data.is_empty() || test_data.is_empty() {
            return Err(ForecastError::DataError("Empty data".to_string()));
        }
        
        // Train on training data
        let trained_model = self.train(train_data)?;
        
        // Forecast for the test period
        let forecast_result = trained_model.forecast(train_data, test_data.len())?;
        
        // Calculate error metrics
        let test_prices = test_data.close_prices();
        let forecasts = forecast_result.values;
        
        let mut sum_abs_error = 0.0;
        let mut sum_squared_error = 0.0;
        let mut sum_abs_pct_error = 0.0;
        
        for i in 0..test_prices.len() {
            let error = test_prices[i] - forecasts[i];
            sum_abs_error += error.abs();
            sum_squared_error += error * error;
            
            if test_prices[i] != 0.0 {
                sum_abs_pct_error += (error / test_prices[i]).abs();
            }
        }
        
        let n = test_prices.len();
        let mae = sum_abs_error / n as f64;
        let mse = sum_squared_error / n as f64;
        let rmse = mse.sqrt();
        let mape = sum_abs_pct_error / n as f64 * 100.0;
        
        Ok(ErrorMetrics {
            mae,
            mse,
            rmse,
            mape,
        })
    }
    
    fn time_granularity(&self) -> TimeGranularity {
        self.time_granularity
    }
    
    fn with_granularity(mut self, granularity: TimeGranularity) -> Self {
        let _ = self.adjust_for_granularity(granularity);
        self
    }
    
    fn adjust_for_granularity(&mut self, granularity: TimeGranularity) -> Result<()> {
        self.time_granularity = granularity;
        
        // Adjust alpha based on granularity
        self.alpha = match granularity {
            TimeGranularity::Daily => {
                // If alpha is too high for daily data, reduce it
                if self.alpha > 0.3 {
                    0.2
                } else {
                    self.alpha
                }
            }
            TimeGranularity::Minute => {
                // If alpha is too low for minute data, increase it
                if self.alpha < 0.3 {
                    0.4
                } else {
                    self.alpha
                }
            }
        };
        
        Ok(())
    }
}
