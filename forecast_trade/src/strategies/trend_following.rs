use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::{ForecastModel, TrainedForecastModel};
use crate::strategies::{BacktestResults, ForecastStrategy, TimeGranularity, TradingSignal};

/// Trend following strategy configuration
#[derive(Debug, Clone)]
pub struct TrendFollowingConfig {
    /// Window size for trend detection
    pub window_size: usize,
    /// Time granularity
    pub time_granularity: TimeGranularity,
    /// Momentum threshold
    pub momentum_threshold: f64,
}

impl Default for TrendFollowingConfig {
    fn default() -> Self {
        Self {
            window_size: 5,
            time_granularity: TimeGranularity::Daily,
            momentum_threshold: 0.01, // 1% for daily
        }
    }
}

/// Trend following strategy
///
/// This strategy generates buy signals when the price is trending upward
/// and sell signals when the price is trending downward.
#[derive(Debug, Clone)]
pub struct TrendFollowingStrategy<M: ForecastModel> {
    /// The forecast model to use
    model: M,
    /// Configuration
    config: TrendFollowingConfig,
}

impl<M: ForecastModel> TrendFollowingStrategy<M> {
    /// Create a new trend following strategy with default time granularity (Daily)
    pub fn new(model: M, window_size: usize) -> Result<Self> {
        if window_size == 0 {
            return Err(ForecastError::ValidationError(
                "Window size must be positive".to_string(),
            ));
        }

        let config = TrendFollowingConfig {
            window_size,
            time_granularity: TimeGranularity::Daily,
            momentum_threshold: 0.01, // 1% for daily
        };

        Ok(Self { model, config })
    }

    /// Create a new trend following strategy with specified time granularity
    pub fn new_with_granularity(
        model: M,
        window_size: usize,
        time_granularity: TimeGranularity,
    ) -> Result<Self> {
        if window_size == 0 {
            return Err(ForecastError::ValidationError(
                "Window size must be positive".to_string(),
            ));
        }

        // Adjust momentum threshold based on time granularity
        let momentum_threshold = match time_granularity {
            TimeGranularity::Daily => 0.01,   // 1% for daily
            TimeGranularity::Minute => 0.001, // 0.1% for minute
        };

        let config = TrendFollowingConfig {
            window_size,
            time_granularity,
            momentum_threshold,
        };

        Ok(Self { model, config })
    }

    /// Create a new trend following strategy with custom configuration
    pub fn new_with_config(model: M, config: TrendFollowingConfig) -> Result<Self> {
        if config.window_size == 0 {
            return Err(ForecastError::ValidationError(
                "Window size must be positive".to_string(),
            ));
        }

        Ok(Self { model, config })
    }
}

impl<M: ForecastModel> ForecastStrategy for TrendFollowingStrategy<M> {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        if data.is_empty() {
            return Err(ForecastError::DataError(
                "Empty time series data".to_string(),
            ));
        }

        // Calculate moving average
        let prices = data.close_prices();
        if prices.len() < self.config.window_size {
            return Err(ForecastError::DataError(format!(
                "Not enough data points. Need at least {} points.",
                self.config.window_size
            )));
        }

        // Train the model on the data
        let trained_model = self.model.train(data)?;

        // Generate forecast
        let forecast = trained_model.forecast(1)?;

        // Generate signals based on the trend
        let mut signals = Vec::with_capacity(prices.len());

        // Fill initial signals with Hold
        for _ in 0..self.config.window_size {
            signals.push(TradingSignal::Hold);
        }

        // Calculate momentum for each point after the initial window
        for i in self.config.window_size..prices.len() {
            let window_start = i - self.config.window_size;
            let window_end = i;

            let window_start_price = prices[window_start];
            let window_end_price = prices[window_end];

            // Calculate momentum as percent change
            let momentum = (window_end_price - window_start_price) / window_start_price;

            // Adjust threshold based on time granularity
            let threshold = self.config.momentum_threshold;

            if momentum > threshold {
                // Strong uptrend - buy signal
                signals.push(TradingSignal::Buy);
            } else if momentum < -threshold {
                // Strong downtrend - sell signal
                signals.push(TradingSignal::Sell);
            } else {
                // No strong trend - hold signal
                signals.push(TradingSignal::Hold);
            }
        }

        Ok(signals)
    }

    fn backtest(&self, data: &TimeSeriesData, initial_balance: f64) -> Result<BacktestResults> {
        // Default parameters based on time granularity
        let (commission_rate, slippage) = match self.time_granularity() {
            TimeGranularity::Daily => (0.001, 0.0005), // 0.1% commission, 0.05% slippage for daily
            TimeGranularity::Minute => (0.0005, 0.001), // 0.05% commission, 0.1% slippage for minute
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
            return Err(ForecastError::DataError(
                "Empty time series data".to_string(),
            ));
        }

        // Generate signals
        let signals = self.generate_signals(data)?;
        let prices = data.close_prices();

        // Run backtest
        let mut balance = initial_balance;
        let mut position = 0.0;
        let mut total_trades = 0;
        let mut winning_trades = 0;
        let mut max_balance = initial_balance;
        let mut max_drawdown = 0.0;

        let mut trade_returns = Vec::new();

        for i in 1..prices.len() {
            let current_price = prices[i];
            let prev_price = prices[i - 1];

            // Portfolio value before the current trade
            let portfolio_value = balance + position * prev_price;

            match signals[i] {
                TradingSignal::Buy if position <= 0.0 => {
                    // Close any short position first
                    if position < 0.0 {
                        let close_value = -position * current_price;
                        let trade_pnl = -position * (prev_price - current_price);
                        let commission = close_value * commission_rate;
                        let slip = close_value * slippage;

                        balance += close_value - commission - slip;

                        if trade_pnl > 0.0 {
                            winning_trades += 1;
                        }
                        total_trades += 1;
                        trade_returns.push(trade_pnl / (portfolio_value));
                    }

                    // Open a long position
                    let shares = balance / current_price;
                    let cost = shares * current_price;
                    let commission = cost * commission_rate;
                    let slip = cost * slippage;

                    balance -= cost + commission + slip;
                    position = shares;
                    total_trades += 1;
                }
                TradingSignal::Sell if position >= 0.0 => {
                    // Close any long position first
                    if position > 0.0 {
                        let close_value = position * current_price;
                        let trade_pnl = position * (current_price - prev_price);
                        let commission = close_value * commission_rate;
                        let slip = close_value * slippage;

                        balance += close_value - commission - slip;

                        if trade_pnl > 0.0 {
                            winning_trades += 1;
                        }
                        total_trades += 1;
                        trade_returns.push(trade_pnl / (portfolio_value));
                    }

                    // Open a short position
                    let shares = balance / current_price;
                    let cost = shares * current_price;
                    let commission = cost * commission_rate;
                    let slip = cost * slippage;

                    balance -= commission + slip;
                    position = -shares;
                    total_trades += 1;
                }
                _ => (), // Hold, do nothing
            }

            // Calculate current portfolio value
            let current_value = balance + position * current_price;

            // Update max balance and drawdown
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
            let close_value = position.abs() * final_price;
            let commission = close_value * commission_rate;
            let slip = close_value * slippage;

            if position > 0.0 {
                balance += close_value - commission - slip;
            } else {
                balance += 2.0 * position.abs() * final_price - commission - slip;
            }
            position = 0.0;
        }

        // Calculate performance metrics
        let final_balance = balance;
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        // Calculate Sharpe ratio if we have returns
        let sharpe_ratio = if !trade_returns.is_empty() {
            let mean_return = trade_returns.iter().sum::<f64>() / trade_returns.len() as f64;
            let variance = trade_returns
                .iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>()
                / trade_returns.len() as f64;
            let std_dev = variance.sqrt();

            if std_dev > 0.0 {
                Some(mean_return / std_dev)
            } else {
                None
            }
        } else {
            None
        };

        // Return the results
        Ok(BacktestResults {
            final_balance,
            total_trades,
            win_rate,
            max_drawdown,
            performance_metrics: crate::strategies::PerformanceMetrics {
                sharpe_ratio,
                sortino_ratio: None, // Not calculated for simplicity
                calmar_ratio: None,  // Not calculated for simplicity
            },
        })
    }

    fn time_granularity(&self) -> TimeGranularity {
        self.config.time_granularity
    }
}
