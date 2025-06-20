//! Forecasting strategy backtesting module
//!
//! This module provides backtesting capabilities specifically tailored for forecasting-based strategies.

use crate::simple_types::{NyxsOwlError, Result, Signal};

/// Comprehensive backtest performance metrics
#[derive(Debug, Clone)]
pub struct BacktestPerformance {
    /// Total return over the backtest period
    pub total_return: f64,
    /// Sharpe ratio - risk-adjusted return measure
    pub sharpe_ratio: f64,
    /// Sortino ratio - downside risk-adjusted return measure
    pub sortino_ratio: f64,
    /// Maximum drawdown experienced
    pub max_drawdown: f64,
    /// Percentage of winning trades
    pub win_rate: f64,
    /// Total number of trades executed
    pub total_trades: usize,
    /// Number of profitable trades
    pub winning_trades: usize,
    /// Number of losing trades
    pub losing_trades: usize,
    /// Average profit per winning trade
    pub avg_win: f64,
    /// Average loss per losing trade
    pub avg_loss: f64,
    /// Profit factor (gross profits / gross losses)
    pub profit_factor: f64,

    // Additional fields expected by examples
    /// Annualized return over the backtest period
    pub annualized_return: f64,
    /// Benchmark return for comparison
    pub benchmark_return: f64,
    /// Strategy volatility (standard deviation of returns)
    pub volatility: f64,
    /// Calmar ratio (annual return / maximum drawdown)
    pub calmar_ratio: f64,
    /// Average return per trade
    pub avg_trade_return: f64,
    /// Best single trade return
    pub best_trade: f64,
    /// Worst single trade return
    pub worst_trade: f64,
}

/// Extended performance metrics for detailed analysis
#[derive(Debug, Clone)]
pub struct ExtendedPerformance {
    /// Annualized return
    pub annualized_return: f64,
    /// Benchmark return for comparison
    pub benchmark_return: f64,
    /// Strategy volatility
    pub volatility: f64,
    /// Calmar ratio (annual return / max drawdown)
    pub calmar_ratio: f64,
    /// Average return per trade
    pub avg_trade_return: f64,
    /// Best single trade return
    pub best_trade: f64,
    /// Worst single trade return
    pub worst_trade: f64,
}

impl BacktestPerformance {
    /// Create a new BacktestPerformance with default values
    pub fn new() -> Self {
        Self {
            total_return: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            max_drawdown: 0.0,
            win_rate: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            avg_win: 0.0,
            avg_loss: 0.0,
            profit_factor: 0.0,

            // Additional fields expected by examples
            annualized_return: 0.0,
            benchmark_return: 0.0,
            volatility: 0.0,
            calmar_ratio: 0.0,
            avg_trade_return: 0.0,
            best_trade: 0.0,
            worst_trade: 0.0,
        }
    }
}

impl Default for BacktestPerformance {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for backtesting
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    /// Initial capital for backtesting
    pub initial_capital: f64,
    /// Transaction cost as percentage (e.g., 0.001 for 0.1%)
    pub transaction_cost: f64, // As percentage (e.g., 0.001 for 0.1%)
    /// Slippage cost as percentage
    pub slippage: f64, // As percentage
    /// Annual risk-free rate for Sharpe ratio calculation
    pub risk_free_rate: f64, // Annual risk-free rate for Sharpe ratio
    /// Fraction of capital to use per trade
    pub position_size: f64, // Fraction of capital to use per trade
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100000.0, // $100,000
            transaction_cost: 0.001,   // 0.1%
            slippage: 0.0005,          // 0.05%
            risk_free_rate: 0.02,      // 2% annually
            position_size: 1.0,        // Use full capital
        }
    }
}

/// Backtesting engine for forecasting strategies
pub struct ForecastBacktester {
    config: BacktestConfig,
}

impl ForecastBacktester {
    /// Create a new backtester with the given configuration
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// Run backtest on signals and price data
    pub fn backtest(
        &self,
        prices: &[f64],
        signals: &[Signal],
        _timestamps: Option<&[String]>,
    ) -> Result<BacktestPerformance> {
        if prices.len() != signals.len() {
            return Err(NyxsOwlError::BacktestError(
                "Prices and signals must have the same length".to_string(),
            ));
        }

        if prices.is_empty() {
            return Err(NyxsOwlError::BacktestError(
                "Cannot backtest with empty data".to_string(),
            ));
        }

        let trades = self.execute_trades(prices, signals)?;
        let returns = self.calculate_returns(&trades, prices)?;
        let performance = self.calculate_performance_metrics(&returns, &trades)?;

        Ok(performance)
    }

    /// Execute trades based on signals
    fn execute_trades(&self, prices: &[f64], signals: &[Signal]) -> Result<Vec<Trade>> {
        let mut trades = Vec::new();
        let mut current_position: Option<Position> = None;

        for (i, (&price, &signal)) in prices.iter().zip(signals.iter()).enumerate() {
            match (current_position.as_ref(), signal) {
                // Open long position
                (None, Signal::Buy) => {
                    let adjusted_price =
                        price * (1.0 + self.config.slippage + self.config.transaction_cost);
                    current_position = Some(Position {
                        entry_price: adjusted_price,
                        entry_index: i,
                        position_type: PositionType::Long,
                        size: self.config.position_size,
                    });
                }

                // Close long position
                (Some(pos), Signal::Sell) if pos.position_type == PositionType::Long => {
                    let adjusted_price =
                        price * (1.0 - self.config.slippage - self.config.transaction_cost);
                    let trade = Trade {
                        entry_price: pos.entry_price,
                        exit_price: adjusted_price,
                        entry_index: pos.entry_index,
                        exit_index: i,
                        position_type: pos.position_type,
                        size: pos.size,
                        pnl: self.calculate_trade_pnl(
                            pos.entry_price,
                            adjusted_price,
                            pos.size,
                            pos.position_type,
                        ),
                    };
                    trades.push(trade);
                    current_position = None;
                }

                // Open short position (if we want to support shorting)
                (None, Signal::Sell) => {
                    let adjusted_price =
                        price * (1.0 - self.config.slippage + self.config.transaction_cost);
                    current_position = Some(Position {
                        entry_price: adjusted_price,
                        entry_index: i,
                        position_type: PositionType::Short,
                        size: self.config.position_size,
                    });
                }

                // Close short position
                (Some(pos), Signal::Buy) if pos.position_type == PositionType::Short => {
                    let adjusted_price =
                        price * (1.0 + self.config.slippage + self.config.transaction_cost);
                    let trade = Trade {
                        entry_price: pos.entry_price,
                        exit_price: adjusted_price,
                        entry_index: pos.entry_index,
                        exit_index: i,
                        position_type: pos.position_type,
                        size: pos.size,
                        pnl: self.calculate_trade_pnl(
                            pos.entry_price,
                            adjusted_price,
                            pos.size,
                            pos.position_type,
                        ),
                    };
                    trades.push(trade);
                    current_position = None;
                }

                // Hold or invalid transitions
                _ => {}
            }
        }

        // Close any remaining position at the end
        if let Some(pos) = current_position {
            let last_price = prices[prices.len() - 1];
            let adjusted_price = match pos.position_type {
                PositionType::Long => {
                    last_price * (1.0 - self.config.slippage - self.config.transaction_cost)
                }
                PositionType::Short => {
                    last_price * (1.0 + self.config.slippage + self.config.transaction_cost)
                }
            };

            let trade = Trade {
                entry_price: pos.entry_price,
                exit_price: adjusted_price,
                entry_index: pos.entry_index,
                exit_index: prices.len() - 1,
                position_type: pos.position_type,
                size: pos.size,
                pnl: self.calculate_trade_pnl(
                    pos.entry_price,
                    adjusted_price,
                    pos.size,
                    pos.position_type,
                ),
            };
            trades.push(trade);
        }

        Ok(trades)
    }

    fn calculate_trade_pnl(
        &self,
        entry_price: f64,
        exit_price: f64,
        size: f64,
        position_type: PositionType,
    ) -> f64 {
        let capital_used = self.config.initial_capital * size;
        let shares = capital_used / entry_price;

        match position_type {
            PositionType::Long => shares * (exit_price - entry_price),
            PositionType::Short => shares * (entry_price - exit_price),
        }
    }

    fn calculate_returns(&self, trades: &[Trade], prices: &[f64]) -> Result<Vec<f64>> {
        let mut portfolio_value = self.config.initial_capital;
        let mut returns = vec![0.0]; // First return is 0

        let mut trade_iter = trades.iter().peekable();

        for i in 1..prices.len() {
            let mut period_pnl = 0.0;

            // Check if any trades were closed in this period
            while let Some(trade) = trade_iter.peek() {
                if trade.exit_index == i {
                    period_pnl += trade.pnl;
                    trade_iter.next();
                } else {
                    break;
                }
            }

            // Calculate return for this period
            let period_return = if portfolio_value > 0.0 {
                period_pnl / portfolio_value
            } else {
                0.0
            };

            portfolio_value += period_pnl;
            returns.push(period_return);
        }

        Ok(returns)
    }

    fn calculate_performance_metrics(
        &self,
        returns: &[f64],
        trades: &[Trade],
    ) -> Result<BacktestPerformance> {
        if returns.is_empty() || trades.is_empty() {
            return Ok(BacktestPerformance::default());
        }

        // Total return
        let total_return = returns.iter().fold(1.0, |acc, &r| acc * (1.0 + r)) - 1.0;

        // Trade statistics
        let winning_trades = trades.iter().filter(|t| t.pnl > 0.0).count();
        let losing_trades = trades.iter().filter(|t| t.pnl < 0.0).count();
        let total_trades = trades.len();
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        let wins: Vec<f64> = trades
            .iter()
            .filter(|t| t.pnl > 0.0)
            .map(|t| t.pnl)
            .collect();
        let losses: Vec<f64> = trades
            .iter()
            .filter(|t| t.pnl < 0.0)
            .map(|t| t.pnl.abs())
            .collect();

        let avg_win = if !wins.is_empty() {
            wins.iter().sum::<f64>() / wins.len() as f64
        } else {
            0.0
        };
        let avg_loss = if !losses.is_empty() {
            losses.iter().sum::<f64>() / losses.len() as f64
        } else {
            0.0
        };

        let gross_profit: f64 = wins.iter().sum();
        let gross_loss: f64 = losses.iter().sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            0.0
        };

        // Risk metrics
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        let std_dev = variance.sqrt();

        // Sharpe ratio (annualized)
        let periods_per_year = 252.0; // Assuming daily data
        let excess_return = mean_return - (self.config.risk_free_rate / periods_per_year);
        let sharpe_ratio = if std_dev > 0.0 {
            excess_return / std_dev * periods_per_year.sqrt()
        } else {
            0.0
        };

        // Sortino ratio
        let downside_returns: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).cloned().collect();
        let downside_deviation = if !downside_returns.is_empty() {
            let downside_variance = downside_returns.iter().map(|r| r.powi(2)).sum::<f64>()
                / downside_returns.len() as f64;
            downside_variance.sqrt()
        } else {
            0.0
        };

        let sortino_ratio = if downside_deviation > 0.0 {
            excess_return / downside_deviation * periods_per_year.sqrt()
        } else {
            0.0
        };

        // Maximum drawdown
        let mut peak = 1.0;
        let mut max_drawdown = 0.0;
        let mut portfolio_value = 1.0;

        for &ret in returns {
            portfolio_value *= 1.0 + ret;
            if portfolio_value > peak {
                peak = portfolio_value;
            }
            let drawdown = (peak - portfolio_value) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        Ok(BacktestPerformance {
            total_return,
            sharpe_ratio,
            sortino_ratio,
            max_drawdown,
            win_rate,
            total_trades,
            winning_trades,
            losing_trades,
            avg_win,
            avg_loss,
            profit_factor,

            // Additional fields expected by examples
            annualized_return: 0.0,
            benchmark_return: 0.0,
            volatility: 0.0,
            calmar_ratio: 0.0,
            avg_trade_return: 0.0,
            best_trade: 0.0,
            worst_trade: 0.0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PositionType {
    Long,
    Short,
}

#[derive(Debug, Clone)]
struct Position {
    entry_price: f64,
    entry_index: usize,
    position_type: PositionType,
    size: f64,
}

#[derive(Debug, Clone)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    entry_index: usize,
    exit_index: usize,
    position_type: PositionType,
    size: f64,
    pnl: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn create_test_data() -> (Vec<f64>, Vec<Signal>) {
        let prices = vec![100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 106.0, 108.0];
        let signals = vec![
            Signal::Hold,
            Signal::Buy,
            Signal::Hold,
            Signal::Hold,
            Signal::Hold,
            Signal::Sell,
            Signal::Hold,
            Signal::Hold,
        ];
        (prices, signals)
    }

    #[test]
    fn test_backtest_config_default() {
        let config = BacktestConfig::default();
        assert_relative_eq!(config.initial_capital, 100000.0);
        assert_relative_eq!(config.transaction_cost, 0.001);
        assert_relative_eq!(config.position_size, 1.0);
    }

    #[test]
    fn test_backtest_performance_creation() {
        let performance = BacktestPerformance::new();
        assert_relative_eq!(performance.total_return, 0.0);
        assert_eq!(performance.total_trades, 0);

        let default_performance = BacktestPerformance::default();
        assert_relative_eq!(default_performance.total_return, 0.0);
    }

    #[test]
    fn test_forecast_backtester_creation() {
        let config = BacktestConfig::default();
        let backtester = ForecastBacktester::new(config);
        assert_relative_eq!(backtester.config.initial_capital, 100000.0);
    }

    #[test]
    fn test_backtest_mismatched_data() {
        let backtester = ForecastBacktester::new(BacktestConfig::default());
        let prices = vec![100.0, 102.0];
        let signals = vec![Signal::Buy, Signal::Hold, Signal::Sell]; // Different length

        let result = backtester.backtest(&prices, &signals, None);
        assert!(matches!(result, Err(NyxsOwlError::BacktestError(_))));
    }

    #[test]
    fn test_backtest_empty_data() {
        let backtester = ForecastBacktester::new(BacktestConfig::default());
        let prices = vec![];
        let signals = vec![];

        let result = backtester.backtest(&prices, &signals, None);
        assert!(matches!(result, Err(NyxsOwlError::BacktestError(_))));
    }

    #[test]
    fn test_simple_backtest() {
        let backtester = ForecastBacktester::new(BacktestConfig::default());
        let (prices, signals) = create_test_data();

        let result = backtester.backtest(&prices, &signals, None);
        assert!(result.is_ok());

        let performance = result.unwrap();
        assert!(performance.total_trades > 0);

        // Should have at least one trade based on our test signals
        assert_eq!(performance.total_trades, 1);

        // Since we bought at 102 and sold at 107, we should have positive return
        assert!(performance.total_return > 0.0);
    }

    #[test]
    fn test_no_trades_scenario() {
        let backtester = ForecastBacktester::new(BacktestConfig::default());
        let prices = vec![100.0, 102.0, 104.0, 103.0];
        let signals = vec![Signal::Hold, Signal::Hold, Signal::Hold, Signal::Hold];

        let result = backtester.backtest(&prices, &signals, None);
        assert!(result.is_ok());

        let performance = result.unwrap();
        assert_eq!(performance.total_trades, 0);
        assert_relative_eq!(performance.total_return, 0.0);
        assert_relative_eq!(performance.win_rate, 0.0);
    }

    #[test]
    fn test_calculate_trade_pnl_long() {
        let backtester = ForecastBacktester::new(BacktestConfig::default());
        let pnl = backtester.calculate_trade_pnl(100.0, 105.0, 1.0, PositionType::Long);

        // With $100k capital, buying at $100, we get 1000 shares
        // Selling at $105, we make $5 per share = $5000 profit
        assert_relative_eq!(pnl, 5000.0, epsilon = 1.0);
    }

    #[test]
    fn test_calculate_trade_pnl_short() {
        let backtester = ForecastBacktester::new(BacktestConfig::default());
        let pnl = backtester.calculate_trade_pnl(100.0, 95.0, 1.0, PositionType::Short);

        // Shorting at $100, covering at $95, we make $5 per share = $5000 profit
        assert_relative_eq!(pnl, 5000.0, epsilon = 1.0);
    }

    #[test]
    fn test_transaction_costs() {
        let config = BacktestConfig {
            transaction_cost: 0.01, // 1% transaction cost
            slippage: 0.0,
            ..Default::default()
        };
        let backtester = ForecastBacktester::new(config);

        let prices = vec![100.0, 110.0]; // 10% price increase
        let signals = vec![Signal::Buy, Signal::Sell];

        let result = backtester.backtest(&prices, &signals, None);
        assert!(result.is_ok());

        let performance = result.unwrap();
        // With 1% transaction cost on both buy and sell, effective return should be less than 10%
        assert!(performance.total_return < 0.08); // Should be around 8% after costs
        assert!(performance.total_return > 0.0); // But still positive
    }
}
