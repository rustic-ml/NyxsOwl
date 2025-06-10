use nyxs_owl::forecasting::backtest::{BacktestConfig, ForecastBacktester};
use nyxs_owl::forecasting::strategies::{ExponentialSmoothingConfig, ExponentialSmoothingStrategy};
use nyxs_owl::simple_types::{Result, Signal};
use polars::prelude::*;
use std::fs;

fn main() -> Result<()> {
    // Create sample data since the Polars CSV scanning API has changed
    let df = create_sample_data()?;

    println!("🦉 NyxsOwl Exponential Smoothing Strategy Example");
    println!("=================================================");

    // Run analysis and backtesting for different configurations
    let configs = vec![
        (
            "Simple",
            ExponentialSmoothingConfig {
                alpha: 0.3,
                beta: None,
                gamma: None,
                ..Default::default()
            },
        ),
        (
            "Holt",
            ExponentialSmoothingConfig {
                alpha: 0.3,
                beta: Some(0.1),
                gamma: None,
                ..Default::default()
            },
        ),
        (
            "Holt-Winters",
            ExponentialSmoothingConfig {
                alpha: 0.3,
                beta: Some(0.1),
                gamma: Some(0.1),
                ..Default::default()
            },
        ),
    ];

    for (config_name, config) in configs {
        println!("\n📊 Testing {config_name} Exponential Smoothing:");

        let mut strategy = ExponentialSmoothingStrategy::new(config);

        match strategy.generate_signals(&df, "close", "timestamp") {
            Ok(signals) => {
                println!("   ✅ Generated {} signals", signals.len());
                analyze_signals(&signals);

                // Run backtest
                if let Ok(prices) = extract_prices(&df) {
                    let backtester = ForecastBacktester::new(BacktestConfig::default());
                    match backtester.backtest(&prices, &signals, None) {
                        Ok(performance) => {
                            println!("   📈 Backtest Results:");
                            println!(
                                "      Total Return: {:.2}%",
                                performance.total_return * 100.0
                            );
                            println!("      Sharpe Ratio: {:.3}", performance.sharpe_ratio);
                            println!(
                                "      Max Drawdown: {:.2}%",
                                performance.max_drawdown * 100.0
                            );
                            println!("      Win Rate: {:.1}%", performance.win_rate * 100.0);
                        }
                        Err(e) => println!("   ❌ Backtest failed: {}", e),
                    }
                }
            }
            Err(e) => println!("   ❌ Signal generation failed: {}", e),
        }
    }

    println!("\n✅ Exponential Smoothing strategy example completed!");
    Ok(())
}

fn analyze_signals(signals: &[Signal]) {
    let long_signals = signals.iter().filter(|s| matches!(s, Signal::Buy)).count();
    let short_signals = signals.iter().filter(|s| matches!(s, Signal::Sell)).count();
    let hold_signals = signals.iter().filter(|s| matches!(s, Signal::Hold)).count();

    println!("   🔍 Signal Analysis:");
    println!("      Buy signals: {}", long_signals);
    println!("      Sell signals: {}", short_signals);
    println!("      Hold signals: {}", hold_signals);

    // Show some sample signals
    println!("   📋 Sample Signals:");
    for (i, signal) in signals.iter().enumerate().take(5) {
        println!("      Signal {}: {:?}", i + 1, signal);
    }
}

fn create_sample_data() -> Result<DataFrame> {
    let n = 252; // One year of daily data
    let mut prices = Vec::with_capacity(n);
    let mut timestamps = Vec::with_capacity(n);

    let mut price = 100.0;
    let mut rng_state = 42u64; // Simple PRNG state

    for i in 0..n {
        // Simple PRNG (linear congruential generator)
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let random = (rng_state as f64 / u64::MAX as f64 - 0.5) * 0.02; // ±1% random change

        price *= 1.0 + random + 0.0002; // Small upward drift
        prices.push(price);
        timestamps.push(format!("2023-{:02}-{:02}", (i / 30) + 1, (i % 30) + 1));
    }

    let df = df! {
        "timestamp" => timestamps,
        "close" => prices,
    }?;

    Ok(df)
}

fn extract_prices(df: &DataFrame) -> Result<Vec<f64>> {
    let close_series = df.column("close").map_err(|e| {
        nyxs_owl::simple_types::NyxsOwlError::DataError(format!(
            "Failed to get close column: {}",
            e
        ))
    })?;

    let prices: Vec<f64> = close_series
        .f64()
        .map_err(|e| {
            nyxs_owl::simple_types::NyxsOwlError::DataError(format!(
                "Failed to convert to f64: {}",
                e
            ))
        })?
        .into_iter()
        .map(|v| v.unwrap_or(0.0))
        .collect();

    Ok(prices)
}
