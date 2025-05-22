//! ARIMA models for time series forecasting

use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::{ForecastModel, ForecastResult, TrainedForecastModel};

/// ARIMA model (AutoRegressive Integrated Moving Average)
#[derive(Debug, Clone)]
pub struct ArimaModel {
    /// Name of the model
    name: String,
    /// AR order (p)
    p: usize,
    /// Differencing order (d)
    d: usize,
    /// MA order (q)
    q: usize,
    /// Fitted AR coefficients
    ar_coefficients: Vec<f64>,
    /// Fitted MA coefficients
    ma_coefficients: Vec<f64>,
    /// Fitted data
    data: Vec<f64>,
    /// Residuals from fitting
    residuals: Vec<f64>,
}

/// Trained ARIMA model
#[derive(Debug, Clone)]
pub struct TrainedArimaModel {
    /// Name of the model
    name: String,
    /// AR order (p)
    p: usize,
    /// Differencing order (d)
    d: usize,
    /// MA order (q)
    q: usize,
    /// Fitted AR coefficients
    ar_coefficients: Vec<f64>,
    /// Fitted MA coefficients
    ma_coefficients: Vec<f64>,
    /// Historical data
    historical_data: Vec<f64>,
    /// Residuals from fitting
    residuals: Vec<f64>,
}

impl ArimaModel {
    /// Create a new ARIMA model
    pub fn new(p: usize, d: usize, q: usize) -> Self {
        Self {
            name: format!("ARIMA({},{},{})", p, d, q),
            p,
            d,
            q,
            ar_coefficients: Vec::new(),
            ma_coefficients: Vec::new(),
            data: Vec::new(),
            residuals: Vec::new(),
        }
    }

    /// Helper function to forecast using AR process
    fn forecast_ar(&self, data: &[f64], coefficients: &[f64], horizon: usize) -> Vec<f64> {
        if data.is_empty() || coefficients.is_empty() {
            return vec![0.0; horizon];
        }

        let mut forecasts = Vec::with_capacity(horizon);
        let p = coefficients.len();

        // Use last p values of the data for initial forecasting
        let mut history = data[data.len().saturating_sub(p)..].to_vec();

        for _ in 0..horizon {
            // Calculate forecast using AR formula
            let mut forecast = 0.0;
            for i in 0..p {
                forecast += coefficients[i] * history[history.len() - 1 - i];
            }

            // Add forecast to history
            history.push(forecast);
            forecasts.push(forecast);
        }

        forecasts
    }
}

impl ForecastModel for ArimaModel {
    type Trained = TrainedArimaModel;

    fn train(&self, data: &TimeSeriesData) -> Result<TrainedArimaModel> {
        let prices = data.close_prices();
        if prices.len() < self.p + self.d + self.q + 1 {
            return Err(ForecastError::ValidationError(format!(
                "Insufficient data for ARIMA({},{},{}). Need at least {} observations.",
                self.p,
                self.d,
                self.q,
                self.p + self.d + self.q + 1
            )));
        }

        // For simplicity, we'll just use a naive AR estimation
        // Real ARIMA would involve differencing, proper estimation, etc.

        // For simplicity, just set some default coefficients
        // In a real implementation, these would be estimated from data
        let ar_coefficients = vec![0.8; self.p];
        let ma_coefficients = vec![0.2; self.q];

        // In a real implementation, we would calculate residuals properly
        let residuals = vec![0.0; prices.len()];

        Ok(TrainedArimaModel {
            name: self.name.clone(),
            p: self.p,
            d: self.d,
            q: self.q,
            ar_coefficients,
            ma_coefficients,
            historical_data: prices.to_vec(),
            residuals,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl TrainedForecastModel for TrainedArimaModel {
    fn forecast(&self, horizon: usize) -> Result<ForecastResult> {
        if self.historical_data.is_empty() {
            return Err(ForecastError::ForecastingError(
                "Model has not been fitted to data".to_string(),
            ));
        }

        // For simplicity, just use AR part for forecasting
        // Real ARIMA would involve more complex forecasting logic

        let forecasts = self.forecast_ar(&self.historical_data, &self.ar_coefficients, horizon);

        Ok(ForecastResult::new(forecasts, horizon)?)
    }

    fn predict(&self, data: &TimeSeriesData) -> Result<ForecastResult> {
        let prices = data.close_prices();
        if prices.is_empty() {
            return Err(ForecastError::DataError(
                "Empty time series data".to_string(),
            ));
        }

        // For simplicity, we'll just produce one-step-ahead forecasts
        // Real ARIMA would involve proper prediction

        let mut predictions = Vec::with_capacity(prices.len());

        // For the first p points, just use the actual values
        for i in 0..self.p.min(prices.len()) {
            predictions.push(prices[i]);
        }

        // For the rest, use AR model to predict
        for i in self.p..prices.len() {
            let mut prediction = 0.0;
            for j in 0..self.p {
                prediction += self.ar_coefficients[j] * prices[i - j - 1];
            }
            predictions.push(prediction);
        }

        Ok(ForecastResult::new(predictions.clone(), predictions.len())?)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl TrainedArimaModel {
    /// Helper function to forecast using AR process
    fn forecast_ar(&self, data: &[f64], coefficients: &[f64], horizon: usize) -> Vec<f64> {
        if data.is_empty() || coefficients.is_empty() {
            return vec![0.0; horizon];
        }

        let mut forecasts = Vec::with_capacity(horizon);
        let p = coefficients.len();

        // Use last p values of the data for initial forecasting
        let mut history = data[data.len().saturating_sub(p)..].to_vec();

        for _ in 0..horizon {
            // Calculate forecast using AR formula
            let mut forecast = 0.0;
            for i in 0..p {
                forecast += coefficients[i] * history[history.len() - 1 - i];
            }

            // Add forecast to history
            history.push(forecast);
            forecasts.push(forecast);
        }

        forecasts
    }
}
