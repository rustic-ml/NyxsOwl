//! Strategy Comparison Example
//!
//! This example demonstrates how to:
//! 1. Create multiple intraday trading strategies
//! 2. Generate signals for each strategy on the same dataset
//! 3. Compare their performance metrics
//! 4. Visualize the results

use minute_trade::utils::{calculate_detailed_performance, generate_minute_data};
use minute_trade::{
    IntradayStrategy, MinuteOhlcv, MomentumBreakoutStrategy, PerformanceMetrics, ScalpingStrategy,
    Signal, TradeError,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate test data with moderate volatility
    println!("Generating test data...");
    // 5 days, 390 minutes per day (6.5-hour trading day), starting price 100, volatility 0.02, slight uptrend 0.0001
    let data = generate_minute_data(5, 390, 100.0, 0.02, 0.0001);

    println!("Generated {} data points across {} days", data.len(), 5);

    // Create different strategies
    let strategies: Vec<Box<dyn IntradayStrategy>> = vec![
        Box::new(ScalpingStrategy::new(5, 0.1)?), // 5-minute, 0.1% threshold
        Box::new(ScalpingStrategy::new(3, 0.05)?), // 3-minute, 0.05% threshold
        Box::new(MomentumBreakoutStrategy::new(30, 1.5)?), // 30-minute, 1.5x volume
        Box::new(MomentumBreakoutStrategy::new(60, 2.0)?), // 60-minute, 2.0x volume
    ];

    // Store performance results for comparison
    let mut performance_results = Vec::new();
    let mut signal_distributions = HashMap::new();

    // Test each strategy
    println!("\nTesting strategies...");
    for strategy in &strategies {
        let name = strategy.name();
        println!("Running strategy: {}", name);

        // Generate signals for this strategy
        let signals = strategy.generate_signals(&data)?;

        // Calculate detailed performance metrics
        let performance = calculate_detailed_performance(&data, &signals, 10000.0, 0.05)?;
        performance_results.push((name.to_string(), performance));

        // Count signal distribution
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
        let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
        signal_distributions.insert(name.to_string(), (buy_count, sell_count, hold_count));
    }

    // Print performance comparison
    println!("\nPerformance Comparison:");
    println!("======================");
    println!(
        "{:<30} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Strategy", "Return %", "Ann. Ret %", "Sharpe", "DrawDown%", "Win Rate%", "# Trades"
    );
    println!("{}", "-".repeat(90));

    for (name, perf) in &performance_results {
        println!(
            "{:<30} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>10}",
            name,
            perf.total_return,
            perf.annualized_return,
            perf.sharpe_ratio,
            perf.max_drawdown,
            perf.win_rate,
            perf.total_trades
        );
    }

    // Print signal distribution
    println!("\nSignal Distribution:");
    println!("===================");
    println!(
        "{:<30} {:>10} {:>10} {:>10} {:>10}",
        "Strategy", "Buy", "Sell", "Hold", "Activity%"
    );
    println!("{}", "-".repeat(70));

    for (name, (buy, sell, hold)) in &signal_distributions {
        let total = buy + sell + hold;
        let activity_pct = if total > 0 {
            (buy + sell) as f64 / total as f64 * 100.0
        } else {
            0.0
        };

        println!(
            "{:<30} {:>10} {:>10} {:>10} {:>10.2}%",
            name, buy, sell, hold, activity_pct
        );
    }

    // Find the best performing strategy
    if let Some((best_name, best_perf)) = performance_results
        .iter()
        .max_by(|(_, a), (_, b)| a.total_return.partial_cmp(&b.total_return).unwrap())
    {
        println!(
            "\nBest Performing Strategy: {} with {:.2}% return",
            best_name, best_perf.total_return
        );
    }

    // Print comparative insights
    println!("\nComparative Insights:");
    println!("====================");

    // Compare higher vs. lower frequency strategies
    let scalping_3m = performance_results
        .iter()
        .find(|(name, _)| name.contains("3m"))
        .map(|(_, perf)| perf.total_return)
        .unwrap_or(0.0);
    let breakout_60m = performance_results
        .iter()
        .find(|(name, _)| name.contains("60m"))
        .map(|(_, perf)| perf.total_return)
        .unwrap_or(0.0);

    if scalping_3m > breakout_60m {
        println!("Higher frequency strategies performed better in this time period.");
    } else {
        println!("Lower frequency strategies performed better in this time period.");
    }

    // Trading frequency vs. profitability
    let mut trade_count_vs_return: Vec<(usize, f64)> = performance_results
        .iter()
        .map(|(_, perf)| (perf.total_trades, perf.total_return))
        .collect();

    trade_count_vs_return.sort_by_key(|(count, _)| *count);

    if trade_count_vs_return.first().unwrap().1 > trade_count_vs_return.last().unwrap().1 {
        println!("Strategies with fewer trades tended to perform better.");
    } else {
        println!("Strategies with more trades tended to perform better.");
    }

    Ok(())
}

/// Generate a hypothetical combined strategy
///
/// This function implements a simple "majority vote" approach where:
/// - If more strategies say Buy than Sell, we Buy
/// - If more strategies say Sell than Buy, we Sell
/// - Otherwise, we Hold
///
/// # Arguments
/// * `data` - OHLCV data points
/// * `strategy_signals` - Vector of signals from different strategies
///
/// # Returns
/// * `Result<PerformanceMetrics, TradeError>` - Performance metrics for the combined strategy
fn _create_combined_strategy(
    data: &[MinuteOhlcv],
    strategy_signals: &[Vec<Signal>],
) -> Result<PerformanceMetrics, TradeError> {
    let len = data.len();
    if strategy_signals.is_empty() {
        return Err(TradeError::InvalidData(
            "No strategy signals provided".to_string(),
        ));
    }

    // Ensure all signal vectors have the same length
    for signals in strategy_signals {
        if signals.len() != len {
            return Err(TradeError::InvalidData(
                "All strategy signals must have the same length as data".to_string(),
            ));
        }
    }

    let mut combined_signals = Vec::with_capacity(len);

    // For each data point, tally the signals from all strategies
    for i in 0..len {
        let mut buy_count = 0;
        let mut sell_count = 0;

        for signals in strategy_signals {
            match signals[i] {
                Signal::Buy => buy_count += 1,
                Signal::Sell => sell_count += 1,
                Signal::Hold => {} // No action
            }
        }

        // Determine the consensus signal
        let signal = if buy_count > sell_count {
            Signal::Buy
        } else if sell_count > buy_count {
            Signal::Sell
        } else {
            Signal::Hold
        };

        combined_signals.push(signal);
    }

    // Calculate performance metrics for the combined strategy
    calculate_detailed_performance(data, &combined_signals, 10000.0, 0.05)
}
