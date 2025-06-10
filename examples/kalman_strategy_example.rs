use nyxs_owl::forecasting::backtest::{BacktestConfig, ForecastBacktester};
use nyxs_owl::forecasting::strategies::kalman_strategy::{KalmanStrategy, KalmanStrategyConfig};
use nyxs_owl::prelude::*;
use polars::prelude::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🎯 NyxsOwl Kalman Filter Strategy Demo");
    println!("====================================");

    // Create sample data
    let df = create_sample_data()?;
    println!("📊 Created {} data points", df.height());

    // Demo basic Kalman strategy
    demo_basic_kalman(&df)?;

    // Demo backtest
    demo_backtest(&df)?;

    println!("✅ All Kalman strategy demos completed successfully!");
    Ok(())
}

fn demo_basic_kalman(df: &DataFrame) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n📈 Basic Kalman Filter Strategy");
    println!("==============================");

    let config = KalmanStrategyConfig {
        process_noise: 1e-4,
        observation_noise: 1e-2,
        initial_uncertainty: 1.0,
        signal_threshold: 0.02,
        use_trend_detection: true,
        min_observations: 10,
        forecast_horizon: 5,
    };

    let mut strategy = KalmanStrategy::new(config);

    // Extract prices for signal generation
    let prices: Vec<f64> = df.column("close")?.f64()?.into_no_null_iter().collect();

    let signals = strategy.generate_signals(&prices, &df, "close")?;

    // Analyze signals
    let long_count = signals.iter().filter(|s| matches!(s, Signal::Buy)).count();
    let short_count = signals.iter().filter(|s| matches!(s, Signal::Sell)).count();
    let hold_count = signals.iter().filter(|s| matches!(s, Signal::Hold)).count();

    println!("  📊 Signal Distribution:");
    println!("    🟢 Long signals: {}", long_count);
    println!("    🔴 Short signals: {}", short_count);
    println!("    ⚪ Hold signals: {}", hold_count);

    Ok(())
}

fn demo_backtest(df: &DataFrame) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n💰 Backtest Demo");
    println!("================");

    let config = KalmanStrategyConfig {
        process_noise: 5e-5,
        observation_noise: 2e-3,
        initial_uncertainty: 0.5,
        signal_threshold: 0.015,
        use_trend_detection: true,
        min_observations: 15,
        forecast_horizon: 10,
    };

    let mut strategy = KalmanStrategy::new(config);

    let prices: Vec<f64> = df.column("close")?.f64()?.into_no_null_iter().collect();

    let signals = strategy.generate_signals(&prices, &df, "close")?;

    // Create backtester
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

    let mut price = 100.0;

    for i in 0..n {
        // Add trend and noise
        let trend = (i as f64 * 0.001);
        let noise = (i as f64 * 0.1).sin() * 1.5 + (i as f64 * 0.03).cos() * 0.8;
        price += trend + noise;

        prices.push(price);
        timestamps.push(format!("2024-01-{:02}", (i % 30) + 1));
    }

    df! {
        "timestamp" => timestamps,
        "close" => prices,
    }
}
