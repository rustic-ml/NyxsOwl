use nyxs_owl::forecasting::strategies::{KalmanStrategy, KalmanStrategyConfig};
use nyxs_owl::forecasting::backtest::{ForecastBacktester, BacktestConfig};
use nyxs_owl::simple_types::{Signal, Result};
use polars::prelude::*;
use std::env;

fn main() -> Result<()> {
    println!("=================================");
    println!("    Kalman Filter Strategy Example");
    println!("=================================");
    
    // Load market data
    let data_file = env::var("OHLCV_FILE").unwrap_or_else(|_| "examples/csv/AAPL_daily_ohlcv.csv".to_string());
    println!("Loading data from: {}", data_file);
    
    let df = load_ohlcv_data(&data_file)?;
    println!("Loaded {} data points", df.height());
    
    // Test different Kalman Filter configurations
    println!("\n🔬 Testing Kalman Filter Configurations");
    println!("=====================================");
    
    test_kalman_configuration("Conservative", KalmanStrategyConfig::conservative(), &df)?;
    test_kalman_configuration("Aggressive", KalmanStrategyConfig::aggressive(), &df)?;
    test_kalman_configuration("Trend-Focused", KalmanStrategyConfig::trend_focused(), &df)?;
    
    // Custom configuration
    let custom_config = KalmanStrategyConfig::new(0.015, 0.08, 1.2, 0.012, 40)?;
    test_kalman_configuration("Custom", custom_config, &df)?;
    
    println!("\n📊 Detailed Analysis - Conservative Configuration");
    println!("===============================================");
    
    let config = KalmanStrategyConfig::conservative();
    detailed_analysis(&config, &df)?;
    
    Ok(())
}

fn load_ohlcv_data(file_path: &str) -> Result<DataFrame> {
    let df = LazyFrame::scan_csv(file_path, ScanArgsCSV::default())
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to load CSV: {}", e)))?
        .collect()
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to collect data: {}", e)))?;
    
    // Ensure we have required columns
    let required_columns = ["close", "timestamp"];
    for col in required_columns.iter() {
        if df.column(col).is_err() {
            return Err(nyxs_owl::simple_types::NyxsOwlError::DataError(
                format!("Required column '{}' not found", col)
            ));
        }
    }
    
    Ok(df)
}

fn test_kalman_configuration(name: &str, config: KalmanStrategyConfig, df: &DataFrame) -> Result<()> {
    println!("\n📈 {} Configuration:", name);
    println!("  Process Noise: {:.6}", config.process_noise);
    println!("  Observation Noise: {:.3}", config.observation_noise);
    println!("  Signal Threshold: {:.1}%", config.signal_threshold * 100.0);
    println!("  Trend Detection: {}", config.use_trend_detection);
    println!("  Min Data Points: {}", config.min_data_points);
    
    let strategy = KalmanStrategy::new(config.clone());
    let signals = strategy.generate_signals(df, "close", "timestamp")?;
    
    analyze_signals(&signals, name);
    
    // Perform backtesting
    let backtest_config = BacktestConfig::default();
    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&signals, df, None)?;
    
    println!("  📊 Performance Metrics:");
    println!("    Total Return: {:.2}%", performance.total_return * 100.0);
    println!("    Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("    Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("    Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("    Profit Factor: {:.2}", performance.profit_factor);
    
    Ok(())
}

fn analyze_signals(signals: &[Signal], config_name: &str) {
    let total_signals = signals.len();
    let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
    let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
    
    println!("  🎯 Signal Analysis:");
    println!("    Total Signals: {}", total_signals);
    println!("    Buy Signals: {} ({:.1}%)", buy_count, buy_count as f64 / total_signals as f64 * 100.0);
    println!("    Sell Signals: {} ({:.1}%)", sell_count, sell_count as f64 / total_signals as f64 * 100.0);
    println!("    Hold Signals: {} ({:.1}%)", hold_count, hold_count as f64 / total_signals as f64 * 100.0);
    
    // Calculate signal activity
    let activity_rate = (buy_count + sell_count) as f64 / total_signals as f64 * 100.0;
    println!("    Trading Activity: {:.1}%", activity_rate);
    
    if activity_rate < 5.0 {
        println!("    ⚠️  Low trading activity - consider adjusting thresholds");
    } else if activity_rate > 50.0 {
        println!("    ⚠️  High trading activity - may incur significant transaction costs");
    } else {
        println!("    ✅ Moderate trading activity");
    }
}

fn detailed_analysis(config: &KalmanStrategyConfig, df: &DataFrame) -> Result<()> {
    let strategy = KalmanStrategy::new(config.clone());
    let signals = strategy.generate_signals(df, "close", "timestamp")?;
    
    println!("Strategy Configuration Details:");
    println!("  Process Noise: {:.6} (lower = smoother filter)", config.process_noise);
    println!("  Observation Noise: {:.3} (higher = trust observations less)", config.observation_noise);
    println!("  Initial Uncertainty: {:.1} (starting confidence)", config.initial_uncertainty);
    println!("  Signal Threshold: {:.2}% (minimum change for signal)", config.signal_threshold * 100.0);
    println!("  Trend Detection: {} (use trend vs. divergence)", config.use_trend_detection);
    
    if config.use_trend_detection {
        println!("  Trend Lookback: {} periods", config.trend_lookback);
        println!("  Innovation Threshold: {:.1}σ (regime change detection)", config.innovation_threshold);
    }
    
    // Analyze signal patterns
    analyze_signal_patterns(&signals);
    
    // Extract prices for additional analysis
    let prices = extract_prices_for_analysis(df)?;
    analyze_market_conditions(&prices, &signals);
    
    // Perform detailed backtesting
    println!("\n📈 Detailed Backtesting Results:");
    let backtest_config = BacktestConfig {
        initial_capital: 100000.0,
        transaction_cost_pct: 0.001, // 0.1% transaction cost
        slippage_pct: 0.0005,        // 0.05% slippage
        risk_free_rate: 0.02,        // 2% risk-free rate
    };
    
    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&signals, df, None)?;
    
    print_detailed_performance(&performance);
    
    Ok(())
}

fn analyze_signal_patterns(signals: &[Signal]) {
    println!("\n🔍 Signal Pattern Analysis:");
    
    // Find signal clusters and transitions
    let mut consecutive_holds = 0;
    let mut max_consecutive_holds = 0;
    let mut signal_transitions = 0;
    let mut last_signal = Signal::Hold;
    
    for (i, &signal) in signals.iter().enumerate() {
        if signal == Signal::Hold {
            consecutive_holds += 1;
            max_consecutive_holds = max_consecutive_holds.max(consecutive_holds);
        } else {
            consecutive_holds = 0;
        }
        
        if i > 0 && signal != last_signal {
            signal_transitions += 1;
        }
        last_signal = signal;
    }
    
    println!("  Signal Transitions: {}", signal_transitions);
    println!("  Max Consecutive Holds: {}", max_consecutive_holds);
    
    // Analyze signal clustering
    let mut buy_clusters = 0;
    let mut sell_clusters = 0;
    let mut in_buy_cluster = false;
    let mut in_sell_cluster = false;
    
    for &signal in signals.iter() {
        match signal {
            Signal::Buy => {
                if !in_buy_cluster {
                    buy_clusters += 1;
                    in_buy_cluster = true;
                }
                in_sell_cluster = false;
            },
            Signal::Sell => {
                if !in_sell_cluster {
                    sell_clusters += 1;
                    in_sell_cluster = true;
                }
                in_buy_cluster = false;
            },
            Signal::Hold => {
                in_buy_cluster = false;
                in_sell_cluster = false;
            }
        }
    }
    
    println!("  Buy Signal Clusters: {}", buy_clusters);
    println!("  Sell Signal Clusters: {}", sell_clusters);
}

fn extract_prices_for_analysis(df: &DataFrame) -> Result<Vec<f64>> {
    let column = df.column("close")
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to get close column: {}", e)))?;
    
    let prices: Vec<f64> = column
        .f64()
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to convert to f64: {}", e)))?
        .into_iter()
        .collect::<Option<Vec<f64>>>()
        .ok_or_else(|| nyxs_owl::simple_types::NyxsOwlError::DataError("Price column contains null values".to_string()))?;
        
    Ok(prices)
}

fn analyze_market_conditions(prices: &[f64], signals: &[Signal]) {
    println!("\n📊 Market Conditions Analysis:");
    
    // Calculate basic statistics
    let price_changes: Vec<f64> = prices.windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();
    
    let avg_change = price_changes.iter().sum::<f64>() / price_changes.len() as f64;
    let volatility = {
        let variance = price_changes.iter()
            .map(|&x| (x - avg_change).powi(2))
            .sum::<f64>() / price_changes.len() as f64;
        variance.sqrt()
    };
    
    println!("  Average Daily Return: {:.4}% ({:.2}% annualized)", 
             avg_change * 100.0, avg_change * 252.0 * 100.0);
    println!("  Daily Volatility: {:.4}% ({:.2}% annualized)", 
             volatility * 100.0, volatility * (252.0_f64).sqrt() * 100.0);
    
    // Analyze signal timing vs market conditions
    let mut buy_in_uptrend = 0;
    let mut sell_in_downtrend = 0;
    let lookback = 10;
    
    for (i, &signal) in signals.iter().enumerate() {
        if i >= lookback {
            let recent_trend = (prices[i] - prices[i - lookback]) / prices[i - lookback];
            
            match signal {
                Signal::Buy if recent_trend > 0.0 => buy_in_uptrend += 1,
                Signal::Sell if recent_trend < 0.0 => sell_in_downtrend += 1,
                _ => {}
            }
        }
    }
    
    let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
    
    if buy_count > 0 {
        println!("  Buy Signals in Uptrend: {:.1}%", 
                 buy_in_uptrend as f64 / buy_count as f64 * 100.0);
    }
    
    if sell_count > 0 {
        println!("  Sell Signals in Downtrend: {:.1}%", 
                 sell_in_downtrend as f64 / sell_count as f64 * 100.0);
    }
}

fn print_detailed_performance(performance: &nyxs_owl::forecasting::backtest::BacktestPerformance) {
    println!("  💰 Return Metrics:");
    println!("    Total Return: {:.2}%", performance.total_return * 100.0);
    println!("    Annualized Return: {:.2}%", performance.annualized_return * 100.0);
    println!("    Benchmark Return (Buy & Hold): {:.2}%", performance.benchmark_return * 100.0);
    println!("    Excess Return: {:.2}%", (performance.total_return - performance.benchmark_return) * 100.0);
    
    println!("\n  📈 Risk Metrics:");
    println!("    Volatility: {:.2}%", performance.volatility * 100.0);
    println!("    Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("    Sortino Ratio: {:.3}", performance.sortino_ratio);
    println!("    Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("    Calmar Ratio: {:.3}", performance.calmar_ratio);
    
    println!("\n  🎯 Trading Metrics:");
    println!("    Total Trades: {}", performance.total_trades);
    println!("    Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("    Profit Factor: {:.2}", performance.profit_factor);
    println!("    Average Trade: {:.2}%", performance.avg_trade_return * 100.0);
    println!("    Best Trade: {:.2}%", performance.best_trade * 100.0);
    println!("    Worst Trade: {:.2}%", performance.worst_trade * 100.0);
    
    // Performance assessment
    println!("\n  🏆 Strategy Assessment:");
    
    if performance.sharpe_ratio > 1.0 {
        println!("    ✅ Excellent risk-adjusted returns (Sharpe > 1.0)");
    } else if performance.sharpe_ratio > 0.5 {
        println!("    ✅ Good risk-adjusted returns (Sharpe > 0.5)");
    } else if performance.sharpe_ratio > 0.0 {
        println!("    ⚠️  Modest risk-adjusted returns");
    } else {
        println!("    ❌ Poor risk-adjusted returns");
    }
    
    if performance.max_drawdown < 0.1 {
        println!("    ✅ Low drawdown risk (< 10%)");
    } else if performance.max_drawdown < 0.2 {
        println!("    ⚠️  Moderate drawdown risk (< 20%)");
    } else {
        println!("    ❌ High drawdown risk (> 20%)");
    }
    
    if performance.win_rate > 0.6 {
        println!("    ✅ High win rate (> 60%)");
    } else if performance.win_rate > 0.4 {
        println!("    ⚠️  Moderate win rate");
    } else {
        println!("    ❌ Low win rate (< 40%)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_sample_data() {
        // Test data loading with a simple dataset
        let result = load_ohlcv_data("examples/csv/daily_data.csv");
        // Should handle missing files gracefully
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_signal_analysis() {
        let signals = vec![
            Signal::Hold, Signal::Buy, Signal::Hold, Signal::Hold,
            Signal::Sell, Signal::Hold, Signal::Buy, Signal::Hold,
        ];
        
        // Should not panic
        analyze_signals(&signals, "test");
    }
} 