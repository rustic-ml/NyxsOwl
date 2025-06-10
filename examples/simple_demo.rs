use nyxs_owl::prelude::*;
use nyxs_owl::forecasting::strategies::arima_strategy::{ArimaStrategy, ArimaStrategyConfig};
use nyxs_owl::forecasting::backtest::{ForecastBacktester, BacktestConfig};
use polars::prelude::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🦉 NyxsOwl Simple Demo");
    println!("======================");

    // Create sample data
    let df = create_sample_data()?;
    println!("📊 Created {} data points", df.height());

    // Demo 1: Basic ARIMA strategy
    demo_arima_strategy(&df)?;
    
    // Demo 2: Backtest performance
    demo_backtest(&df)?;

    println!("✅ Demo completed successfully!");
    Ok(())
}

fn demo_arima_strategy(df: &DataFrame) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n📈 ARIMA Strategy Demo");
    println!("====================");

    // Use default configuration for simplicity
    let config = ArimaStrategyConfig::default();
    let mut strategy = ArimaStrategy::new(config);
    
    // Generate signals
    let signals = strategy.generate_signals(df, "close", "timestamp")?;
    
    // Count signal types
    let buy_count = signals.iter().filter(|s| matches!(s, Signal::Buy)).count();
    let sell_count = signals.iter().filter(|s| matches!(s, Signal::Sell)).count();
    let hold_count = signals.iter().filter(|s| matches!(s, Signal::Hold)).count();
    
    println!("  📊 Generated {} signals:", signals.len());
    println!("    🟢 Buy: {}", buy_count);
    println!("    🔴 Sell: {}", sell_count);
    println!("    ⚪ Hold: {}", hold_count);
    
    Ok(())
}

fn demo_backtest(df: &DataFrame) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n💰 Backtest Demo");
    println!("================");

    let config = ArimaStrategyConfig::default();
    let mut strategy = ArimaStrategy::new(config);
    let signals = strategy.generate_signals(df, "close", "timestamp")?;
    
    // Extract prices for backtesting
    let prices: Vec<f64> = df.column("close")?
        .f64()?
        .into_no_null_iter()
        .collect();
    
    // Create backtester
    let backtest_config = BacktestConfig {
        initial_capital: 10000.0,
        transaction_cost: 0.001,
        slippage: 0.0005,
        risk_free_rate: 0.02,
        position_size: 0.1,
    };
    
    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&prices, &signals, None)?;
    
    println!("  📊 Results:");
    println!("    💰 Total Return: {:.2}%", performance.total_return * 100.0);
    println!("    📈 Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("    📉 Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("    🎯 Win Rate: {:.1}%", performance.win_rate * 100.0);
    
    Ok(())
}

fn create_sample_data() -> PolarsResult<DataFrame> {
    let n = 100;
    let mut prices = Vec::with_capacity(n);
    let mut timestamps = Vec::with_capacity(n);
    
    let mut price = 100.0;
    
    for i in 0..n {
        // Simple trending price series with noise
        let trend = i as f64 * 0.01;
        let noise = (i as f64 * 0.1).sin() * 2.0;
        price += trend + noise;
        
        prices.push(price);
        timestamps.push(format!("2024-01-{:02}", (i % 30) + 1));
    }
    
    df! {
        "timestamp" => timestamps,
        "close" => prices,
    }
} 