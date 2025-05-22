//! Backtest utility functions and structures
//!
//! This module provides common functionality for backtesting trading strategies.

use crate::data::TimeSeriesData;
use crate::error::Result;
use crate::strategies::{BacktestResult, TradingSignal};

/// Trade record type for tracking executed trades
pub type TradeRecord = (usize, TradingSignal, f64, f64);

/// Run a backtest simulation with the given signals and parameters
pub fn run_backtest(
    data: &TimeSeriesData,
    signals: &[TradingSignal],
    initial_capital: f64,
    commission_rate: f64,
    slippage: f64,
) -> Result<BacktestResult> {
    let prices = data.close_prices();
    
    if prices.is_empty() || signals.is_empty() {
        return Ok(BacktestResult {
            final_balance: initial_capital,
            total_return: 0.0,
            max_drawdown: 0.0,
            win_rate: 0.0,
            equity_curve: vec![initial_capital],
            trades: 0,
            performance_metrics: None,
        });
    }
    
    // Run backtest simulation
    let mut balance = initial_capital;
    let mut position = 0.0;
    let mut trades = Vec::new();
    let mut equity_curve = Vec::with_capacity(prices.len());
    
    for (i, &signal) in signals.iter().enumerate() {
        let price = prices[i];
        
        // Apply slippage to execution price
        let buy_price = price * (1.0 + slippage);
        let sell_price = price * (1.0 - slippage);
        
        match signal {
            TradingSignal::Buy if position <= 0.0 => {
                // Close short position if any
                if position < 0.0 {
                    let profit = -position * (sell_price - (if i > 0 { prices[i-1] } else { price }));
                    balance += profit;
                    
                    // Apply commission
                    let commission = (-position * sell_price) * commission_rate;
                    balance -= commission;
                    
                    trades.push((i, TradingSignal::Buy, profit, commission));
                }
                
                // Enter long position
                let shares = balance / buy_price;
                let commission = (shares * buy_price) * commission_rate;
                balance -= commission;
                position = shares;
                
                trades.push((i, TradingSignal::Buy, 0.0, commission));
            },
            TradingSignal::Sell if position >= 0.0 => {
                // Close long position if any
                if position > 0.0 {
                    let profit = position * (sell_price - (if i > 0 { prices[i-1] } else { price }));
                    balance += profit;
                    
                    // Apply commission
                    let commission = (position * sell_price) * commission_rate;
                    balance -= commission;
                    
                    trades.push((i, TradingSignal::Sell, profit, commission));
                }
                
                // Enter short position
                let shares = balance / sell_price;
                let commission = (shares * sell_price) * commission_rate;
                balance -= commission;
                position = -shares;
                
                trades.push((i, TradingSignal::Sell, 0.0, commission));
            },
            _ => {}
        }
        
        // Update equity curve
        let equity = if position > 0.0 {
            balance + position * price
        } else if position < 0.0 {
            balance - position * price
        } else {
            balance
        };
        
        equity_curve.push(equity);
    }
    
    // Calculate metrics
    let final_balance = *equity_curve.last().unwrap_or(&initial_capital);
    let max_balance = equity_curve.iter().fold(initial_capital, |max, &x| max.max(x));
    let max_drawdown = equity_curve.iter().enumerate().fold(0.0, |max_dd, (i, &equity)| {
        let subsequent_min = equity_curve[i..].iter().fold(equity, |min, &x| min.min(x));
        let dd = (equity - subsequent_min) / equity;
        max_dd.max(dd)
    });
    
    // Calculate win rate
    let profitable_trades = trades.iter()
        .filter(|&&(_, _, profit, _)| profit > 0.0)
        .count();
    let win_rate = if trades.is_empty() {
        0.0
    } else {
        profitable_trades as f64 / trades.len() as f64
    };
    
    // Calculate performance metrics
    let performance_metrics = calculate_performance_metrics(&equity_curve, &trades);
    
    Ok(BacktestResult {
        final_balance,
        total_return: (final_balance - initial_capital) / initial_capital,
        max_drawdown,
        win_rate,
        equity_curve,
        trades: trades.len(),
        performance_metrics: Some(performance_metrics),
    })
}

/// Calculate advanced performance metrics from equity curve and trades
fn calculate_performance_metrics(
    equity_curve: &[f64],
    trades: &[TradeRecord],
) -> crate::strategies::PerformanceMetrics {
    // Calculate returns
    let returns: Vec<f64> = if equity_curve.len() > 1 {
        equity_curve.windows(2)
            .map(|w| (w[1] / w[0]) - 1.0)
            .collect()
    } else {
        Vec::new()
    };
    
    // Calculate Sharpe ratio (assuming risk-free rate of 0)
    let sharpe_ratio = if !returns.is_empty() {
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
            
        if variance > 0.0 {
            Some(mean_return / variance.sqrt())
        } else {
            None
        }
    } else {
        None
    };
    
    // Calculate Sortino ratio (downside risk only)
    let sortino_ratio = if !returns.is_empty() {
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let negative_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .map(|&r| r)
            .collect();
            
        if !negative_returns.is_empty() {
            let downside_variance = negative_returns.iter()
                .map(|r| (r - 0.0).powi(2))
                .sum::<f64>() / negative_returns.len() as f64;
                
            if downside_variance > 0.0 {
                Some(mean_return / downside_variance.sqrt())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Calculate Calmar ratio
    let calmar_ratio = if let Some(first) = equity_curve.first() {
        let last = equity_curve.last().unwrap_or(first);
        let total_return = (last - first) / first;
        
        if equity_curve.len() > 1 {
            let max_drawdown = equity_curve.iter().enumerate().fold(0.0, |max_dd, (i, &equity)| {
                let subsequent_min = equity_curve[i..].iter().fold(equity, |min, &x| min.min(x));
                let dd = (equity - subsequent_min) / equity;
                max_dd.max(dd)
            });
            
            if max_drawdown > 0.0 {
                Some(total_return / max_drawdown)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    crate::strategies::PerformanceMetrics {
        sharpe_ratio,
        sortino_ratio,
        calmar_ratio,
    }
} 