//! Adapters for OxiDiviner forecasting models
//!
//! This module provides adapter implementations for the various forecasting models
//! provided by the OxiDiviner crate, making them compatible with our ForecastModel trait.

use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::{ErrorMetrics, ForecastModel, ForecastResult};
use crate::strategies::TimeGranularity;
use crate::impl_box_clone;

// Import the main crate
use oxidiviner::prelude::*;
// Import subcrates
extern crate oxidiviner_exponential_smoothing;
extern crate oxidiviner_moving_average;
extern crate oxidiviner_autoregressive;
extern crate oxidiviner_garch;

/// Adapter for OxiDiviner's exponential smoothing model
#[derive(Clone, Debug)]
pub struct ExponentialSmoothingAdapter {
    /// The alpha parameter (level smoothing)
    alpha: f64,
    /// The beta parameter (trend smoothing, optional)
    beta: Option<f64>,
    /// The gamma parameter (seasonal smoothing, optional)
    gamma: Option<f64>,
    /// Seasonal period (optional)
    seasonal_period: Option<usize>,
    /// The trained model (if available)
    model: Option<oxidiviner_exponential_smoothing::ExponentialSmoothingModel>,
    /// Time granularity for the model
    granularity: TimeGranularity,
}

impl ExponentialSmoothingAdapter {
    /// Create a new simple exponential smoothing adapter
    pub fn new(alpha: f64) -> Result<Self> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(ForecastError::InvalidParameter(
                "Alpha must be between 0 and 1".to_string(),
            ));
        }

        Ok(Self {
            alpha,
            beta: None,
            gamma: None,
            seasonal_period: None,
            model: None,
            granularity: TimeGranularity::Daily,
        })
    }
    
    /// Create a new Holt exponential smoothing adapter (with trend)
    pub fn holt(alpha: f64, beta: f64) -> Result<Self> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(ForecastError::InvalidParameter(
                "Alpha must be between 0 and 1".to_string(),
            ));
        }
        
        if beta <= 0.0 || beta >= 1.0 {
            return Err(ForecastError::InvalidParameter(
                "Beta must be between 0 and 1".to_string(),
            ));
        }

        Ok(Self {
            alpha,
            beta: Some(beta),
            gamma: None,
            seasonal_period: None,
            model: None,
            granularity: TimeGranularity::Daily,
        })
    }
    
    /// Create a new Holt-Winters exponential smoothing adapter (with trend and seasonality)
    pub fn holt_winters(alpha: f64, beta: f64, gamma: f64, seasonal_period: usize) -> Result<Self> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(ForecastError::InvalidParameter(
                "Alpha must be between 0 and 1".to_string(),
            ));
        }
        
        if beta <= 0.0 || beta >= 1.0 {
            return Err(ForecastError::InvalidParameter(
                "Beta must be between 0 and 1".to_string(),
            ));
        }
        
        if gamma <= 0.0 || gamma >= 1.0 {
            return Err(ForecastError::InvalidParameter(
                "Gamma must be between 0 and 1".to_string(),
            ));
        }
        
        if seasonal_period < 2 {
            return Err(ForecastError::InvalidParameter(
                "Seasonal period must be at least 2".to_string(),
            ));
        }

        Ok(Self {
            alpha,
            beta: Some(beta),
            gamma: Some(gamma),
            seasonal_period: Some(seasonal_period),
            model: None,
            granularity: TimeGranularity::Daily,
        })
    }
    
    /// Set the model's time granularity
    pub fn with_granularity(mut self, granularity: TimeGranularity) -> Self {
        self.granularity = granularity;
        self
    }
}

impl ForecastModel for ExponentialSmoothingAdapter {
    fn train(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>> {
        let values = data.close_prices();
        let dates = data.timestamps().to_vec();
        
        // Create and train the model
        let mut model = if let (Some(beta), Some(gamma), Some(period)) = (self.beta, self.gamma, self.seasonal_period) {
            // Holt-Winters
            oxidiviner_exponential_smoothing::ExponentialSmoothingModel::holt_winters_additive(
                self.alpha, beta, gamma, period, None
            ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ETS model: {}", e)))?
        } else if let Some(beta) = self.beta {
            // Holt
            oxidiviner_exponential_smoothing::ExponentialSmoothingModel::holt(
                self.alpha, beta, None
            ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ETS model: {}", e)))?
        } else {
            // Simple exponential smoothing
            oxidiviner_exponential_smoothing::ExponentialSmoothingModel::simple(
                self.alpha, None
            ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ETS model: {}", e)))?
        };
        
        // Create OxiDiviner TimeSeriesData
        let ox_data = oxidiviner::TimeSeriesData::new(
            dates, 
            values.to_vec(),
            "training_data"
        ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create time series: {}", e)))?;
        
        // Fit the model
        model.fit(&ox_data)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to train ETS model: {}", e)))?;
        
        let mut trained = self.clone();
        trained.model = Some(model);
        
        Ok(Box::new(trained))
    }
    
    fn forecast(&self, _data: &TimeSeriesData, periods: usize) -> Result<ForecastResult> {
        let model = if let Some(model) = &self.model {
            model
        } else {
            return Err(ForecastError::InvalidParameter(
                "Model not trained. Call train() first.".to_string(),
            ));
        };
        
        // Generate forecast
        let forecast = model.forecast(periods)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to generate forecast: {}", e)))?;
        
        Ok(ForecastResult {
            values: forecast,
            confidence_intervals: None,
            error_metrics: None,
        })
    }
    
    fn validate(&self, train_data: &TimeSeriesData, test_data: &TimeSeriesData) -> Result<ErrorMetrics> {
        let train_values = train_data.close_prices();
        let test_values = test_data.close_prices();
        let train_dates = train_data.timestamps().to_vec();
        
        // Create and train the model
        let mut model = if let (Some(beta), Some(gamma), Some(period)) = (self.beta, self.gamma, self.seasonal_period) {
            // Holt-Winters
            oxidiviner_exponential_smoothing::ExponentialSmoothingModel::holt_winters_additive(
                self.alpha, beta, gamma, period, None
            ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ETS model: {}", e)))?
        } else if let Some(beta) = self.beta {
            // Holt
            oxidiviner_exponential_smoothing::ExponentialSmoothingModel::holt(
                self.alpha, beta, None
            ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ETS model: {}", e)))?
        } else {
            // Simple exponential smoothing
            oxidiviner_exponential_smoothing::ExponentialSmoothingModel::simple(
                self.alpha, None
            ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ETS model: {}", e)))?
        };
        
        // Create OxiDiviner TimeSeriesData
        let ox_train_data = oxidiviner::TimeSeriesData::new(
            train_dates, 
            train_values.to_vec(),
            "train_data"
        ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create time series: {}", e)))?;
        
        // Fit model
        model.fit(&ox_train_data)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to train model: {}", e)))?;
        
        // Generate forecasts
        let forecasts = model.forecast(test_values.len())
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to generate forecast: {}", e)))?;
        
        // Calculate error metrics
        let mae = mean_absolute_error(&test_values, &forecasts);
        let mse = mean_squared_error(&test_values, &forecasts);
        let rmse = root_mean_squared_error(&test_values, &forecasts);
        let mape = mean_absolute_percentage_error(&test_values, &forecasts);
        
        Ok(ErrorMetrics {
            mae,
            mse,
            rmse,
            mape,
        })
    }
    
    fn time_granularity(&self) -> TimeGranularity {
        self.granularity
    }
    
    // Implement box_clone using the macro
    impl_box_clone!(ExponentialSmoothingAdapter);
    
    fn adjust_for_granularity(&mut self, granularity: TimeGranularity) -> Result<()> {
        self.granularity = granularity;
        
        // Adjust seasonal period if present
        if let Some(period) = self.seasonal_period {
            let new_period = match (self.granularity, granularity) {
                (TimeGranularity::Daily, TimeGranularity::Minute) => {
                    // Convert daily seasonality to minute
                    period * 24 * 60 // days to minutes
                },
                (TimeGranularity::Minute, TimeGranularity::Daily) => {
                    // Convert minute seasonality to daily
                    period / (24 * 60) // minutes to days, minimum 1
                },
                _ => period // No change needed
            };
            
            self.seasonal_period = Some(new_period.max(2)); // Ensure minimum seasonality of 2
        }
        
        Ok(())
    }
}

/// Adapter for OxiDiviner's moving average model
#[derive(Clone, Debug)]
pub struct MovingAverageAdapter {
    /// The window size for the moving average
    window_size: usize,
    /// The trained model (if available)
    model: Option<oxidiviner_moving_average::MovingAverageModel>,
    /// Time granularity for the model
    granularity: TimeGranularity,
}

impl MovingAverageAdapter {
    /// Create a new moving average adapter
    pub fn new(window_size: usize) -> Result<Self> {
        if window_size < 2 {
            return Err(ForecastError::InvalidParameter(
                "Window size must be at least 2".to_string(),
            ));
        }

        Ok(Self {
            window_size,
            model: None,
            granularity: TimeGranularity::Daily,
        })
    }
    
    /// Set the model's time granularity
    pub fn with_granularity(mut self, granularity: TimeGranularity) -> Self {
        self.granularity = granularity;
        self
    }
}

impl ForecastModel for MovingAverageAdapter {
    fn train(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>> {
        let values = data.close_prices();
        let dates = data.timestamps().to_vec();
        
        // Create model
        let mut model = oxidiviner_moving_average::MovingAverageModel::new(self.window_size)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to create MA model: {}", e)))?;
        
        // Create OxiDiviner TimeSeriesData
        let ox_data = oxidiviner::TimeSeriesData::new(
            dates, 
            values.to_vec(),
            "training_data"
        ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create time series: {}", e)))?;
        
        // Fit the model
        model.fit(&ox_data)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to train MA model: {}", e)))?;
        
        let mut trained = self.clone();
        trained.model = Some(model);
        
        Ok(Box::new(trained))
    }
    
    fn forecast(&self, _data: &TimeSeriesData, periods: usize) -> Result<ForecastResult> {
        let model = if let Some(model) = &self.model {
            model
        } else {
            return Err(ForecastError::InvalidParameter(
                "Model not trained. Call train() first.".to_string(),
            ));
        };
        
        // Generate forecast
        let forecast = model.forecast(periods)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to generate forecast: {}", e)))?;
        
        Ok(ForecastResult {
            values: forecast,
            confidence_intervals: None,
            error_metrics: None,
        })
    }
    
    fn validate(&self, train_data: &TimeSeriesData, test_data: &TimeSeriesData) -> Result<ErrorMetrics> {
        let train_values = train_data.close_prices();
        let test_values = test_data.close_prices();
        let train_dates = train_data.timestamps().to_vec();
        
        // Create model
        let mut model = oxidiviner_moving_average::MovingAverageModel::new(self.window_size)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to create MA model: {}", e)))?;
        
        // Create OxiDiviner TimeSeriesData
        let ox_train_data = oxidiviner::TimeSeriesData::new(
            train_dates, 
            train_values.to_vec(),
            "train_data"
        ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create time series: {}", e)))?;
        
        // Fit model
        model.fit(&ox_train_data)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to train model: {}", e)))?;
        
        // Generate forecasts
        let forecasts = model.forecast(test_values.len())
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to generate forecast: {}", e)))?;
        
        // Calculate error metrics
        let mae = mean_absolute_error(&test_values, &forecasts);
        let mse = mean_squared_error(&test_values, &forecasts);
        let rmse = root_mean_squared_error(&test_values, &forecasts);
        let mape = mean_absolute_percentage_error(&test_values, &forecasts);
        
        Ok(ErrorMetrics {
            mae,
            mse,
            rmse,
            mape,
        })
    }
    
    fn time_granularity(&self) -> TimeGranularity {
        self.granularity
    }
    
    // Implement box_clone using the macro
    impl_box_clone!(MovingAverageAdapter);
    
    fn adjust_for_granularity(&mut self, granularity: TimeGranularity) -> Result<()> {
        self.granularity = granularity;
        
        // Adjust window size based on granularity changes
        if self.granularity != granularity {
            match (self.granularity, granularity) {
                (TimeGranularity::Daily, TimeGranularity::Minute) => {
                    // Increase window size for minute data
                    self.window_size = self.window_size * 24;
                },
                (TimeGranularity::Minute, TimeGranularity::Daily) => {
                    // Decrease window size for daily data
                    self.window_size = (self.window_size / 24).max(2);
                },
                _ => () // No change needed
            }
        }
        
        Ok(())
    }
}

/// Adapter for OxiDiviner's ARIMA model
#[derive(Clone, Debug)]
pub struct ArimaAdapter {
    /// Autoregressive order (p)
    p: usize,
    /// Differencing order (d)
    d: usize,
    /// Moving average order (q)
    q: usize,
    /// Seasonal autoregressive order (P)
    seasonal_p: Option<usize>,
    /// Seasonal differencing order (D)
    seasonal_d: Option<usize>,
    /// Seasonal moving average order (Q)
    seasonal_q: Option<usize>,
    /// Seasonal period
    seasonal_period: Option<usize>,
    /// The trained model (if available)
    model: Option<oxidiviner_autoregressive::ARIMAModel>,
    /// Time granularity for the model
    granularity: TimeGranularity,
}

impl ArimaAdapter {
    /// Create a new ARIMA adapter with default parameters (ARIMA(1,1,1))
    pub fn new() -> Result<Self> {
        Ok(Self {
            p: 1,
            d: 1,
            q: 1,
            seasonal_p: None,
            seasonal_d: None,
            seasonal_q: None,
            seasonal_period: None,
            model: None,
            granularity: TimeGranularity::Daily,
        })
    }
    
    /// Create a new ARIMA adapter with custom parameters
    pub fn with_params(p: usize, d: usize, q: usize) -> Result<Self> {
        Ok(Self {
            p,
            d,
            q,
            seasonal_p: None,
            seasonal_d: None,
            seasonal_q: None,
            seasonal_period: None,
            model: None,
            granularity: TimeGranularity::Daily,
        })
    }
    
    /// Create a new seasonal ARIMA (SARIMA) adapter
    pub fn sarima(p: usize, d: usize, q: usize, seasonal_p: usize, seasonal_d: usize, seasonal_q: usize, seasonal_period: usize) -> Result<Self> {
        if seasonal_period < 2 {
            return Err(ForecastError::InvalidParameter(
                "Seasonal period must be at least 2".to_string(),
            ));
        }
        
        Ok(Self {
            p,
            d,
            q,
            seasonal_p: Some(seasonal_p),
            seasonal_d: Some(seasonal_d),
            seasonal_q: Some(seasonal_q),
            seasonal_period: Some(seasonal_period),
            model: None,
            granularity: TimeGranularity::Daily,
        })
    }
    
    /// Set the model's time granularity
    pub fn with_granularity(mut self, granularity: TimeGranularity) -> Self {
        self.granularity = granularity;
        self
    }
}

impl ForecastModel for ArimaAdapter {
    fn train(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>> {
        let values = data.close_prices();
        let dates = data.timestamps().to_vec();
        
        // Create model
        let mut model = if let (Some(sp), Some(sd), Some(sq), Some(period)) = (self.seasonal_p, self.seasonal_d, self.seasonal_q, self.seasonal_period) {
            // Seasonal ARIMA
            oxidiviner_autoregressive::ARIMAModel::with_seasonal(self.p, self.d, self.q, sp, sd, sq, period, true)
                .map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ARIMA model: {}", e)))?
        } else {
            // Regular ARIMA
            oxidiviner_autoregressive::ARIMAModel::new(self.p, self.d, self.q, true)
                .map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ARIMA model: {}", e)))?
        };
        
        // Create OxiDiviner TimeSeriesData
        let ox_data = oxidiviner::TimeSeriesData::new(
            dates, 
            values.to_vec(),
            "training_data"
        ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create time series: {}", e)))?;
        
        // Fit the model
        model.fit(&ox_data)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to train ARIMA model: {}", e)))?;
        
        let mut trained = self.clone();
        trained.model = Some(model);
        
        Ok(Box::new(trained))
    }
    
    fn forecast(&self, _data: &TimeSeriesData, periods: usize) -> Result<ForecastResult> {
        let model = if let Some(model) = &self.model {
            model
        } else {
            return Err(ForecastError::InvalidParameter(
                "Model not trained. Call train() first.".to_string(),
            ));
        };
        
        // Generate forecast
        let forecast = model.forecast(periods)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to generate forecast: {}", e)))?;
        
        Ok(ForecastResult {
            values: forecast,
            confidence_intervals: None,
            error_metrics: None,
        })
    }
    
    fn validate(&self, train_data: &TimeSeriesData, test_data: &TimeSeriesData) -> Result<ErrorMetrics> {
        let train_values = train_data.close_prices();
        let test_values = test_data.close_prices();
        let train_dates = train_data.timestamps().to_vec();
        
        // Create model
        let mut model = if let (Some(sp), Some(sd), Some(sq), Some(period)) = (self.seasonal_p, self.seasonal_d, self.seasonal_q, self.seasonal_period) {
            // Seasonal ARIMA
            oxidiviner_autoregressive::ARIMAModel::with_seasonal(self.p, self.d, self.q, sp, sd, sq, period, true)
                .map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ARIMA model: {}", e)))?
        } else {
            // Regular ARIMA
            oxidiviner_autoregressive::ARIMAModel::new(self.p, self.d, self.q, true)
                .map_err(|e| ForecastError::InvalidParameter(format!("Failed to create ARIMA model: {}", e)))?
        };
        
        // Create OxiDiviner TimeSeriesData
        let ox_train_data = oxidiviner::TimeSeriesData::new(
            train_dates, 
            train_values.to_vec(),
            "train_data"
        ).map_err(|e| ForecastError::InvalidParameter(format!("Failed to create time series: {}", e)))?;
        
        // Fit model
        model.fit(&ox_train_data)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to train model: {}", e)))?;
        
        // Generate forecasts
        let forecasts = model.forecast(test_values.len())
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to generate forecast: {}", e)))?;
        
        // Calculate error metrics
        let mae = mean_absolute_error(&test_values, &forecasts);
        let mse = mean_squared_error(&test_values, &forecasts);
        let rmse = root_mean_squared_error(&test_values, &forecasts);
        let mape = mean_absolute_percentage_error(&test_values, &forecasts);
        
        Ok(ErrorMetrics {
            mae,
            mse,
            rmse,
            mape,
        })
    }
    
    fn time_granularity(&self) -> TimeGranularity {
        self.granularity
    }
    
    // Implement box_clone using the macro
    impl_box_clone!(ArimaAdapter);
    
    fn adjust_for_granularity(&mut self, granularity: TimeGranularity) -> Result<()> {
        self.granularity = granularity;
        
        // Adjust seasonal period if present
        if let Some(period) = self.seasonal_period {
            let new_period = match (self.granularity, granularity) {
                (TimeGranularity::Daily, TimeGranularity::Minute) => {
                    // Convert daily seasonality to minute
                    period * 24 * 60 // days to minutes
                },
                (TimeGranularity::Minute, TimeGranularity::Daily) => {
                    // Convert minute seasonality to daily
                    period / (24 * 60) // minutes to days, minimum 1
                },
                _ => period // No change needed
            };
            
            self.seasonal_period = Some(new_period.max(2)); // Ensure minimum seasonality of 2
        }
        
        Ok(())
    }
}

/// Adapter for OxiDiviner's GARCH model
#[derive(Clone, Debug)]
pub struct GarchAdapter {
    /// ARCH order (p)
    p: usize,
    /// GARCH order (q)
    q: usize,
    /// The trained model (if available)
    model: Option<oxidiviner_garch::GARCHModel>,
    /// Time granularity for the model
    granularity: TimeGranularity,
}

impl GarchAdapter {
    /// Create a new GARCH adapter with default parameters (GARCH(1,1))
    pub fn new() -> Result<Self> {
        Ok(Self {
            p: 1,
            q: 1,
            model: None,
            granularity: TimeGranularity::Daily,
        })
    }
    
    /// Create a new GARCH adapter with custom parameters
    pub fn with_params(p: usize, q: usize) -> Result<Self> {
        Ok(Self {
            p,
            q,
            model: None,
            granularity: TimeGranularity::Daily,
        })
    }
    
    /// Set the model's time granularity
    pub fn with_granularity(mut self, granularity: TimeGranularity) -> Self {
        self.granularity = granularity;
        self
    }
}

impl ForecastModel for GarchAdapter {
    fn train(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>> {
        let values = data.close_prices();
        
        // Calculate returns
        let returns: Vec<f64> = values.windows(2)
            .map(|w| (w[1] / w[0]) - 1.0)
            .collect();
        
        // Create model
        let mut model = oxidiviner_garch::GARCHModel::new(self.p, self.q, None)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to create GARCH model: {}", e)))?;
        
        // Fit the model
        model.fit(&returns, None)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to train GARCH model: {}", e)))?;
        
        let mut trained = self.clone();
        trained.model = Some(model);
        
        Ok(Box::new(trained))
    }
    
    fn forecast(&self, data: &TimeSeriesData, periods: usize) -> Result<ForecastResult> {
        let values = data.close_prices();
        
        // Calculate returns
        let returns: Vec<f64> = values.windows(2)
            .map(|w| (w[1] / w[0]) - 1.0)
            .collect();
        
        let model = if let Some(model) = &self.model {
            model
        } else {
            return Err(ForecastError::InvalidParameter(
                "Model not trained. Call train() first.".to_string(),
            ));
        };
        
        // Generate volatility forecast
        let forecast = model.forecast_variance(periods)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to generate forecast: {}", e)))?;
        
        Ok(ForecastResult {
            values: forecast,
            confidence_intervals: None,
            error_metrics: None,
        })
    }
    
    fn validate(&self, train_data: &TimeSeriesData, test_data: &TimeSeriesData) -> Result<ErrorMetrics> {
        let train_values = train_data.close_prices();
        let test_values = test_data.close_prices();
        
        // Calculate returns
        let train_returns: Vec<f64> = train_values.windows(2)
            .map(|w| (w[1] / w[0]) - 1.0)
            .collect();
        
        // Create model
        let mut model = oxidiviner_garch::GARCHModel::new(self.p, self.q, None)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to create GARCH model: {}", e)))?;
        
        // Fit model
        model.fit(&train_returns, None)
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to train model: {}", e)))?;
        
        // Generate variance forecasts
        let forecasts = model.forecast_variance(test_values.len())
            .map_err(|e| ForecastError::InvalidParameter(format!("Failed to generate forecast: {}", e)))?;
        
        // Calculate actual volatility from test data
        let test_returns: Vec<f64> = test_values.windows(2)
            .map(|w| (w[1] / w[0]) - 1.0)
            .collect();
        
        let actual_variance: Vec<f64> = test_returns.iter().map(|r| r.powi(2)).collect();
        
        // Calculate error metrics
        let mae = mean_absolute_error(&actual_variance, &forecasts);
        let mse = mean_squared_error(&actual_variance, &forecasts);
        let rmse = root_mean_squared_error(&actual_variance, &forecasts);
        let mape = mean_absolute_percentage_error(&actual_variance, &forecasts);
        
        Ok(ErrorMetrics {
            mae,
            mse,
            rmse,
            mape,
        })
    }
    
    fn time_granularity(&self) -> TimeGranularity {
        self.granularity
    }
    
    // Implement box_clone using the macro
    impl_box_clone!(GarchAdapter);
    
    fn adjust_for_granularity(&mut self, granularity: TimeGranularity) -> Result<()> {
        self.granularity = granularity;
        Ok(())
    }
}

// Implement metrics functions that were used in OxiDiviner 0.2
fn mean_absolute_error(actual: &[f64], predicted: &[f64]) -> f64 {
    let n = actual.len().min(predicted.len());
    if n == 0 {
        return 0.0;
    }
    
    let sum: f64 = actual.iter().zip(predicted.iter())
        .take(n)
        .map(|(a, p)| (a - p).abs())
        .sum();
    
    sum / n as f64
}

fn mean_squared_error(actual: &[f64], predicted: &[f64]) -> f64 {
    let n = actual.len().min(predicted.len());
    if n == 0 {
        return 0.0;
    }
    
    let sum: f64 = actual.iter().zip(predicted.iter())
        .take(n)
        .map(|(a, p)| (a - p).powi(2))
        .sum();
    
    sum / n as f64
}

fn root_mean_squared_error(actual: &[f64], predicted: &[f64]) -> f64 {
    mean_squared_error(actual, predicted).sqrt()
}

fn mean_absolute_percentage_error(actual: &[f64], predicted: &[f64]) -> f64 {
    let n = actual.len().min(predicted.len());
    if n == 0 {
        return 0.0;
    }
    
    let sum: f64 = actual.iter().zip(predicted.iter())
        .take(n)
        .filter(|(a, _)| a.abs() > 1e-10) // Avoid division by zero or very small numbers
        .map(|(a, p)| ((a - p).abs() / a.abs()))
        .sum();
    
    (sum / n as f64) * 100.0 // Convert to percentage
} 