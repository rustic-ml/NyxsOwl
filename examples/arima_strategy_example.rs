use nyxs_owl::forecasting::strategies::{ArimaStrategy, ArimaStrategyConfig};
use nyxs_owl::forecasting::backtest::{ForecastBacktester, BacktestConfig};
use nyxs_owl::simple_types::{Signal, Result};
use polars::prelude::*;

fn main() -> Result<()> {
    println!("🦉 NyxsOwl ARIMA Strategy Example");
    println!("==================================");

    // Create sample data
    let df = create_sample_data()?;
    println!("📊 Created sample data with {} rows", df.height());

    // Test different ARIMA configurations
    test_arima_configurations(&df)?;

    // Run backtesting example
    run_backtest_example(&df)?;

    println!("✅ ARIMA strategy example completed!");
    Ok(())
}

fn test_arima_configurations(df: &DataFrame) -> Result<()> {
    println!("\n🔧 Testing Different ARIMA Configurations:");

    // Conservative configuration
    let conservative_config = ArimaStrategyConfig {
        p: 1,
        d: 1,
        q: 1,
        threshold: 0.03,
        min_data_points: 50,
        forecast_horizon: 1,
        forecast_confidence: 0.8,
        
        // New adaptive features (disabled for conservative approach)
        dynamic_threshold: false,
        volatility_lookback: 20,
        volatility_multiplier: 2.0,
        min_threshold: 0.005,
        max_threshold: 0.05,
        model_selection: false,
        max_p: 3,
        max_q: 3,
        outlier_detection: false,
        outlier_threshold: 2.5,
        
        // Enhanced features (disabled for conservative)
        confidence_intervals: false,
        confidence_level: 0.95,
        ensemble_models: 1,
        trend_confirmation: false,
        momentum_filter: false,
        regime_detection: false,
        adaptive_refit: false,
        refit_frequency: 50,
    };

    // Aggressive configuration with new features
    let aggressive_config = ArimaStrategyConfig {
        p: 2,
        d: 1,
        q: 2,
        threshold: 0.015,
        min_data_points: 30,
        forecast_horizon: 3,
        forecast_confidence: 0.7,
        
        // Enable adaptive features
        dynamic_threshold: true,
        volatility_lookback: 30,
        volatility_multiplier: 2.5,
        min_threshold: 0.002,
        max_threshold: 0.03,
        model_selection: true,
        max_p: 5,
        max_q: 5,
        outlier_detection: true,
        outlier_threshold: 3.0,
        
        // Enable enhanced features
        confidence_intervals: true,
        confidence_level: 0.95,
        ensemble_models: 3,
        trend_confirmation: true,
        momentum_filter: true,
        regime_detection: true,
        adaptive_refit: true,
        refit_frequency: 25,
    };

    // Test conservative strategy
    println!("\n📈 Conservative ARIMA Strategy:");
    let mut conservative_strategy = ArimaStrategy::new(conservative_config);
    let conservative_signals = conservative_strategy.generate_signals(df, "close", "timestamp")?;
    analyze_signals(&conservative_signals, "Conservative");

    // Test aggressive strategy
    println!("\n📈 Aggressive ARIMA Strategy (with OxiDiviner 1.2.0 features):");
    let mut aggressive_strategy = ArimaStrategy::new(aggressive_config);
    let aggressive_signals = aggressive_strategy.generate_signals(df, "close", "timestamp")?;
    analyze_signals(&aggressive_signals, "Aggressive");

    Ok(())
}

fn create_sample_data() -> PolarsResult<DataFrame> {
    // Create synthetic price data with trend and noise
    let n = 200;
    let mut prices = Vec::with_capacity(n);
    let mut timestamps = Vec::with_capacity(n);
    
    let base_price = 100.0;
    let trend = 0.02; // 2% trend per period
    
    for i in 0..n {
        let noise = (i as f64 * 0.1).sin() * 2.0 + rand::random::<f64>() * 1.0 - 0.5;
        let price = base_price + (i as f64 * trend) + noise;
        prices.push(price);
        timestamps.push(format!("2024-01-{:02}", (i % 30) + 1));
    }
    
    df! {
        "timestamp" => timestamps,
        "close" => prices,
    }
}

fn analyze_signals(signals: &[Signal], _config_name: &str) {
    let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
    let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
    
    println!("  📊 Signal Distribution:");
    println!("    🟢 Buy signals: {}", buy_count);
    println!("    🔴 Sell signals: {}", sell_count);
    println!("    ⚪ Hold signals: {}", hold_count);
    println!("    📈 Activity rate: {:.1}%", 
             ((buy_count + sell_count) as f64 / signals.len() as f64) * 100.0);
}

fn run_backtest_example(df: &DataFrame) -> Result<()> {
    println!("\n💰 Backtesting Example:");
    
    // Create ARIMA strategy with balanced configuration
    let config = ArimaStrategyConfig {
        p: 1,
        d: 1,
        q: 1,
        threshold: 0.02,
        min_data_points: 50,
        forecast_horizon: 1,
        forecast_confidence: 0.75,
        
        // Enable some adaptive features for better performance
        dynamic_threshold: true,
        volatility_lookback: 25,
        volatility_multiplier: 2.0,
        min_threshold: 0.003,
        max_threshold: 0.04,
        model_selection: true,
        max_p: 3,
        max_q: 3,
        outlier_detection: true,
        outlier_threshold: 2.5,
        
        // Enable confidence intervals and ensemble
        confidence_intervals: true,
        confidence_level: 0.95,
        ensemble_models: 3,
        trend_confirmation: false,
        momentum_filter: true,
        regime_detection: true,
        adaptive_refit: true,
        refit_frequency: 30,
    };
    
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
        transaction_cost: 0.001, // 0.1%
        slippage: 0.0005,        // 0.05%
        risk_free_rate: 0.02,    // 2% annual
        position_size: 0.95,     // 95% of capital
    };
    
    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&prices, &signals, None)?;
    
    // Display results
    println!("  📊 Backtest Results:");
    println!("    💰 Total Return: {:.2}%", performance.total_return * 100.0);
    println!("    📈 Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("    📉 Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("    🎯 Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("    🔄 Total Trades: {}", performance.total_trades);
    
    if performance.total_trades > 0 {
        println!("    💵 Profit Factor: {:.2}", performance.profit_factor);
        println!("    📊 Avg Win: {:.2}%", performance.avg_win * 100.0);
        println!("    📊 Avg Loss: {:.2}%", performance.avg_loss * 100.0);
    }

    Ok(())
}

// Simple random number generator for demo
mod rand {
    use std::cell::Cell;
    
    thread_local! {
        static SEED: Cell<u64> = Cell::new(1);
    }
    
    pub fn random<T>() -> T 
    where 
        T: From<f64>
    {
        SEED.with(|seed| {
            let mut s = seed.get();
            s = s.wrapping_mul(1103515245).wrapping_add(12345);
            seed.set(s);
            T::from((s as f64) / (u64::MAX as f64))
        })
    }
} 