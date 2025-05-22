//! A strategy based on ARIMA forecasts for time series with stationarity and seasonal effects

use crate::backtest;
use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::ForecastModel;
use crate::strategies::{BacktestResult, BaseStrategy, ForecastStrategy, TimeGranularity, TradingSignal};

/// A trading strategy based on ARIMA models, suitable for trending markets with seasonal patterns
#[derive(Clone)]
pub struct ArimaStrategy<M: ForecastModel> {
    /// Base strategy implementation
    base: BaseStrategy<M>,
    /// The threshold for triggering buy/sell signals
    threshold: f64,
    /// Whether to use percentage change or absolute value change
    use_percentage: bool,
    /// Number of periods to forecast ahead
    forecast_horizon: usize,
}

impl<M: ForecastModel> ArimaStrategy<M> {
    /// Create a new ARIMA-based strategy
    pub fn new(model: M, threshold: f64) -> Result<Self> {
        if threshold <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Threshold must be positive".to_string(),
            ));
        }

        let forecast_horizon = match model.time_granularity() {
            TimeGranularity::Daily => 1,   // 1 day ahead
            TimeGranularity::Minute => 5,  // 5 minutes ahead
        };

        Ok(Self {
            base: BaseStrategy::new("ARIMA Strategy", model),
            threshold,
            use_percentage: true,
            forecast_horizon,
        })
    }

    /// Create a new ARIMA-based strategy with specific granularity
    pub fn new_with_granularity(
        model: M,
        threshold: f64,
        granularity: TimeGranularity,
    ) -> Result<Self> {
        if threshold <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Threshold must be positive".to_string(),
            ));
        }
        
        // Adjust forecast horizon based on granularity
        let forecast_horizon = match granularity {
            TimeGranularity::Daily => 1,   // 1 day ahead
            TimeGranularity::Minute => 5,  // 5 minutes ahead
        };
        
        Ok(Self {
            base: BaseStrategy::new_with_granularity("ARIMA Strategy", model, granularity),
            threshold,
            use_percentage: true,
            forecast_horizon,
        })
    }

    /// Set whether to use percentage change instead of absolute change
    pub fn with_percentage(mut self, use_percentage: bool) -> Self {
        self.use_percentage = use_percentage;
        self
    }

    /// Set the forecast horizon (number of periods to forecast ahead)
    pub fn with_forecast_horizon(mut self, horizon: usize) -> Result<Self> {
        if horizon == 0 {
            return Err(ForecastError::InvalidParameter(
                "Forecast horizon must be positive".to_string(),
            ));
        }
        self.forecast_horizon = horizon;
        Ok(self)
    }
}

impl<M: ForecastModel> ForecastStrategy for ArimaStrategy<M> {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        // Get the trained model
        let trained_model = self.base.get_trained_model(data)?;

        // Generate forecast
        let forecast = trained_model.forecast(data, self.forecast_horizon)?;
        
        // Generate signals based on the forecasted values
        let mut signals = Vec::with_capacity(data.len());
        let prices = data.close_prices();
        
        // Fill with Hold signals for all historical data points except the last one
        for _ in 0..(data.len() - 1) {
            signals.push(TradingSignal::Hold);
        }
        
        // Get the last price and forecasted next price
        let last_price = *prices.last().unwrap();
        let next_price = forecast.values[0];
        
        // Calculate change
        let change = if self.use_percentage {
            (next_price - last_price) / last_price * 100.0
        } else {
            next_price - last_price
        };
        
        // Generate signal based on forecasted change
        let signal = if change > self.threshold {
            TradingSignal::Buy
        } else if change < -self.threshold {
            TradingSignal::Sell
        } else {
            TradingSignal::Hold
        };
        
        signals.push(signal);
        
        Ok(signals)
    }
    
    fn backtest_with_params(
        &self,
        data: &TimeSeriesData,
        initial_capital: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Result<BacktestResult> {
        let signals = self.generate_signals(data)?;
        backtest::run_backtest(data, &signals, initial_capital, commission_rate, slippage)
    }

    fn time_granularity(&self) -> TimeGranularity {
        self.base.time_granularity
    }
} 