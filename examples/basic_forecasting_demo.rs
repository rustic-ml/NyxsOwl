use nyxs_owl::forecasting::backtest::{BacktestConfig, ForecastBacktester};
use nyxs_owl::forecasting::strategies::arima_strategy::{ArimaStrategy, ArimaStrategyConfig};
use nyxs_owl::prelude::*;
use polars::prelude::*;

/// Basic forecasting demo without heavy Polars dependencies
/// This demonstrates the structure and approach while avoiding complex API issues

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🦉 NyxsOwl Basic Forecasting Demo");
    println!("=================================");

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

    // Use lenient configuration for demo
    let config = ArimaStrategyConfig {
        min_data_points: 50, // Reduced from default
        ..ArimaStrategyConfig::default()
    };
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

    let config = ArimaStrategyConfig {
        min_data_points: 50, // Reduced from default
        ..ArimaStrategyConfig::default()
    };
    let mut strategy = ArimaStrategy::new(config);
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    // Extract prices for backtesting
    let prices: Vec<f64> = df.column("close")?.f64()?.into_no_null_iter().collect();

    // The ARIMA strategy only generates signals from min_data_points onwards
    // We need to pad the beginning with Hold signals to match the price length
    let mut padded_signals = Vec::with_capacity(prices.len());

    // Add Hold signals for the initial period where no forecasts are generated
    for _ in 0..50 {
        padded_signals.push(Signal::Hold);
    }

    // Add the actual generated signals
    padded_signals.extend_from_slice(&signals);

    // Ensure we have the same number of signals as prices
    if padded_signals.len() != prices.len() {
        // If we still have a mismatch, pad with Hold signals to match
        while padded_signals.len() < prices.len() {
            padded_signals.push(Signal::Hold);
        }
        // If we have too many signals, truncate
        if padded_signals.len() > prices.len() {
            padded_signals.truncate(prices.len());
        }
    }

    // Create backtester
    let backtest_config = BacktestConfig {
        initial_capital: 10000.0,
        transaction_cost: 0.001,
        slippage: 0.0005,
        risk_free_rate: 0.02,
        position_size: 0.1,
    };

    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&prices, &padded_signals, None)?;

    println!("  📊 Results:");
    println!(
        "    💰 Total Return: {:.2}%",
        performance.total_return * 100.0
    );
    println!("    📈 Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!(
        "    📉 Max Drawdown: {:.2}%",
        performance.max_drawdown * 100.0
    );
    println!("    🎯 Win Rate: {:.1}%", performance.win_rate * 100.0);

    Ok(())
}

fn create_sample_data() -> PolarsResult<DataFrame> {
    let n = 150; // Increased data size
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
