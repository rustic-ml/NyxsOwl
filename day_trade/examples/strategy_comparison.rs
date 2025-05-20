//! Strategy Comparison Example
//!
//! This example demonstrates how to:
//! 1. Create different types of trading strategies (buy, sell, and hold)
//! 2. Generate signals from each strategy
//! 3. Compare their performance across the same dataset
//! 4. Display summary statistics

use day_trade::utils::{calculate_basic_performance, generate_test_data};
use day_trade::{
    DailyOhlcv, GridTradingStrategy, MACrossover, MeanReversionStrategy, Signal, TradingStrategy,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate test data with medium volatility
    println!("Generating test data...");
    let data = generate_test_data(200, 100.0, 0.05);

    // Create different strategies
    let buy_strategy = MACrossover::new(10, 30)?; // Fast/slow MA crossover (buy-focused)
    let sell_strategy = MeanReversionStrategy::default(); // Mean reversion (sell-focused)
    let hold_strategy = GridTradingStrategy::default(); // Grid trading (hold/range-focused)

    // Generate signals for each strategy
    println!("Generating signals for different strategies...");
    let buy_signals = buy_strategy.generate_signals(&data)?;
    let sell_signals = sell_strategy.generate_signals(&data)?;
    let hold_signals = hold_strategy.generate_signals(&data)?;

    // Calculate performance for each strategy
    let buy_performance = buy_strategy.calculate_performance(&data, &buy_signals)?;
    let sell_performance = sell_strategy.calculate_performance(&data, &sell_signals)?;
    let hold_performance = hold_strategy.calculate_performance(&data, &hold_signals)?;

    // Display performance results
    println!("\nPerformance Results:");
    println!("--------------------");
    println!("Buy Strategy (MA Crossover): {:.2}%", buy_performance);
    println!("Sell Strategy (Mean Reversion): {:.2}%", sell_performance);
    println!("Hold Strategy (Grid Trading): {:.2}%", hold_performance);

    // Display signal distribution
    println!("\nSignal Distribution:");
    println!("-------------------");
    print_signal_distribution("Buy Strategy", &buy_signals);
    print_signal_distribution("Sell Strategy", &sell_signals);
    print_signal_distribution("Hold Strategy", &hold_signals);

    // Create a combined strategy (consensus approach)
    println!("\nCombined Strategy Analysis:");
    println!("---------------------------");
    analyze_combined_strategy(&data, &buy_signals, &sell_signals, &hold_signals)?;

    Ok(())
}

/// Print the distribution of signals for a strategy
fn print_signal_distribution(name: &str, signals: &[Signal]) {
    let mut counts = HashMap::new();
    for &signal in signals {
        *counts.entry(signal).or_insert(0) += 1;
    }

    let buy_count = *counts.get(&Signal::Buy).unwrap_or(&0);
    let sell_count = *counts.get(&Signal::Sell).unwrap_or(&0);
    let hold_count = *counts.get(&Signal::Hold).unwrap_or(&0);

    let total = signals.len() as f64;

    println!(
        "{}: Buy: {} ({:.1}%), Sell: {} ({:.1}%), Hold: {} ({:.1}%)",
        name,
        buy_count,
        buy_count as f64 / total * 100.0,
        sell_count,
        sell_count as f64 / total * 100.0,
        hold_count,
        hold_count as f64 / total * 100.0
    );
}

/// Create and analyze a combined strategy based on consensus
fn analyze_combined_strategy(
    data: &[DailyOhlcv],
    buy_signals: &[Signal],
    sell_signals: &[Signal],
    hold_signals: &[Signal],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create consensus signals where we require at least 2 strategies to agree
    let mut consensus_signals = Vec::with_capacity(data.len());

    for i in 0..data.len() {
        let signals = [buy_signals[i], sell_signals[i], hold_signals[i]];
        let buy_votes = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_votes = signals.iter().filter(|&&s| s == Signal::Sell).count();

        let consensus = if buy_votes >= 2 {
            Signal::Buy
        } else if sell_votes >= 2 {
            Signal::Sell
        } else {
            Signal::Hold
        };

        consensus_signals.push(consensus);
    }

    // Calculate performance of the consensus strategy
    let consensus_performance = calculate_basic_performance(data, &consensus_signals, 10000.0)?;

    println!(
        "Consensus Strategy Performance: {:.2}%",
        consensus_performance
    );
    print_signal_distribution("Consensus Strategy", &consensus_signals);

    Ok(())
}
