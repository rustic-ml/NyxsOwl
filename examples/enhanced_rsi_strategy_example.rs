//! Enhanced RSI Strategy Example
//!
//! This example demonstrates how to use the Enhanced RSI Strategy with NyxsOwl,
//! showcasing both basic usage and advanced configuration options.
//!
//! The Enhanced RSI Strategy provides:
//! - Dual RSI confirmation system
//! - Dynamic threshold adjustment based on market volatility
//! - Trend filtering to avoid counter-trend trades
//! - Configurable signal strength thresholds
//! - Real-time streaming updates support

use nyxs_owl::prelude::*;
use nyxs_owl::technical_strategies::momentum::{
    enhanced_rsi_signals, enhanced_rsi_signals_with_config, EnhancedRsiConfig, EnhancedRsiStrategy,
};
use nyxs_owl::technical_strategies::{Strategy, StrategyConfig, TechnicalStrategy};
use polars::prelude::*;
use std::error::Error;

fn main() -> std::result::Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    println!("=== Enhanced RSI Strategy Example ===\n");

    // Create sample market data with realistic price movements
    let sample_data = create_sample_data();
    println!(
        "📊 Created sample data with {} data points",
        sample_data.height()
    );

    // Example 1: Basic Enhanced RSI Strategy with default parameters
    println!("\n1️⃣ Basic Enhanced RSI Strategy (Default Parameters)");
    basic_enhanced_rsi_example(&sample_data)?;

    // Example 2: Enhanced RSI Strategy with custom parameters
    println!("\n2️⃣ Enhanced RSI Strategy with Custom Parameters");
    custom_parameters_example(&sample_data)?;

    // Example 3: Advanced configuration with trend filtering and dynamic thresholds
    println!("\n3️⃣ Advanced Enhanced RSI Configuration");
    advanced_configuration_example(&sample_data)?;

    // Example 4: Signal analysis and filtering
    println!("\n4️⃣ Signal Analysis and Filtering");
    signal_analysis_example(&sample_data)?;

    // Example 5: Real-world usage pattern
    println!("\n5️⃣ Real-World Usage Pattern");
    real_world_example(&sample_data)?;

    println!("\n✅ Enhanced RSI Strategy example completed successfully!");
    Ok(())
}

/// Creates realistic sample market data for demonstration
fn create_sample_data() -> DataFrame {
    let num_points = 200;
    let mut prices = Vec::with_capacity(num_points);
    let mut highs = Vec::with_capacity(num_points);
    let mut lows = Vec::with_capacity(num_points);
    let mut volumes = Vec::with_capacity(num_points);

    let mut base_price = 100.0;
    let mut trend = 0.05; // Base trend

    for i in 0..num_points {
        // Add some realistic price movement with trend and noise
        let noise = (i as f64 * 0.1).sin() * 2.0 + (i as f64 * 0.05).cos() * 1.5;
        let daily_change = trend + noise * 0.3;

        base_price *= 1.0 + daily_change / 100.0;

        // Occasionally reverse the trend to create interesting patterns
        if i % 50 == 0 && i > 0 {
            trend = -trend * 0.8;
        }

        let high = base_price * (1.0 + (i as f64 * 0.2).sin().abs() * 0.02);
        let low = base_price * (1.0 - (i as f64 * 0.3).cos().abs() * 0.02);
        let volume = 10000.0 + (i as f64 * 0.1).sin() * 5000.0;

        prices.push(base_price);
        highs.push(high);
        lows.push(low);
        volumes.push(volume.abs());
    }

    df! {
        "close" => prices,
        "high" => highs,
        "low" => lows,
        "volume" => volumes
    }
    .expect("Failed to create sample data")
}

/// Example 1: Basic Enhanced RSI Strategy with default parameters
fn basic_enhanced_rsi_example(data: &DataFrame) -> std::result::Result<(), Box<dyn Error>> {
    println!("   Using default parameters (RSI 14/21, thresholds 30/70)");

    // Simple usage with default parameters
    let signals = enhanced_rsi_signals(data)?;

    println!("   📈 Generated {} signals", signals.len());

    // Count signal types
    let buy_signals = signals.iter().filter(|s| s.signal == Signal::Buy).count();
    let sell_signals = signals.iter().filter(|s| s.signal == Signal::Sell).count();
    let hold_signals = signals.iter().filter(|s| s.signal == Signal::Hold).count();

    println!("   🟢 Buy signals: {}", buy_signals);
    println!("   🔴 Sell signals: {}", sell_signals);
    println!("   ⚪ Hold signals: {}", hold_signals);

    // Show first few signals with metadata
    println!("   📋 First 5 signals:");
    for (i, signal) in signals.iter().take(5).enumerate() {
        println!(
            "      {}: {:?} (strength: {:.2}, confidence: {:.2})",
            i, signal.signal, signal.strength, signal.confidence
        );
        if let Some(primary_rsi) = signal.metadata.get("primary_rsi") {
            println!("         Primary RSI: {:.2}", primary_rsi);
        }
    }

    Ok(())
}

/// Example 2: Enhanced RSI Strategy with custom parameters
fn custom_parameters_example(data: &DataFrame) -> std::result::Result<(), Box<dyn Error>> {
    println!("   Using custom parameters (RSI 10/20, thresholds 25/75)");

    // Custom parameters for more sensitive signals
    let signals = enhanced_rsi_signals_with_config(data, 10, 20, 25.0, 75.0)?;

    println!("   📈 Generated {} signals", signals.len());

    // Analyze signal strength distribution
    let high_confidence_signals = signals.iter().filter(|s| s.confidence > 0.7).count();

    let medium_confidence_signals = signals
        .iter()
        .filter(|s| s.confidence > 0.5 && s.confidence <= 0.7)
        .count();

    println!(
        "   🎯 High confidence signals (>0.7): {}",
        high_confidence_signals
    );
    println!(
        "   📊 Medium confidence signals (0.5-0.7): {}",
        medium_confidence_signals
    );

    Ok(())
}

/// Example 3: Advanced configuration with trend filtering and dynamic thresholds
fn advanced_configuration_example(data: &DataFrame) -> std::result::Result<(), Box<dyn Error>> {
    println!("   Using advanced configuration with trend filtering and dynamic thresholds");

    // Create advanced configuration
    let config = StrategyConfig::new()
        .with_parameter("primary_period", 14i64)
        .with_parameter("secondary_period", 28i64)
        .with_parameter("oversold_threshold", 25.0)
        .with_parameter("overbought_threshold", 75.0)
        .with_parameter("dynamic_thresholds", true)
        .with_parameter("min_signal_strength", 0.7)
        .with_parameter("trend_filtering", true)
        .with_parameter("trend_lookback", 30i64);

    let enhanced_config = EnhancedRsiConfig {
        primary_period: 14,
        secondary_period: 28,
        oversold_threshold: 25.0,
        overbought_threshold: 75.0,
        dynamic_thresholds: true,
        min_signal_strength: 0.7,
        trend_filtering: true,
        trend_lookback: 30,
    };

    let strategy = EnhancedRsiStrategy::with_enhanced_config(config, enhanced_config);

    // Validate parameters
    strategy.validate_parameters()?;
    println!("   ✅ Parameters validated successfully");

    // Generate enhanced signals
    let signals = strategy.generate_enhanced_signals(data)?;
    println!("   📈 Generated {} high-quality signals", signals.len());

    // Get indicator values for analysis
    let indicators = strategy.get_indicator_values(data)?;
    if let Some(primary_rsi) = indicators.get("primary_rsi") {
        println!("   📊 Primary RSI series length: {}", primary_rsi.len());
    }

    // Show signals with trend information
    println!("   🔍 Signals with trend analysis:");
    for (i, signal) in signals.iter().enumerate().take(3) {
        if signal.signal != Signal::Hold {
            println!("      Signal {}: {:?}", i, signal.signal);
            println!("         Confidence: {:.3}", signal.confidence);
            println!("         Strength: {:.3}", signal.strength);
            if let Some(trend) = signal.metadata.get("trend_strength") {
                println!("         Trend strength: {:.3}", trend);
            }
        }
    }

    Ok(())
}

/// Example 4: Signal analysis and filtering
fn signal_analysis_example(data: &DataFrame) -> std::result::Result<(), Box<dyn Error>> {
    println!("   Analyzing and filtering signals by quality");

    let strategy =
        EnhancedRsiStrategy::new(StrategyConfig::new().with_parameter("min_signal_strength", 0.5));

    let all_signals = strategy.generate_enhanced_signals(data)?;

    // Filter signals by different criteria
    let high_quality_signals: Vec<_> = all_signals.iter().filter(|s| s.confidence > 0.8).collect();

    let medium_quality_signals: Vec<_> = all_signals
        .iter()
        .filter(|s| s.confidence > 0.6 && s.confidence <= 0.8)
        .collect();

    let strong_signals: Vec<_> = all_signals.iter().filter(|s| s.strength > 0.7).collect();

    println!("   📊 Signal Quality Analysis:");
    println!("      Total signals: {}", all_signals.len());
    println!(
        "      High quality (confidence > 0.8): {}",
        high_quality_signals.len()
    );
    println!(
        "      Medium quality (confidence 0.6-0.8): {}",
        medium_quality_signals.len()
    );
    println!(
        "      Strong signals (strength > 0.7): {}",
        strong_signals.len()
    );

    // Analyze buy vs sell signal quality
    let quality_buy_signals = high_quality_signals
        .iter()
        .filter(|s| s.signal == Signal::Buy)
        .count();

    let quality_sell_signals = high_quality_signals
        .iter()
        .filter(|s| s.signal == Signal::Sell)
        .count();

    println!("   📈 High Quality Signal Breakdown:");
    println!("      Buy signals: {}", quality_buy_signals);
    println!("      Sell signals: {}", quality_sell_signals);

    Ok(())
}

/// Example 5: Real-world usage pattern
fn real_world_example(data: &DataFrame) -> std::result::Result<(), Box<dyn Error>> {
    println!("   Demonstrating real-world usage pattern");

    // Configuration suitable for day trading
    let day_trading_config = StrategyConfig::new()
        .with_parameter("primary_period", 9i64) // Faster RSI
        .with_parameter("secondary_period", 14i64) // Standard RSI for confirmation
        .with_parameter("oversold_threshold", 20.0) // More aggressive thresholds
        .with_parameter("overbought_threshold", 80.0)
        .with_parameter("dynamic_thresholds", true) // Adapt to volatility
        .with_parameter("min_signal_strength", 0.75) // High quality signals only
        .with_parameter("trend_filtering", true); // Avoid counter-trend trades

    let strategy = EnhancedRsiStrategy::new(day_trading_config);

    println!("   📋 Strategy Configuration:");
    println!("      Name: {}", strategy.name());
    println!("      Description: {}", strategy.description());
    println!("      Required columns: {:?}", strategy.required_columns());
    println!("      Minimum data points: {}", strategy.min_data_points());

    // Generate signals
    let signals = strategy.generate_enhanced_signals(data)?;

    // Simulate a simple trading scenario
    let mut position = 0; // 0 = no position, 1 = long, -1 = short
    let mut trades = 0;
    let mut last_signal_index = 0;

    println!("   💼 Simulated Trading Scenario:");

    for (i, signal) in signals.iter().enumerate() {
        if signal.signal != Signal::Hold && i > last_signal_index + 5 {
            // Avoid rapid trading
            match (position, signal.signal) {
                (0, Signal::Buy) => {
                    position = 1;
                    trades += 1;
                    println!(
                        "      Trade {}: OPEN LONG at index {} (confidence: {:.2})",
                        trades, i, signal.confidence
                    );
                    last_signal_index = i;
                }
                (0, Signal::Sell) => {
                    position = -1;
                    trades += 1;
                    println!(
                        "      Trade {}: OPEN SHORT at index {} (confidence: {:.2})",
                        trades, i, signal.confidence
                    );
                    last_signal_index = i;
                }
                (1, Signal::Sell) => {
                    position = 0;
                    println!(
                        "      Trade {}: CLOSE LONG at index {} (confidence: {:.2})",
                        trades, i, signal.confidence
                    );
                    last_signal_index = i;
                }
                (-1, Signal::Buy) => {
                    position = 0;
                    println!(
                        "      Trade {}: CLOSE SHORT at index {} (confidence: {:.2})",
                        trades, i, signal.confidence
                    );
                    last_signal_index = i;
                }
                _ => {} // No action needed
            }

            if trades >= 5 {
                // Limit output for readability
                break;
            }
        }
    }

    println!("      📊 Total simulated trades: {}", trades);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_data_creation() {
        let data = create_sample_data();
        assert!(data.height() > 0);
        assert!(data.get_column_names().contains(&"close"));
    }

    #[test]
    fn test_basic_enhanced_rsi_example() {
        let data = create_sample_data();
        let result = basic_enhanced_rsi_example(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_parameters_example() {
        let data = create_sample_data();
        let result = custom_parameters_example(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_advanced_configuration_example() {
        let data = create_sample_data();
        let result = advanced_configuration_example(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_analysis_example() {
        let data = create_sample_data();
        let result = signal_analysis_example(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_real_world_example() {
        let data = create_sample_data();
        let result = real_world_example(&data);
        assert!(result.is_ok());
    }
}
