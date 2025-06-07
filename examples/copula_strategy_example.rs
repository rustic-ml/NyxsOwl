use nyxs_owl::forecasting::strategies::{CopulaStrategy, CopulaStrategyConfig, CopulaType, CopulaStrategyType};
use nyxs_owl::forecasting::backtest::{ForecastBacktester, BacktestConfig};
use nyxs_owl::simple_types::{Signal, Result};
use polars::prelude::*;
use std::env;

fn main() -> Result<()> {
    println!("===================================");
    println!("    Copula Strategy Example");
    println!("===================================");
    
    // Load multi-asset data
    println!("Loading multi-asset data...");
    let asset_data = load_multi_asset_data()?;
    
    for (asset, df) in &asset_data {
        println!("  {}: {} data points", asset, df.height());
    }
    
    // Create combined DataFrame for analysis
    let combined_df = create_combined_dataframe(&asset_data)?;
    println!("Combined dataset: {} rows\n", combined_df.height());
    
    // Test different copula types
    println!("🔬 Testing Copula Types");
    println!("=======================");
    
    test_copula_type("Gaussian Copula", CopulaType::Gaussian, &combined_df)?;
    test_copula_type("Student-t Copula", CopulaType::StudentT(5.0), &combined_df)?;
    test_copula_type("Clayton Copula", CopulaType::Clayton(2.0), &combined_df)?;
    test_copula_type("Gumbel Copula", CopulaType::Gumbel(2.0), &combined_df)?;
    
    // Test different strategy types
    println!("\n🎛️ Testing Strategy Types");
    println!("=========================");
    
    test_strategy_type("Pairs Trading", CopulaStrategyConfig::pairs_trading("AAPL", "MSFT"), &combined_df)?;
    test_strategy_type("Statistical Arbitrage", 
        CopulaStrategyConfig::statistical_arbitrage(vec![
            ("AAPL".to_string(), "MSFT".to_string()),
            ("GOOGL".to_string(), "AMZN".to_string()),
        ]), &combined_df)?;
    test_strategy_type("Portfolio Optimization", 
        CopulaStrategyConfig::portfolio_optimization(vec![
            "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string()
        ]), &combined_df)?;
    
    // Detailed analysis
    println!("\n📊 Detailed Copula Analysis");
    println!("===========================");
    
    detailed_copula_analysis(&combined_df)?;
    
    // Educational content
    println!("\n📚 Copula Models Explained");
    println!("==========================");
    
    explain_copula_models();
    
    Ok(())
}

fn load_multi_asset_data() -> Result<Vec<(String, DataFrame)>> {
    let assets = vec!["AAPL", "MSFT", "GOOGL", "AMZN"];
    let mut asset_data = Vec::new();
    
    for asset in assets {
        let file_path = format!("examples/csv/{}_daily_ohlcv.csv", asset);
        
        match load_single_asset_data(&file_path) {
            Ok(df) => {
                asset_data.push((asset.to_string(), df));
            },
            Err(_) => {
                // Try alternative naming
                let alt_path = format!("examples/csv/{}_daily.csv", asset.to_lowercase());
                match load_single_asset_data(&alt_path) {
                    Ok(df) => {
                        asset_data.push((asset.to_string(), df));
                    },
                    Err(_) => {
                        println!("Warning: Could not load data for {}, skipping", asset);
                    }
                }
            }
        }
    }
    
    if asset_data.is_empty() {
        // Fallback to default data
        let default_file = env::var("OHLCV_FILE")
            .unwrap_or_else(|_| "examples/csv/AAPL_daily_ohlcv.csv".to_string());
        let df = load_single_asset_data(&default_file)?;
        asset_data.push(("DEFAULT".to_string(), df));
    }
    
    Ok(asset_data)
}

fn load_single_asset_data(file_path: &str) -> Result<DataFrame> {
    let df = LazyFrame::scan_csv(file_path, ScanArgsCSV::default())
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to load CSV: {}", e)))?
        .collect()
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to collect data: {}", e)))?;
    
    // Ensure we have required columns
    let required_columns = ["close", "timestamp"];
    for col in required_columns.iter() {
        if df.column(col).is_err() {
            return Err(nyxs_owl::simple_types::NyxsOwlError::DataError(
                format!("Required column '{}' not found in {}", col, file_path)
            ));
        }
    }
    
    Ok(df)
}

fn create_combined_dataframe(asset_data: &[(String, DataFrame)]) -> Result<DataFrame> {
    if asset_data.is_empty() {
        return Err(nyxs_owl::simple_types::NyxsOwlError::DataError(
            "No asset data available".to_string()
        ));
    }
    
    // Start with the first asset's timestamp
    let mut combined = asset_data[0].1.clone();
    
    // Add price columns for each asset
    for (asset, df) in asset_data.iter() {
        let close_column = df.column("close")
            .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to get close column: {}", e)))?;
        
        let new_column_name = format!("{}_close", asset);
        combined = combined.with_column(
            close_column.clone().alias(&new_column_name)
        ).map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to add column: {}", e)))?;
    }
    
    Ok(combined)
}

fn test_copula_type(name: &str, copula_type: CopulaType, df: &DataFrame) -> Result<()> {
    println!("\n📈 {} Analysis:", name);
    
    explain_copula_type(&copula_type);
    
    // Get available price columns
    let price_columns = get_price_columns(df);
    if price_columns.len() < 2 {
        println!("  ⚠️ Not enough price columns for copula analysis");
        return Ok(());
    }
    
    let asset_pairs = vec![(price_columns[0].clone(), price_columns[1].clone())];
    
    let config = CopulaStrategyConfig {
        copula_type,
        strategy_type: CopulaStrategyType::PairsTrading,
        asset_pairs,
        lookback_window: 60,
        correlation_threshold: 0.7,
        signal_threshold: 0.02,
        min_data_points: 100,
        rolling_window: 30,
        confidence_level: 0.95,
        risk_adjustment: 1.0,
    };
    
    let strategy = CopulaStrategy::new(config);
    let signals = strategy.generate_signals(df, &price_columns, "timestamp")?;
    
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
    
    Ok(())
}

fn test_strategy_type(name: &str, config: CopulaStrategyConfig, df: &DataFrame) -> Result<()> {
    println!("\n📈 {} Strategy:", name);
    
    explain_strategy_type(&config.strategy_type);
    
    // Get available price columns
    let price_columns = get_price_columns(df);
    if price_columns.len() < 2 {
        println!("  ⚠️ Not enough price columns for strategy analysis");
        return Ok(());
    }
    
    println!("  Configuration:");
    println!("    Copula Type: {:?}", config.copula_type);
    println!("    Asset Pairs: {}", config.asset_pairs.len());
    println!("    Correlation Threshold: {:.2}", config.correlation_threshold);
    println!("    Signal Threshold: {:.3}", config.signal_threshold);
    println!("    Lookback Window: {}", config.lookback_window);
    
    let strategy = CopulaStrategy::new(config);
    let signals = strategy.generate_signals(df, &price_columns, "timestamp")?;
    
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
    
    analyze_copula_characteristics(&config, &signals);
    
    Ok(())
}

fn get_price_columns(df: &DataFrame) -> Vec<String> {
    df.get_column_names()
        .iter()
        .filter(|name| name.ends_with("_close") || **name == "close")
        .map(|&name| name.to_string())
        .collect()
}

fn explain_copula_type(copula_type: &CopulaType) {
    match copula_type {
        CopulaType::Gaussian => {
            println!("  🔄 Gaussian Copula:");
            println!("    • Models linear correlation between assets");
            println!("    • Symmetric dependence structure");
            println!("    • Good for normal market conditions");
        },
        CopulaType::StudentT(df) => {
            println!("  📊 Student-t Copula (df={:.1}):", df);
            println!("    • Captures tail dependence");
            println!("    • Heavier tails than Gaussian");
            println!("    • Good for crisis periods and extreme events");
        },
        CopulaType::Clayton(alpha) => {
            println!("  📉 Clayton Copula (α={:.1}):", alpha);
            println!("    • Strong lower tail dependence");
            println!("    • Weak upper tail dependence");
            println!("    • Good for downside risk modeling");
        },
        CopulaType::Gumbel(alpha) => {
            println!("  📈 Gumbel Copula (α={:.1}):", alpha);
            println!("    • Strong upper tail dependence");
            println!("    • Weak lower tail dependence");
            println!("    • Good for modeling bull market extremes");
        },
        CopulaType::Frank(alpha) => {
            println!("  ⚖️ Frank Copula (α={:.1}):", alpha);
            println!("    • Symmetric dependence");
            println!("    • No tail dependence");
            println!("    • Good for moderate dependencies");
        },
    }
}

fn explain_strategy_type(strategy_type: &CopulaStrategyType) {
    match strategy_type {
        CopulaStrategyType::PairsTrading => {
            println!("  🔄 Pairs Trading Strategy:");
            println!("    • Trades based on correlation deviations");
            println!("    • Long/short positions in correlated assets");
            println!("    • Mean-reverting spread trading");
        },
        CopulaStrategyType::StatisticalArbitrage => {
            println!("  📊 Statistical Arbitrage Strategy:");
            println!("    • Multi-asset correlation breakdowns");
            println!("    • Complex arbitrage opportunities");
            println!("    • Risk management through diversification");
        },
        CopulaStrategyType::PortfolioOptimization => {
            println!("  💼 Portfolio Optimization Strategy:");
            println!("    • Dynamic asset allocation");
            println!("    • Correlation-based rebalancing");
            println!("    • Risk-adjusted portfolio construction");
        },
        CopulaStrategyType::RiskManagement => {
            println!("  🛡️ Risk Management Strategy:");
            println!("    • Tail dependency monitoring");
            println!("    • Crisis detection through correlations");
            println!("    • Defensive positioning");
        },
    }
}

fn analyze_signals(signals: &[Signal], strategy_name: &str) {
    let total_signals = signals.len();
    let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
    let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
    
    println!("  🎯 Signal Analysis:");
    println!("    Total Signals: {}", total_signals);
    println!("    Buy Signals: {} ({:.1}%)", buy_count, buy_count as f64 / total_signals as f64 * 100.0);
    println!("    Sell Signals: {} ({:.1}%)", sell_count, sell_count as f64 / total_signals as f64 * 100.0);
    println!("    Hold Signals: {} ({:.1}%)", hold_count, hold_count as f64 / total_signals as f64 * 100.0);
    
    // Calculate signal activity
    let activity_rate = (buy_count + sell_count) as f64 / total_signals as f64 * 100.0;
    println!("    Trading Activity: {:.1}%", activity_rate);
    
    // Copula-specific assessment
    match activity_rate {
        rate if rate < 3.0 => println!("    📊 Low Activity - Strong correlation persistence"),
        rate if rate < 10.0 => println!("    📈 Moderate Activity - Typical correlation variations"),
        rate if rate < 20.0 => println!("    🚀 Active - Dynamic correlation changes"),
        _ => println!("    ⚠️ Very Active - High correlation instability"),
    }
}

fn analyze_copula_characteristics(config: &CopulaStrategyConfig, signals: &[Signal]) {
    println!("  🔍 Copula Strategy Characteristics:");
    
    // Strategy assessment based on configuration
    match config.strategy_type {
        CopulaStrategyType::PairsTrading => {
            println!("    Strategy Focus: Pairs trading efficiency");
            println!("    Expected Behavior: Mean-reverting signals");
        },
        CopulaStrategyType::StatisticalArbitrage => {
            println!("    Strategy Focus: Multi-asset arbitrage");
            println!("    Expected Behavior: Correlation breakdown exploitation");
        },
        CopulaStrategyType::PortfolioOptimization => {
            println!("    Strategy Focus: Dynamic allocation");
            println!("    Expected Behavior: Rebalancing signals");
        },
        CopulaStrategyType::RiskManagement => {
            println!("    Strategy Focus: Risk reduction");
            println!("    Expected Behavior: Defensive positioning");
        },
    }
    
    // Copula type assessment
    match config.copula_type {
        CopulaType::Gaussian => {
            println!("    Copula Strength: Linear correlation modeling");
        },
        CopulaType::StudentT(_) => {
            println!("    Copula Strength: Tail dependence and crisis modeling");
        },
        CopulaType::Clayton(_) => {
            println!("    Copula Strength: Downside risk management");
        },
        CopulaType::Gumbel(_) => {
            println!("    Copula Strength: Upside momentum capture");
        },
        CopulaType::Frank(_) => {
            println!("    Copula Strength: Symmetric dependence modeling");
        },
    }
    
    // Signal stability analysis
    let signal_changes = signals.windows(2)
        .filter(|window| window[0] != window[1])
        .count();
    
    let change_rate = signal_changes as f64 / signals.len() as f64 * 100.0;
    println!("    Signal Stability: {:.1}% change rate", change_rate);
    
    if change_rate < 5.0 {
        println!("    Signal Behavior: Very stable correlation patterns");
    } else if change_rate < 15.0 {
        println!("    Signal Behavior: Stable with occasional adjustments");
    } else {
        println!("    Signal Behavior: Dynamic correlation tracking");
    }
}

fn detailed_copula_analysis(df: &DataFrame) -> Result<()> {
    let config = CopulaStrategyConfig::pairs_trading("AAPL_close", "MSFT_close");
    let strategy = CopulaStrategy::new(config.clone());
    
    let price_columns = get_price_columns(df);
    if price_columns.len() < 2 {
        println!("⚠️ Not enough price columns for detailed analysis");
        return Ok(());
    }
    
    let signals = strategy.generate_signals(df, &price_columns, "timestamp")?;
    
    println!("Comprehensive Copula Analysis:");
    println!("=============================");
    
    println!("  Strategy Configuration:");
    println!("    Copula Type: {:?}", config.copula_type);
    println!("    Strategy Type: {:?}", config.strategy_type);
    println!("    Asset Pairs: {}", config.asset_pairs.len());
    println!("    Correlation Threshold: {:.2}", config.correlation_threshold);
    
    // Analyze correlation patterns
    analyze_correlation_patterns(df, &price_columns)?;
    
    // Analyze dependency structure
    analyze_dependency_structure(df, &price_columns)?;
    
    // Strategy performance analysis
    analyze_strategy_effectiveness(&signals, df, &price_columns)?;
    
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

fn analyze_correlation_patterns(df: &DataFrame, price_columns: &[String]) -> Result<()> {
    println!("\n📊 Correlation Patterns Analysis:");
    
    if price_columns.len() < 2 {
        println!("  ⚠️ Need at least 2 assets for correlation analysis");
        return Ok(());
    }
    
    // Extract prices for the first two assets
    let prices1 = extract_column_data(df, &price_columns[0])?;
    let prices2 = extract_column_data(df, &price_columns[1])?;
    
    // Calculate returns
    let returns1: Vec<f64> = prices1.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
    let returns2: Vec<f64> = prices2.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
    
    // Calculate overall correlation
    let correlation = calculate_correlation(&returns1, &returns2)?;
    println!("  Overall Correlation: {:.3}", correlation);
    
    // Rolling correlation analysis
    let window = 60;
    let mut rolling_correlations = Vec::new();
    
    for i in window..returns1.len() {
        let window_returns1 = &returns1[i-window..i];
        let window_returns2 = &returns2[i-window..i];
        
        if let Ok(corr) = calculate_correlation(window_returns1, window_returns2) {
            rolling_correlations.push(corr);
        }
    }
    
    if !rolling_correlations.is_empty() {
        let avg_rolling_corr = rolling_correlations.iter().sum::<f64>() / rolling_correlations.len() as f64;
        let max_corr = rolling_correlations.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_corr = rolling_correlations.iter().cloned().fold(f64::INFINITY, f64::min);
        
        println!("  Rolling Correlation Stats:");
        println!("    Average: {:.3}", avg_rolling_corr);
        println!("    Range: {:.3} to {:.3}", min_corr, max_corr);
        println!("    Stability: {:.3}", 1.0 - (max_corr - min_corr));
        
        // Correlation regime analysis
        analyze_correlation_regimes(&rolling_correlations);
    }
    
    Ok(())
}

fn calculate_correlation(x: &[f64], y: &[f64]) -> Result<f64> {
    if x.len() != y.len() || x.is_empty() {
        return Err(nyxs_owl::simple_types::NyxsOwlError::DataError(
            "Input vectors must have the same non-zero length".to_string()
        ));
    }
    
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    
    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;
    
    for (xi, yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        sum_xy += dx * dy;
        sum_x2 += dx * dx;
        sum_y2 += dy * dy;
    }
    
    let correlation = sum_xy / (sum_x2.sqrt() * sum_y2.sqrt());
    Ok(correlation)
}

fn analyze_correlation_regimes(correlations: &[f64]) {
    println!("    Correlation Regimes:");
    
    let high_corr_threshold = 0.7;
    let low_corr_threshold = 0.3;
    
    let high_corr_periods = correlations.iter().filter(|&&c| c > high_corr_threshold).count();
    let low_corr_periods = correlations.iter().filter(|&&c| c.abs() < low_corr_threshold).count();
    let medium_corr_periods = correlations.len() - high_corr_periods - low_corr_periods;
    
    println!("      High Correlation (>{:.1}): {:.1}%", high_corr_threshold, 
             high_corr_periods as f64 / correlations.len() as f64 * 100.0);
    println!("      Medium Correlation: {:.1}%", 
             medium_corr_periods as f64 / correlations.len() as f64 * 100.0);
    println!("      Low Correlation (<{:.1}): {:.1}%", low_corr_threshold,
             low_corr_periods as f64 / correlations.len() as f64 * 100.0);
    
    // Regime persistence
    let mut regime_changes = 0;
    let mut current_regime = if correlations[0] > high_corr_threshold {
        "High"
    } else if correlations[0].abs() < low_corr_threshold {
        "Low"
    } else {
        "Medium"
    };
    
    for &corr in correlations.iter().skip(1) {
        let new_regime = if corr > high_corr_threshold {
            "High"
        } else if corr.abs() < low_corr_threshold {
            "Low"
        } else {
            "Medium"
        };
        
        if new_regime != current_regime {
            regime_changes += 1;
            current_regime = new_regime;
        }
    }
    
    println!("      Regime Changes: {} ({:.1}%)", regime_changes, 
             regime_changes as f64 / correlations.len() as f64 * 100.0);
}

fn analyze_dependency_structure(df: &DataFrame, price_columns: &[String]) -> Result<()> {
    println!("\n🔗 Dependency Structure Analysis:");
    
    if price_columns.len() < 2 {
        return Ok(());
    }
    
    // Extract data for analysis
    let prices1 = extract_column_data(df, &price_columns[0])?;
    let prices2 = extract_column_data(df, &price_columns[1])?;
    
    // Calculate returns
    let returns1: Vec<f64> = prices1.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
    let returns2: Vec<f64> = prices2.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
    
    // Tail dependence analysis
    analyze_tail_dependence(&returns1, &returns2);
    
    // Rank correlation analysis
    analyze_rank_correlation(&returns1, &returns2)?;
    
    Ok(())
}

fn analyze_tail_dependence(returns1: &[f64], returns2: &[f64]) {
    println!("  Tail Dependence Analysis:");
    
    // Convert to ranks for tail analysis
    let ranks1 = convert_to_ranks(returns1);
    let ranks2 = convert_to_ranks(returns2);
    
    // Lower tail dependence (bottom 10%)
    let threshold = 0.1;
    let mut lower_tail_count = 0;
    let mut lower_tail_joint = 0;
    
    for (r1, r2) in ranks1.iter().zip(ranks2.iter()) {
        if *r1 <= threshold {
            lower_tail_count += 1;
            if *r2 <= threshold {
                lower_tail_joint += 1;
            }
        }
    }
    
    let lower_tail_dep = if lower_tail_count > 0 {
        lower_tail_joint as f64 / lower_tail_count as f64
    } else {
        0.0
    };
    
    // Upper tail dependence (top 10%)
    let upper_threshold = 0.9;
    let mut upper_tail_count = 0;
    let mut upper_tail_joint = 0;
    
    for (r1, r2) in ranks1.iter().zip(ranks2.iter()) {
        if *r1 >= upper_threshold {
            upper_tail_count += 1;
            if *r2 >= upper_threshold {
                upper_tail_joint += 1;
            }
        }
    }
    
    let upper_tail_dep = if upper_tail_count > 0 {
        upper_tail_joint as f64 / upper_tail_count as f64
    } else {
        0.0
    };
    
    println!("    Lower Tail Dependence: {:.3}", lower_tail_dep);
    println!("    Upper Tail Dependence: {:.3}", upper_tail_dep);
    
    // Interpretation
    if lower_tail_dep > 0.3 {
        println!("    🔴 Strong downside contagion risk");
    } else if lower_tail_dep > 0.1 {
        println!("    🟡 Moderate downside dependence");
    } else {
        println!("    🟢 Limited downside contagion");
    }
    
    if upper_tail_dep > 0.3 {
        println!("    🔵 Strong upside momentum coupling");
    } else if upper_tail_dep > 0.1 {
        println!("    🟡 Moderate upside dependence");
    } else {
        println!("    🟢 Independent upside movements");
    }
}

fn convert_to_ranks(data: &[f64]) -> Vec<f64> {
    let mut indexed_data: Vec<(usize, f64)> = data.iter().enumerate().map(|(i, &x)| (i, x)).collect();
    indexed_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    let mut ranks = vec![0.0; data.len()];
    for (rank, (original_index, _)) in indexed_data.iter().enumerate() {
        ranks[*original_index] = (rank + 1) as f64 / data.len() as f64;
    }
    
    ranks
}

fn analyze_rank_correlation(returns1: &[f64], returns2: &[f64]) -> Result<()> {
    println!("  Rank-Based Correlation:");
    
    let ranks1 = convert_to_ranks(returns1);
    let ranks2 = convert_to_ranks(returns2);
    
    let spearman_corr = calculate_correlation(&ranks1, &ranks2)?;
    println!("    Spearman Correlation: {:.3}", spearman_corr);
    
    // Compare with Pearson correlation
    let pearson_corr = calculate_correlation(returns1, returns2)?;
    println!("    Pearson Correlation: {:.3}", pearson_corr);
    
    let difference = (spearman_corr - pearson_corr).abs();
    println!("    Difference: {:.3}", difference);
    
    if difference > 0.1 {
        println!("    📊 Significant non-linear dependence detected");
    } else {
        println!("    📈 Primarily linear dependence structure");
    }
    
    Ok(())
}

fn analyze_strategy_effectiveness(signals: &[Signal], df: &DataFrame, price_columns: &[String]) -> Result<()> {
    println!("\n📈 Strategy Effectiveness Analysis:");
    
    // Signal timing analysis
    analyze_signal_timing_effectiveness(signals, df, price_columns)?;
    
    // Risk-return profile
    analyze_copula_risk_return(signals, df, price_columns)?;
    
    Ok(())
}

fn analyze_signal_timing_effectiveness(signals: &[Signal], df: &DataFrame, price_columns: &[String]) -> Result<()> {
    if price_columns.is_empty() {
        return Ok(());
    }
    
    let prices = extract_column_data(df, &price_columns[0])?;
    let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
    
    println!("  Signal Timing Effectiveness:");
    
    let mut correct_signals = 0;
    let mut total_trading_signals = 0;
    let forecast_horizon = 5; // Look 5 periods ahead
    
    for (i, &signal) in signals.iter().enumerate() {
        if signal != Signal::Hold && i + forecast_horizon < returns.len() {
            total_trading_signals += 1;
            let future_return: f64 = returns[i..i+forecast_horizon].iter().sum();
            
            let correct = match signal {
                Signal::Buy => future_return > 0.0,
                Signal::Sell => future_return < 0.0,
                Signal::Hold => true,
            };
            
            if correct {
                correct_signals += 1;
            }
        }
    }
    
    if total_trading_signals > 0 {
        let accuracy = correct_signals as f64 / total_trading_signals as f64 * 100.0;
        println!("    Forecast Accuracy: {:.1}% ({}/{})", accuracy, correct_signals, total_trading_signals);
        
        if accuracy > 60.0 {
            println!("    ✅ Excellent copula signal accuracy");
        } else if accuracy > 50.0 {
            println!("    ✅ Good copula signal accuracy");
        } else {
            println!("    ⚠️ Copula signals need improvement");
        }
    }
    
    Ok(())
}

fn analyze_copula_risk_return(signals: &[Signal], df: &DataFrame, price_columns: &[String]) -> Result<()> {
    if price_columns.is_empty() {
        return Ok(());
    }
    
    println!("  Risk-Return Profile:");
    
    let prices = extract_column_data(df, &price_columns[0])?;
    let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
    
    let mut strategy_returns = Vec::new();
    let mut position = 0.0; // 0 = no position, 1 = long, -1 = short
    
    for (i, &signal) in signals.iter().enumerate() {
        match signal {
            Signal::Buy => position = 1.0,
            Signal::Sell => position = -1.0,
            Signal::Hold => {}, // Keep current position
        }
        
        if i > 0 && i <= returns.len() {
            let strategy_return = position * returns[i-1];
            strategy_returns.push(strategy_return);
        }
    }
    
    if !strategy_returns.is_empty() {
        let avg_return = strategy_returns.iter().sum::<f64>() / strategy_returns.len() as f64;
        let volatility = {
            let variance = strategy_returns.iter()
                .map(|&r| (r - avg_return).powi(2))
                .sum::<f64>() / strategy_returns.len() as f64;
            variance.sqrt()
        };
        
        println!("    Average Return: {:.4}% daily", avg_return * 100.0);
        println!("    Strategy Volatility: {:.3}% daily", volatility * 100.0);
        
        if volatility > 0.0 {
            let sharpe_ratio = avg_return / volatility * (252.0_f64).sqrt();
            println!("    Sharpe Ratio: {:.3}", sharpe_ratio);
        }
    }
    
    Ok(())
}

fn extract_column_data(df: &DataFrame, column_name: &str) -> Result<Vec<f64>> {
    let column = df.column(column_name)
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to get column {}: {}", column_name, e)))?;
    
    let data: Vec<f64> = column
        .f64()
        .map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to convert to f64: {}", e)))?
        .into_iter()
        .collect::<Option<Vec<f64>>>()
        .ok_or_else(|| nyxs_owl::simple_types::NyxsOwlError::DataError("Column contains null values".to_string()))?;
        
    Ok(data)
}

fn print_detailed_performance(performance: &nyxs_owl::forecasting::backtest::BacktestPerformance) {
    println!("  💰 Return Analysis:");
    println!("    Total Return: {:.2}%", performance.total_return * 100.0);
    println!("    Annualized Return: {:.2}%", performance.annualized_return * 100.0);
    println!("    Benchmark (B&H): {:.2}%", performance.benchmark_return * 100.0);
    println!("    Alpha: {:.2}%", (performance.total_return - performance.benchmark_return) * 100.0);
    
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
    println!("    Avg Trade Return: {:.3}%", performance.avg_trade_return * 100.0);
    
    // Copula-specific assessment
    println!("\n  🏆 Copula Strategy Assessment:");
    
    if performance.total_trades > 50 {
        println!("    ✅ Active correlation-based trading");
    } else if performance.total_trades > 20 {
        println!("    ✅ Moderate trading activity");
    } else {
        println!("    ⚠️ Low trading activity - consider parameter adjustment");
    }
    
    if performance.sharpe_ratio > 0.8 {
        println!("    ✅ Excellent risk-adjusted returns from copula signals");
    } else if performance.sharpe_ratio > 0.5 {
        println!("    ✅ Good dependency structure exploitation");
    } else {
        println!("    ⚠️ Copula model may need refinement");
    }
}

fn explain_copula_models() {
    println!("Copula Models for Trading Explained:");
    println!("===================================");
    
    println!("\n🎯 What are Copulas?");
    println!("  • Mathematical functions that link marginal distributions to joint distribution");
    println!("  • Separate dependency structure from individual asset characteristics");
    println!("  • Model how assets move together, not just their individual movements");
    println!("  • Essential for portfolio risk management and multi-asset strategies");
    
    println!("\n📊 Types of Copulas:");
    
    println!("\n  🔄 Gaussian Copula:");
    println!("    • Based on multivariate normal distribution");
    println!("    • Parameter: correlation matrix");
    println!("    • Symmetric dependence, no tail dependence");
    println!("    • Best for: Normal market conditions, linear relationships");
    
    println!("\n  📈 Student-t Copula:");
    println!("    • Based on multivariate t-distribution");
    println!("    • Parameters: correlation matrix + degrees of freedom");
    println!("    • Symmetric tail dependence");
    println!("    • Best for: Crisis periods, extreme market events");
    
    println!("\n  📉 Clayton Copula:");
    println!("    • C(u,v) = (u⁻ᵅ + v⁻ᵅ - 1)⁻¹/ᵅ");
    println!("    • Parameter: α > 0 (higher α = stronger dependence)");
    println!("    • Strong lower tail dependence, weak upper tail");
    println!("    • Best for: Downside risk modeling, bear markets");
    
    println!("\n  🚀 Gumbel Copula:");
    println!("    • C(u,v) = exp(-[(-ln u)ᵅ + (-ln v)ᵅ]¹/ᵅ)");
    println!("    • Parameter: α ≥ 1 (higher α = stronger dependence)");
    println!("    • Strong upper tail dependence, weak lower tail");
    println!("    • Best for: Bull market modeling, momentum strategies");
    
    println!("\n  ⚖️ Frank Copula:");
    println!("    • C(u,v) = -α⁻¹ ln(1 + (e⁻ᵅᵘ-1)(e⁻ᵅᵛ-1)/(e⁻ᵅ-1))");
    println!("    • Parameter: α ∈ ℝ (α=0 → independence)");
    println!("    • Symmetric dependence, no tail dependence");
    println!("    • Best for: Moderate dependencies, stable relationships");
    
    println!("\n🎯 Trading Applications:");
    
    println!("\n  📈 Pairs Trading:");
    println!("    • Model the joint distribution of two assets");
    println!("    • Trade when actual relationship deviates from copula prediction");
    println!("    • Long asset with low conditional probability, short the other");
    
    println!("\n  📊 Statistical Arbitrage:");
    println!("    • Multi-asset copula models for complex relationships");
    println!("    • Identify mispricing across multiple correlated instruments");
    println!("    • Risk management through diversified arbitrage portfolio");
    
    println!("\n  💼 Portfolio Optimization:");
    println!("    • Dynamic correlation matrix from copula forecasts");
    println!("    • Optimize portfolio weights based on changing dependencies");
    println!("    • Stress testing under different dependency scenarios");
    
    println!("\n  🛡️ Risk Management:");
    println!("    • Tail dependency analysis for extreme risk assessment");
    println!("    • Crisis contagion modeling");
    println!("    • Dynamic hedging based on dependency changes");
    
    println!("\n⚙️ Implementation Strategy:");
    
    println!("\n  1️⃣ Model Selection:");
    println!("    • Start with Gaussian for baseline analysis");
    println!("    • Use Student-t for crisis-prone markets");
    println!("    • Clayton for downside protection strategies");
    println!("    • Gumbel for momentum/trend following");
    
    println!("\n  2️⃣ Parameter Estimation:");
    println!("    • Rolling window estimation (60-250 observations)");
    println!("    • Maximum likelihood or method of moments");
    println!("    • Regular re-calibration for parameter drift");
    
    println!("\n  3️⃣ Signal Generation:");
    println!("    • Compare actual vs predicted conditional distributions");
    println!("    • Trade when probability deviations exceed threshold");
    println!("    • Combine with traditional technical indicators");
    
    println!("\n  4️⃣ Risk Control:");
    println!("    • Position sizing based on copula-implied correlation");
    println!("    • Stop losses when dependency structure breaks down");
    println!("    • Diversification across multiple copula strategies");
    
    println!("\n📚 Advanced Techniques:");
    println!("  • Time-varying copulas (regime-switching)");
    println!("  • Vine copulas for high-dimensional dependencies");
    println!("  • Copula-based VaR and CVaR calculation");
    println!("  • Machine learning for copula parameter prediction");
    
    println!("\n⚠️ Common Pitfalls:");
    println!("  ❌ Using wrong copula type for market regime");
    println!("  ❌ Ignoring parameter instability over time");
    println!("  ❌ Over-relying on historical dependency patterns");
    println!("  ❌ Neglecting transaction costs in high-frequency strategies");
} 