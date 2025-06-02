//! Performance calculation utilities
//!
//! Centralized, optimized performance calculation functions to reduce code duplication
//! and improve accuracy across all strategies.

use chrono::{DateTime, Utc};

#[cfg(feature = "day-trading")]
use crate::day_trade::{DailyOhlcv, Signal as DaySignal, TradeError as DayTradeError};

#[cfg(feature = "minute-trading")]
use crate::minute_trade::{Signal as MinuteSignal, TradeError as MinuteTradeError};

/// Performance metrics for trading strategies
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    /// Total return as a percentage
    pub total_return: f64,
    /// Maximum drawdown as a percentage
    pub max_drawdown: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Number of trades
    pub trade_count: usize,
    /// Win rate as a percentage
    pub win_rate: f64,
    /// Average profit per trade
    pub avg_profit_per_trade: f64,
    /// Volatility (annual)
    pub volatility: f64,
    /// Start date
    pub start_date: DateTime<Utc>,
    /// End date  
    pub end_date: DateTime<Utc>,
}

/// Input validation helper for day trading
#[cfg(feature = "day-trading")]
#[inline]
fn validate_daily_inputs(data_len: usize, signals_len: usize) -> Result<(), DayTradeError> {
    if data_len != signals_len {
        return Err(DayTradeError::InvalidData(
            "Data and signals arrays must be the same length".to_string(),
        ));
    }

    if data_len <= 1 {
        return Err(DayTradeError::InsufficientData(
            "Need at least 2 data points to calculate performance".to_string(),
        ));
    }

    Ok(())
}

/// Optimized basic performance calculation for daily trading
///
/// This replaces the duplicated calculate_performance functions across strategies
/// with a single, optimized implementation.
#[cfg(feature = "day-trading")]
pub fn calculate_daily_performance(
    data: &[DailyOhlcv],
    signals: &[DaySignal],
    initial_cash: f64,
    commission: f64,
) -> Result<f64, DayTradeError> {
    validate_daily_inputs(data.len(), signals.len())?;

    let mut portfolio_value = initial_cash;
    let mut position = 0.0;
    let mut cash = initial_cash;

    // Pre-allocate for better performance
    let mut in_position = false;

    for (i, &signal) in signals.iter().enumerate().skip(1).take(data.len() - 1) {
        let price = data[i].data.open;

        match signal {
            DaySignal::Buy if !in_position => {
                // Buy with all available cash, accounting for commission
                let shares = cash / (price * (1.0 + commission / 100.0));
                position = shares;
                cash = 0.0;
                in_position = true;
            }
            DaySignal::Sell if in_position => {
                // Sell all shares, accounting for commission
                cash = position * price * (1.0 - commission / 100.0);
                position = 0.0;
                in_position = false;
            }
            _ => {} // Hold or invalid signal
        }
    }

    // Calculate final portfolio value
    if let Some(last_data) = data.last() {
        portfolio_value = cash + position * last_data.data.close * (1.0 - commission / 100.0);
    }

    Ok((portfolio_value / initial_cash - 1.0) * 100.0)
}

/// Calculate performance metrics for minute trading strategies
#[cfg(feature = "minute-trading")]
pub fn calculate_minute_performance(
    data: &[crate::minute_trade::MinuteOhlcv],
    signals: &[crate::minute_trade::Signal],
    initial_cash: f64,
    commission_rate: f64,
) -> Result<f64, crate::minute_trade::TradeError> {
    validate_inputs(data.len(), signals.len())?;

    let mut portfolio_value = initial_cash;
    let mut position = 0.0;
    let mut cash = initial_cash;
    let mut in_position = false;

    for (i, &signal) in signals.iter().enumerate().skip(1).take(data.len() - 1) {
        let price = data[i].data.open;

        match signal {
            crate::minute_trade::Signal::Buy if position == 0.0 => {
                let shares = cash / (price * (1.0 + commission_rate / 100.0));
                position = shares;
                cash = 0.0;
                in_position = true;
            }
            crate::minute_trade::Signal::Sell if in_position => {
                cash = position * price * (1.0 - commission_rate / 100.0);
                position = 0.0;
                in_position = false;
            }
            _ => {}
        }
    }

    if let Some(last_data) = data.last() {
        portfolio_value = cash + position * last_data.data.close * (1.0 - commission_rate / 100.0);
    }

    Ok((portfolio_value / initial_cash - 1.0) * 100.0)
}

/// Calculate detailed performance metrics for minute trading strategies
#[cfg(feature = "minute-trading")]
pub fn calculate_detailed_minute_performance(
    data: &[crate::minute_trade::MinuteOhlcv],
    signals: &[crate::minute_trade::Signal],
    initial_cash: f64,
    commission_rate: f64,
) -> Result<PerformanceMetrics, crate::minute_trade::TradeError> {
    validate_inputs(data.len(), signals.len())?;

    // Pre-allocate collections for better performance
    let mut trades = Vec::with_capacity(signals.len() / 4); // Estimate: 1 trade per 4 signals
    let mut portfolio_values = Vec::with_capacity(data.len());
    let mut daily_returns = Vec::with_capacity(data.len());

    let mut cash = initial_cash;
    let mut position = 0.0;
    let mut current_trade: Option<crate::minute_trade::Trade> = None;

    portfolio_values.push(initial_cash);

    for (i, &signal) in signals.iter().enumerate().skip(1).take(data.len() - 1) {
        let price = data[i].data.open;
        let timestamp = data[i].timestamp;

        match signal {
            MinuteSignal::Buy if position == 0.0 => {
                let commission_cost = cash * commission_rate / 100.0;
                let shares = (cash - commission_cost) / price;
                position = shares;
                cash = 0.0;

                current_trade = Some(crate::minute_trade::Trade {
                    entry_time: timestamp,
                    entry_price: price,
                    exit_time: None,
                    exit_price: None,
                    size: shares,
                    is_long: true,
                    pnl: None,
                });
            }
            MinuteSignal::Sell if position > 0.0 => {
                let gross_proceeds = position * price;
                let commission_cost = gross_proceeds * commission_rate / 100.0;
                cash = gross_proceeds - commission_cost;

                if let Some(mut trade) = current_trade.take() {
                    trade.exit_time = Some(timestamp);
                    trade.exit_price = Some(price);
                    trade.pnl = Some(cash - initial_cash);
                    trades.push(trade);
                }

                position = 0.0;
            }
            _ => {} // Hold
        }

        // Calculate current portfolio value
        let current_value = cash + position * price;
        portfolio_values.push(current_value);

        // Calculate daily return
        if i > 0 {
            let return_rate = (current_value / portfolio_values[i - 1] - 1.0) * 100.0;
            daily_returns.push(return_rate);
        }
    }

    // Calculate performance metrics efficiently
    let final_value = portfolio_values.last().copied().unwrap_or(initial_cash);
    let total_return = (final_value / initial_cash - 1.0) * 100.0;

    // Calculate additional metrics using optimized algorithms
    let (max_drawdown, sharpe_ratio, volatility) =
        calculate_risk_metrics(&portfolio_values, &daily_returns);

    let (win_rate, profit_factor) = {
        #[cfg(feature = "minute-trading")]
        {
            calculate_trade_metrics(&trades)
        }
        #[cfg(not(feature = "minute-trading"))]
        {
            (0.0, 0.0)
        }
    };

    let annualized_return = if data.len() > 252 * 24 * 60 {
        // More than a year of minute data
        total_return * (252.0 * 24.0 * 60.0) / data.len() as f64
    } else {
        total_return
    };

    Ok(PerformanceMetrics {
        total_return,
        max_drawdown,
        sharpe_ratio,
        trade_count: trades.len(),
        win_rate,
        avg_profit_per_trade: if !trades.is_empty() {
            final_value / trades.len() as f64
        } else {
            0.0
        },
        volatility,
        start_date: data
            .first()
            .map(|d| d.timestamp)
            .unwrap_or_else(|| Utc::now()),
        end_date: data
            .last()
            .map(|d| d.timestamp)
            .unwrap_or_else(|| Utc::now()),
    })
}

/// Optimized risk metrics calculation
fn calculate_risk_metrics(portfolio_values: &[f64], daily_returns: &[f64]) -> (f64, f64, f64) {
    if portfolio_values.len() < 2 {
        return (0.0, 0.0, 0.0);
    }

    // Calculate maximum drawdown efficiently
    let mut max_value = portfolio_values[0];
    let mut max_drawdown: f64 = 0.0;

    for &value in portfolio_values.iter().skip(1) {
        max_value = max_value.max(value);
        let drawdown = (max_value - value) / max_value;
        max_drawdown = max_drawdown.max(drawdown);
    }

    // Calculate volatility and Sharpe ratio
    let (volatility, sharpe_ratio) = if !daily_returns.is_empty() {
        let mean_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
        let variance = daily_returns
            .iter()
            .map(|&r| (r - mean_return).powi(2))
            .sum::<f64>()
            / (daily_returns.len() - 1).max(1) as f64;
        let vol = variance.sqrt();
        let sharpe = if vol > 0.0 { mean_return / vol } else { 0.0 };
        (vol, sharpe)
    } else {
        (0.0, 0.0)
    };

    (max_drawdown * 100.0, sharpe_ratio, volatility)
}

/// Optimized trade metrics calculation
#[cfg(feature = "minute-trading")]
fn calculate_trade_metrics(trades: &[crate::minute_trade::Trade]) -> (f64, f64) {
    if trades.is_empty() {
        return (0.0, 0.0);
    }

    let winning_trades = trades.iter().filter(|t| t.pnl.unwrap_or(0.0) > 0.0).count();
    let win_rate = (winning_trades as f64 / trades.len() as f64) * 100.0;

    let gross_profit: f64 = trades
        .iter()
        .filter_map(|t| t.pnl)
        .filter(|&pnl| pnl > 0.0)
        .sum();

    let gross_loss: f64 = trades
        .iter()
        .filter_map(|t| t.pnl)
        .filter(|&pnl| pnl < 0.0)
        .map(|pnl| pnl.abs())
        .sum();

    let profit_factor = if gross_loss > 0.0 {
        gross_profit / gross_loss
    } else if gross_profit > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    (win_rate, profit_factor)
}

/// Input validation helper
#[cfg(feature = "minute-trading")]
#[inline]
fn validate_inputs(
    data_len: usize,
    signals_len: usize,
) -> Result<(), crate::minute_trade::TradeError> {
    if data_len != signals_len {
        return Err(crate::minute_trade::TradeError::InvalidData(
            "Data and signals arrays must be the same length".to_string(),
        ));
    }

    if data_len <= 1 {
        return Err(crate::minute_trade::TradeError::InsufficientData(
            "Need at least 2 data points to calculate performance".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "day-trading")]
    use crate::day_trade::OhlcvData;
    #[cfg(feature = "minute-trading")]
    use crate::minute_trade::OhlcvData as MinuteOhlcvData;
    use chrono::{DateTime, Utc};

    #[cfg(feature = "day-trading")]
    #[test]
    fn test_daily_performance_calculation() {
        let data = create_test_daily_data();
        let signals = vec![
            DaySignal::Buy,
            DaySignal::Hold,
            DaySignal::Sell,
            DaySignal::Hold,
        ];

        let performance = calculate_daily_performance(&data, &signals, 10000.0, 0.1).unwrap();
        assert!(performance.abs() < 100.0); // Reasonable performance range
    }

    #[cfg(feature = "minute-trading")]
    #[test]
    fn test_minute_performance_calculation() {
        let data = create_test_minute_data();
        let signals = vec![
            MinuteSignal::Buy,
            MinuteSignal::Hold,
            MinuteSignal::Sell,
            MinuteSignal::Hold,
        ];

        let performance = calculate_minute_performance(&data, &signals, 10000.0, 0.1).unwrap();
        assert!(performance.abs() < 100.0); // Reasonable performance range
    }

    #[cfg(feature = "day-trading")]
    fn create_test_daily_data() -> Vec<DailyOhlcv> {
        vec![
            DailyOhlcv {
                date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                data: OhlcvData {
                    open: 100.0,
                    high: 105.0,
                    low: 99.0,
                    close: 102.0,
                    volume: 1000,
                },
            },
            DailyOhlcv {
                date: chrono::NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
                data: OhlcvData {
                    open: 102.0,
                    high: 106.0,
                    low: 101.0,
                    close: 104.0,
                    volume: 1100,
                },
            },
            DailyOhlcv {
                date: chrono::NaiveDate::from_ymd_opt(2023, 1, 3).unwrap(),
                data: OhlcvData {
                    open: 104.0,
                    high: 107.0,
                    low: 103.0,
                    close: 106.0,
                    volume: 1200,
                },
            },
            DailyOhlcv {
                date: chrono::NaiveDate::from_ymd_opt(2023, 1, 4).unwrap(),
                data: OhlcvData {
                    open: 106.0,
                    high: 109.0,
                    low: 105.0,
                    close: 108.0,
                    volume: 1300,
                },
            },
        ]
    }

    #[cfg(feature = "minute-trading")]
    fn create_test_minute_data() -> Vec<crate::minute_trade::MinuteOhlcv> {
        vec![
            crate::minute_trade::MinuteOhlcv {
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T09:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                data: MinuteOhlcvData {
                    open: 100.0,
                    high: 105.0,
                    low: 99.0,
                    close: 102.0,
                    volume: 1000.0,
                },
            },
            crate::minute_trade::MinuteOhlcv {
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T09:31:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                data: MinuteOhlcvData {
                    open: 102.0,
                    high: 106.0,
                    low: 101.0,
                    close: 104.0,
                    volume: 1100.0,
                },
            },
            crate::minute_trade::MinuteOhlcv {
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T09:32:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                data: MinuteOhlcvData {
                    open: 104.0,
                    high: 107.0,
                    low: 103.0,
                    close: 106.0,
                    volume: 1200.0,
                },
            },
            crate::minute_trade::MinuteOhlcv {
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T09:33:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                data: MinuteOhlcvData {
                    open: 106.0,
                    high: 109.0,
                    low: 105.0,
                    close: 108.0,
                    volume: 1300.0,
                },
            },
        ]
    }
}
