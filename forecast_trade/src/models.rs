//! Forecasting models for time series data

use crate::data::TimeSeriesData;
use crate::error::Result;
use crate::strategies::TimeGranularity;

/// Result of a forecast operation
#[derive(Debug, Clone)]
pub struct ForecastResult {
    /// Forecasted values
    pub values: Vec<f64>,
    /// Forecast confidence intervals (optional)
    pub confidence_intervals: Option<(Vec<f64>, Vec<f64>)>,
    /// Forecast error metrics
    pub error_metrics: Option<ErrorMetrics>,
}

/// Error metrics for forecast evaluation
#[derive(Debug, Clone)]
pub struct ErrorMetrics {
    /// Mean Absolute Error
    pub mae: f64,
    /// Mean Squared Error
    pub mse: f64,
    /// Root Mean Squared Error
    pub rmse: f64,
    /// Mean Absolute Percentage Error
    pub mape: f64,
}

/// Common interface for forecasting models
pub trait ForecastModel {
    /// Train the model on time series data
    fn train(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>>;
    
    /// Generate a forecast for a specified number of periods
    fn forecast(&self, data: &TimeSeriesData, periods: usize) -> Result<ForecastResult>;
    
    /// Validate the model on test data
    fn validate(&self, train_data: &TimeSeriesData, test_data: &TimeSeriesData) -> Result<ErrorMetrics>;
    
    /// Get the model's time granularity preference
    fn time_granularity(&self) -> TimeGranularity {
        // Default implementation returns Daily
        TimeGranularity::Daily
    }
    
    /// Create a boxed clone of this model (replacement for Clone trait which is not object-safe)
    fn box_clone(&self) -> Box<dyn ForecastModel>;
    
    /// Adjust model parameters based on time granularity
    fn adjust_for_granularity(&mut self, granularity: TimeGranularity) -> Result<()>;
}

// Macro to help implement box_clone for ForecastModel implementors
#[macro_export]
macro_rules! impl_box_clone {
    ($type:ty) => {
        fn box_clone(&self) -> Box<dyn ForecastModel> {
            Box::new(self.clone())
        }
    };
}

// Export only the OxiDiviner module, removing our custom implementations
pub mod oxidiviner; 