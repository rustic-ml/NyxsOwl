//! Trading strategies based on forecasting models
//!
//! This module provides a framework for implementing trading strategies based on forecasting models.
//! It includes the core traits, common implementations, and utility functions for strategy development.

#[cfg(any(feature = "day-trading", feature = "minute-trading"))]
use crate::day_trade::{DailyOhlcv, Signal};
use crate::forecast_trade::data::TimeSeriesData;
use crate::forecast_trade::error::Result;
use crate::forecast_trade::models::ForecastModel;

/// Trading signal types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingSignal {
    /// Buy signal
    Buy,
    /// Sell signal
    Sell,
    /// Hold/neutral signal
    Hold,
}

/// Time granularity for trading strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeGranularity {
    /// Daily trading data
    Daily,
    /// Minute-level trading data
    Minute,
}

impl TimeGranularity {
    /// Get default commission rate for this granularity
    pub fn default_commission_rate(&self) -> f64 {
        match self {
            TimeGranularity::Daily => 0.001,   // 0.1% commission for daily
            TimeGranularity::Minute => 0.0005, // 0.05% commission for minute
        }
    }

    /// Get default slippage for this granularity
    pub fn default_slippage(&self) -> f64 {
        match self {
            TimeGranularity::Daily => 0.0005,  // 0.05% slippage for daily
            TimeGranularity::Minute => 0.0002, // 0.02% slippage for minute
        }
    }
}

/// Result of a strategy backtest
#[derive(Debug, Clone)]
pub struct BacktestResult {
    /// Final account balance
    pub final_balance: f64,
    /// Total percentage return
    pub total_return: f64,
    /// Maximum drawdown (percentage)
    pub max_drawdown: f64,
    /// Win rate (percentage of profitable trades)
    pub win_rate: f64,
    /// Equity curve (account balance over time)
    pub equity_curve: Vec<f64>,
    /// Number of trades executed
    pub trades: usize,
    /// Performance metrics
    pub performance_metrics: Option<PerformanceMetrics>,
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

/// Common interface for trading strategies based on forecasting models
pub trait ForecastStrategy {
    /// Generate trading signals based on forecasts
    fn generate_signals(&self, data: &TimeSeriesData) -> Result<Vec<TradingSignal>>;

    /// Backtest the strategy on historical data
    fn backtest(&self, data: &TimeSeriesData, initial_capital: f64) -> Result<BacktestResult> {
        // Get default commission and slippage based on granularity
        let granularity = self.time_granularity();
        let commission_rate = granularity.default_commission_rate();
        let slippage = granularity.default_slippage();

        self.backtest_with_params(data, initial_capital, commission_rate, slippage)
    }

    /// Backtest with custom parameters for commission and slippage
    fn backtest_with_params(
        &self,
        data: &TimeSeriesData,
        initial_balance: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Result<BacktestResult>;

    /// Get the strategy's time granularity preference
    fn time_granularity(&self) -> TimeGranularity;

    /// Generate signals with daily OHLCV data
    #[cfg(feature = "day-trading")]
    fn generate_signals_daily(
        &self,
        data: &[crate::day_trade::DailyOhlcv],
    ) -> Result<Vec<TradingSignal>> {
        let time_series = Self::convert_daily_to_time_series(data)?;
        self.generate_signals(&time_series)
    }

    /// Generate signals with minute OHLCV data
    #[cfg(feature = "minute-trading")]
    fn generate_signals_minute(
        &self,
        data: &[crate::minute_trade::MinuteOhlcv],
    ) -> Result<Vec<TradingSignal>> {
        let time_series = Self::convert_minute_to_time_series(data)?;
        self.generate_signals(&time_series)
    }

    /// Run backtest with daily OHLCV data
    #[cfg(feature = "day-trading")]
    fn backtest_daily(
        &self,
        data: &[crate::day_trade::DailyOhlcv],
        initial_balance: f64,
    ) -> Result<BacktestResult> {
        let time_series = Self::convert_daily_to_time_series(data)?;
        self.backtest(&time_series, initial_balance)
    }

    /// Run backtest with minute OHLCV data
    #[cfg(feature = "minute-trading")]
    fn backtest_minute(
        &self,
        data: &[crate::minute_trade::MinuteOhlcv],
        initial_balance: f64,
    ) -> Result<BacktestResult> {
        let time_series = Self::convert_minute_to_time_series(data)?;
        self.backtest(&time_series, initial_balance)
    }

    /// Helper method to convert daily OHLCV data to TimeSeriesData
    #[cfg(feature = "day-trading")]
    fn convert_daily_to_time_series(
        data: &[crate::day_trade::DailyOhlcv],
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
    #[cfg(feature = "minute-trading")]
    fn convert_minute_to_time_series(
        data: &[crate::minute_trade::MinuteOhlcv],
    ) -> Result<TimeSeriesData> {
        let dates = data.iter().map(|d| d.timestamp).collect();

        let ohlc_data = data
            .iter()
            .map(|d| (d.data.open, d.data.high, d.data.low, d.data.close))
            .collect();

        TimeSeriesData::new_ohlc(dates, ohlc_data)
    }
}

/// Base strategy implementation that reduces boilerplate code
#[derive(Debug)]
pub struct BaseStrategy<M>
where
    M: ForecastModel + Clone + ForecastStrategy + 'static,
{
    /// Strategy name
    pub name: String,
    /// Forecast model
    pub model: M,
    /// Time granularity
    pub time_granularity: TimeGranularity,
    /// Whether the model has been trained
    pub is_trained: bool,
}

impl<M> BaseStrategy<M>
where
    M: ForecastModel + Clone + ForecastStrategy + 'static,
{
    /// Create a new base strategy
    pub fn new(name: &str, model: M) -> Self {
        let granularity = ForecastModel::time_granularity(&model);
        Self {
            name: name.to_string(),
            model,
            time_granularity: granularity,
            is_trained: false,
        }
    }

    /// Create a new base strategy with specific granularity
    pub fn new_with_granularity(name: &str, model: M, granularity: TimeGranularity) -> Self {
        Self {
            name: name.to_string(),
            model,
            time_granularity: granularity,
            is_trained: false,
        }
    }

    /// Get the trained model, training it if necessary
    pub fn get_trained_model(&self, data: &TimeSeriesData) -> Result<Box<dyn ForecastModel>> {
        if !self.is_trained {
            self.model.train(data)
        } else {
            Ok(Box::new(self.model.clone()))
        }
    }
}

impl<M> Clone for BaseStrategy<M>
where
    M: ForecastModel + Clone + ForecastStrategy + 'static,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            model: self.model.clone(),
            time_granularity: self.time_granularity,
            is_trained: self.is_trained,
        }
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

    fn backtest_with_params(
        &self,
        data: &TimeSeriesData,
        initial_balance: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Result<BacktestResult> {
        if data.is_empty() {
            return Err(crate::forecast_trade::ForecastError::DataError(
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

        let performance_metrics = if let Some(sharpe) = sharpe_ratio {
            Some(PerformanceMetrics {
                sharpe_ratio: Some(sharpe),
                sortino_ratio: None,
                calmar_ratio: None,
            })
        } else {
            None
        };

        Ok(BacktestResult {
            final_balance: balance,
            total_return: (balance - initial_balance) / initial_balance,
            max_drawdown,
            win_rate,
            equity_curve: Vec::new(),
            trades: total_trades,
            performance_metrics,
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

    fn backtest_with_params(
        &self,
        data: &TimeSeriesData,
        initial_balance: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Result<BacktestResult> {
        if data.is_empty() {
            return Err(crate::forecast_trade::ForecastError::DataError(
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

        let performance_metrics = if let Some(sharpe) = sharpe_ratio {
            Some(PerformanceMetrics {
                sharpe_ratio: Some(sharpe),
                sortino_ratio: None,
                calmar_ratio: None,
            })
        } else {
            None
        };

        Ok(BacktestResult {
            final_balance: balance,
            total_return: (balance - initial_balance) / initial_balance,
            max_drawdown,
            win_rate,
            equity_curve: Vec::new(),
            trades: total_trades,
            performance_metrics,
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

    fn backtest_with_params(
        &self,
        data: &TimeSeriesData,
        initial_balance: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Result<BacktestResult> {
        if data.is_empty() {
            return Err(crate::forecast_trade::ForecastError::DataError(
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

        let performance_metrics = if let Some(sharpe) = sharpe_ratio {
            Some(PerformanceMetrics {
                sharpe_ratio: Some(sharpe),
                sortino_ratio: None,
                calmar_ratio: None,
            })
        } else {
            None
        };

        Ok(BacktestResult {
            final_balance: balance,
            total_return: (balance - initial_balance) / initial_balance,
            max_drawdown,
            win_rate,
            equity_curve: Vec::new(),
            trades: total_trades,
            performance_metrics,
        })
    }

    fn time_granularity(&self) -> TimeGranularity {
        self.time_granularity
    }
}

// Export individual strategy modules
pub mod arima_strategy;
pub mod mean_reversion;
pub mod trend_following;
pub mod volatility_breakout;
pub mod volatility_strategy;
