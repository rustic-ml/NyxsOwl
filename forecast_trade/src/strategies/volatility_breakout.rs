use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::ForecastModel;
use crate::strategies::{BacktestResults, ForecastStrategy, PerformanceMetrics, TimeGranularity, TradingSignal};

/// Configuration for volatility breakout strategy
#[derive(Debug, Clone)]
pub struct VolatilityBreakoutConfig {
    /// Volatility multiplier for breakout threshold
    pub volatility_multiplier: f64,
    /// Lookback period for calculating volatility
    pub lookback_period: usize,
    /// Whether to use ATR (true) or standard deviation (false)
    pub use_atr: bool,
    /// Whether to use trailing stops
    pub use_trailing_stop: bool,
    /// Trailing stop multiplier
    pub trailing_stop_multiplier: f64,
}

impl VolatilityBreakoutConfig {
    /// Create a default configuration for daily data
    pub fn default_daily() -> Self {
        Self {
            volatility_multiplier: 1.5,
            lookback_period: 20,
            use_atr: true,
            use_trailing_stop: true,
            trailing_stop_multiplier: 2.0,
        }
    }
    
    /// Create a default configuration for minute data
    pub fn default_minute() -> Self {
        Self {
            volatility_multiplier: 2.0, // Higher for minute data due to noise
            lookback_period: 60,        // 1 hour of minute data
            use_atr: true,
            use_trailing_stop: true,
            trailing_stop_multiplier: 1.5, // Tighter stops for minute data
        }
    }
    
    /// Create a configuration based on time granularity
    pub fn for_granularity(granularity: TimeGranularity) -> Self {
        match granularity {
            TimeGranularity::Daily => Self::default_daily(),
            TimeGranularity::Minute => Self::default_minute(),
        }
    }
}

/// Volatility breakout strategy
#[derive(Debug, Clone)]
pub struct VolatilityBreakoutStrategy<M: ForecastModel> {
    /// Forecast model
    model: M,
    /// Strategy configuration
    config: VolatilityBreakoutConfig,
    /// Time granularity
    time_granularity: TimeGranularity,
}

impl<M: ForecastModel> VolatilityBreakoutStrategy<M> {
    /// Create a new volatility breakout strategy
    pub fn new(model: M, volatility_multiplier: f64) -> Result<Self> {
        if volatility_multiplier <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Volatility multiplier must be positive".to_string(),
            ));
        }

        let mut config = VolatilityBreakoutConfig::default_daily();
        config.volatility_multiplier = volatility_multiplier;

        Ok(Self {
            model,
            config,
            time_granularity: TimeGranularity::Daily,
        })
    }

    /// Create a new volatility breakout strategy with custom configuration
    pub fn new_with_config(model: M, config: VolatilityBreakoutConfig) -> Self {
        Self {
            model,
            config,
            time_granularity: TimeGranularity::Daily,
        }
    }

    /// Create a new volatility breakout strategy for a specific time granularity
    pub fn new_with_granularity(
        model: M,
        volatility_multiplier: f64,
        time_granularity: TimeGranularity,
    ) -> Result<Self> {
        if volatility_multiplier <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Volatility multiplier must be positive".to_string(),
            ));
        }

        let mut config = VolatilityBreakoutConfig::for_granularity(time_granularity);
        config.volatility_multiplier = volatility_multiplier;

        Ok(Self {
            model,
            config,
            time_granularity,
        })
    }

    /// Calculate Average True Range (ATR)
    fn calculate_atr(&self, data: &TimeSeriesData, period: usize) -> Vec<f64> {
        if data.len() < 2 {
            return vec![0.0; data.len()];
        }

        let close_prices = data.close_prices();
        let high_prices = data.high_prices().unwrap_or_else(|| close_prices.clone());
        let low_prices = data.low_prices().unwrap_or_else(|| close_prices.clone());
        
        let mut true_ranges = Vec::with_capacity(data.len());
        true_ranges.push(high_prices[0] - low_prices[0]); // First TR is just the range
        
        for i in 1..data.len() {
            // True Range is max of:
            // 1. Current High - Current Low
            // 2. |Current High - Previous Close|
            // 3. |Current Low - Previous Close|
            let tr = (high_prices[i] - low_prices[i])
                .max((high_prices[i] - close_prices[i - 1]).abs())
                .max((low_prices[i] - close_prices[i - 1]).abs());
                
            true_ranges.push(tr);
        }
        
        // Calculate moving average of true ranges
        let mut atr = Vec::with_capacity(data.len());
        for i in 0..data.len() {
            if i < period - 1 {
                // Not enough data for full period
                atr.push(true_ranges[..=i].iter().sum::<f64>() / (i + 1) as f64);
            } else {
                // Full period
                atr.push(true_ranges[i - (period - 1)..=i].iter().sum::<f64>() / period as f64);
            }
        }
        
        atr
    }
}

impl<M: ForecastModel> ForecastStrategy for VolatilityBreakoutStrategy<M> {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        if data.len() < self.config.lookback_period {
            return Err(ForecastError::DataError(format!(
                "Insufficient data. Need at least {} data points",
                self.config.lookback_period
            )));
        }
        
        let prices = data.close_prices();
        let mut signals = vec![TradingSignal::Hold; prices.len()];
        
        // Calculate volatility measure (ATR or standard deviation)
        let volatility = if self.config.use_atr {
            self.calculate_atr(data, self.config.lookback_period)
        } else {
            // Calculate rolling standard deviation
            let mut std_devs = Vec::with_capacity(prices.len());
            for i in 0..prices.len() {
                if i < self.config.lookback_period - 1 {
                    std_devs.push(0.0);
                    continue;
                }
                
                let window = &prices[i - (self.config.lookback_period - 1)..=i];
                let mean = window.iter().sum::<f64>() / window.len() as f64;
                let variance = window.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum::<f64>() / window.len() as f64;
                    
                std_devs.push(variance.sqrt());
            }
            std_devs
        };
        
        // Use the model to get a forecast
        let model_result = self.model.train(data)?;
        let forecast_result = model_result.forecast(data, 1)?;
        let forecast = if !forecast_result.values.is_empty() {
            forecast_result.values[0]
        } else {
            prices[prices.len() - 1] // Use last price if no forecast
        };
        
        // Generate signals based on volatility breakouts
        for i in self.config.lookback_period..prices.len() {
            let breakout_threshold = volatility[i] * self.config.volatility_multiplier;
            
            // Calculate if we have a breakout
            if prices[i] > prices[i - 1] + breakout_threshold {
                // Upward breakout
                signals[i] = TradingSignal::Buy;
            } else if prices[i] < prices[i - 1] - breakout_threshold {
                // Downward breakout
                signals[i] = TradingSignal::Sell;
            }
            
            // Apply trailing stops if enabled
            if self.config.use_trailing_stop && i > 0 {
                let trail_threshold = volatility[i] * self.config.trailing_stop_multiplier;
                
                // Check if we need to exit a long position
                if signals[i - 1] == TradingSignal::Buy && 
                   prices[i] < prices[i - 1] - trail_threshold {
                    signals[i] = TradingSignal::Sell;
                }
                
                // Check if we need to exit a short position
                if signals[i - 1] == TradingSignal::Sell && 
                   prices[i] > prices[i - 1] + trail_threshold {
                    signals[i] = TradingSignal::Buy;
                }
            }
        }
        
        Ok(signals)
    }

    fn backtest(&self, data: &TimeSeriesData, initial_balance: f64) -> Result<BacktestResults> {
        // Default commission and slippage values based on time granularity
        let (commission_rate, slippage) = match self.time_granularity {
            TimeGranularity::Daily => (0.001, 0.0005), // 0.1% commission, 0.05% slippage
            TimeGranularity::Minute => (0.0005, 0.001), // 0.05% commission, 0.1% slippage
        };

        self.backtest_with_params(data, initial_balance, commission_rate, slippage)
    }

    fn backtest_with_params(
        &self,
        data: &TimeSeriesData,
        initial_balance: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Result<BacktestResults> {
        if data.is_empty() {
            return Err(ForecastError::DataError("Empty data".to_string()));
        }

        let signals = self.generate_signals(data)?;
        let prices = data.close_prices();

        // Simple backtesting logic
        let mut balance = initial_balance;
        let mut position = 0.0;
        let mut total_trades = 0;
        let mut winning_trades = 0;
        let mut max_balance = initial_balance;
        let mut max_drawdown = 0.0;

        // For calculating Sharpe ratio
        let mut trade_returns = Vec::new();

        for i in 1..prices.len() {
            let current_price = prices[i];
            let prev_price = prices[i - 1];

            // Current portfolio value
            let portfolio_value = balance + position * prev_price;

            match signals[i] {
                TradingSignal::Buy if position <= 0.0 => {
                    // Close any short position
                    if position < 0.0 {
                        let trade_value = (-position) * current_price;
                        let trade_pnl = -position * (prev_price - current_price);
                        let commission = trade_value * commission_rate;
                        let slip = trade_value * slippage;

                        balance += trade_value - commission - slip;

                        if trade_pnl > 0.0 {
                            winning_trades += 1;
                        }

                        total_trades += 1;
                        trade_returns.push(trade_pnl / portfolio_value);
                    }

                    // Open long position
                    let shares = balance / current_price;
                    let cost = shares * current_price;
                    let commission = cost * commission_rate;
                    let slip = cost * slippage;

                    balance -= cost + commission + slip;
                    position = shares;
                    total_trades += 1;
                }
                TradingSignal::Sell if position >= 0.0 => {
                    // Close any long position
                    if position > 0.0 {
                        let trade_value = position * current_price;
                        let trade_pnl = position * (current_price - prev_price);
                        let commission = trade_value * commission_rate;
                        let slip = trade_value * slippage;

                        balance += trade_value - commission - slip;

                        if trade_pnl > 0.0 {
                            winning_trades += 1;
                        }

                        total_trades += 1;
                        trade_returns.push(trade_pnl / portfolio_value);
                    }

                    // Open short position
                    let shares = balance / current_price;
                    let cost = shares * current_price;
                    let commission = cost * commission_rate;
                    let slip = cost * slippage;

                    balance -= commission + slip;
                    position = -shares;
                    total_trades += 1;
                }
                _ => (), // Hold signal, do nothing
            }

            // Update maximum drawdown
            let current_value = balance + position * current_price;
            if current_value > max_balance {
                max_balance = current_value;
            } else {
                let drawdown = (max_balance - current_value) / max_balance;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
        }

        // Close any remaining position at the end
        let final_price = prices[prices.len() - 1];
        if position != 0.0 {
            let position_value = position.abs() * final_price;
            let commission = position_value * commission_rate;
            let slip = position_value * slippage;

            if position > 0.0 {
                balance += position_value - commission - slip;
            } else {
                balance += 2.0 * position.abs() * final_price - commission - slip;
            }
        }

        // Calculate win rate
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        // Calculate Sharpe ratio
        let sharpe_ratio = if !trade_returns.is_empty() {
            let mean_return = trade_returns.iter().sum::<f64>() / trade_returns.len() as f64;
            let variance = trade_returns
                .iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>()
                / trade_returns.len() as f64;

            if variance > 0.0 {
                Some(mean_return / variance.sqrt())
            } else {
                None
            }
        } else {
            None
        };

        Ok(BacktestResults {
            final_balance: balance,
            total_trades,
            win_rate,
            max_drawdown,
            performance_metrics: PerformanceMetrics {
                sharpe_ratio,
                sortino_ratio: None, // Not implemented
                calmar_ratio: None,  // Not implemented
            },
        })
    }

    fn time_granularity(&self) -> TimeGranularity {
        self.time_granularity
    }
}
