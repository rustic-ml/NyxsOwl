//! Advanced Technical Indicators Example
//!
//! This example demonstrates the implementation of advanced technical indicators
//! specifically researched for day trading and short-term trading strategies.
//! 
//! Featured indicators:
//! - Williams %R: 71.7% win rate reliability (highest among oscillators)
//! - ATR (Average True Range): Essential for volatility measurement and position sizing
//!
//! These indicators were selected based on 2024 research into the most effective
//! technical indicators for day trading.

use nyxs_owl::technical_strategies::oscillators::{WilliamsRStrategy, WilliamsRConfig};
use nyxs_owl::technical_strategies::volatility::{ATRStrategy, ATRConfig};
use nyxs_owl::technical_strategies::{TechnicalStrategy, TechnicalSignal};
use nyxs_owl::simple_types::Signal;
use polars::prelude::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Technical Indicators for Day Trading");
    println!("================================================");
    
    // Create sample market data with realistic price movements
    let sample_data = create_realistic_market_data()?;
    println!("📊 Created sample market data with {} data points", sample_data.height());
    
    // Demonstrate Williams %R Strategy
    demonstrate_williams_r_strategy(&sample_data)?;
    
    // Demonstrate ATR Strategy  
    demonstrate_atr_strategy(&sample_data)?;
    
    // Demonstrate combined strategy approach
    demonstrate_combined_strategy(&sample_data)?;
    
    Ok(())
}

fn create_realistic_market_data() -> PolarsResult<DataFrame> {
    let num_points = 100;
    let mut prices = Vec::new();
    let mut highs = Vec::new();
    let mut lows = Vec::new();
    let mut volumes = Vec::new();
    
    // Start with base price
    let mut base_price = 100.0;
    
    for i in 0..num_points {
        // Simulate realistic price movements with trend and volatility
        let trend = 0.02 * (i as f64 / 20.0).sin(); // Trending component
        let volatility = 2.0 * ((i as f64 * 0.3).cos() + 1.0); // Changing volatility
        let random_factor = ((i * 17) % 100) as f64 / 100.0 - 0.5; // Pseudo-random
        
        let price_change = trend + volatility * random_factor * 0.1;
        base_price += price_change;
        
        // Create OHLC data
        let close = base_price;
        let high = close + volatility * 0.5 * (random_factor.abs() + 0.2);
        let low = close - volatility * 0.5 * (random_factor.abs() + 0.2);
        let volume = 10000.0 + 5000.0 * random_factor.abs();
        
        prices.push(close);
        highs.push(high);
        lows.push(low);
        volumes.push(volume);
    }
    
    // Create timestamps
    let dates: Vec<String> = (0..num_points)
        .map(|i| format!("2024-01-{:02} 09:{:02}:00", (i / 60) + 1, i % 60))
        .collect();
    
    df! {
        "timestamp" => dates,
        "high" => highs,
        "low" => lows,
        "close" => prices,
        "volume" => volumes,
    }
}

fn demonstrate_williams_r_strategy(data: &DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📈 Williams %R Oscillator Strategy");
    println!("==================================");
    println!("Research shows Williams %R has 71.7% win rate reliability");
    
    // Create Williams %R strategy with optimized parameters
    let config = WilliamsRConfig {
        period: 14,
        overbought_threshold: -20.0,
        oversold_threshold: -80.0,
        middle_threshold: -50.0,
    };
    
    let strategy = WilliamsRStrategy::new(config);
    
    // Validate parameters
    strategy.validate_parameters()?;
    println!("✅ Strategy parameters validated");
    
    // Generate signals
    let signals = strategy.generate_enhanced_signals(data)?;
    println!("📊 Generated {} signals", signals.len());
    
    // Get indicator values
    let indicators = strategy.get_indicator_values(data)?;
    if let Some(williams_r_series) = indicators.get("williams_r") {
        let williams_r_values = williams_r_series.f64()?;
        
        // Analyze signal distribution
        let buy_signals = signals.iter().filter(|s| s.signal == Signal::Buy).count();
        let sell_signals = signals.iter().filter(|s| s.signal == Signal::Sell).count();
        let hold_signals = signals.iter().filter(|s| s.signal == Signal::Hold).count();
        
        println!("📈 Buy signals: {}", buy_signals);
        println!("📉 Sell signals: {}", sell_signals);
        println!("⏸️  Hold signals: {}", hold_signals);
        
        // Show some example signals with Williams %R values
        println!("\n🔍 Sample Signals:");
        for (i, signal) in signals.iter().enumerate().take(20).skip(15) {
            if signal.signal != Signal::Hold {
                let wr_value = williams_r_values.get(i).unwrap_or(0.0);
                println!("  Index {}: {:?} (Williams %R: {:.2}, Strength: {:.2}, Confidence: {:.2})", 
                    i, signal.signal, wr_value, signal.strength, signal.confidence);
            }
        }
    }
    
    // Calculate performance metrics
    let performance = strategy.get_performance_metrics(data, &signals)?;
    println!("\n📊 Performance Metrics:");
    println!("  Total Return: {:.2}%", performance.total_return * 100.0);
    println!("  Win Rate: {:.2}%", performance.win_rate * 100.0);
    println!("  Sharpe Ratio: {:.2}", performance.sharpe_ratio);
    println!("  Total Trades: {}", performance.total_trades);
    
    Ok(())
}

fn demonstrate_atr_strategy(data: &DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 ATR (Average True Range) Strategy");
    println!("====================================");
    println!("Essential for volatility measurement and position sizing");
    
    // Create ATR strategy with day trading parameters
    let config = ATRConfig {
        period: 14,
        expansion_threshold: 1.5,
        contraction_threshold: 0.7,
        stop_loss_multiplier: 2.0,
        position_size_multiplier: 0.02,
    };
    
    let strategy = ATRStrategy::new(config);
    
    // Validate parameters
    strategy.validate_parameters()?;
    println!("✅ Strategy parameters validated");
    
    // Generate signals
    let signals = strategy.generate_enhanced_signals(data)?;
    println!("📊 Generated {} signals", signals.len());
    
    // Get indicator values
    let indicators = strategy.get_indicator_values(data)?;
    if let Some(atr_series) = indicators.get("atr") {
        let atr_values = atr_series.f64()?;
        
        // Analyze volatility patterns
        let avg_atr: f64 = atr_values.iter().filter(|&&x| x > 0.0).sum::<f64>() 
            / atr_values.iter().filter(|&&x| x > 0.0).count() as f64;
        
        println!("📈 Average ATR: {:.4}", avg_atr);
        
        // Show signals with ATR-based metadata
        println!("\n🔍 Sample ATR-based Signals:");
        for (i, signal) in signals.iter().enumerate().take(25).skip(20) {
            if signal.signal != Signal::Hold {
                let atr_value = atr_values.get(i).unwrap_or(0.0);
                let position_size = signal.metadata.get("position_size").unwrap_or(&0.0);
                let stop_loss_long = signal.metadata.get("stop_loss_long").unwrap_or(&0.0);
                let volatility_ratio = signal.metadata.get("volatility_ratio").unwrap_or(&1.0);
                
                println!("  Index {}: {:?}", i, signal.signal);
                println!("    ATR: {:.4}, Volatility Ratio: {:.2}", atr_value, volatility_ratio);
                println!("    Position Size: {:.2}, Stop Loss: {:.2}", position_size, stop_loss_long);
                println!("    Strength: {:.2}, Confidence: {:.2}", signal.strength, signal.confidence);
            }
        }
    }
    
    // Demonstrate position sizing calculations
    let close_prices = data.column("close")?.f64()?;
    if let Some(current_price) = close_prices.get(50) {
        if current_price > 0.0 {
            let atr_values = strategy.calculate_atr_values(data)?;
            if let Some(&current_atr) = atr_values.get(50) {
                if current_atr > 0.0 {
                    let position_size = strategy.calculate_position_size(current_price, current_atr, 10000.0);
                    let stop_loss_long = strategy.calculate_stop_loss(current_price, current_atr, true);
                    let stop_loss_short = strategy.calculate_stop_loss(current_price, current_atr, false);
                    
                    println!("\n💰 Position Sizing Example (Price: {:.2}):", current_price);
                    println!("  ATR: {:.4}", current_atr);
                    println!("  Recommended Position Size: {:.2} shares", position_size);
                    println!("  Long Stop Loss: {:.2}", stop_loss_long);
                    println!("  Short Stop Loss: {:.2}", stop_loss_short);
                }
            }
        }
    }
    
    Ok(())
}

fn demonstrate_combined_strategy(data: &DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Combined Strategy Approach");
    println!("=============================");
    println!("Using Williams %R for signals and ATR for position sizing");
    
    // Create both strategies
    let williams_r_strategy = WilliamsRStrategy::new(WilliamsRConfig::default());
    let atr_strategy = ATRStrategy::new(ATRConfig::default());
    
    // Get signals from Williams %R
    let williams_r_signals = williams_r_strategy.generate_enhanced_signals(data)?;
    
    // Get ATR values for position sizing
    let atr_values = atr_strategy.calculate_atr_values(data)?;
    let close_prices = data.column("close")?.f64()?;
    
    // Combine the strategies
    let mut combined_trades = Vec::new();
    
    for (i, signal) in williams_r_signals.iter().enumerate() {
        if signal.signal != Signal::Hold && i < atr_values.len() {
            let current_price = close_prices.get(i).unwrap_or(0.0);
            let current_atr = atr_values[i];
            
            if current_price > 0.0 && current_atr > 0.0 {
                let position_size = atr_strategy.calculate_position_size(current_price, current_atr, 10000.0);
                let stop_loss = atr_strategy.calculate_stop_loss(
                    current_price, 
                    current_atr, 
                    signal.signal == Signal::Buy
                );
                
                combined_trades.push(CombinedTrade {
                    index: i,
                    signal: signal.signal,
                    price: current_price,
                    williams_r_strength: signal.strength,
                    williams_r_confidence: signal.confidence,
                    atr: current_atr,
                    position_size,
                    stop_loss,
                });
            }
        }
    }
    
    println!("🎯 Generated {} combined trades", combined_trades.len());
    
    // Show top trades by confidence
    combined_trades.sort_by(|a, b| b.williams_r_confidence.partial_cmp(&a.williams_r_confidence).unwrap());
    
    println!("\n🏆 Top 5 Trades by Confidence:");
    for (rank, trade) in combined_trades.iter().take(5).enumerate() {
        println!("  {}. Index {}: {:?} at {:.2}", rank + 1, trade.index, trade.signal, trade.price);
        println!("     Williams %R Confidence: {:.2}, Strength: {:.2}", 
            trade.williams_r_confidence, trade.williams_r_strength);
        println!("     ATR: {:.4}, Position Size: {:.2}, Stop Loss: {:.2}", 
            trade.atr, trade.position_size, trade.stop_loss);
    }
    
    // Calculate risk metrics
    let total_risk: f64 = combined_trades.iter()
        .map(|t| (t.price - t.stop_loss).abs() * t.position_size)
        .sum();
    
    println!("\n⚠️  Risk Analysis:");
    println!("  Total Risk Exposure: ${:.2}", total_risk);
    println!("  Average Risk per Trade: ${:.2}", total_risk / combined_trades.len() as f64);
    
    Ok(())
}

#[derive(Debug)]
struct CombinedTrade {
    index: usize,
    signal: Signal,
    price: f64,
    williams_r_strength: f64,
    williams_r_confidence: f64,
    atr: f64,
    position_size: f64,
    stop_loss: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_runs_without_panic() {
        // This test ensures the example can run without panicking
        let result = std::panic::catch_unwind(|| {
            let data = create_realistic_market_data().unwrap();
            let williams_r_strategy = WilliamsRStrategy::new(WilliamsRConfig::default());
            let _signals = williams_r_strategy.generate_enhanced_signals(&data).unwrap();
        });
        
        assert!(result.is_ok());
    }
} 