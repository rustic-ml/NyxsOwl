//! Moving average models for time series forecasting

use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::{ErrorMetrics, ForecastModel, ForecastResult};
use crate::strategies::TimeGranularity;

/// Simple Moving Average model
#[derive(Debug, Clone)]
pub struct MovingAverage {
    /// Window size for the moving average
    window_size: usize,
    /// Time granularity
    time_granularity: TimeGranularity,
}

impl MovingAverage {
    /// Create a new moving average model with the specified window size
    pub fn new(window_size: usize) -> Result<Self> {
        if window_size == 0 {
            return Err(ForecastError::InvalidParameter(
                "Window size must be greater than 0".to_string(),
            ));
        }

        Ok(Self {
            window_size,
            time_granularity: TimeGranularity::Daily,
        })
    }

    /// Create a new moving average model for minute data
    pub fn new_minute(window_size: usize) -> Result<Self> {
        let mut model = Self::new(window_size)?;
        model.time_granularity = TimeGranularity::Minute;
        Ok(model)
    }

    /// Create a new moving average with default parameters for the given granularity
    pub fn with_default_params(granularity: TimeGranularity) -> Result<Self> {
        match granularity {
            TimeGranularity::Daily => Self::new(20),   // 20-day moving average
            TimeGranularity::Minute => Self::new(60),  // 60-minute moving average
        }
    }
}

impl ForecastModel for MovingAverage {
    fn train(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>> {
        if data.is_empty() {
            return Err(ForecastError::DataError("Empty data".to_string()));
        }

        if data.len() < self.window_size {
            return Err(ForecastError::DataError(format!(
                "Not enough data points. Need at least {} points for the window size.",
                self.window_size
            )));
        }

        // No additional training needed for moving average
        Ok(Box::new(self.clone()))
    }

    fn forecast(&self, data: &TimeSeriesData, periods: usize) -> Result<ForecastResult> {
        if data.is_empty() {
            return Err(ForecastError::DataError("Empty data".to_string()));
        }

        if data.len() < self.window_size {
            return Err(ForecastError::DataError(format!(
                "Not enough data points. Need at least {} points for the window size.",
                self.window_size
            )));
        }

        let prices = data.close_prices();
        
        // Calculate the last moving average
        let last_window = &prices[prices.len() - self.window_size..];
        let last_ma = last_window.iter().sum::<f64>() / self.window_size as f64;
        
        // For simple moving average, forecast is constant
        let values = vec![last_ma; periods];
        
        // Calculate in-sample error metrics
        let mut forecasts = Vec::with_capacity(prices.len());
        
        // Fill initial points with NaN equivalent (not enough data for MA)
        for _ in 0..self.window_size - 1 {
            forecasts.push(f64::NAN);
        }
        
        // Calculate moving averages
        for i in self.window_size - 1..prices.len() {
            let window = &prices[i - (self.window_size - 1)..=i];
            let ma = window.iter().sum::<f64>() / self.window_size as f64;
            forecasts.push(ma);
        }
        
        // Calculate error metrics (skipping initial NaN values)
        let mut sum_abs_error = 0.0;
        let mut sum_squared_error = 0.0;
        let mut sum_abs_pct_error = 0.0;
        let mut count = 0;
        
        for i in self.window_size..prices.len() {
            let error = prices[i] - forecasts[i - 1];
            sum_abs_error += error.abs();
            sum_squared_error += error * error;
            
            if prices[i] != 0.0 {
                sum_abs_pct_error += (error / prices[i]).abs();
            }
            
            count += 1;
        }
        
        let mae = sum_abs_error / count as f64;
        let mse = sum_squared_error / count as f64;
        let rmse = mse.sqrt();
        let mape = sum_abs_pct_error / count as f64 * 100.0;
        
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
        
        // Train on training data (no actual training for MA)
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
        
        // Adjust window size based on granularity
        self.window_size = match granularity {
            TimeGranularity::Daily => {
                // If window is too large for daily data, reduce it
                if self.window_size > 60 {
                    20 // Default to 20-day MA
                } else {
                    self.window_size
                }
            }
            TimeGranularity::Minute => {
                // If window is too small for minute data, increase it
                if self.window_size < 20 {
                    60 // Default to 60-minute MA
                } else {
                    self.window_size
                }
            }
        };
        
        Ok(())
    }
}
