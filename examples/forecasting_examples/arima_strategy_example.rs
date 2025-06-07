// examples/forecasting_examples/arima_strategy_example.rs
use nyxs_owl::forecasting::strategies::{ArimaStrategy, ArimaStrategyConfig};
use nyxs_owl::forecasting::backtest::{ForecastBacktester, BacktestConfig};
use nyxs_owl::forecasting::utils::extract_numeric_series;
use nyxs_owl::simple_types::{Signal, Result};
use polars::prelude::*;
use std::env;
use std::path::Path;

fn main() -> Result<()> {
    env_logger::init();
    
    println!("ARIMA Strategy Example");
    println!("======================");
    
    // Load data
    let data_file = env::var("OHLCV_FILE").unwrap_or_else(|_| "AAPL_daily_ohlcv.csv".to_string());
    let data_path = format!("examples/csv/{}", data_file);
    let df = load_ohlcv_data(&data_path)?;
    
    println!("Loaded {} rows of data from {}", df.height(), data_file);
    
    // Show data sample
    println!("\nData sample:");
    println!("{}", df.head(Some(5)));
    
    // Configure strategy with different parameter sets
    let configs = vec![
        ("Conservative", ArimaStrategyConfig {
            p: 1,
            d: 1,
            q: 1,
            threshold: 0.02,
            min_data_points: 60,
            forecast_horizon: 1,
            forecast_confidence: 0.8,
        }),
        ("Moderate", ArimaStrategyConfig {
            p: 2,
            d: 1,
            q: 1,
            threshold: 0.015,
            min_data_points: 80,
            forecast_horizon: 1,
            forecast_confidence: 0.75,
        }),
        ("Aggressive", ArimaStrategyConfig {
            p: 2,
            d: 1,
            q: 2,
            threshold: 0.01,
            min_data_points: 100,
            forecast_horizon: 1,
            forecast_confidence: 0.7,
        }),
    ];
    
    println!("\nTesting different ARIMA configurations:");
    println!("{:-<80}", "");
    
    for (name, config) in configs {
        println!("\n{} Configuration:", name);
        println!("  ARIMA({}, {}, {})", config.p, config.d, config.q);
        println!("  Threshold: {:.1}%", config.threshold * 100.0);
        println!("  Min data points: {}", config.min_data_points);
        
        // Test strategy
        test_arima_strategy(&df, config, name)?;
    }
    
    // Demonstrate parameter sensitivity
    println!("\n\nParameter Sensitivity Analysis:");
    println!("{:-<80}", "");
    demonstrate_parameter_sensitivity(&df)?;
    
    Ok(())
}

fn load_ohlcv_data(path: &str) -> Result<DataFrame> {
    if !Path::new(path).exists() {
        return Err(nyxs_owl::simple_types::NyxsOwlError::DataError(
            format!("Data file not found: {}. Please ensure the file exists in examples/csv/", path)
        ));
    }
    
    LazyFrame::scan_csv(path, ScanArgsCSV::default())
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to load CSV: {}", e)))?
        .collect()
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to collect data: {}", e)))
}

fn test_arima_strategy(df: &DataFrame, config: ArimaStrategyConfig, config_name: &str) -> Result<()> {
    // Initialize strategy
    let strategy = ArimaStrategy::new(config.clone());
    
    // Generate signals
    let start_time = std::time::Instant::now();
    let signals = strategy.generate_signals(df, "close", "timestamp")?;
    let generation_time = start_time.elapsed();
    
    // Analyze signals
    analyze_signals(&signals, config_name);
    
    // Backtest strategy
    let prices = extract_numeric_series(df, "close")?;
    let performance = backtest_strategy(&prices, &signals)?;
    print_performance_metrics(&performance, config_name);
    
    println!("  Signal generation time: {:.2}ms", generation_time.as_millis());
    
    // Compare with buy-and-hold
    let buy_and_hold_return = calculate_buy_and_hold_return(&prices);
    println!("  Buy-and-hold return: {:.2}%", buy_and_hold_return * 100.0);
    
    let alpha = performance.total_return - buy_and_hold_return;
    println!("  Alpha (vs buy-and-hold): {:.2}%", alpha * 100.0);
    
    Ok(())
}

fn analyze_signals(signals: &[Signal], config_name: &str) {
    let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
    let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
    
    println!("  Signal Analysis:");
    println!("    Buy signals: {} ({:.1}%)", buy_count, buy_count as f64 / signals.len() as f64 * 100.0);
    println!("    Sell signals: {} ({:.1}%)", sell_count, sell_count as f64 / signals.len() as f64 * 100.0);
    println!("    Hold signals: {} ({:.1}%)", hold_count, hold_count as f64 / signals.len() as f64 * 100.0);
    println!("    Total signals: {}", signals.len());
    
    // Calculate signal transition patterns
    let mut transitions = std::collections::HashMap::new();
    for window in signals.windows(2) {
        let key = format!("{:?} -> {:?}", window[0], window[1]);
        *transitions.entry(key).or_insert(0) += 1;
    }
    
    println!("    Common transitions:");
    let mut sorted_transitions: Vec<_> = transitions.iter().collect();
    sorted_transitions.sort_by(|a, b| b.1.cmp(a.1));
    for (transition, count) in sorted_transitions.iter().take(3) {
        println!("      {}: {}", transition, count);
    }
}

fn backtest_strategy(prices: &[f64], signals: &[Signal]) -> Result<nyxs_owl::forecasting::backtest::BacktestPerformance> {
    let config = BacktestConfig {
        initial_capital: 100000.0,
        transaction_cost: 0.001,  // 0.1%
        slippage: 0.0005,        // 0.05%
        risk_free_rate: 0.02,    // 2%
        position_size: 1.0,
    };
    
    let backtester = ForecastBacktester::new(config);
    backtester.backtest(prices, signals, None)
}

fn print_performance_metrics(performance: &nyxs_owl::forecasting::backtest::BacktestPerformance, config_name: &str) {
    println!("  Backtest Results:");
    println!("    Total Return: {:.2}%", performance.total_return * 100.0);
    println!("    Sharpe Ratio: {:.2}", performance.sharpe_ratio);
    println!("    Sortino Ratio: {:.2}", performance.sortino_ratio);
    println!("    Maximum Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("    Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("    Total Trades: {}", performance.total_trades);
    
    if performance.total_trades > 0 {
        println!("    Winning Trades: {}", performance.winning_trades);
        println!("    Losing Trades: {}", performance.losing_trades);
        println!("    Average Win: ${:.2}", performance.avg_win);
        println!("    Average Loss: ${:.2}", performance.avg_loss);
        println!("    Profit Factor: {:.2}", performance.profit_factor);
    }
}

fn calculate_buy_and_hold_return(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }
    
    let first_price = prices[0];
    let last_price = prices[prices.len() - 1];
    (last_price - first_price) / first_price
}

fn demonstrate_parameter_sensitivity(df: &DataFrame) -> Result<()> {
    println!("\nThreshold Sensitivity (ARIMA(1,1,1)):");
    
    let thresholds = vec![0.005, 0.01, 0.015, 0.02, 0.025, 0.03];
    let prices = extract_numeric_series(df, "close")?;
    
    for threshold in thresholds {
        let config = ArimaStrategyConfig {
            threshold,
            ..Default::default()
        };
        
        let strategy = ArimaStrategy::new(config);
        let signals = strategy.generate_signals(df, "close", "timestamp")?;
        
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
        
        if buy_count > 0 || sell_count > 0 {
            let performance = backtest_strategy(&prices, &signals)?;
            println!("  Threshold {:.1}%: {} trades, {:.2}% return, {:.2} Sharpe", 
                threshold * 100.0, 
                performance.total_trades, 
                performance.total_return * 100.0,
                performance.sharpe_ratio
            );
        } else {
            println!("  Threshold {:.1}%: No trades generated", threshold * 100.0);
        }
    }
    
    println!("\nModel Order Sensitivity (threshold=1.5%):");
    
    let model_orders = vec![(1,1,1), (2,1,1), (1,1,2), (2,1,2), (3,1,1), (1,1,3)];
    
    for (p, d, q) in model_orders {
        let config = ArimaStrategyConfig {
            p, d, q,
            threshold: 0.015,
            ..Default::default()
        };
        
        let strategy = ArimaStrategy::new(config);
        match strategy.generate_signals(df, "close", "timestamp") {
            Ok(signals) => {
                let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
                let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
                
                if buy_count > 0 || sell_count > 0 {
                    match backtest_strategy(&prices, &signals) {
                        Ok(performance) => {
                            println!("  ARIMA({},{},{}): {} trades, {:.2}% return, {:.2} Sharpe", 
                                p, d, q,
                                performance.total_trades, 
                                performance.total_return * 100.0,
                                performance.sharpe_ratio
                            );
                        },
                        Err(e) => println!("  ARIMA({},{},{}): Backtest error: {}", p, d, q, e),
                    }
                } else {
                    println!("  ARIMA({},{},{}): No trades generated", p, d, q);
                }
            },
            Err(e) => println!("  ARIMA({},{},{}): Signal generation error: {}", p, d, q, e),
        }
    }
    
    Ok(())
} 