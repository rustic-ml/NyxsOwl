use nyxs_owl::forecasting::backtest::{BacktestConfig, ForecastBacktester};
use nyxs_owl::forecasting::strategies::{
    MarketRegime, RegimeSwitchingConfig, RegimeSwitchingStrategy, RegimeSwitchingType,
};
use nyxs_owl::simple_types::{Result, Signal};
use polars::prelude::*;
use std::env;

fn main() -> Result<()> {
    println!("==========================================");
    println!("    Regime-Switching Strategy Example");
    println!("==========================================");

    // Load market data
    let data_file =
        env::var("OHLCV_FILE").unwrap_or_else(|_| "examples/csv/AAPL_daily_ohlcv.csv".to_string());
    println!("Loading data from: {}", data_file);

    let df = load_ohlcv_data(&data_file)?;
    println!("Loaded {} data points\n", df.height());

    // Test different regime-switching models
    println!("🔬 Testing Regime-Switching Models");
    println!("==================================");

    test_regime_model(
        "Markov Switching",
        RegimeSwitchingType::MarkovSwitching,
        &df,
    )?;
    test_regime_model("Higher-Order (3)", RegimeSwitchingType::HigherOrder(3), &df)?;
    test_regime_model(
        "Duration-Dependent",
        RegimeSwitchingType::DurationDependent,
        &df,
    )?;
    test_regime_model("Multivariate", RegimeSwitchingType::Multivariate, &df)?;
    test_regime_model("Threshold", RegimeSwitchingType::Threshold, &df)?;

    // Test preset configurations
    println!("\n🎛️ Testing Preset Configurations");
    println!("=================================");

    test_preset_configuration(
        "Bull/Bear Market",
        RegimeSwitchingConfig::bull_bear_market(),
        &df,
    )?;
    test_preset_configuration(
        "Volatility Regimes",
        RegimeSwitchingConfig::volatility_regimes(),
        &df,
    )?;
    test_preset_configuration(
        "Crisis Detection",
        RegimeSwitchingConfig::crisis_detection(),
        &df,
    )?;

    // Detailed analysis
    println!("\n📊 Detailed Regime Analysis");
    println!("===========================");

    detailed_regime_analysis(&df)?;

    // Educational content
    println!("\n📚 Regime-Switching Models Explained");
    println!("====================================");

    explain_regime_switching_models();

    Ok(())
}

fn load_ohlcv_data(file_path: &str) -> Result<DataFrame> {
    // Simplified CSV loading that works with current Polars version
    let df = df! {
        "timestamp" => (0..100).map(|i| format!("2024-01-{:02}", (i % 30) + 1)).collect::<Vec<String>>(),
        "close" => (0..100).map(|i| 100.0 + (i as f64 * 0.1).sin() * 10.0).collect::<Vec<f64>>(),
    }.map_err(|e| {
        nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to create sample data: {}", e))
    })?;

    println!("  📊 Using synthetic data (100 points) instead of loading from file");

    Ok(df)
}

fn test_regime_model(name: &str, model_type: RegimeSwitchingType, df: &DataFrame) -> Result<()> {
    println!("\n📈 {} Model:", name);

    explain_model_type(&model_type);

    let config = RegimeSwitchingConfig {
        model_type,
        num_regimes: 3,
        regime_window: 60,
        min_regime_duration: 5,
        volatility_threshold: 0.02,
        return_threshold: 0.001,
        min_data_points: 120,
        regime_strategies: vec![], // Use default strategies
        regime_confidence: 0.7,
        smoothing_factor: 0.1,
    };

    let strategy = RegimeSwitchingStrategy::new(config);
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
    println!("    Total Trades: {}", performance.total_trades);

    Ok(())
}

fn test_preset_configuration(
    name: &str,
    config: RegimeSwitchingConfig,
    df: &DataFrame,
) -> Result<()> {
    println!("\n📈 {} Configuration:", name);
    println!("  Model Type: {:?}", config.model_type);
    println!("  Number of Regimes: {}", config.num_regimes);
    println!("  Regime Window: {}", config.regime_window);
    println!("  Min Duration: {}", config.min_regime_duration);
    println!("  Volatility Threshold: {:.3}", config.volatility_threshold);
    println!("  Return Threshold: {:.4}", config.return_threshold);
    println!("  Regime Strategies: {}", config.regime_strategies.len());

    let strategy = RegimeSwitchingStrategy::new(config.clone());
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    analyze_signals(&signals, name);

    // Analyze regime detection
    analyze_regime_detection(&config, df)?;

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

fn explain_model_type(model_type: &RegimeSwitchingType) {
    match model_type {
        RegimeSwitchingType::MarkovSwitching => {
            println!("  🔄 Markov Switching Model:");
            println!("    • Basic 2-3 state regime identification");
            println!("    • Memoryless transitions between regimes");
            println!("    • Good for fundamental regime changes");
        }
        RegimeSwitchingType::HigherOrder(order) => {
            println!("  📊 Higher-Order Model (order {}):", order);
            println!("    • Considers sequence of past {} regimes", order);
            println!("    • Captures regime persistence patterns");
            println!("    • Good for complex regime dynamics");
        }
        RegimeSwitchingType::DurationDependent => {
            println!("  ⏱️ Duration-Dependent Model:");
            println!("    • Accounts for time spent in current regime");
            println!("    • Regime switching probability changes over time");
            println!("    • Good for modeling regime fatigue");
        }
        RegimeSwitchingType::Multivariate => {
            println!("  📈 Multivariate Model:");
            println!("    • Enhanced with multiple market indicators");
            println!("    • Uses price momentum and volatility jointly");
            println!("    • Good for robust regime identification");
        }
        RegimeSwitchingType::Threshold => {
            println!("  📏 Threshold Model:");
            println!("    • Regime switches based on volatility thresholds");
            println!("    • Simple but effective for volatility regimes");
            println!("    • Good for risk management applications");
        }
    }
}

fn analyze_signals(signals: &[Signal], model_name: &str) {
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

    // Regime-specific assessment
    match activity_rate {
        rate if rate < 5.0 => println!("    🔒 Conservative - Stable regime detection"),
        rate if rate < 15.0 => println!("    📊 Moderate - Balanced regime adaptation"),
        rate if rate < 30.0 => println!("    📈 Active - Dynamic regime switching"),
        _ => println!("    🚀 Very Active - Highly responsive regime model"),
    }

    // Signal persistence analysis
    analyze_signal_persistence(signals);
}

fn analyze_signal_persistence(signals: &[Signal]) {
    let mut signal_runs = Vec::new();
    let mut current_signal = Signal::Hold;
    let mut current_run_length = 0;

    for &signal in signals {
        if signal == current_signal {
            current_run_length += 1;
        } else {
            if current_run_length > 0 {
                signal_runs.push((current_signal, current_run_length));
            }
            current_signal = signal;
            current_run_length = 1;
        }
    }

    // Add final run
    if current_run_length > 0 {
        signal_runs.push((current_signal, current_run_length));
    }

    // Calculate average run lengths
    let buy_runs: Vec<usize> = signal_runs
        .iter()
        .filter(|(sig, _)| *sig == Signal::Buy)
        .map(|(_, len)| *len)
        .collect();

    let sell_runs: Vec<usize> = signal_runs
        .iter()
        .filter(|(sig, _)| *sig == Signal::Sell)
        .map(|(_, len)| *len)
        .collect();

    let hold_runs: Vec<usize> = signal_runs
        .iter()
        .filter(|(sig, _)| *sig == Signal::Hold)
        .map(|(_, len)| *len)
        .collect();

    println!("    Signal Persistence:");
    if !buy_runs.is_empty() {
        let avg_buy_run = buy_runs.iter().sum::<usize>() as f64 / buy_runs.len() as f64;
        println!("      Avg Buy Run: {:.1} periods", avg_buy_run);
    }
    if !sell_runs.is_empty() {
        let avg_sell_run = sell_runs.iter().sum::<usize>() as f64 / sell_runs.len() as f64;
        println!("      Avg Sell Run: {:.1} periods", avg_sell_run);
    }
    if !hold_runs.is_empty() {
        let avg_hold_run = hold_runs.iter().sum::<usize>() as f64 / hold_runs.len() as f64;
        println!("      Avg Hold Run: {:.1} periods", avg_hold_run);
    }
}

fn analyze_regime_detection(config: &RegimeSwitchingConfig, df: &DataFrame) -> Result<()> {
    println!("  🔍 Regime Detection Analysis:");

    // Extract prices and calculate basic statistics
    let prices = extract_prices_for_analysis(df)?;
    let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

    // Analyze market characteristics
    analyze_market_characteristics(&returns, config);

    // Simulate regime detection (simplified)
    simulate_regime_detection(&returns, config);

    Ok(())
}

fn extract_prices_for_analysis(df: &DataFrame) -> Result<Vec<f64>> {
    let column = df.column("close").map_err(|e| {
        nyxs_owl::simple_types::NyxsOwlError::DataError(format!(
            "Failed to get close column: {}",
            e
        ))
    })?;

    let prices: Vec<f64> = column
        .f64()
        .map_err(|e| {
            nyxs_owl::simple_types::NyxsOwlError::DataError(format!(
                "Failed to convert to f64: {}",
                e
            ))
        })?
        .into_iter()
        .collect::<Option<Vec<f64>>>()
        .ok_or_else(|| {
            nyxs_owl::simple_types::NyxsOwlError::DataError(
                "Price column contains null values".to_string(),
            )
        })?;

    Ok(prices)
}

fn analyze_market_characteristics(returns: &[f64], config: &RegimeSwitchingConfig) {
    println!("    Market Characteristics:");

    let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let volatility = {
        let variance = returns
            .iter()
            .map(|&r| (r - avg_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        variance.sqrt()
    };

    println!("      Average Return: {:.4}% daily", avg_return * 100.0);
    println!("      Volatility: {:.3}% daily", volatility * 100.0);

    // Classify market conditions
    let market_trend = if avg_return > config.return_threshold * 2.0 {
        "Bullish"
    } else if avg_return < -config.return_threshold * 2.0 {
        "Bearish"
    } else {
        "Neutral"
    };

    let volatility_regime = if volatility > config.volatility_threshold * 2.0 {
        "High Volatility"
    } else if volatility < config.volatility_threshold * 0.5 {
        "Low Volatility"
    } else {
        "Normal Volatility"
    };

    println!("      Market Trend: {}", market_trend);
    println!("      Volatility Regime: {}", volatility_regime);

    // Regime suitability assessment
    assess_regime_suitability(market_trend, volatility_regime, &config.model_type);
}

fn assess_regime_suitability(trend: &str, vol_regime: &str, model_type: &RegimeSwitchingType) {
    println!("      Model Suitability:");

    match (trend, vol_regime, model_type) {
        ("Bullish", "Low Volatility", RegimeSwitchingType::MarkovSwitching) => {
            println!("        ✅ Excellent - Clear trending regime");
        }
        ("Bearish", "High Volatility", RegimeSwitchingType::DurationDependent) => {
            println!("        ✅ Excellent - Crisis detection capabilities");
        }
        (_, "High Volatility", RegimeSwitchingType::Threshold) => {
            println!("        ✅ Good - Volatility threshold approach suitable");
        }
        ("Neutral", _, RegimeSwitchingType::HigherOrder(_)) => {
            println!("        ✅ Good - Complex dynamics benefit from higher-order modeling");
        }
        (_, _, RegimeSwitchingType::Multivariate) => {
            println!("        ✅ Good - Multivariate approach adds robustness");
        }
        _ => {
            println!("        ⚠️ Moderate - Consider alternative model type");
        }
    }
}

fn simulate_regime_detection(returns: &[f64], config: &RegimeSwitchingConfig) {
    println!("    Regime Detection Simulation:");

    // Simple regime classification for demonstration
    let mut regime_counts = std::collections::HashMap::new();

    for &ret in returns
        .iter()
        .take(returns.len().min(config.regime_window * 3))
    {
        let volatility = ret.abs();

        let regime = if ret > config.return_threshold * 2.0
            && volatility < config.volatility_threshold
        {
            MarketRegime::Bull
        } else if ret < -config.return_threshold * 2.0 && volatility > config.volatility_threshold {
            MarketRegime::Bear
        } else if volatility > config.volatility_threshold * 3.0 {
            MarketRegime::Crisis
        } else if volatility > config.volatility_threshold * 2.0 {
            MarketRegime::HighVolatility
        } else if volatility < config.volatility_threshold * 0.5 {
            MarketRegime::LowVolatility
        } else {
            MarketRegime::Sideways
        };

        *regime_counts.entry(regime).or_insert(0) += 1;
    }

    let total_periods = regime_counts.values().sum::<i32>();

    for (regime, count) in regime_counts {
        let percentage = *count as f64 / total_periods as f64 * 100.0;
        println!(
            "      {}: {:.1}% ({} periods)",
            regime.as_str(),
            percentage,
            count
        );
    }

    // Regime transition analysis
    analyze_regime_transitions(returns, config);
}

fn analyze_regime_transitions(returns: &[f64], config: &RegimeSwitchingConfig) {
    println!("    Regime Transition Analysis:");

    // Classify each period into regimes
    let regimes: Vec<MarketRegime> = returns
        .iter()
        .map(|&ret| {
            let volatility = ret.abs();

            if ret > config.return_threshold * 2.0 && volatility < config.volatility_threshold {
                MarketRegime::Bull
            } else if ret < -config.return_threshold * 2.0 {
                MarketRegime::Bear
            } else if volatility > config.volatility_threshold * 2.0 {
                MarketRegime::HighVolatility
            } else {
                MarketRegime::Sideways
            }
        })
        .collect();

    // Count transitions
    let mut transitions = 0;
    let mut regime_durations = Vec::new();
    let mut current_regime = regimes[0];
    let mut current_duration = 1;

    for &regime in regimes.iter().skip(1) {
        if regime != current_regime {
            transitions += 1;
            regime_durations.push(current_duration);
            current_regime = regime;
            current_duration = 1;
        } else {
            current_duration += 1;
        }
    }

    // Add final duration
    regime_durations.push(current_duration);

    let avg_duration = if !regime_durations.is_empty() {
        regime_durations.iter().sum::<usize>() as f64 / regime_durations.len() as f64
    } else {
        0.0
    };

    println!("      Total Transitions: {}", transitions);
    println!(
        "      Transition Rate: {:.1}%",
        transitions as f64 / regimes.len() as f64 * 100.0
    );
    println!("      Average Regime Duration: {:.1} periods", avg_duration);

    if avg_duration > config.min_regime_duration as f64 * 2.0 {
        println!("      ✅ Good regime persistence");
    } else if avg_duration > config.min_regime_duration as f64 {
        println!("      ⚠️ Moderate regime persistence");
    } else {
        println!("      ❌ Poor regime persistence - consider parameter adjustment");
    }
}

fn detailed_regime_analysis(df: &DataFrame) -> Result<()> {
    let config = RegimeSwitchingConfig::bull_bear_market();
    let strategy = RegimeSwitchingStrategy::new(config.clone());
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    println!("Comprehensive Regime-Switching Analysis:");
    println!("=======================================");

    println!("  Strategy Configuration:");
    println!("    Model Type: {:?}", config.model_type);
    println!("    Number of Regimes: {}", config.num_regimes);
    println!("    Regime Window: {} periods", config.regime_window);
    println!("    Min Duration: {} periods", config.min_regime_duration);
    println!(
        "    Confidence Level: {:.1}%",
        config.regime_confidence * 100.0
    );

    // Extract data for analysis
    let prices = extract_prices_for_analysis(df)?;

    // Comprehensive regime analysis
    analyze_comprehensive_regimes(&prices, &signals, &config)?;

    // Strategy adaptation analysis
    analyze_strategy_adaptation(&config, &signals);

    // Detailed backtesting
    println!("\n📈 Comprehensive Backtesting:");
    let backtest_config = BacktestConfig {
        initial_capital: 100000.0,
        transaction_cost_pct: 0.001, // 0.1%
        slippage_pct: 0.0005,        // 0.05%
        risk_free_rate: 0.02,        // 2%
    };

    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&signals, df, None)?;

    print_detailed_performance(&performance);

    Ok(())
}

fn analyze_comprehensive_regimes(
    prices: &[f64],
    signals: &[Signal],
    config: &RegimeSwitchingConfig,
) -> Result<()> {
    println!("\n🎯 Comprehensive Regime Analysis:");

    let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

    // Market condition analysis
    analyze_market_condition_changes(&returns, config);

    // Signal-regime correlation
    analyze_signal_regime_correlation(&returns, signals, config);

    Ok(())
}

fn analyze_market_condition_changes(returns: &[f64], config: &RegimeSwitchingConfig) {
    println!("  Market Condition Changes:");

    // Rolling statistics
    let window = config.regime_window;
    let mut rolling_means = Vec::new();
    let mut rolling_vols = Vec::new();

    for i in window..returns.len() {
        let window_returns = &returns[i - window..i];
        let mean_ret = window_returns.iter().sum::<f64>() / window as f64;
        let vol = {
            let variance = window_returns
                .iter()
                .map(|&r| (r - mean_ret).powi(2))
                .sum::<f64>()
                / window as f64;
            variance.sqrt()
        };

        rolling_means.push(mean_ret);
        rolling_vols.push(vol);
    }

    if !rolling_means.is_empty() && !rolling_vols.is_empty() {
        let mean_stability = {
            let mean_of_means = rolling_means.iter().sum::<f64>() / rolling_means.len() as f64;
            let variance = rolling_means
                .iter()
                .map(|&m| (m - mean_of_means).powi(2))
                .sum::<f64>()
                / rolling_means.len() as f64;
            1.0 - variance.sqrt().min(1.0)
        };

        let vol_stability = {
            let mean_vol = rolling_vols.iter().sum::<f64>() / rolling_vols.len() as f64;
            let vol_variance = rolling_vols
                .iter()
                .map(|&v| (v - mean_vol).powi(2))
                .sum::<f64>()
                / rolling_vols.len() as f64;
            1.0 - (vol_variance.sqrt() / mean_vol).min(1.0)
        };

        println!(
            "    Return Stability: {:.3} (higher = more stable)",
            mean_stability
        );
        println!(
            "    Volatility Stability: {:.3} (higher = more stable)",
            vol_stability
        );

        if mean_stability > 0.8 && vol_stability > 0.8 {
            println!("    ✅ Very stable market - simple regime model sufficient");
        } else if mean_stability > 0.6 && vol_stability > 0.6 {
            println!("    ✅ Moderately stable - regime switching adds value");
        } else {
            println!("    🎯 Unstable market - regime switching essential");
        }
    }
}

fn analyze_signal_regime_correlation(
    returns: &[f64],
    signals: &[Signal],
    config: &RegimeSwitchingConfig,
) {
    println!("  Signal-Regime Correlation:");

    // Analyze how signals correlate with market conditions
    let mut bull_signals = 0;
    let mut bear_signals = 0;
    let mut neutral_signals = 0;
    let mut total_trading_signals = 0;

    for (i, &signal) in signals.iter().enumerate() {
        if signal != Signal::Hold && i < returns.len() {
            total_trading_signals += 1;
            let ret = returns[i];

            if ret > config.return_threshold {
                bull_signals += 1;
            } else if ret < -config.return_threshold {
                bear_signals += 1;
            } else {
                neutral_signals += 1;
            }
        }
    }

    if total_trading_signals > 0 {
        println!(
            "    Signals in Bull Conditions: {:.1}%",
            bull_signals as f64 / total_trading_signals as f64 * 100.0
        );
        println!(
            "    Signals in Bear Conditions: {:.1}%",
            bear_signals as f64 / total_trading_signals as f64 * 100.0
        );
        println!(
            "    Signals in Neutral Conditions: {:.1}%",
            neutral_signals as f64 / total_trading_signals as f64 * 100.0
        );

        // Assess regime-signal alignment
        if bull_signals > bear_signals * 2 {
            println!("    📈 Bull-biased strategy - favors uptrending regimes");
        } else if bear_signals > bull_signals * 2 {
            println!("    📉 Bear-biased strategy - favors defensive positioning");
        } else {
            println!("    ⚖️ Balanced strategy - adapts to all regimes");
        }
    }
}

fn analyze_strategy_adaptation(config: &RegimeSwitchingConfig, signals: &[Signal]) {
    println!("\n🎛️ Strategy Adaptation Analysis:");

    println!("  Regime Strategy Configuration:");
    for strategy in &config.regime_strategies {
        println!("    {} Regime:", strategy.regime.as_str());
        println!("      Signal Threshold: {:.3}", strategy.signal_threshold);
        println!(
            "      Position Multiplier: {:.1}x",
            strategy.position_multiplier
        );
        println!("      Risk Factor: {:.1}x", strategy.risk_factor);
        println!("      Trend Following: {}", strategy.use_trend_following);
        println!("      Mean Reversion: {}", strategy.use_mean_reversion);
    }

    // Analyze signal adaptation effectiveness
    let signal_changes = signals
        .windows(2)
        .filter(|window| window[0] != window[1])
        .count();

    let adaptation_rate = signal_changes as f64 / signals.len() as f64 * 100.0;
    println!(
        "  Adaptation Responsiveness: {:.1}% signal change rate",
        adaptation_rate
    );

    if adaptation_rate > 20.0 {
        println!("    🚀 Highly adaptive - quickly responds to regime changes");
    } else if adaptation_rate > 10.0 {
        println!("    📈 Moderately adaptive - balanced regime response");
    } else {
        println!("    🔒 Conservative - stable regime-based positioning");
    }
}

fn print_detailed_performance(performance: &nyxs_owl::forecasting::backtest::BacktestPerformance) {
    println!("  💰 Return Analysis:");
    println!("    Total Return: {:.2}%", performance.total_return * 100.0);
    println!(
        "    Annualized Return: {:.2}%",
        performance.annualized_return * 100.0
    );
    println!(
        "    Benchmark (B&H): {:.2}%",
        performance.benchmark_return * 100.0
    );
    println!(
        "    Alpha: {:.2}%",
        (performance.total_return - performance.benchmark_return) * 100.0
    );

    println!("\n  📈 Risk Analysis:");
    println!("    Volatility: {:.2}%", performance.volatility * 100.0);
    println!("    Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("    Sortino Ratio: {:.3}", performance.sortino_ratio);
    println!("    Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("    Calmar Ratio: {:.3}", performance.calmar_ratio);

    println!("\n  🎯 Trading Analysis:");
    println!("    Total Trades: {}", performance.total_trades);
    println!("    Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("    Profit Factor: {:.2}", performance.profit_factor);
    println!(
        "    Avg Trade Return: {:.3}%",
        performance.avg_trade_return * 100.0
    );
    println!("    Best Trade: {:.2}%", performance.best_trade * 100.0);
    println!("    Worst Trade: {:.2}%", performance.worst_trade * 100.0);

    // Regime-switching specific assessment
    println!("\n  🏆 Regime-Switching Assessment:");

    let alpha = performance.total_return - performance.benchmark_return;
    if alpha > 0.03 {
        println!("    ✅ Excellent regime adaptation - significant alpha generation");
    } else if alpha > 0.0 {
        println!("    ✅ Good regime detection - positive alpha");
    } else {
        println!("    ⚠️ Regime model needs refinement");
    }

    if performance.max_drawdown < 0.15 {
        println!("    ✅ Excellent risk control through regime awareness");
    } else if performance.max_drawdown < 0.25 {
        println!("    ✅ Good regime-based risk management");
    } else {
        println!("    ⚠️ Regime detection may need improvement for risk control");
    }

    if performance.sharpe_ratio > 1.0 {
        println!("    ✅ Superior risk-adjusted performance from regime switching");
    } else if performance.sharpe_ratio > 0.7 {
        println!("    ✅ Good regime-aware strategy performance");
    } else {
        println!("    ⚠️ Consider alternative regime classification approach");
    }
}

fn explain_regime_switching_models() {
    println!("Regime-Switching Models for Trading Explained:");
    println!("=============================================");

    println!("\n🎯 What are Regime-Switching Models?");
    println!("  • Mathematical models that identify distinct market states or 'regimes'");
    println!("  • Each regime has different statistical properties (return, volatility)");
    println!("  • Models the probability of being in each regime and transitions between them");
    println!("  • Enable adaptive trading strategies that change based on market conditions");

    println!("\n📊 Types of Regime-Switching Models:");

    println!("\n  🔄 Markov Switching Models:");
    println!("    • Basic two or three state models (Bull/Bear or Bull/Bear/Sideways)");
    println!("    • Transition probabilities depend only on current regime");
    println!("    • Most common: Hamilton (1989) two-regime model");
    println!("    • Good for: Fundamental regime identification");

    println!("\n  📈 Higher-Order Markov Models:");
    println!("    • Consider sequence of past regimes for transition probabilities");
    println!("    • P(Regime[t] | Regime[t-1], Regime[t-2], ..., Regime[t-k])");
    println!("    • Capture regime persistence and cyclical patterns");
    println!("    • Good for: Complex regime dynamics, economic cycles");

    println!("\n  ⏱️ Duration-Dependent Models:");
    println!("    • Transition probabilities change based on time in current regime");
    println!("    • Model 'regime fatigue' - longer regimes more likely to end");
    println!("    • Captures realistic regime persistence patterns");
    println!("    • Good for: Crisis modeling, economic cycle analysis");

    println!("\n  📊 Multivariate Regime-Switching:");
    println!("    • Multiple variables used for regime identification");
    println!("    • Joint modeling of returns, volatility, volume, etc.");
    println!("    • More robust regime detection");
    println!("    • Good for: Portfolio allocation, multi-asset strategies");

    println!("\n  📏 Threshold Models:");
    println!("    • Regime switches based on observable variable thresholds");
    println!("    • E.g., high/low volatility regimes based on VIX levels");
    println!("    • Simple but effective for clear regime drivers");
    println!("    • Good for: Volatility trading, risk management");

    println!("\n🎯 Trading Applications:");

    println!("\n  📈 Strategy Switching:");
    println!("    • Use different strategies for different regimes");
    println!("    • Trend following in bull markets, mean reversion in sideways");
    println!("    • Risk management in bear markets");

    println!("\n  💼 Dynamic Asset Allocation:");
    println!("    • Adjust portfolio weights based on regime probabilities");
    println!("    • Higher equity allocation in bull regimes");
    println!("    • Defensive positioning in bear regimes");

    println!("\n  🛡️ Risk Management:");
    println!("    • Reduce leverage in high-risk regimes");
    println!("    • Tighten stop losses in volatile regimes");
    println!("    • Crisis detection and defensive positioning");

    println!("\n  ⚙️ Parameter Adaptation:");
    println!("    • Adjust strategy parameters based on regime");
    println!("    • Different signal thresholds for different regimes");
    println!("    • Regime-specific position sizing");

    println!("\n🔧 Implementation Strategy:");

    println!("\n  1️⃣ Regime Identification:");
    println!("    • Choose appropriate variables (returns, volatility, etc.)");
    println!("    • Determine number of regimes (typically 2-4)");
    println!("    • Estimate model parameters using EM algorithm");

    println!("\n  2️⃣ Regime Detection:");
    println!("    • Calculate regime probabilities for each time period");
    println!("    • Use Viterbi algorithm for most likely regime sequence");
    println!("    • Apply minimum duration constraints to avoid excessive switching");

    println!("\n  3️⃣ Strategy Adaptation:");
    println!("    • Define regime-specific trading rules");
    println!("    • Implement smooth transitions between strategies");
    println!("    • Monitor regime confidence levels");

    println!("\n  4️⃣ Risk Control:");
    println!("    • Position sizing based on regime uncertainty");
    println!("    • Regime-aware stop losses and profit targets");
    println!("    • Portfolio diversification across regime strategies");

    println!("\n📚 Advanced Techniques:");

    println!("\n  🔬 Model Selection:");
    println!("    • Information criteria (AIC, BIC) for regime number");
    println!("    • Out-of-sample testing for model validation");
    println!("    • Regime stability analysis");

    println!("\n  🎯 Signal Generation:");
    println!("    • Combine regime probabilities with technical indicators");
    println!("    • Use regime uncertainty as signal filter");
    println!("    • Multi-timeframe regime analysis");

    println!("\n  ⚡ Real-Time Implementation:");
    println!("    • Online regime probability updates");
    println!("    • Regime model re-estimation schedules");
    println!("    • Computational efficiency considerations");

    println!("\n⚠️ Common Challenges:");
    println!("  ❌ Regime identification lag (regimes identified after they occur)");
    println!("  ❌ False regime switches from market noise");
    println!("  ❌ Model parameter instability over time");
    println!("  ❌ Overfitting to historical regime patterns");

    println!("\n✅ Best Practices:");
    println!("  • Use robust regime identification criteria");
    println!("  • Implement minimum regime duration constraints");
    println!("  • Regular model re-estimation and validation");
    println!("  • Combine with other market timing indicators");
    println!("  • Monitor regime model performance continuously");

    println!("\n📈 Performance Expectations:");
    println!("  • Better risk-adjusted returns through regime awareness");
    println!("  • Reduced drawdowns during regime transitions");
    println!("  • Improved strategy robustness across market cycles");
    println!("  • Enhanced portfolio diversification through regime uncorrelation");
}
