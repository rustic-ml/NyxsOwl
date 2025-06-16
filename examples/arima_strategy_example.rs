use nyxs_owl::forecasting::backtest::{BacktestConfig, ForecastBacktester};
use nyxs_owl::forecasting::strategies::arima_strategy::{ArimaStrategy, ArimaStrategyConfig};
use nyxs_owl::prelude::*;
use polars::prelude::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🎯 NyxsOwl ARIMA Strategy Demo");
    println!("==============================");

    // Create sample market data
    let df = create_sample_data()?;
    println!("📊 Created {} data points", df.height());

    // Demo basic ARIMA strategy
    demo_basic_strategy(&df)?;

    // Demo backtest
    demo_backtest(&df)?;

    println!("✅ All demos completed successfully!");
    Ok(())
}

fn demo_basic_strategy(df: &DataFrame) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n📈 Basic ARIMA Strategy");
    println!("======================");

    // Create simple ARIMA configuration
    let config = ArimaStrategyConfig::default();
    let mut strategy = ArimaStrategy::new(config);

    // Generate signals
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    // Analyze signals - using the new simplified Signal enum
    let buy_count = signals.iter().filter(|&s| *s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&s| *s == Signal::Sell).count();
    let hold_count = signals.iter().filter(|&s| *s == Signal::Hold).count();

    println!("  📊 Signal Distribution:");
    println!("    🟢 Buy signals: {}", buy_count);
    println!("    🔴 Sell signals: {}", sell_count);
    println!("    ⚪ Hold signals: {}", hold_count);

    Ok(())
}

fn demo_backtest(df: &DataFrame) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n💰 Backtest Demo");
    println!("================");

    let config = ArimaStrategyConfig::default();
    let mut strategy = ArimaStrategy::new(config);
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    // Extract prices
    let prices: Vec<f64> = df.column("close")?.f64()?.into_no_null_iter().collect();

    // Create backtester with simple config
    let backtest_config = BacktestConfig {
        initial_capital: 10000.0,
        transaction_cost: 0.001,
        slippage: 0.0005,
        risk_free_rate: 0.02,
        position_size: 0.95,
    };

    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&prices, &signals, None)?;

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
    let n = 200;
    let mut prices = Vec::with_capacity(n);
    let mut timestamps = Vec::with_capacity(n);

    let base_price = 100.0;
    let trend = 0.02;

    for i in 0..n {
        let noise = (i as f64 * 0.1).sin() * 2.0 + (i as f64 * 0.01).cos();
        let price = base_price + (i as f64 * trend) + noise;
        prices.push(price);
        timestamps.push(format!("2024-01-{:02}", (i % 30) + 1));
    }

    df! {
        "timestamp" => timestamps,
        "close" => prices,
    }
}
