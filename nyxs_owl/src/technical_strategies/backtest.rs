use crate::technical_strategies::{TechnicalSignal, TechnicalStrategy};
use polars::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Comprehensive performance metrics for strategy backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub consecutive_wins: usize,
    pub consecutive_losses: usize,
    pub volatility: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
}

/// Individual trade record for detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub entry_date: String,
    pub exit_date: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub position_size: f64,
    pub direction: TradeDirection,
    pub pnl: f64,
    pub pnl_percent: f64,
    pub holding_period: i32,
    pub entry_signal_strength: f64,
    pub exit_reason: ExitReason,
}

/// Trade direction enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeDirection {
    Long,
    Short,
}

/// Reason for exiting a trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExitReason {
    Signal,
    StopLoss,
    TakeProfit,
    TimeLimit,
    EndOfData,
}

/// Backtesting configuration parameters
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub position_size_method: PositionSizeMethod,
    pub transaction_cost_percent: f64,
    pub slippage_percent: f64,
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub max_holding_period: Option<i32>,
    pub risk_free_rate: f64,
}

/// Methods for determining position size
#[derive(Debug, Clone)]
pub enum PositionSizeMethod {
    FixedAmount(f64),
    PercentOfCapital(f64),
    VolatilityAdjusted(f64), // ATR-based sizing
    KellyOptimal(f64),       // Kelly criterion
}

/// Complete backtesting results
#[derive(Debug, Clone)]
pub struct BacktestResults {
    pub metrics: PerformanceMetrics,
    pub trades: Vec<TradeRecord>,
    pub equity_curve: Vec<f64>,
    pub drawdown_curve: Vec<f64>,
    pub dates: Vec<String>,
    pub monthly_returns: HashMap<String, f64>,
    pub strategy_name: String,
}

/// Main backtesting engine
pub struct Backtester {
    config: BacktestConfig,
    current_capital: f64,
    current_position: Option<Position>,
    trades: Vec<TradeRecord>,
    equity_curve: Vec<f64>,
    dates: Vec<String>,
}

/// Internal position tracking
#[derive(Debug, Clone)]
struct Position {
    direction: TradeDirection,
    entry_price: f64,
    entry_date: String,
    size: f64,
    entry_signal_strength: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100_000.0,
            position_size_method: PositionSizeMethod::PercentOfCapital(0.1),
            transaction_cost_percent: 0.001, // 0.1%
            slippage_percent: 0.0005,        // 0.05%
            stop_loss_percent: None,
            take_profit_percent: None,
            max_holding_period: None,
            risk_free_rate: 0.02, // 2%
        }
    }
}

impl Backtester {
    /// Create a new backtester with configuration
    pub fn new(config: BacktestConfig) -> Self {
        Self {
            current_capital: config.initial_capital,
            config,
            current_position: None,
            trades: Vec::new(),
            equity_curve: Vec::new(),
            dates: Vec::new(),
        }
    }

    /// Run backtest on a technical strategy
    pub fn run_backtest<T: TechnicalStrategy>(
        &mut self,
        strategy: &T,
        data: &DataFrame,
    ) -> PolarsResult<BacktestResults> {
        // Generate signals
        let result = strategy.generate_signals(data)?;
        let signals = result.signals;
        
        // Extract required columns
        let dates = data
            .column("date")?
            .str()?
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| PolarsError::ComputeError("Invalid date column".into()))?;
        
        let prices = data
            .column("close")?
            .f64()?
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| PolarsError::ComputeError("Invalid price column".into()))?;

        // Process each signal
        for (i, (&price, date)) in prices.iter().zip(dates.iter()).enumerate() {
            if let Some(signal) = signals.get(i) {
                self.process_signal(signal, price, date, i)?;
            }
            
            // Update equity curve
            let current_equity = self.calculate_current_equity(price);
            self.equity_curve.push(current_equity);
            self.dates.push(date.to_string());
        }

        // Close any remaining position
        if let Some(_) = &self.current_position {
            let last_price = *prices.last().unwrap();
            let last_date = dates.last().unwrap();
            self.close_position(last_price, last_date, ExitReason::EndOfData)?;
        }

        Ok(self.generate_results(&strategy.name()))
    }

    /// Process a trading signal
    fn process_signal(
        &mut self,
        signal: &TechnicalSignal,
        price: f64,
        date: &str,
        index: usize,
    ) -> PolarsResult<()> {
        match &self.current_position {
            None => {
                // No current position, check for entry signals
                if signal.strength > 0.5 && signal.confidence > 0.6 {
                    let direction = if signal.strength > 0.0 {
                        TradeDirection::Long
                    } else {
                        TradeDirection::Short
                    };
                    
                    self.open_position(direction, price, date, signal.strength)?;
                }
            }
            Some(position) => {
                // Check exit conditions
                let should_exit = self.should_exit_position(signal, price, date, index)?;
                
                if should_exit.0 {
                    self.close_position(price, date, should_exit.1)?;
                }
            }
        }
        
        Ok(())
    }

    /// Open a new position
    fn open_position(
        &mut self,
        direction: TradeDirection,
        price: f64,
        date: &str,
        signal_strength: f64,
    ) -> PolarsResult<()> {
        let position_value = self.calculate_position_size(price);
        let adjusted_price = self.apply_slippage_and_costs(price, true);
        
        self.current_position = Some(Position {
            direction,
            entry_price: adjusted_price,
            entry_date: date.to_string(),
            size: position_value / adjusted_price,
            entry_signal_strength: signal_strength,
        });
        
        self.current_capital -= position_value + self.calculate_transaction_costs(position_value);
        
        Ok(())
    }

    /// Close the current position
    fn close_position(
        &mut self,
        price: f64,
        date: &str,
        exit_reason: ExitReason,
    ) -> PolarsResult<()> {
        if let Some(position) = self.current_position.take() {
            let adjusted_price = self.apply_slippage_and_costs(price, false);
            let position_value = position.size * adjusted_price;
            
            let pnl = match position.direction {
                TradeDirection::Long => position_value - (position.size * position.entry_price),
                TradeDirection::Short => (position.size * position.entry_price) - position_value,
            };
            
            let transaction_costs = self.calculate_transaction_costs(position_value);
            let net_pnl = pnl - transaction_costs;
            
            self.current_capital += position_value - transaction_costs;
            
            let trade = TradeRecord {
                entry_date: position.entry_date,
                exit_date: date.to_string(),
                entry_price: position.entry_price,
                exit_price: adjusted_price,
                position_size: position.size,
                direction: position.direction,
                pnl: net_pnl,
                pnl_percent: net_pnl / (position.size * position.entry_price) * 100.0,
                holding_period: 1, // Simplified - would need actual date calculation
                entry_signal_strength: position.entry_signal_strength,
                exit_reason,
            };
            
            self.trades.push(trade);
        }
        
        Ok(())
    }

    /// Determine if position should be exited
    fn should_exit_position(
        &self,
        signal: &TechnicalSignal,
        current_price: f64,
        _date: &str,
        _index: usize,
    ) -> PolarsResult<(bool, ExitReason)> {
        if let Some(position) = &self.current_position {
            // Signal-based exit
            match position.direction {
                TradeDirection::Long if signal.strength < -0.3 => {
                    return Ok((true, ExitReason::Signal));
                }
                TradeDirection::Short if signal.strength > 0.3 => {
                    return Ok((true, ExitReason::Signal));
                }
                _ => {}
            }
            
            // Stop loss check
            if let Some(stop_loss_pct) = self.config.stop_loss_percent {
                let loss_pct = match position.direction {
                    TradeDirection::Long => {
                        (position.entry_price - current_price) / position.entry_price
                    }
                    TradeDirection::Short => {
                        (current_price - position.entry_price) / position.entry_price
                    }
                };
                
                if loss_pct >= stop_loss_pct {
                    return Ok((true, ExitReason::StopLoss));
                }
            }
            
            // Take profit check
            if let Some(take_profit_pct) = self.config.take_profit_percent {
                let profit_pct = match position.direction {
                    TradeDirection::Long => {
                        (current_price - position.entry_price) / position.entry_price
                    }
                    TradeDirection::Short => {
                        (position.entry_price - current_price) / position.entry_price
                    }
                };
                
                if profit_pct >= take_profit_pct {
                    return Ok((true, ExitReason::TakeProfit));
                }
            }
        }
        
        Ok((false, ExitReason::Signal))
    }

    /// Calculate position size based on configuration
    fn calculate_position_size(&self, price: f64) -> f64 {
        match &self.config.position_size_method {
            PositionSizeMethod::FixedAmount(amount) => *amount,
            PositionSizeMethod::PercentOfCapital(percent) => self.current_capital * percent,
            PositionSizeMethod::VolatilityAdjusted(risk_percent) => {
                // Simplified - would need ATR calculation
                self.current_capital * risk_percent
            }
            PositionSizeMethod::KellyOptimal(win_rate) => {
                // Simplified Kelly criterion
                let kelly_fraction = (win_rate * 2.0) - 1.0;
                self.current_capital * kelly_fraction.max(0.01).min(0.25)
            }
        }
    }

    /// Apply slippage and transaction costs to price
    fn apply_slippage_and_costs(&self, price: f64, is_buy: bool) -> f64 {
        let slippage_adjustment = if is_buy {
            1.0 + self.config.slippage_percent
        } else {
            1.0 - self.config.slippage_percent
        };
        
        price * slippage_adjustment
    }

    /// Calculate transaction costs
    fn calculate_transaction_costs(&self, position_value: f64) -> f64 {
        position_value * self.config.transaction_cost_percent
    }

    /// Calculate current portfolio equity
    fn calculate_current_equity(&self, current_price: f64) -> f64 {
        let mut equity = self.current_capital;
        
        if let Some(position) = &self.current_position {
            let position_value = position.size * current_price;
            equity += position_value;
        }
        
        equity
    }

    /// Generate final backtest results
    fn generate_results(&self, strategy_name: &str) -> BacktestResults {
        let metrics = self.calculate_performance_metrics();
        let drawdown_curve = self.calculate_drawdown_curve();
        
        BacktestResults {
            metrics,
            trades: self.trades.clone(),
            equity_curve: self.equity_curve.clone(),
            drawdown_curve,
            dates: self.dates.clone(),
            monthly_returns: HashMap::new(), // Simplified
            strategy_name: strategy_name.to_string(),
        }
    }

    /// Calculate comprehensive performance metrics
    fn calculate_performance_metrics(&self) -> PerformanceMetrics {
        let initial_capital = self.config.initial_capital;
        let final_capital = self.equity_curve.last().copied().unwrap_or(initial_capital);
        
        let total_return = (final_capital - initial_capital) / initial_capital;
        let winning_trades = self.trades.iter().filter(|t| t.pnl > 0.0).count();
        let losing_trades = self.trades.len() - winning_trades;
        let win_rate = if self.trades.is_empty() {
            0.0
        } else {
            winning_trades as f64 / self.trades.len() as f64
        };
        
        let gross_profit: f64 = self.trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();
        let gross_loss: f64 = self.trades.iter().filter(|t| t.pnl < 0.0).map(|t| t.pnl.abs()).sum();
        let profit_factor = if gross_loss > 0.0 { gross_profit / gross_loss } else { f64::INFINITY };
        
        let average_win = if winning_trades > 0 {
            gross_profit / winning_trades as f64
        } else {
            0.0
        };
        
        let average_loss = if losing_trades > 0 {
            gross_loss / losing_trades as f64
        } else {
            0.0
        };
        
        // Calculate max drawdown
        let mut peak = initial_capital;
        let mut max_drawdown = 0.0;
        
        for &equity in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
        
        // Calculate volatility (simplified)
        let returns: Vec<f64> = self.equity_curve
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();
        
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let volatility = variance.sqrt();
        
        // Calculate Sharpe ratio
        let excess_return = total_return - self.config.risk_free_rate;
        let sharpe_ratio = if volatility > 0.0 {
            excess_return / volatility
        } else {
            0.0
        };
        
        PerformanceMetrics {
            total_return,
            annualized_return: total_return, // Simplified
            sharpe_ratio,
            max_drawdown,
            win_rate,
            profit_factor,
            total_trades: self.trades.len(),
            winning_trades,
            losing_trades,
            average_win,
            average_loss,
            largest_win: self.trades.iter().map(|t| t.pnl).fold(0.0, f64::max),
            largest_loss: self.trades.iter().map(|t| t.pnl).fold(0.0, f64::min),
            consecutive_wins: 0, // Simplified
            consecutive_losses: 0, // Simplified
            volatility,
            sortino_ratio: sharpe_ratio, // Simplified
            calmar_ratio: if max_drawdown > 0.0 { total_return / max_drawdown } else { 0.0 },
        }
    }

    /// Calculate drawdown curve
    fn calculate_drawdown_curve(&self) -> Vec<f64> {
        let mut peak = self.config.initial_capital;
        let mut drawdown_curve = Vec::new();
        
        for &equity in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            drawdown_curve.push(drawdown);
        }
        
        drawdown_curve
    }
}

/// Utility function to run a quick backtest
pub fn quick_backtest<T: TechnicalStrategy>(
    strategy: &T,
    data: &DataFrame,
    initial_capital: Option<f64>,
) -> PolarsResult<BacktestResults> {
    let config = BacktestConfig {
        initial_capital: initial_capital.unwrap_or(100_000.0),
        ..Default::default()
    };
    
    let mut backtester = Backtester::new(config);
    backtester.run_backtest(strategy, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technical_strategies::volume::VWAPStrategy;

    #[test]
    fn test_backtest_config_default() {
        let config = BacktestConfig::default();
        assert_eq!(config.initial_capital, 100_000.0);
        assert_eq!(config.transaction_cost_percent, 0.001);
    }

    #[test]
    fn test_performance_metrics_calculation() {
        // Test with empty trades
        let backtester = Backtester::new(BacktestConfig::default());
        let metrics = backtester.calculate_performance_metrics();
        assert_eq!(metrics.total_trades, 0);
        assert_eq!(metrics.win_rate, 0.0);
    }
}
