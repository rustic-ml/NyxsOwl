//! OxiDiviner Integration
//!
//! This module provides integration with the OxiDiviner forecasting library,
//! offering simplified access to various time series forecasting models.

use oxidiviner::{TimeSeriesData as OxiTimeSeriesData, quick};
use crate::forecast_trade::{
    ForecastError, TimeGranularity, 
    data::TimeSeriesData,
    models::{ForecastModel, ForecastResult, ErrorMetrics, Result},
    strategies::{ForecastStrategy, TradingSignal, BacktestResult},
};
use std::collections::HashMap;

/// High-level convenience API for OxiDiviner integration
pub mod easy {
    use super::*;
    use chrono::{DateTime, Utc};

    /// Convert our TimeSeriesData to OxiDiviner TimeSeriesData
    fn convert_to_oxidiviner_data(
        dates: Vec<DateTime<Utc>>,
        values: Vec<f64>,
        name: &str,
    ) -> Result<OxiTimeSeriesData> {
        OxiTimeSeriesData::new(dates, values, name)
            .map_err(|e| ForecastError::InvalidParameter(format!("Data conversion failed: {}", e)))
    }

    /// Quick ARIMA forecast
    pub fn arima_forecast(
        dates: Vec<DateTime<Utc>>,
        values: Vec<f64>,
        forecast_periods: usize,
    ) -> Result<Vec<f64>> {
        let ts_data = convert_to_oxidiviner_data(dates, values, "arima_forecast")?;
        let forecast = quick::arima(ts_data, forecast_periods)
            .map_err(|e| ForecastError::InvalidParameter(format!("ARIMA forecast failed: {}", e)))?;
        Ok(forecast)
    }

    /// Quick Moving Average forecast
    pub fn ma_forecast(
        dates: Vec<DateTime<Utc>>,
        values: Vec<f64>,
        forecast_periods: usize,
        window: Option<usize>,
    ) -> Result<Vec<f64>> {
        let ts_data = convert_to_oxidiviner_data(dates, values, "ma_forecast")?;
        let forecast = quick::moving_average(ts_data, forecast_periods, window)
            .map_err(|e| ForecastError::InvalidParameter(format!("MA forecast failed: {}", e)))?;
        Ok(forecast)
    }

    /// Quick Exponential Smoothing forecast
    pub fn es_forecast(
        dates: Vec<DateTime<Utc>>,
        values: Vec<f64>,
        forecast_periods: usize,
        alpha: Option<f64>,
    ) -> Result<Vec<f64>> {
        let ts_data = convert_to_oxidiviner_data(dates, values, "es_forecast")?;
        let forecast = quick::exponential_smoothing(ts_data, forecast_periods, alpha)
            .map_err(|e| ForecastError::InvalidParameter(format!("ES forecast failed: {}", e)))?;
        Ok(forecast)
    }

    /// Automatic model selection and forecasting
    pub fn auto_forecast(
        dates: Vec<DateTime<Utc>>,
        values: Vec<f64>,
        forecast_periods: usize,
    ) -> Result<(Vec<f64>, String)> {
        let ts_data = convert_to_oxidiviner_data(dates, values, "auto_forecast")?;
        let (forecast, model_name) = quick::auto_select(ts_data, forecast_periods)
            .map_err(|e| ForecastError::InvalidParameter(format!("Auto forecast failed: {}", e)))?;
        Ok((forecast, model_name))
    }

    /// Convert our TimeSeriesData to OxiDiviner TimeSeriesData
    pub fn convert_to_oxidiviner(data: &TimeSeriesData) -> Result<OxiTimeSeriesData> {
        let dates = data.timestamps().to_vec();
        let values = data.close_prices().to_vec();
        convert_to_oxidiviner_data(dates, values, "converted_data")
    }

    /// Exponential smoothing forecast using quick API
    pub fn exponential_smoothing_forecast(
        dates: Vec<DateTime<Utc>>,
        values: Vec<f64>,
        forecast_periods: usize,
        alpha: Option<f64>,
    ) -> Result<Vec<f64>> {
        let oxi_data = convert_to_oxidiviner_data(dates, values, "es_data")?;
        let forecast = quick::exponential_smoothing(oxi_data, forecast_periods, alpha)
            .map_err(|e| ForecastError::ModelError(e.to_string()))?;
        Ok(forecast)
    }

    /// Moving average forecast using quick API
    pub fn moving_average_forecast(
        dates: Vec<DateTime<Utc>>,
        values: Vec<f64>,
        forecast_periods: usize,
        window: Option<usize>,
    ) -> Result<Vec<f64>> {
        let oxi_data = convert_to_oxidiviner_data(dates, values, "ma_data")?;
        let forecast = quick::moving_average(oxi_data, forecast_periods, window)
            .map_err(|e| ForecastError::ModelError(e.to_string()))?;
        Ok(forecast)
    }
}

/// Simplified adapter for OxiDiviner's quick API
#[derive(Debug, Clone)]
pub struct OxiDivinerAdapter {
    /// The model type to use
    model_type: String,
    /// Model parameters
    params: HashMap<String, f64>,
    /// Time granularity for the model
    granularity: TimeGranularity,
    /// Last training data (for forecasting)
    last_data: Option<OxiTimeSeriesData>,
}

impl OxiDivinerAdapter {
    /// Create a new ARIMA adapter
    pub fn arima() -> Result<Self> {
        Ok(Self {
            model_type: "arima".to_string(),
            params: HashMap::new(),
            granularity: TimeGranularity::Daily,
            last_data: None,
        })
    }

    /// Create a new Moving Average adapter
    pub fn moving_average(window: Option<usize>) -> Result<Self> {
        let mut params = HashMap::new();
        if let Some(w) = window {
            params.insert("window".to_string(), w as f64);
        }
        
        Ok(Self {
            model_type: "ma".to_string(),
            params,
            granularity: TimeGranularity::Daily,
            last_data: None,
        })
    }

    /// Create a new Exponential Smoothing adapter
    pub fn exponential_smoothing(alpha: Option<f64>) -> Result<Self> {
        let mut params = HashMap::new();
        if let Some(a) = alpha {
            params.insert("alpha".to_string(), a);
        }
        
        Ok(Self {
            model_type: "es".to_string(),
            params,
            granularity: TimeGranularity::Daily,
            last_data: None,
        })
    }

    /// Create an auto-selecting adapter
    pub fn auto() -> Result<Self> {
        Ok(Self {
            model_type: "auto".to_string(),
            params: HashMap::new(),
            granularity: TimeGranularity::Daily,
            last_data: None,
        })
    }

    /// Create a new SARIMA adapter (falls back to ARIMA for now)
    pub fn sarima(
        _p: usize, _d: usize, _q: usize,
        _seasonal_p: usize, _seasonal_d: usize, _seasonal_q: usize,
        _seasonal_period: usize
    ) -> Result<Self> {
        Self::arima() // Fall back to ARIMA since OxiDiviner doesn't have SARIMA yet
    }

    /// Create a GARCH adapter with parameters (falls back to auto)
    pub fn with_params(_p: usize, _q: usize) -> Result<Self> {
        Self::auto()
    }

    /// Set time granularity
    pub fn with_granularity(mut self, granularity: TimeGranularity) -> Self {
        self.granularity = granularity;
        self
    }
}

impl ForecastModel for OxiDivinerAdapter {
    fn train(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>> {
        let ox_data = easy::convert_to_oxidiviner(data)?;
        let mut adapter = self.clone();
        adapter.last_data = Some(ox_data);
        Ok(Box::new(adapter))
    }

    fn forecast(&self, _data: &TimeSeriesData, periods: usize) -> Result<ForecastResult> {
        let ox_data = self.last_data.as_ref().ok_or_else(|| {
            ForecastError::InvalidParameter("Model not trained. Call train() first.".to_string())
        })?;

        let forecast = match self.model_type.as_str() {
            "arima" => {
                quick::arima(ox_data.clone(), periods)
                    .map_err(|e| ForecastError::InvalidParameter(format!("ARIMA forecast failed: {}", e)))?
            },
            "ma" => {
                let window = self.params.get("window").map(|&w| w as usize);
                quick::moving_average(ox_data.clone(), periods, window)
                    .map_err(|e| ForecastError::InvalidParameter(format!("MA forecast failed: {}", e)))?
            },
            "es" => {
                let alpha = self.params.get("alpha").copied();
                quick::exponential_smoothing(ox_data.clone(), periods, alpha)
                    .map_err(|e| ForecastError::InvalidParameter(format!("ES forecast failed: {}", e)))?
            },
            "auto" => {
                let (forecast, _model_name) = quick::auto_select(ox_data.clone(), periods)
                    .map_err(|e| ForecastError::InvalidParameter(format!("Auto forecast failed: {}", e)))?;
                forecast
            },
            _ => {
                return Err(ForecastError::InvalidParameter(
                    format!("Unknown model type: {}", self.model_type)
                ));
            }
        };

        // Generate confidence intervals (simplified approach)
        let confidence_lower = forecast.iter().map(|&v| v * 0.95).collect();
        let confidence_upper = forecast.iter().map(|&v| v * 1.05).collect();

        Ok(ForecastResult {
            forecasts: forecast,
            confidence_intervals: Some((confidence_lower, confidence_upper)),
            model_info: format!("OxiDiviner {} model", self.model_type),
        })
    }

    fn validate(&self, train_data: &TimeSeriesData, test_data: &TimeSeriesData) -> Result<ErrorMetrics> {
        let trained_model = self.train(train_data)?;
        let forecast_result = trained_model.forecast(train_data, test_data.close_prices().len())?;
        
        let predicted = &forecast_result.forecasts;
        let actual = test_data.close_prices();

        Ok(ErrorMetrics {
            mae: mean_absolute_error(&actual, predicted),
            mse: mean_squared_error(&actual, predicted),
            rmse: root_mean_squared_error(&actual, predicted),
            mape: mean_absolute_percentage_error(&actual, predicted),
        })
    }

    fn time_granularity(&self) -> TimeGranularity {
        self.granularity
    }

    fn adjust_for_granularity(&mut self, granularity: TimeGranularity) -> Result<()> {
        self.granularity = granularity;
        Ok(())
    }

    fn name(&self) -> &str {
        &self.model_type
    }
}

impl ForecastStrategy for OxiDivinerAdapter {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        // Simple signal generation based on the forecast trend
        let trained_model = self.train(data)?;
        let forecast_result = trained_model.forecast(data, 1)?;
        
        if forecast_result.forecasts.is_empty() {
            return Ok(vec![TradingSignal::Hold; data.len()]);
        }
        
        let mut signals = vec![TradingSignal::Hold; data.len() - 1];
        let prices = data.close_prices();
        let current_price = prices.last().unwrap();
        let forecast_price = forecast_result.forecasts[0];
        
        // Simple signal: Buy if forecast is higher, Sell if lower, Hold otherwise
        let threshold = 0.01; // 1% threshold
        let change = (forecast_price - current_price) / current_price;
        
        let signal = if change > threshold {
            TradingSignal::Buy
        } else if change < -threshold {
            TradingSignal::Sell
        } else {
            TradingSignal::Hold
        };
        
        signals.push(signal);
        Ok(signals)
    }

    fn backtest(&self, data: &TimeSeriesData, initial_balance: f64) -> Result<BacktestResult> {
        self.backtest_with_params(data, initial_balance, 0.001, 0.0005)
    }

    fn backtest_with_params(
        &self,
        data: &TimeSeriesData,
        initial_balance: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Result<BacktestResult> {
        let signals = self.generate_signals(data)?;
        
        // Simple backtesting logic
        let prices = data.close_prices();
        let mut balance = initial_balance;
        let mut position: f64 = 0.0;
        let mut total_trades = 0;
        let mut winning_trades = 0;

        for i in 1..prices.len() {
            let price = prices[i];
            
            match signals[i] {
                TradingSignal::Buy if position <= 0.0 => {
                    // Close short position if any, then buy
                    if position < 0.0 {
                        balance += position.abs() * price * (1.0 - commission_rate - slippage);
                    }
                    // Open long position
                    let shares = balance / (price * (1.0 + commission_rate + slippage));
                    balance -= shares * price * (1.0 + commission_rate + slippage);
                    position = shares;
                    total_trades += 1;
                }
                TradingSignal::Sell if position >= 0.0 => {
                    // Close long position if any, then short
                    if position > 0.0 {
                        balance += position * price * (1.0 - commission_rate - slippage);
                    }
                    // Open short position
                    let shares = balance / (price * (1.0 + commission_rate + slippage));
                    position = -shares;
                    total_trades += 1;
                }
                _ => {}
            }
        }

        // Close final position
        let final_price = prices[prices.len() - 1];
        if position > 0.0 {
            balance += position * final_price * (1.0 - commission_rate - slippage);
        } else if position < 0.0 {
            balance += position.abs() * final_price * (1.0 - commission_rate - slippage);
        }

        let total_return = (balance - initial_balance) / initial_balance;
        let win_rate = if total_trades > 0 { winning_trades as f64 / total_trades as f64 } else { 0.0 };

        Ok(BacktestResult {
            final_balance: balance,
            total_return,
            max_drawdown: 0.0, // Simplified - not calculated
            win_rate,
            equity_curve: Vec::new(),
            trades: total_trades,
            performance_metrics: None,
        })
    }

    fn time_granularity(&self) -> TimeGranularity {
        self.granularity
    }
}

// Type aliases for backward compatibility
pub type ExponentialSmoothingAdapter = OxiDivinerAdapter;
pub type MovingAverageAdapter = OxiDivinerAdapter;  
pub type ArimaAdapter = OxiDivinerAdapter;
pub type GarchAdapter = OxiDivinerAdapter;
pub type ETSAdapter = OxiDivinerAdapter;
pub type ARAdapter = OxiDivinerAdapter;

// Constructor functions to avoid method conflicts
pub mod constructors {
    use super::*;

    /// Create a new ExponentialSmoothingAdapter
    pub fn exponential_smoothing(alpha: f64) -> Result<ExponentialSmoothingAdapter> {
        OxiDivinerAdapter::exponential_smoothing(Some(alpha))
    }

    /// Create a new MovingAverageAdapter
    pub fn moving_average(window_size: usize) -> Result<MovingAverageAdapter> {
        OxiDivinerAdapter::moving_average(Some(window_size))
    }

    /// Create a new ArimaAdapter
    pub fn arima() -> Result<ArimaAdapter> {
        OxiDivinerAdapter::arima()
    }

    /// Create a new GarchAdapter (falls back to auto)
    pub fn garch() -> Result<GarchAdapter> {
        OxiDivinerAdapter::auto()
    }

    /// Create a new ETSAdapter (falls back to auto)
    pub fn ets(_seasonal_period: usize) -> Result<ETSAdapter> {
        OxiDivinerAdapter::auto()
    }

    /// Create a new ARAdapter (falls back to auto)
    pub fn ar(_order: usize) -> Result<ARAdapter> {
        OxiDivinerAdapter::auto()
    }
}

// Helper functions for error metrics
fn mean_absolute_error(actual: &[f64], predicted: &[f64]) -> f64 {
    if actual.len() != predicted.len() || actual.is_empty() {
        return f64::NAN;
    }

    let sum: f64 = actual
        .iter()
        .zip(predicted.iter())
        .map(|(a, p)| (a - p).abs())
        .sum();

    sum / actual.len() as f64
}

fn mean_squared_error(actual: &[f64], predicted: &[f64]) -> f64 {
    if actual.len() != predicted.len() || actual.is_empty() {
        return f64::NAN;
    }

    let sum: f64 = actual
        .iter()
        .zip(predicted.iter())
        .map(|(a, p)| (a - p).powi(2))
        .sum();

    sum / actual.len() as f64
}

fn root_mean_squared_error(actual: &[f64], predicted: &[f64]) -> f64 {
    mean_squared_error(actual, predicted).sqrt()
}

fn mean_absolute_percentage_error(actual: &[f64], predicted: &[f64]) -> f64 {
    if actual.len() != predicted.len() || actual.is_empty() {
        return f64::NAN;
    }

    let sum: f64 = actual
        .iter()
        .zip(predicted.iter())
        .map(|(a, p)| {
            if a.abs() < f64::EPSILON {
                0.0 // Avoid division by zero
            } else {
                ((a - p) / a).abs()
            }
        })
        .sum();

    let n = actual.len();
    (sum / n as f64) * 100.0 // Convert to percentage
}

