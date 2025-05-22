use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::{ForecastModel, TrainedForecastModel};
use crate::strategies::{BacktestResults, ForecastStrategy, TimeGranularity, TradingSignal};
use std::marker::PhantomData;

/// Mean reversion strategy configuration
#[derive(Debug, Clone)]
pub struct MeanReversionConfig {
    /// Threshold for triggering signals (standard deviations)
    pub threshold: f64,
    /// Time granularity
    pub time_granularity: TimeGranularity,
    /// Lookback period (adjusts based on time granularity)
    pub lookback_period: usize,
}

impl Default for MeanReversionConfig {
    fn default() -> Self {
        Self {
            threshold: 2.0,
            time_granularity: TimeGranularity::Daily,
            lookback_period: 20, // Default for daily
        }
    }
}

/// Mean reversion strategy
///
/// This strategy generates buy signals when the price falls significantly below
/// its moving average, and sell signals when it rises significantly above it.
#[derive(Debug, Clone)]
pub struct MeanReversionStrategy<M: ForecastModel> {
    /// The forecast model to use
    model: M,
    /// Configuration
    config: MeanReversionConfig,
}

impl<M: ForecastModel> MeanReversionStrategy<M> {
    /// Create a new mean reversion strategy with default time granularity (Daily)
    pub fn new(model: M, threshold: f64) -> Result<Self> {
        if threshold <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Threshold must be positive".to_string(),
            ));
        }

        let config = MeanReversionConfig {
            threshold,
            time_granularity: TimeGranularity::Daily,
            lookback_period: 20, // Default for daily
        };

        Ok(Self { model, config })
    }

    /// Create a new mean reversion strategy with specified time granularity
    pub fn new_with_granularity(
        model: M,
        threshold: f64,
        time_granularity: TimeGranularity,
    ) -> Result<Self> {
        if threshold <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Threshold must be positive".to_string(),
            ));
        }

        // Adjust lookback period based on time granularity
        let lookback_period = match time_granularity {
            TimeGranularity::Daily => 20,  // 20 days
            TimeGranularity::Minute => 60, // 60 minutes
        };

        let config = MeanReversionConfig {
            threshold,
            time_granularity,
            lookback_period,
        };

        Ok(Self { model, config })
    }

    /// Create a new mean reversion strategy with custom configuration
    pub fn new_with_config(model: M, config: MeanReversionConfig) -> Result<Self> {
        if config.threshold <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Threshold must be positive".to_string(),
            ));
        }

        Ok(Self { model, config })
    }
}

impl<M: ForecastModel> ForecastStrategy for MeanReversionStrategy<M> {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        if data.is_empty() {
            return Err(ForecastError::DataError(
                "Empty time series data".to_string(),
            ));
        }

        // Adjust parameters based on time granularity
        let lookback = self.config.lookback_period;

        // Calculate moving average
        let prices = data.close_prices();
        if prices.len() < lookback {
            return Err(ForecastError::DataError(format!(
                "Not enough data points. Need at least {} points.",
                lookback
            )));
        }

        // Train the model on the data
        let trained_model = self.model.train(data)?;

        // Get the model's predictions
        let predictions = trained_model.predict(data)?;

        // Calculate the standard deviation
        let mut std_dev = 0.0;
        let mut sum_squared_diff = 0.0;
        for i in 0..prices.len() {
            let diff = prices[i] - predictions.values()[i];
            sum_squared_diff += diff * diff;
        }
        std_dev = (sum_squared_diff / prices.len() as f64).sqrt();

        // Generate signals based on the difference between actual and predicted
        let mut signals = Vec::with_capacity(prices.len());

        for i in 0..prices.len() {
            let diff = prices[i] - predictions.values()[i];
            let normalized_diff = diff / std_dev;

            if normalized_diff < -self.config.threshold {
                // Price is significantly below prediction - buy signal
                signals.push(TradingSignal::Buy);
            } else if normalized_diff > self.config.threshold {
                // Price is significantly above prediction - sell signal
                signals.push(TradingSignal::Sell);
            } else {
                // Price is within normal range - hold signal
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
