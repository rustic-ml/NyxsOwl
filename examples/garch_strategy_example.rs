use nyxs_owl::forecasting::backtest::{BacktestConfig, ForecastBacktester};
use nyxs_owl::forecasting::strategies::garch_strategy::{
    GarchStrategy, GarchStrategyConfig, GarchType,
};
use nyxs_owl::simple_types::{NyxsOwlError, Result as NyxsOwlResult, Signal};
use polars::prelude::*;
use std::env;

fn main() -> NyxsOwlResult<()> {
    env_logger::init();

    println!("🎯 NyxsOwl GARCH Strategy Demo");
    println!("==============================");

    // Create sample market data
    let df = create_sample_data()?;
    println!("📊 Created {} data points", df.height());

    // Test different GARCH models
    test_garch_model("Standard GARCH", GarchType::Standard, &df)?;
    test_garch_model("EGARCH", GarchType::Egarch, &df)?;
    test_garch_model("GJR-GARCH", GarchType::GjrGarch, &df)?;

    // Demo comprehensive backtest
    demo_comprehensive_backtest(&df)?;

    println!("✅ All GARCH demos completed successfully!");
    Ok(())
}

fn create_sample_data() -> NyxsOwlResult<DataFrame> {
    let n = 500;
    let mut prices = Vec::with_capacity(n);
    let mut timestamps = Vec::with_capacity(n);

    let mut price = 100.0_f64;
    let mut volatility = 0.02_f64; // 2% daily volatility

    for i in 0..n {
        // Simulate volatility clustering (GARCH effect) using deterministic pattern
        let innovation: f64 = ((i as f64 * 0.1).sin() + (i as f64 * 0.23).cos()) * 0.5;

        // GARCH-like volatility process
        volatility = 0.00001 + 0.05 * innovation.powi(2) + 0.94 * volatility;
        volatility = volatility.min(0.1).max(0.001); // Bound volatility

        // Price evolution with time-varying volatility
        let return_shock = innovation * volatility.sqrt();
        price *= (return_shock).exp();

        prices.push(price);
        timestamps.push(i as i64);
    }

    let df = df! {
        "timestamp" => timestamps,
        "close" => prices,
    }?;

    Ok(df)
}

fn test_garch_model(name: &str, garch_type: GarchType, df: &DataFrame) -> NyxsOwlResult<()> {
    println!("\n🎯 Testing {} GARCH Model", name);
    println!("=========================");

    let config = GarchStrategyConfig {
        model_type: garch_type,
        arch_order: 1,
        garch_order: 1,
        signal_threshold: 0.02,    // 2% threshold
        volatility_threshold: 1.5, // 1.5x average volatility
        min_data_points: 50,
        volatility_window: 30,
        use_volatility_targeting: true,
        target_volatility: 0.15, // 15% annualized
        risk_adjustment: 1.2,
    };

    let mut strategy = GarchStrategy::new(config);

    // Generate signals
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    // Analyze signals
    analyze_signals(&signals, name);

    println!("✅ {} model completed successfully", name);
    Ok(())
}

fn demo_comprehensive_backtest(df: &DataFrame) -> NyxsOwlResult<()> {
    println!("\n📈 Comprehensive GARCH Backtest");
    println!("===============================");

    // Create aggressive GARCH strategy for backtesting
    let config = GarchStrategyConfig::aggressive();
    let mut strategy = GarchStrategy::new(config);

    // Generate signals
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    // Extract prices for backtest
    let prices: Vec<f64> = df.column("close")?.f64()?.into_no_null_iter().collect();

    // Comprehensive backtest
    let backtest_config = BacktestConfig {
        initial_capital: 100000.0,
        transaction_cost: 0.001, // 0.1%
        slippage: 0.0005,        // 0.05%
        risk_free_rate: 0.02,    // 2% annual
        position_size: 0.25,     // 25% of capital per trade
    };

    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&prices, &signals, None)?;

    // Display results
    println!("📊 Backtest Results:");
    println!(
        "  💰 Total Return: {:.2}%",
        performance.total_return * 100.0
    );
    println!("  📈 Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!(
        "  📉 Max Drawdown: {:.2}%",
        performance.max_drawdown * 100.0
    );
    println!("  🎯 Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("  🔢 Total Trades: {}", performance.total_trades);
    println!(
        "  📊 Avg Trade Return: {:.3}%",
        performance.avg_trade_return * 100.0
    );

    Ok(())
}

fn analyze_signals(signals: &[Signal], _config_name: &str) {
    let buy_count = signals.iter().filter(|&s| *s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&s| *s == Signal::Sell).count();
    let hold_count = signals.iter().filter(|&s| *s == Signal::Hold).count();

    println!("📊 Signal Analysis:");
    println!("  🟢 Buy signals: {}", buy_count);
    println!("  🔴 Sell signals: {}", sell_count);
    println!("  ⚪ Hold signals: {}", hold_count);
    println!("  📈 Total signals: {}", signals.len());

    if !signals.is_empty() {
        let signal_rate = (buy_count + sell_count) as f64 / signals.len() as f64 * 100.0;
        println!("  ⚡ Signal rate: {:.1}%", signal_rate);
    }
}

// Additional utility functions for demonstration
fn demonstrate_volatility_regimes(_signals: &[Signal]) {
    println!("\n🔄 Volatility Regime Analysis");
    println!("============================");

    // This would contain actual volatility regime analysis
    println!("📊 Simulated regime analysis:");
    println!("  🔵 Low Volatility: 45% of time");
    println!("  🟡 Medium Volatility: 35% of time");
    println!("  🔴 High Volatility: 20% of time");
}

fn demonstrate_position_sizing(_signals: &[Signal], _config: &GarchStrategyConfig) {
    println!("\n💰 Position Sizing Analysis");
    println!("===========================");

    println!("📊 Volatility-adjusted position sizes:");
    println!("  📉 Low volatility periods: 100% base position");
    println!("  📊 Normal volatility: 80% base position");
    println!("  📈 High volatility: 50% base position");
}
