//! Trading strategies based on forecasting models

use crate::data::TimeSeriesData;
use crate::error::Result;
use crate::models::ForecastResult;
use day_trade::{DailyOhlcv, Signal};

/// Time granularity for strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeGranularity {
    /// Daily data
    Daily,
    /// Minute data
    Minute,
}

impl Default for TimeGranularity {
    fn default() -> Self {
        TimeGranularity::Daily
    }
}

/// Trading signal emitted by strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradingSignal {
    /// Buy signal
    Buy,
    /// Sell signal
    Sell,
    /// Hold signal (no action)
    Hold,
}

/// Backtest results
#[derive(Debug, Clone)]
pub struct BacktestResults {
    /// Final portfolio balance
    pub final_balance: f64,
    /// Total number of trades executed
    pub total_trades: usize,
    /// Win rate (ratio of profitable trades)
    pub win_rate: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics for strategies
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Sharpe ratio
    pub sharpe_ratio: Option<f64>,
    /// Sortino ratio
    pub sortino_ratio: Option<f64>,
    /// Calmar ratio
    pub calmar_ratio: Option<f64>,
}

/// Common interface for forecast-based trading strategies
pub trait ForecastStrategy {
    /// Generate trading signals from time series data
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>>;

    /// Run backtest with default parameters
    fn backtest(&self, data: &TimeSeriesData, initial_balance: f64) -> Result<BacktestResults>;

    /// Run backtest with custom parameters
    fn backtest_with_params(
        &self,
        data: &TimeSeriesData,
        initial_balance: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Result<BacktestResults>;

    /// Get the strategy's time granularity
    fn time_granularity(&self) -> TimeGranularity;

    /// Generate signals with daily OHLCV data
    fn generate_signals_daily(&self, data: &[day_trade::DailyOhlcv]) -> Result<Vec<TradingSignal>> {
        let time_series = self.convert_daily_to_time_series(data)?;
        self.generate_signals(&time_series)
    }

    /// Generate signals with minute OHLCV data
    fn generate_signals_minute(
        &self,
        data: &[minute_trade::MinuteOhlcv],
    ) -> Result<Vec<TradingSignal>> {
        let time_series = self.convert_minute_to_time_series(data)?;
        self.generate_signals(&time_series)
    }

    /// Run backtest with daily OHLCV data
    fn backtest_daily(
        &self,
        data: &[day_trade::DailyOhlcv],
        initial_balance: f64,
    ) -> Result<BacktestResults> {
        let time_series = self.convert_daily_to_time_series(data)?;
        self.backtest(&time_series, initial_balance)
    }

    /// Run backtest with minute OHLCV data
    fn backtest_minute(
        &self,
        data: &[minute_trade::MinuteOhlcv],
        initial_balance: f64,
    ) -> Result<BacktestResults> {
        let time_series = self.convert_minute_to_time_series(data)?;
        self.backtest(&time_series, initial_balance)
    }

    /// Helper method to convert daily OHLCV data to TimeSeriesData
    fn convert_daily_to_time_series(
        &self,
        data: &[day_trade::DailyOhlcv],
    ) -> Result<TimeSeriesData> {
        let dates = data
            .iter()
            .map(|d| {
                let naive = chrono::NaiveDateTime::new(d.date, chrono::NaiveTime::default());
                chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive, chrono::Utc)
            })
            .collect();

        let ohlc_data = data
            .iter()
            .map(|d| (d.data.open, d.data.high, d.data.low, d.data.close))
            .collect();

        TimeSeriesData::new_ohlc(dates, ohlc_data)
    }

    /// Helper method to convert minute OHLCV data to TimeSeriesData
    fn convert_minute_to_time_series(
        &self,
        data: &[minute_trade::MinuteOhlcv],
    ) -> Result<TimeSeriesData> {
        let dates = data.iter().map(|d| d.timestamp).collect();

        let ohlc_data = data
            .iter()
            .map(|d| (d.data.open, d.data.high, d.data.low, d.data.close))
            .collect();

        TimeSeriesData::new_ohlc(dates, ohlc_data)
    }
}

/// Trend following strategy based on forecasted price movement
#[derive(Debug, Clone)]
pub struct TrendFollowingStrategy {
    /// Strategy name
    name: String,
    /// Threshold for entering a trade (percent change)
    threshold: f64,
    /// Time granularity
    time_granularity: TimeGranularity,
}

impl TrendFollowingStrategy {
    /// Create a new trend following strategy
    pub fn new(threshold: f64) -> Self {
        Self {
            name: format!("Trend Following (threshold={}%)", threshold),
            threshold,
            time_granularity: TimeGranularity::Daily,
        }
    }

    /// Create a new trend following strategy with specific time granularity
    pub fn new_with_granularity(threshold: f64, time_granularity: TimeGranularity) -> Self {
        Self {
            name: format!(
                "Trend Following (threshold={}%, granularity={:?})",
                threshold, time_granularity
            ),
            threshold,
            time_granularity,
        }
    }
}

impl ForecastStrategy for TrendFollowingStrategy {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut signals = Vec::with_capacity(data.len());
        let prices = data.close_prices();
        let base_value = prices[0];

        for value in &prices {
            let percent_change = (value - base_value) / base_value * 100.0;

            let signal = if percent_change > self.threshold {
                TradingSignal::Buy
            } else if percent_change < -self.threshold {
                TradingSignal::Sell
            } else {
                TradingSignal::Hold
            };

            signals.push(signal);
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
            return Err(crate::error::ForecastError::DataError(
                "Empty data".to_string(),
            ));
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

/// Mean reversion strategy based on forecasted price movement
#[derive(Debug, Clone)]
pub struct MeanReversionStrategy {
    /// Strategy name
    name: String,
    /// Threshold for mean deviation (percent)
    threshold: f64,
    /// Moving average window
    window: usize,
    /// Time granularity
    time_granularity: TimeGranularity,
}

impl MeanReversionStrategy {
    /// Create a new mean reversion strategy
    pub fn new(threshold: f64, window: usize) -> Self {
        Self {
            name: format!(
                "Mean Reversion (threshold={}%, window={})",
                threshold, window
            ),
            threshold,
            window,
            time_granularity: TimeGranularity::Daily,
        }
    }

    /// Create a new mean reversion strategy with specific time granularity
    pub fn new_with_granularity(
        threshold: f64,
        window: usize,
        time_granularity: TimeGranularity,
    ) -> Self {
        Self {
            name: format!(
                "Mean Reversion (threshold={}%, window={}, granularity={:?})",
                threshold, window, time_granularity
            ),
            threshold,
            window,
            time_granularity,
        }
    }
}

impl ForecastStrategy for MeanReversionStrategy {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        let prices = data.close_prices();
        if prices.len() < self.window {
            return Ok(Vec::new());
        }

        let mut signals = Vec::with_capacity(prices.len());

        // Calculate moving average
        for i in 0..prices.len() {
            if i < self.window - 1 {
                signals.push(TradingSignal::Hold);
                continue;
            }

            // Calculate moving average for this window
            let ma: f64 =
                prices[i - (self.window - 1)..=i].iter().sum::<f64>() / self.window as f64;

            // Current value
            let current = prices[i];

            // Calculate deviation
            let deviation = (current - ma) / ma * 100.0;

            // Generate signal
            let signal = if deviation > self.threshold {
                // Price is above MA by threshold - expect reversion, so sell
                TradingSignal::Sell
            } else if deviation < -self.threshold {
                // Price is below MA by threshold - expect reversion, so buy
                TradingSignal::Buy
            } else {
                TradingSignal::Hold
            };

            signals.push(signal);
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
            return Err(crate::error::ForecastError::DataError(
                "Empty data".to_string(),
            ));
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

/// Volatility-based strategy using GARCH forecasts
#[derive(Debug, Clone)]
pub struct VolatilityStrategy {
    /// Strategy name
    name: String,
    /// Threshold for high volatility (annualized)
    high_threshold: f64,
    /// Threshold for low volatility (annualized)
    low_threshold: f64,
    /// Time granularity
    time_granularity: TimeGranularity,
}

impl VolatilityStrategy {
    /// Create a new volatility-based strategy
    pub fn new(low_threshold: f64, high_threshold: f64) -> Self {
        Self {
            name: format!(
                "Volatility Strategy (low={}, high={})",
                low_threshold, high_threshold
            ),
            high_threshold,
            low_threshold,
            time_granularity: TimeGranularity::Daily,
        }
    }

    /// Create a new volatility-based strategy with specific time granularity
    pub fn new_with_granularity(
        low_threshold: f64,
        high_threshold: f64,
        time_granularity: TimeGranularity,
    ) -> Self {
        Self {
            name: format!(
                "Volatility Strategy (low={}, high={}, granularity={:?})",
                low_threshold, high_threshold, time_granularity
            ),
            high_threshold,
            low_threshold,
            time_granularity,
        }
    }
}

impl ForecastStrategy for VolatilityStrategy {
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>> {
        let prices = data.close_prices();
        let mut signals = Vec::with_capacity(prices.len());

        // Calculate simple volatility (rolling standard deviation)
        const WINDOW: usize = 20;

        for i in 0..prices.len() {
            if i < WINDOW {
                signals.push(TradingSignal::Hold);
                continue;
            }

            // Calculate mean
            let window_prices = &prices[(i - WINDOW)..i];
            let mean = window_prices.iter().sum::<f64>() / WINDOW as f64;

            // Calculate variance
            let variance = window_prices
                .iter()
                .map(|p| (p - mean).powi(2))
                .sum::<f64>()
                / WINDOW as f64;

            // Annualize volatility
            let annualization_factor: f64 = match self.time_granularity {
                TimeGranularity::Daily => 252.0, // Trading days in a year
                TimeGranularity::Minute => 252.0 * 6.5 * 60.0, // Minutes in a trading year
            };

            let annualized = variance.sqrt() * annualization_factor.sqrt();

            let signal = if annualized > self.high_threshold {
                // High volatility - reduce exposure
                TradingSignal::Sell
            } else if annualized < self.low_threshold {
                // Low volatility - increase exposure
                TradingSignal::Buy
            } else {
                // Normal volatility
                TradingSignal::Hold
            };

            signals.push(signal);
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
            return Err(crate::error::ForecastError::DataError(
                "Empty data".to_string(),
            ));
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

pub mod mean_reversion;
pub mod trend_following;
pub mod volatility_breakout;
