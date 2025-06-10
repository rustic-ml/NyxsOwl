use nyxs_owl::{
    forecasting::{
        backtest::{BacktestConfig, ForecastBacktester},
        strategies::ensemble_strategy::{EnsembleMethod, EnsembleStrategy, EnsembleStrategyConfig},
    },
    simple_types::{Result, Signal},
};
use polars::prelude::*;

fn main() -> Result<()> {
    println!("🎯 NyxsOwl Ensemble Strategy Example");
    println!("===================================");

    // Create synthetic data for demonstration
    println!("📊 Creating synthetic data for demonstration...");
    let df = create_synthetic_data(250)?;
    println!("✅ Data created successfully! Shape: {} rows", df.height());

    // Test different ensemble methods
    test_ensemble_method("Simple Average", EnsembleMethod::SimpleAverage, &df)?;
    test_ensemble_method("Median", EnsembleMethod::Median, &df)?;
    test_ensemble_method("Majority Vote", EnsembleMethod::MajorityVote, &df)?;
    test_ensemble_method("Best Model", EnsembleMethod::BestModel, &df)?;

    // Test preset configurations
    println!("\n🎛️  Testing Preset Configurations:");
    println!("===============================");

    test_preset_configuration("Conservative", EnsembleStrategyConfig::conservative(), &df)?;
    test_preset_configuration("Aggressive", EnsembleStrategyConfig::aggressive(), &df)?;
    test_preset_configuration("Balanced", EnsembleStrategyConfig::balanced(), &df)?;

    // Detailed analysis
    println!("\n📊 Detailed Analysis - Balanced Configuration");
    println!("===========================================");

    detailed_ensemble_analysis(&df)?;

    Ok(())
}

fn create_synthetic_data(length: usize) -> Result<DataFrame> {
    // Generate synthetic price data for demonstration
    let mut prices = Vec::with_capacity(length);
    let mut price = 100.0;

    for i in 0..length {
        // Add trend and noise
        let trend = 0.001 * (i as f64).sin() * 0.1;
        let noise = (i as f64 * 0.1).sin() * 0.02;
        price *= 1.0 + trend + noise;
        prices.push(price);
    }

    let timestamps: Vec<String> = (0..length)
        .map(|i| format!("2023-01-{:02} 09:30:00", (i % 30) + 1))
        .collect();

    df! {
        "timestamp" => timestamps,
        "close" => prices.clone(),
        "high" => prices.iter().map(|p| p * 1.02).collect::<Vec<_>>(),
        "low" => prices.iter().map(|p| p * 0.98).collect::<Vec<_>>(),
        "open" => prices.clone(),
        "volume" => vec![1000i64; length],
    }
    .map_err(|e| {
        nyxs_owl::simple_types::NyxsOwlError::DataError(format!(
            "Failed to create synthetic data: {}",
            e
        ))
    })
}

fn test_ensemble_method(name: &str, method: EnsembleMethod, df: &DataFrame) -> Result<()> {
    println!("\n📈 {} Method:", name);

    let config = EnsembleStrategyConfig {
        method,
        signal_threshold: 0.015,
        min_data_points: 120,
        min_confidence: 0.6,
        ..Default::default()
    };

    print_method_details(&config.method);

    let strategy = EnsembleStrategy::new(config);
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    analyze_signals(&signals);

    // Perform backtesting with correct API
    let prices = extract_prices_for_analysis(df)?;
    let backtest_config = BacktestConfig::default();
    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&prices, &signals, None)?;

    println!("  📊 Performance Metrics:");
    println!("    Total Return: {:.2}%", performance.total_return * 100.0);
    println!("    Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("    Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("    Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("    Total Trades: {}", performance.total_trades);

    Ok(())
}

fn test_preset_configuration(
    name: &str,
    config: EnsembleStrategyConfig,
    df: &DataFrame,
) -> Result<()> {
    println!("\n📈 {} Configuration:", name);

    println!("  Method: {:?}", config.method);
    println!(
        "  Signal Threshold: {:.2}%",
        config.signal_threshold * 100.0
    );
    println!("  Min Confidence: {:.1}%", config.min_confidence * 100.0);
    println!(
        "  Models: ARIMA={}, ES={}, Kalman={}",
        config.model_config.use_arima,
        config.model_config.use_exponential_smoothing,
        config.model_config.use_kalman
    );

    let strategy = EnsembleStrategy::new(config);
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    analyze_signals(&signals);

    // Perform backtesting with correct API
    let prices = extract_prices_for_analysis(df)?;
    let backtest_config = BacktestConfig::default();
    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&prices, &signals, None)?;

    println!("  📊 Performance Metrics:");
    println!("    Total Return: {:.2}%", performance.total_return * 100.0);
    println!("    Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("    Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("    Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("    Profit Factor: {:.2}", performance.profit_factor);

    Ok(())
}

fn print_method_details(method: &EnsembleMethod) {
    match method {
        EnsembleMethod::SimpleAverage => {
            println!("  🔄 Simple Average: Equal weight to all models");
        }
        EnsembleMethod::WeightedAverage(weights) => {
            println!("  ⚖️  Weighted Average: Custom weights {:?}", weights);
        }
        EnsembleMethod::Median => {
            println!("  📊 Median: Middle value of all predictions");
        }
        EnsembleMethod::BestModel => {
            println!("  🏆 Best Model: Dynamic selection based on performance");
        }
        EnsembleMethod::MajorityVote => {
            println!("  🗳️  Majority Vote: Most common signal wins");
        }
        EnsembleMethod::Stacking => {
            println!("  🧠 Stacking: Meta-learner (not implemented)");
        }
    }
}

fn analyze_signals(signals: &[Signal]) {
    let total_signals = signals.len();
    let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
    let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();

    println!("  🎯 Signal Analysis:");
    println!("    Total Signals: {}", total_signals);
    println!(
        "    Buy Signals: {} ({:.1}%)",
        buy_count,
        buy_count as f64 / total_signals as f64 * 100.0
    );
    println!(
        "    Sell Signals: {} ({:.1}%)",
        sell_count,
        sell_count as f64 / total_signals as f64 * 100.0
    );
    println!(
        "    Hold Signals: {} ({:.1}%)",
        hold_count,
        hold_count as f64 / total_signals as f64 * 100.0
    );

    // Calculate signal activity
    let activity_rate = (buy_count + sell_count) as f64 / total_signals as f64 * 100.0;
    println!("    Trading Activity: {:.1}%", activity_rate);

    // Assess activity level
    match activity_rate {
        rate if rate < 5.0 => println!("    📉 Very Conservative - Low activity"),
        rate if rate < 15.0 => println!("    📊 Conservative - Moderate activity"),
        rate if rate < 30.0 => println!("    📈 Active - High activity"),
        _ => println!("    🚀 Very Active - Very high activity"),
    }
}

fn detailed_ensemble_analysis(df: &DataFrame) -> Result<()> {
    let config = EnsembleStrategyConfig::balanced();
    let strategy = EnsembleStrategy::new(config.clone());
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    println!("Configuration Details:");
    println!("  Aggregation Method: {:?}", config.method);
    println!(
        "  Signal Threshold: {:.2}%",
        config.signal_threshold * 100.0
    );
    println!(
        "  Minimum Confidence: {:.1}%",
        config.min_confidence * 100.0
    );
    println!(
        "  Performance Window: {} periods",
        config.performance_window
    );

    println!("\nModel Configuration:");
    println!("  ✅ ARIMA: {}", config.model_config.use_arima);
    println!(
        "  ✅ Exponential Smoothing: {}",
        config.model_config.use_exponential_smoothing
    );
    println!("  ✅ Kalman Filter: {}", config.model_config.use_kalman);

    // Analyze signal patterns
    analyze_signal_patterns(&signals);

    // Extract prices for market analysis
    let prices = extract_prices_for_analysis(df)?;
    analyze_ensemble_performance(&prices, &signals);

    // Detailed backtesting with correct field names
    println!("\n📈 Comprehensive Backtesting:");
    let backtest_config = BacktestConfig {
        initial_capital: 100000.0,
        transaction_cost: 0.001, // Fixed field name
        slippage: 0.0005,        // Fixed field name
        risk_free_rate: 0.02,    // 2%
        position_size: 1.0,      // Full position size
    };

    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&prices, &signals, None)?;

    print_comprehensive_performance(&performance);

    Ok(())
}

fn analyze_signal_patterns(signals: &[Signal]) {
    println!("\n🔍 Signal Pattern Analysis:");

    // Find signal streaks
    let mut current_signal = Signal::Hold;
    let mut current_streak = 0;
    let mut max_buy_streak = 0;
    let mut max_sell_streak = 0;
    let mut max_hold_streak = 0;

    for &signal in signals {
        if signal == current_signal {
            current_streak += 1;
        } else {
            match current_signal {
                Signal::Buy => max_buy_streak = max_buy_streak.max(current_streak),
                Signal::Sell => max_sell_streak = max_sell_streak.max(current_streak),
                Signal::Hold => max_hold_streak = max_hold_streak.max(current_streak),
            }
            current_signal = signal;
            current_streak = 1;
        }
    }

    // Handle final streak
    match current_signal {
        Signal::Buy => max_buy_streak = max_buy_streak.max(current_streak),
        Signal::Sell => max_sell_streak = max_sell_streak.max(current_streak),
        Signal::Hold => max_hold_streak = max_hold_streak.max(current_streak),
    }

    println!("  📊 Signal Streaks:");
    println!("    Max Buy Streak: {} periods", max_buy_streak);
    println!("    Max Sell Streak: {} periods", max_sell_streak);
    println!("    Max Hold Streak: {} periods", max_hold_streak);
}

fn extract_prices_for_analysis(df: &DataFrame) -> Result<Vec<f64>> {
    let close_column = df.column("close").map_err(|e| {
        nyxs_owl::simple_types::NyxsOwlError::DataError(format!(
            "Failed to get close column: {}",
            e
        ))
    })?;

    let prices: Vec<f64> = close_column
        .f64()
        .map_err(|e| {
            nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to cast to f64: {}", e))
        })?
        .into_no_null_iter()
        .collect();

    Ok(prices)
}

fn analyze_ensemble_performance(prices: &[f64], signals: &[Signal]) {
    println!("\n📈 Market Analysis:");

    let start_price = prices[0];
    let end_price = prices[prices.len() - 1];
    let total_return = (end_price - start_price) / start_price * 100.0;

    println!("  📊 Market Performance:");
    println!("    Start Price: ${:.2}", start_price);
    println!("    End Price: ${:.2}", end_price);
    println!("    Buy & Hold Return: {:.2}%", total_return);

    // Calculate price volatility
    let returns: Vec<f64> = prices
        .windows(2)
        .map(|window| (window[1] - window[0]) / window[0])
        .collect();

    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns
        .iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;
    let volatility = variance.sqrt() * 100.0;

    println!("    Daily Volatility: {:.2}%", volatility);

    // Analyze signal timing
    let buy_signals: Vec<usize> = signals
        .iter()
        .enumerate()
        .filter(|(_, &signal)| signal == Signal::Buy)
        .map(|(i, _)| i)
        .collect();

    let sell_signals: Vec<usize> = signals
        .iter()
        .enumerate()
        .filter(|(_, &signal)| signal == Signal::Sell)
        .map(|(i, _)| i)
        .collect();

    println!("  🎯 Signal Timing:");
    println!(
        "    Buy Signals at periods: {:?}",
        &buy_signals[..buy_signals.len().min(5)]
    );
    println!(
        "    Sell Signals at periods: {:?}",
        &sell_signals[..sell_signals.len().min(5)]
    );
}

fn print_comprehensive_performance(
    performance: &nyxs_owl::forecasting::backtest::BacktestPerformance,
) {
    println!("📊 Comprehensive Performance Report:");
    println!("===================================");

    // Returns section - using available fields
    println!("📈 Returns:");
    println!("    Total Return: {:.2}%", performance.total_return * 100.0);

    // Risk section
    println!("\n⚖️ Risk Metrics:");
    println!("    Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("    Sortino Ratio: {:.3}", performance.sortino_ratio);
    println!("    Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);

    // Trading statistics
    println!("\n📊 Trading Statistics:");
    println!("    Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("    Total Trades: {}", performance.total_trades);
    println!("    Winning Trades: {}", performance.winning_trades);
    println!("    Losing Trades: {}", performance.losing_trades);
    println!("    Average Win: ${:.2}", performance.avg_win);
    println!("    Average Loss: ${:.2}", performance.avg_loss);
    println!("    Profit Factor: {:.2}", performance.profit_factor);

    // Performance assessment
    println!("\n🎯 Performance Assessment:");

    if performance.sharpe_ratio > 1.0 {
        println!("    📈 Excellent risk-adjusted returns (Sharpe > 1.0)");
    } else if performance.sharpe_ratio > 0.5 {
        println!("    📊 Good risk-adjusted returns (Sharpe > 0.5)");
    } else {
        println!("    📉 Below-average risk-adjusted returns");
    }

    if performance.max_drawdown < 0.05 {
        println!("    🛡️  Low drawdown risk (< 5%)");
    } else if performance.max_drawdown < 0.15 {
        println!("    ⚠️  Moderate drawdown risk (5-15%)");
    } else {
        println!("    🚨 High drawdown risk (> 15%)");
    }

    if performance.win_rate > 0.6 {
        println!("    ✅ High win rate (> 60%)");
    } else if performance.win_rate > 0.45 {
        println!("    📊 Moderate win rate (45-60%)");
    } else {
        println!("    📉 Low win rate (< 45%)");
    }
}
