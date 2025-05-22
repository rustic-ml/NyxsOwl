use crate::data::TimeSeriesData;
use crate::error::{ForecastError, Result};
use crate::models::{ForecastModel, TrainedForecastModel};
use crate::strategies::{BacktestResults, ForecastStrategy, TimeGranularity, TradingSignal};

/// Volatility breakout strategy configuration
#[derive(Debug, Clone)]
pub struct VolatilityBreakoutConfig {
    /// Volatility multiplier for breakout bands
    pub volatility_multiplier: f64,
    /// Time granularity
    pub time_granularity: TimeGranularity,
    /// Lookback period for volatility calculation
    pub lookback_period: usize,
}

impl Default for VolatilityBreakoutConfig {
    fn default() -> Self {
        Self {
            volatility_multiplier: 1.5,
            time_granularity: TimeGranularity::Daily,
            lookback_period: 20, // Default for daily
        }
    }
}

/// Volatility breakout strategy
///
/// This strategy generates buy signals when the price breaks out above a volatility band
/// and sell signals when the price breaks below a volatility band.
#[derive(Debug, Clone)]
pub struct VolatilityBreakoutStrategy<M: ForecastModel> {
    /// The forecast model to use
    model: M,
    /// Configuration
    config: VolatilityBreakoutConfig,
}

impl<M: ForecastModel> VolatilityBreakoutStrategy<M> {
    /// Create a new volatility breakout strategy with default time granularity (Daily)
    pub fn new(model: M, volatility_multiplier: f64) -> Result<Self> {
        if volatility_multiplier <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Volatility multiplier must be positive".to_string(),
            ));
        }

        let config = VolatilityBreakoutConfig {
            volatility_multiplier,
            time_granularity: TimeGranularity::Daily,
            lookback_period: 20, // Default for daily
        };

        Ok(Self { model, config })
    }

    /// Create a new volatility breakout strategy with specified time granularity
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

        // Adjust lookback period based on time granularity
        let lookback_period = match time_granularity {
            TimeGranularity::Daily => 20,  // 20 days
            TimeGranularity::Minute => 60, // 60 minutes
        };

        let config = VolatilityBreakoutConfig {
            volatility_multiplier,
            time_granularity,
            lookback_period,
        };

        Ok(Self { model, config })
    }

    /// Create a new volatility breakout strategy with custom configuration
    pub fn new_with_config(model: M, config: VolatilityBreakoutConfig) -> Result<Self> {
        if config.volatility_multiplier <= 0.0 {
            return Err(ForecastError::InvalidParameter(
                "Volatility multiplier must be positive".to_string(),
            ));
        }

        Ok(Self { model, config })
    }
}

impl<M: ForecastModel> ForecastStrategy for VolatilityBreakoutStrategy<M> {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        if data.is_empty() {
            return Err(ForecastError::DataError(
                "Empty time series data".to_string(),
            ));
        }

        // Calculate volatility
        let prices = data.close_prices();
        if prices.len() < self.config.lookback_period {
            return Err(ForecastError::DataError(format!(
                "Not enough data points. Need at least {} points.",
                self.config.lookback_period
            )));
        }

        // Train the model on the data
        let trained_model = self.model.train(data)?;

        // Get the model's predictions
        let predictions = trained_model.predict(data)?;

        // Calculate the standard deviation (volatility)
        let mut volatility = Vec::with_capacity(prices.len());

        // Initial volatility calculation using lookback window
        for i in self.config.lookback_period..prices.len() {
            let window = &prices[(i - self.config.lookback_period)..i];
            let mean = window.iter().sum::<f64>() / window.len() as f64;
            let variance = window
                .iter()
                .map(|&price| (price - mean).powi(2))
                .sum::<f64>()
                / window.len() as f64;
            volatility.push(variance.sqrt());
        }

        // Generate signals based on volatility bands
        let mut signals = Vec::with_capacity(prices.len());

        // Fill initial signals with Hold for the lookback period
        for _ in 0..self.config.lookback_period {
            signals.push(TradingSignal::Hold);
        }

        // Generate signals for the rest of the data
        for i in 0..(prices.len() - self.config.lookback_period) {
            let current_price = prices[i + self.config.lookback_period];
            let predicted_price = predictions.values()[i + self.config.lookback_period];
            let current_volatility = volatility[i];

            // Calculate volatility bands
            let upper_band =
                predicted_price + self.config.volatility_multiplier * current_volatility;
            let lower_band =
                predicted_price - self.config.volatility_multiplier * current_volatility;

            if current_price > upper_band {
                // Price breaks above upper band - buy signal
                signals.push(TradingSignal::Buy);
            } else if current_price < lower_band {
                // Price breaks below lower band - sell signal
                signals.push(TradingSignal::Sell);
            } else {
                // Price is within bands - hold signal
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
