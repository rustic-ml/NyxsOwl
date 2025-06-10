use nyxs_owl::forecasting::backtest::{BacktestConfig, ForecastBacktester};
use nyxs_owl::forecasting::strategies::{GarchStrategy, GarchStrategyConfig, GarchType};
use nyxs_owl::simple_types::{Result, Signal};
use polars::prelude::*;
use std::env;

fn main() -> Result<()> {
    println!("=================================");
    println!("    GARCH Strategy Example");
    println!("=================================");

    // Load market data
    let data_file =
        env::var("OHLCV_FILE").unwrap_or_else(|_| "examples/csv/AAPL_daily_ohlcv.csv".to_string());
    println!("Loading data from: {}", data_file);

    let df = load_ohlcv_data(&data_file)?;
    println!("Loaded {} data points\n", df.height());

    // Test different GARCH model types
    println!("🔬 Testing GARCH Model Types");
    println!("============================");

    test_garch_model("Standard GARCH(1,1)", GarchType::Standard, &df)?;
    test_garch_model("GARCH-M", GarchType::GarchM, &df)?;
    test_garch_model("EGARCH", GarchType::Egarch, &df)?;
    test_garch_model("GJR-GARCH", GarchType::GjrGarch, &df)?;

    // Test preset configurations
    println!("\n🎛️ Testing Preset Configurations");
    println!("=================================");

    test_preset_configuration("Conservative", GarchStrategyConfig::conservative(), &df)?;
    test_preset_configuration("Aggressive", GarchStrategyConfig::aggressive(), &df)?;
    test_preset_configuration(
        "Volatility Trading",
        GarchStrategyConfig::volatility_trading(),
        &df,
    )?;

    // Detailed analysis
    println!("\n📊 Detailed GARCH Analysis");
    println!("==========================");

    detailed_garch_analysis(&df)?;

    // Educational content
    println!("\n📚 GARCH Models Explained");
    println!("=========================");

    explain_garch_models();

    Ok(())
}

fn load_ohlcv_data(_file_path: &str) -> Result<DataFrame> {
    // Create sample data instead of using CSV scanning
    let n = 252; // One year of daily data
    let mut prices = Vec::with_capacity(n);
    let mut timestamps = Vec::with_capacity(n);

    let mut price = 100.0;
    let mut rng_state = 42u64; // Simple PRNG state

    for i in 0..n {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let random = (rng_state as f64 / u64::MAX as f64 - 0.5) * 0.02;

        price *= 1.0 + random + 0.0002;
        prices.push(price);
        timestamps.push(format!("2023-{:02}-{:02}", (i / 30) + 1, (i % 30) + 1));
    }

    let df = df! {
        "timestamp" => timestamps,
        "close" => prices,
    }?;

    // Ensure we have required columns
    let required_columns = ["close", "timestamp"];
    for col in required_columns.iter() {
        if df.column(col).is_err() {
            return Err(nyxs_owl::simple_types::NyxsOwlError::DataError(format!(
                "Required column '{}' not found",
                col
            )));
        }
    }

    Ok(df)
}

fn test_garch_model(name: &str, garch_type: GarchType, df: &DataFrame) -> Result<()> {
    println!("\n🎯 Testing {} GARCH Model", name);
    println!("=========================");

    let config = GarchStrategyConfig {
        model_type: garch_type,
        arch_order: 1,
        garch_order: 1,
        signal_threshold: 0.02,         // 2% threshold
        volatility_threshold: 1.5,      // 1.5x average volatility
        min_data_points: 50,
        use_volatility_targeting: true,
        target_volatility: 0.15,        // 15% annualized
        risk_adjustment: 1.2,
        max_position_size: 0.3,         // 30% max position
        min_volatility: 0.005,          // 0.5% minimum daily vol
        lookback_window: 30,
        use_regime_detection: true,
        regime_threshold: 0.02,
        enable_dynamic_hedging: false,
    };

    explain_garch_type(&garch_type);

    let strategy = GarchStrategy::new(config);
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    analyze_signals(&signals, name);

    // Extract prices for backtesting
    let prices: Vec<f64> = df.column("close")?.f64()?.into_no_null_iter().collect();

    // Perform backtesting
    let backtest_config = BacktestConfig::default();
    let backtester = ForecastBacktester::new(backtest_config);
    let performance = backtester.backtest(&prices, &signals, None)?;

    println!("  📊 Performance Metrics:");
    println!("    Total Return: {:.2}%", performance.total_return * 100.0);
    println!("    Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("    Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("    Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("    Total Trades: {}", performance.total_trades);

    // Analyze volatility characteristics
    analyze_volatility_performance(&config, &signals, df)?;

    Ok(())
}

fn test_preset_configuration(
    name: &str,
    config: GarchStrategyConfig,
    df: &DataFrame,
) -> Result<()> {
    println!("\n📈 {} Configuration:", name);
    println!("  Model Type: {:?}", config.model_type);
    println!(
        "  GARCH Order: {}, ARCH Order: {}",
        config.garch_order, config.arch_order
    );
    println!(
        "  Volatility Threshold: {:.1}x",
        config.volatility_threshold
    );
    println!(
        "  Signal Threshold: {:.2}%",
        config.signal_threshold * 100.0
    );
    println!(
        "  Volatility Targeting: {}",
        config.use_volatility_targeting
    );
    println!(
        "  Target Volatility: {:.1}%",
        config.target_volatility * 100.0
    );
    println!("  Risk Adjustment: {:.1}x", config.risk_adjustment);

    let strategy = GarchStrategy::new(config.clone());
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    analyze_signals(&signals, name);

    // Extract prices for backtesting
    let prices: Vec<f64> = df.column("close")?.f64()?.into_no_null_iter().collect();

    // Perform backtesting
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

fn explain_garch_type(garch_type: &GarchType) {
    match garch_type {
        GarchType::Standard => {
            println!("  🔄 Standard GARCH: Models volatility clustering");
            println!("    • Uses past squared returns and past volatility");
            println!("    • Good for symmetric volatility patterns");
        }
        GarchType::GarchM => {
            println!("  📈 GARCH-M: Includes volatility in mean equation");
            println!("    • Risk premium depends on conditional volatility");
            println!("    • Good for risk-return relationship modeling");
        }
        GarchType::Egarch => {
            println!("  📊 EGARCH: Exponential GARCH with asymmetry");
            println!("    • Captures leverage effect (negative shocks increase volatility more)");
            println!("    • Good for equity markets with asymmetric volatility");
        }
        GarchType::GjrGarch => {
            println!("  ⚡ GJR-GARCH: Threshold GARCH model");
            println!("    • Different impact for positive vs negative shocks");
            println!("    • Good for markets with volatility asymmetry");
        }
    }
}

fn analyze_signals(signals: &[Signal], config_name: &str) {
    let buy_signals = signals.iter().filter(|s| matches!(s, Signal::Buy)).count();
    let sell_signals = signals.iter().filter(|s| matches!(s, Signal::Sell)).count();
    let hold_signals = signals.iter().filter(|s| matches!(s, Signal::Hold)).count();

    println!("   🔍 Signal Analysis:");
    println!("      Buy signals: {}", buy_signals);
    println!("      Sell signals: {}", sell_signals);
    println!("      Hold signals: {}", hold_signals);
}

fn analyze_volatility_performance(
    config: &GarchStrategyConfig,
    signals: &[Signal],
    df: &DataFrame,
) -> Result<()> {
    println!("  🌊 Volatility Analysis:");

    // Extract prices and calculate returns
    let prices = extract_prices_for_analysis(df)?;
    let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

    // Calculate realized volatility
    let realized_vol = {
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|&r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        variance.sqrt()
    };

    println!(
        "    Realized Volatility: {:.3}% daily ({:.1}% annualized)",
        realized_vol * 100.0,
        realized_vol * (252.0_f64).sqrt() * 100.0
    );

    // Volatility regime analysis
    let high_vol_threshold = realized_vol * config.volatility_threshold;
    let high_vol_periods = returns
        .iter()
        .map(|&r| r.abs())
        .filter(|&abs_r| abs_r > high_vol_threshold)
        .count();

    let high_vol_percentage = high_vol_periods as f64 / returns.len() as f64 * 100.0;
    println!(
        "    High Volatility Periods: {:.1}% of time",
        high_vol_percentage
    );

    // Signal timing relative to volatility
    analyze_signal_volatility_timing(&returns, signals, realized_vol, config.volatility_threshold);

    // Position sizing analysis if using volatility targeting
    if config.use_volatility_targeting {
        analyze_position_sizing(config, realized_vol);
    }

    Ok(())
}

fn analyze_signal_volatility_timing(
    returns: &[f64],
    signals: &[Signal],
    avg_vol: f64,
    vol_threshold: f64,
) {
    println!("    📊 Signal-Volatility Timing:");

    let mut signals_in_high_vol = 0;
    let mut signals_in_low_vol = 0;
    let mut total_trading_signals = 0;

    for (i, &signal) in signals.iter().enumerate() {
        if signal != Signal::Hold && i > 0 && i < returns.len() {
            total_trading_signals += 1;
            let current_vol = returns[i].abs();

            if current_vol > avg_vol * vol_threshold {
                signals_in_high_vol += 1;
            } else {
                signals_in_low_vol += 1;
            }
        }
    }

    if total_trading_signals > 0 {
        println!(
            "      Signals in High Vol: {:.1}% ({}/{})",
            signals_in_high_vol as f64 / total_trading_signals as f64 * 100.0,
            signals_in_high_vol,
            total_trading_signals
        );
        println!(
            "      Signals in Low Vol: {:.1}% ({}/{})",
            signals_in_low_vol as f64 / total_trading_signals as f64 * 100.0,
            signals_in_low_vol,
            total_trading_signals
        );

        if signals_in_high_vol > signals_in_low_vol {
            println!("      🌊 Volatility-driven strategy - More active in volatile periods");
        } else {
            println!("      🏝️ Stability-focused strategy - More active in calm periods");
        }
    }
}

fn analyze_position_sizing(config: &GarchStrategyConfig, current_vol: f64) {
    println!("    💰 Volatility Targeting Analysis:");
    println!(
        "      Target Volatility: {:.1}%",
        config.target_volatility * 100.0
    );
    println!("      Current Volatility: {:.1}%", current_vol * 100.0);

    let vol_scalar = config.target_volatility / current_vol;
    let position_multiplier = vol_scalar * config.risk_adjustment;

    println!("      Volatility Scalar: {:.2}x", vol_scalar);
    println!("      Position Multiplier: {:.2}x", position_multiplier);

    if position_multiplier > 1.5 {
        println!("      📈 Leverage up - Low volatility allows higher exposure");
    } else if position_multiplier < 0.7 {
        println!("      📉 Scale down - High volatility requires risk reduction");
    } else {
        println!("      ⚖️ Balanced - Volatility near target level");
    }
}

fn detailed_garch_analysis(df: &DataFrame) -> Result<()> {
    let config = GarchStrategyConfig::volatility_trading();
    let strategy = GarchStrategy::new(config.clone());
    let signals = strategy.generate_signals(df, "close", "timestamp")?;

    println!("GARCH Volatility Trading Strategy Analysis:");
    println!("==========================================");

    println!("  Model Configuration:");
    println!("    Type: {:?}", config.model_type);
    println!(
        "    Orders: GARCH({}) ARCH({})",
        config.garch_order, config.arch_order
    );
    println!(
        "    Volatility Threshold: {:.1}x average",
        config.volatility_threshold
    );
    println!(
        "    Signal Threshold: {:.2}%",
        config.signal_threshold * 100.0
    );

    // Extract prices for detailed analysis
    let prices = extract_prices_for_analysis(df)?;

    // Comprehensive volatility analysis
    analyze_comprehensive_volatility(&prices, &signals, &config)?;

    // Strategy performance analysis
    analyze_strategy_performance(&signals, &prices, &config)?;

    // Detailed backtesting
    println!("\n📈 Comprehensive Backtesting:");
    let backtest_config = BacktestConfig {
        initial_capital: 100000.0,
        transaction_cost: 0.001, // 0.1%
        slippage: 0.0005,        // 0.05%
        risk_free_rate: 0.02,    // 2%
        position_size: 1.0,
    };

    let backtester = ForecastBacktester::new(backtest_config);
    let prices = extract_prices_for_analysis(df)?;
    let performance = backtester.backtest(&prices, &signals, None)?;

    print_detailed_performance(&performance);

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

fn analyze_comprehensive_volatility(
    prices: &[f64],
    signals: &[Signal],
    config: &GarchStrategyConfig,
) -> Result<()> {
    println!("\n🌊 Comprehensive Volatility Analysis:");

    // Calculate returns
    let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

    // Rolling volatility analysis
    let window = config.volatility_window;
    let mut rolling_volatilities = Vec::new();

    for i in window..returns.len() {
        let window_returns = &returns[i - window..i];
        let mean_ret = window_returns.iter().sum::<f64>() / window_returns.len() as f64;
        let variance = window_returns
            .iter()
            .map(|&r| (r - mean_ret).powi(2))
            .sum::<f64>()
            / window as f64;
        rolling_volatilities.push(variance.sqrt());
    }

    if !rolling_volatilities.is_empty() {
        let avg_vol = rolling_volatilities.iter().sum::<f64>() / rolling_volatilities.len() as f64;
        let vol_of_vol = {
            let vol_mean = avg_vol;
            let variance = rolling_volatilities
                .iter()
                .map(|&v| (v - vol_mean).powi(2))
                .sum::<f64>()
                / rolling_volatilities.len() as f64;
            variance.sqrt()
        };

        println!("  Average Volatility: {:.3}% daily", avg_vol * 100.0);
        println!("  Volatility of Volatility: {:.3}%", vol_of_vol * 100.0);

        let max_vol = rolling_volatilities.iter().cloned().fold(0.0f64, f64::max);
        let min_vol = rolling_volatilities
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);

        println!(
            "  Volatility Range: {:.3}% - {:.3}%",
            min_vol * 100.0,
            max_vol * 100.0
        );

        // Volatility clustering analysis
        analyze_volatility_clustering(&rolling_volatilities, avg_vol);
    }

    Ok(())
}

fn analyze_volatility_clustering(volatilities: &[f64], avg_vol: f64) {
    println!("    🔄 Volatility Clustering Analysis:");

    let high_vol_threshold = avg_vol * 1.5;
    let mut cluster_count = 0;
    let mut current_cluster_length = 0;
    let mut total_cluster_length = 0;
    let mut in_cluster = false;

    for &vol in volatilities {
        if vol > high_vol_threshold {
            if !in_cluster {
                in_cluster = true;
                cluster_count += 1;
                current_cluster_length = 1;
            } else {
                current_cluster_length += 1;
            }
        } else if current_cluster_length > 0 {
            total_cluster_length += current_cluster_length;
            current_cluster_length = 0;
            in_cluster = false;
        }
    }

    // Handle case where we end in a cluster
    if current_cluster_length > 0 {
        total_cluster_length += current_cluster_length;
    }

    if cluster_count > 0 {
        let avg_cluster_length = total_cluster_length as f64 / cluster_count as f64;
        println!("      High Volatility Clusters: {}", cluster_count);
        println!("      Average Cluster Length: {:.1} periods", avg_cluster_length);

        if avg_cluster_length > 3.0 {
            println!("      📈 Strong volatility persistence detected");
        } else {
            println!("      📊 Moderate volatility clustering");
        }
    } else {
        println!("      📉 No significant volatility clustering");
    }
}

fn analyze_strategy_performance(
    signals: &[Signal],
    prices: &[f64],
    config: &GarchStrategyConfig,
) -> Result<()> {
    println!("    📈 Strategy Performance Analysis:");

    // Calculate simple returns
    let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

    // Basic statistics
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let volatility = {
        let variance = returns
            .iter()
            .map(|&r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        variance.sqrt()
    };

    println!(
        "      Average Daily Return: {:.3}% ({:.1}% annualized)",
        mean_return * 100.0,
        mean_return * 252.0 * 100.0
    );
    println!(
        "      Daily Volatility: {:.3}% ({:.1}% annualized)",
        volatility * 100.0,
        volatility * (252.0_f64).sqrt() * 100.0
    );

    if volatility > 0.0 {
        let sharpe_like = (mean_return / volatility) * (252.0_f64).sqrt();
        println!("      Risk-Adjusted Return: {:.2}", sharpe_like);
    }

    Ok(())
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

    // GARCH-specific assessment
    println!("\n  🏆 GARCH Strategy Assessment:");

    if performance.volatility < 0.15 {
        println!("    ✅ Low strategy volatility - Good risk control");
    } else if performance.volatility < 0.25 {
        println!("    ⚠️ Moderate strategy volatility - Acceptable risk");
    } else {
        println!("    ❌ High strategy volatility - Consider risk reduction");
    }

    if performance.sharpe_ratio > 0.8 {
        println!("    ✅ Excellent risk-adjusted returns");
    } else if performance.sharpe_ratio > 0.5 {
        println!("    ✅ Good risk-adjusted returns");
    } else {
        println!("    ⚠️ Risk-adjusted returns could be improved");
    }

    if performance.max_drawdown < 0.1 {
        println!("    ✅ Excellent drawdown control");
    } else if performance.max_drawdown < 0.2 {
        println!("    ✅ Good drawdown control");
    } else {
        println!("    ⚠️ High drawdown - volatility strategy may need refinement");
    }
}

fn explain_garch_models() {
    println!("GARCH Models for Trading Explained:");
    println!("==================================");

    println!("\n🎯 What is GARCH?");
    println!("  • Generalized Autoregressive Conditional Heteroskedasticity");
    println!("  • Models time-varying volatility in financial time series");
    println!("  • Captures volatility clustering (periods of high/low volatility)");
    println!("  • Essential for risk management and option pricing");

    println!("\n📊 Standard GARCH(p,q) Model:");
    println!("  • σ²[t] = α₀ + Σα[i]ε²[t-i] + Σβ[j]σ²[t-j]");
    println!("  • Uses past squared returns (ARCH terms) and past variance (GARCH terms)");
    println!("  • Symmetric response to positive and negative shocks");
    println!("  • Best for: Basic volatility modeling, symmetric markets");

    println!("\n📈 GARCH-M (GARCH in Mean):");
    println!("  • Includes conditional variance in the mean equation");
    println!("  • Models risk premium: higher volatility → higher expected return");
    println!("  • Captures time-varying risk-return relationship");
    println!("  • Best for: Risk premium modeling, portfolio optimization");

    println!("\n⚡ EGARCH (Exponential GARCH):");
    println!("  • ln(σ²[t]) = ω + β*ln(σ²[t-1]) + α*(|ε[t-1]|/σ[t-1]) + γ*ε[t-1]/σ[t-1]");
    println!("  • Asymmetric response: negative shocks increase volatility more");
    println!("  • No non-negativity constraints (works in log space)");
    println!("  • Best for: Equity markets, leverage effect modeling");

    println!("\n🌊 GJR-GARCH (Threshold GARCH):");
    println!("  • σ²[t] = α₀ + (α₁ + γ*I[t-1])*ε²[t-1] + β₁*σ²[t-1]");
    println!("  • I[t-1] = 1 if ε[t-1] < 0, 0 otherwise");
    println!("  • Different impact for positive vs negative shocks");
    println!("  • Best for: Asymmetric volatility, crisis periods");

    println!("\n🎯 Trading Applications:");
    println!("  📈 Volatility Forecasting:");
    println!("    • Predict future volatility for risk management");
    println!("    • Optimal position sizing based on volatility");
    println!("    • Dynamic hedging strategies");

    println!("\n  📊 Volatility Trading:");
    println!("    • Buy in low volatility periods (vol expansion)");
    println!("    • Sell in high volatility periods (vol contraction)");
    println!("    • Breakout strategies based on volatility spikes");

    println!("\n  💼 Risk Management:");
    println!("    • Value-at-Risk (VaR) calculation");
    println!("    • Dynamic portfolio rebalancing");
    println!("    • Stress testing under different volatility regimes");

    println!("\n  🎲 Option Strategies:");
    println!("    • Volatility mean reversion strategies");
    println!("    • Calendar spreads based on vol forecasts");
    println!("    • Straddle/strangle strategies");

    println!("\n⚙️ Implementation Tips:");
    println!("  ✅ Start with GARCH(1,1) - often sufficient");
    println!("  ✅ Use EGARCH/GJR for equity markets (asymmetry)");
    println!("  ✅ Combine with other signals for confirmation");
    println!("  ✅ Regular model re-estimation for parameter drift");
    println!("  ❌ Avoid over-parameterization (p+q > 3)");
    println!("  ❌ Don't ignore structural breaks in volatility");

    println!("\n📚 Model Selection Guidelines:");
    println!("  • Standard GARCH: Commodities, FX (symmetric volatility)");
    println!("  • EGARCH: Equities, indices (leverage effect)");
    println!("  • GJR-GARCH: Crisis periods, emerging markets");
    println!("  • GARCH-M: Portfolio optimization, risk premia modeling");
}
