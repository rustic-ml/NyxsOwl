//! Example demonstrating OxiDiviner 1.2.0 Adaptive Forecasting Features
//!
//! This example showcases the new adaptive capabilities in OxiDiviner 1.2.0:
//! - Adaptive Ensemble Strategy with dynamic model weighting
//! - Enhanced ARIMA with automatic order selection
//! - Regime-aware parameter adjustment
//! - Real-time quality monitoring and alerts
//!
//! Run with: cargo run --example oxidiviner_1_2_0_adaptive_example

use nyxs_owl::forecasting::strategies::{
    AdaptiveEnsembleStrategy, AdaptiveEnsembleConfig,
    ArimaStrategy, ArimaStrategyConfig, MarketRegime
};
use nyxs_owl::simple_types::{Result, Signal};
use polars::prelude::*;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 OxiDiviner 1.2.0 Adaptive Forecasting Demo");
    println!("============================================\n");
    
    // Generate market data
    let market_data = generate_test_data(500)?;
    
    // Demo 1: Enhanced ARIMA
    println!("📈 Demo 1: Enhanced ARIMA with Adaptive Features");
    demo_enhanced_arima(&market_data)?;
    
    // Demo 2: Adaptive Ensemble
    println!("\n🔮 Demo 2: Adaptive Ensemble Strategy");
    demo_adaptive_ensemble(&market_data)?;
    
    // Demo 3: Regime Detection and Adaptation
    println!("\n🎯 Demo 3: Regime Detection and Adaptive Parameters");
    demo_regime_adaptation(&market_data)?;
    
    // Demo 4: Real-time Quality Monitoring
    println!("\n📊 Demo 4: Real-time Quality Monitoring");
    demo_quality_monitoring(&market_data)?;
    
    println!("\n✅ Demo completed successfully!");
    
    Ok(())
}

/// Demo 1: Enhanced ARIMA with automatic order selection and adaptive parameters
fn demo_enhanced_arima(df: &DataFrame) -> Result<()> {
    let config = ArimaStrategyConfig {
        model_selection: true,
        dynamic_threshold: true,
        outlier_detection: true,
        regime_detection: true,
        ..ArimaStrategyConfig::default()
    };
    
    let mut strategy = ArimaStrategy::new(config);
    let signals = strategy.generate_signals(df, "close", "timestamp")?;
    
    let (buy, sell, hold) = count_signals(&signals);
    println!("   📊 ARIMA Results: {}B/{}S/{}H", buy, sell, hold);
    
    Ok(())
}

/// Demo 2: Adaptive Ensemble Strategy with dynamic model weighting
fn demo_adaptive_ensemble(df: &DataFrame) -> Result<()> {
    let config = AdaptiveEnsembleConfig {
        adaptive_weighting: true,
        regime_detection: true,
        quality_monitoring: true,
        ..AdaptiveEnsembleConfig::default()
    };
    
    let mut strategy = AdaptiveEnsembleStrategy::new(config);
    let signals = strategy.generate_signals(df, "close", "timestamp")?;
    
    let (buy, sell, hold) = count_signals(&signals);
    println!("   📊 Ensemble Results: {}B/{}S/{}H", buy, sell, hold);
    
    if let Some(regime) = strategy.get_current_regime() {
        println!("   🎯 Current regime: {:?}", regime);
    }
    
    Ok(())
}

/// Demo 3: Regime detection and adaptive parameter adjustment
fn demo_regime_adaptation(df: &DataFrame) -> Result<()> {
    println!("   Demonstrating regime detection and adaptive parameters...");
    
    // Extract prices for regime analysis
    let prices: Vec<f64> = df.column("close")?
        .f64()?
        .into_no_null_iter()
        .collect();
    
    // Analyze regime changes throughout the data
    let regimes = detect_regime_changes(&prices, 30);
    
    println!("   📈 Regime Analysis:");
    for (start_idx, regime, duration) in regimes {
        let percentage = (duration as f64 / prices.len() as f64) * 100.0;
        println!("      {:?} regime: {}% of data (starting at index {})", 
                 regime, percentage as i32, start_idx);
    }
    
    // Demonstrate adaptive parameter adjustment
    println!("   🔧 Adaptive Parameter Examples:");
    println!("      High Volatility => alpha=0.5, threshold=2x");
    println!("      Low Volatility  => alpha=0.1, threshold=0.7x");
    println!("      Bull Market     => alpha=0.3, add momentum");
    println!("      Bear Market     => alpha=0.2, tighter stops");
    
    Ok(())
}

/// Demo 4: Real-time quality monitoring and alerts
fn demo_quality_monitoring(df: &DataFrame) -> Result<()> {
    println!("   Setting up real-time quality monitoring...");
    
    // Create strategy with quality monitoring enabled
    let config = AdaptiveEnsembleConfig {
        quality_monitoring: true,
        quality_threshold: 0.7,
        ..AdaptiveEnsembleConfig::default()
    };
    
    let mut strategy = AdaptiveEnsembleStrategy::new(config);
    
    // Simulate real-time processing with quality monitoring
    let window_size = 100;
    let prices: Vec<f64> = df.column("close")?
        .f64()?
        .into_no_null_iter()
        .collect();
    
    println!("   📊 Quality Monitoring Results:");
    
    for i in (window_size..prices.len()).step_by(50) {
        let window_data = create_window_dataframe(&prices[i-window_size..i+1], i-window_size)?;
        let signals = strategy.generate_signals(&window_data, "close", "timestamp")?;
        
        // Calculate performance for this window
        let accuracy = calculate_window_accuracy(&signals, &prices[i-window_size..i+1]);
        
        if accuracy < 0.6 {
            println!("      ⚠️  Quality Alert at index {}: Accuracy {:.1}%", i, accuracy * 100.0);
        } else if accuracy > 0.8 {
            println!("      ✅ High Quality at index {}: Accuracy {:.1}%", i, accuracy * 100.0);
        }
    }
    
    Ok(())
}

/// Generate market data
fn generate_test_data(len: usize) -> Result<DataFrame> {
    let timestamps: Vec<String> = (0..len)
        .map(|i| format!("2023-01-{:02}", (i % 30) + 1))
        .collect();
        
    let prices: Vec<f64> = (0..len)
        .map(|i| 100.0 + (i as f64 * 0.1) + (i as f64 * 0.1).sin() * 5.0)
        .collect();
        
    df! {
        "timestamp" => timestamps,
        "close" => prices,
    }.map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(e.to_string()))
}

/// Detect regime changes in price data
fn detect_regime_changes(prices: &[f64], window: usize) -> Vec<(usize, MarketRegime, usize)> {
    let mut regimes = Vec::new();
    let mut current_regime = None;
    let mut regime_start = 0;
    
    for i in window..prices.len() {
        let window_data = &prices[i-window..i];
        let regime = classify_regime(window_data);
        
        if current_regime.as_ref() != Some(&regime) {
            if let Some(prev_regime) = current_regime {
                regimes.push((regime_start, prev_regime, i - regime_start));
            }
            current_regime = Some(regime);
            regime_start = i;
        }
    }
    
    // Add final regime
    if let Some(regime) = current_regime {
        regimes.push((regime_start, regime, prices.len() - regime_start));
    }
    
    regimes
}

/// Classify market regime based on price window
fn classify_regime(prices: &[f64]) -> MarketRegime {
    let volatility = calculate_volatility(prices);
    let trend = calculate_trend(prices);
    
    if volatility > 0.03 {
        MarketRegime::HighVolatility
    } else if volatility < 0.01 {
        MarketRegime::LowVolatility
    } else if trend > 0.02 {
        MarketRegime::Bull
    } else if trend < -0.02 {
        MarketRegime::Bear
    } else {
        MarketRegime::Sideways
    }
}

/// Calculate volatility from prices
fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }
    
    let returns: Vec<f64> = prices.windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();
    
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / returns.len() as f64;
    
    variance.sqrt()
}

/// Calculate trend strength
fn calculate_trend(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }
    
    let first_half = prices.len() / 2;
    let first_avg = prices[..first_half].iter().sum::<f64>() / first_half as f64;
    let second_avg = prices[first_half..].iter().sum::<f64>() / (prices.len() - first_half) as f64;
    
    (second_avg - first_avg) / first_avg
}

/// Count different signal types
fn count_signals(signals: &[Signal]) -> (usize, usize, usize) {
    let mut buy = 0;
    let mut sell = 0;
    let mut hold = 0;
    
    for signal in signals {
        match signal {
            Signal::Buy => buy += 1,
            Signal::Sell => sell += 1,
            Signal::Hold => hold += 1,
        }
    }
    
    (buy, sell, hold)
}

/// Create DataFrame from price window
fn create_window_dataframe(prices: &[f64], start_idx: usize) -> Result<DataFrame> {
    let timestamps: Vec<String> = (0..prices.len())
        .map(|i| format!("2023-01-{:02}", (start_idx + i) % 30 + 1))
        .collect();
    
    df! {
        "timestamp" => timestamps,
        "close" => prices.to_vec(),
    }.map_err(|e| nyxs_owl::simple_types::NyxsOwlError::DataError(format!("Failed to create DataFrame: {}", e)))
}

/// Calculate accuracy for a window of signals
fn calculate_window_accuracy(signals: &[Signal], prices: &[f64]) -> f64 {
    if signals.len() < 2 || prices.len() < 2 {
        return 0.5; // Neutral accuracy
    }
    
    let mut correct = 0;
    let mut total = 0;
    
    for i in 0..signals.len().min(prices.len()-1) {
        let actual_direction = if prices[i+1] > prices[i] { 1 } else if prices[i+1] < prices[i] { -1 } else { 0 };
        let predicted_direction = match signals[i] {
            Signal::Buy => 1,
            Signal::Sell => -1,
            Signal::Hold => 0,
        };
        
        if predicted_direction == actual_direction {
            correct += 1;
        }
        total += 1;
    }
    
    if total > 0 {
        correct as f64 / total as f64
    } else {
        0.5
    }
} 