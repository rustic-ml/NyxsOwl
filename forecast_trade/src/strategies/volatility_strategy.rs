//! A volatility-based trading strategy using GARCH models

use crate::backtest;
use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::ForecastModel;
use crate::models::oxidiviner::GarchAdapter;
use crate::strategies::{BacktestResult, BaseStrategy, ForecastStrategy, TimeGranularity, TradingSignal};

/// A trading strategy based on volatility forecasts from GARCH models
#[derive(Clone)]
pub struct VolatilityStrategy<M: ForecastModel> {
    /// Base strategy implementation
    base: BaseStrategy<M>,
    /// The volatility threshold multiplier for triggering signals
    threshold_multiplier: f64,
    /// Lookback window for calculating moving averages
    lookback_window: usize,
}

impl<M: ForecastModel> VolatilityStrategy<M> {
    /// Create a new volatility-based strategy
    pub fn new(model: M, threshold_multiplier: f64) -> Result<Self> {
        if threshold_multiplier <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Threshold multiplier must be positive".to_string(),
            ));
        }

        // Default lookback window based on model's granularity
        let lookback_window = match model.time_granularity() {
            TimeGranularity::Daily => 20,   // 20 days for daily data
            TimeGranularity::Minute => 60,  // 60 minutes for minute data
        };

        Ok(Self {
            base: BaseStrategy::new("Volatility Strategy", model),
            threshold_multiplier,
            lookback_window,
        })
    }

    /// Create a new volatility-based strategy with specific granularity
    pub fn new_with_granularity(
        model: M,
        threshold_multiplier: f64,
        granularity: TimeGranularity,
    ) -> Result<Self> {
        if threshold_multiplier <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Threshold multiplier must be positive".to_string(),
            ));
        }
        
        // Adjust lookback window based on granularity
        let lookback_window = match granularity {
            TimeGranularity::Daily => 20,   // 20 days
            TimeGranularity::Minute => 60,  // 60 minutes
        };
        
        Ok(Self {
            base: BaseStrategy::new_with_granularity("Volatility Strategy", model, granularity),
            threshold_multiplier,
            lookback_window,
        })
    }

    /// Set custom lookback window
    pub fn with_lookback_window(mut self, window: usize) -> Result<Self> {
        if window < 5 {
            return Err(ForecastError::InvalidParameter(
                "Lookback window must be at least 5".to_string(),
            ));
        }
        self.lookback_window = window;
        Ok(self)
    }
    
    /// Calculate historical price volatility using standard deviation
    fn calculate_volatility(&self, prices: &[f64], window: usize) -> Vec<f64> {
        if prices.len() <= window {
            return vec![0.0; prices.len()];
        }
        
        let mut volatility = vec![0.0; window - 1];
        
        for i in window..=prices.len() {
            let window_slice = &prices[i - window..i];
            let mean = window_slice.iter().sum::<f64>() / window_slice.len() as f64;
            let variance = window_slice.iter()
                .map(|&p| (p - mean).powi(2))
                .sum::<f64>() / window_slice.len() as f64;
            
            volatility.push(variance.sqrt());
        }
        
        volatility
    }
    
    /// Calculate returns from price series
    fn calculate_returns(&self, prices: &[f64]) -> Vec<f64> {
        if prices.len() <= 1 {
            return Vec::new();
        }
        
        let mut returns = vec![0.0];
        returns.extend(
            prices.windows(2)
                .map(|w| (w[1] / w[0]) - 1.0)
        );
        
        returns
    }
}

impl<M: ForecastModel> ForecastStrategy for VolatilityStrategy<M> {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        if data.len() <= self.lookback_window {
            return Err(ForecastError::ValidationError(
                format!("Insufficient data. Need at least {} data points", self.lookback_window + 1)
            ));
        }
        
        // Get trained model
        let trained_model = self.base.get_trained_model(data)?;
        
        // Generate volatility forecast
        let forecast = trained_model.forecast(data, 1)?;
        let forecasted_volatility = forecast.values[0];
        
        let prices = data.close_prices();
        let returns = self.calculate_returns(prices);
        let historical_volatility = self.calculate_volatility(prices, self.lookback_window);
        
        // Calculate moving average of price
        let lookback = self.lookback_window.min(prices.len() - 1);
        let ma_period = (lookback as f64 / 4.0).max(5.0) as usize;
        
        let mut price_ma = vec![0.0; ma_period - 1];
        for i in ma_period..=prices.len() {
            let window_slice = &prices[i - ma_period..i];
            let mean = window_slice.iter().sum::<f64>() / window_slice.len() as f64;
            price_ma.push(mean);
        }
        
        // Generate signals based on volatility forecast and price trend
        let mut signals = Vec::with_capacity(data.len());
        
        // Fill with Hold signals for data points where we don't have enough history
        for _ in 0..(data.len() - 1) {
            signals.push(TradingSignal::Hold);
        }
        
        // Get the current price and volatility
        let current_price = *prices.last().unwrap();
        let current_ma = *price_ma.last().unwrap();
        let historical_vol = *historical_volatility.last().unwrap();
        
        // Generate signal based on volatility forecast and price trend
        let signal = if forecasted_volatility > historical_vol * self.threshold_multiplier {
            // High volatility expected
            if current_price > current_ma {
                // Price above MA in high volatility - bullish breakout potential
                TradingSignal::Buy
            } else {
                // Price below MA in high volatility - bearish breakdown potential
                TradingSignal::Sell
            }
        } else {
            // Normal or low volatility - stay neutral
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

/// Convenience function to create a VolatilityStrategy with GARCH model
pub fn create_garch_strategy(
    p: usize, q: usize,
    threshold_multiplier: f64,
    granularity: TimeGranularity
) -> Result<VolatilityStrategy<GarchAdapter>> {
    let model = GarchAdapter::with_params(p, q)?
        .with_granularity(granularity);
    
    VolatilityStrategy::new_with_granularity(model, threshold_multiplier, granularity)
} 