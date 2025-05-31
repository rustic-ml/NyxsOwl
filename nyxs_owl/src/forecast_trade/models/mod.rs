use crate::forecast_trade::{ForecastError, TimeGranularity, data::TimeSeriesData};

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

/// Forecast result containing predictions and metadata
#[derive(Debug, Clone)]
pub struct ForecastResult {
    /// The forecasted values
    pub forecasts: Vec<f64>,
    /// Optional confidence intervals (lower, upper bounds)
    pub confidence_intervals: Option<(Vec<f64>, Vec<f64>)>,
    /// Information about the model used
    pub model_info: String,
}

/// Result type for forecast operations
pub type Result<T> = std::result::Result<T, ForecastError>;

/// Trait for cloning boxed forecast models
pub trait BoxClone {
    fn box_clone(&self) -> Box<dyn ForecastModel>;
}

impl<T> BoxClone for T
where
    T: 'static + ForecastModel + Clone,
{
    fn box_clone(&self) -> Box<dyn ForecastModel> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ForecastModel> {
    fn clone(&self) -> Box<dyn ForecastModel> {
        // Call the BoxClone trait method
        BoxClone::box_clone(self.as_ref())
    }
}

/// Macro to implement BoxClone for a type
#[macro_export]
macro_rules! impl_box_clone {
    ($name:ty) => {
        impl $crate::forecast_trade::models::BoxClone for $name {
            fn box_clone(&self) -> Box<dyn $crate::forecast_trade::ForecastModel> {
                Box::new(self.clone())
            }
        }
    };
}

/// Trait for forecast models
pub trait ForecastModel: Send + Sync + BoxClone {
    /// Train the model on historical data
    fn train(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>>;
    
    /// Generate forecasts for the given number of periods
    fn forecast(&self, data: &TimeSeriesData, periods: usize) -> Result<ForecastResult>;
    
    /// Validate the model using train/test split
    fn validate(&self, train_data: &TimeSeriesData, test_data: &TimeSeriesData) -> Result<ErrorMetrics>;
    
    /// Get the time granularity for this model
    fn time_granularity(&self) -> TimeGranularity;
    
    /// Adjust model for different time granularities
    fn adjust_for_granularity(&mut self, granularity: TimeGranularity) -> Result<()>;

    /// Get model name
    fn name(&self) -> &str;

    /// Get model parameters (if any)
    fn parameters(&self) -> Option<Vec<f64>> {
        None
    }
}

pub mod oxidiviner; 