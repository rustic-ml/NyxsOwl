// examples/multi_factor_strategy_example.rs
//! Multi-Factor Technical Strategy Example
//!
//! This example demonstrates how to combine multiple technical indicators
//! to create a more robust trading strategy using the NyxsOwl library.

use nyxs_owl::simple_types::{Result as NyxsOwlResult, Signal};
use nyxs_owl::technical_strategies::{
    multi_factor::MultiFactorStrategy, pattern_recognition::CandlestickPatternStrategy,
    volume::VWAPStrategy, CombinationMethod, SignalFilter, TechnicalSignal, TechnicalStrategy,
};
use nyxs_owl::technical_strategies::{Strategy, StrategyConfig};
use polars::prelude::*;

fn main() -> NyxsOwlResult<()> {
    println!("🔄 Multi-Factor Technical Strategy Example");
    println!("=========================================\n");

    // Generate sample OHLCV data with realistic price movements
    let data = generate_sample_data()?;
    println!(
        "📊 Generated {} periods of sample OHLCV data",
        data.height()
    );

    // 1. Individual Strategy Analysis
    println!("\n1️⃣ Individual Strategy Analysis");
    println!("-------------------------------");

    // VWAP-based volume strategy
    let vwap_config = StrategyConfig::new()
        .with_parameter("signal_threshold", 0.02)
        .with_parameter("min_volume_ratio", 1.5);

    let vwap_strategy = VWAPStrategy::new(vwap_config);
    let vwap_signals = vwap_strategy.generate_enhanced_signals(&data)?;
    analyze_strategy_signals("VWAP Volume Strategy", &vwap_signals);

    // Pattern recognition strategy
    let pattern_config = StrategyConfig::new().with_parameter("min_data_points", 5);

    let pattern_strategy = CandlestickPatternStrategy::new(pattern_config);
    let pattern_signals = pattern_strategy.generate_enhanced_signals(&data)?;
    analyze_strategy_signals("Pattern Recognition Strategy", &pattern_signals);

    // 2. Multi-Factor Strategy Combination
    println!("\n2️⃣ Multi-Factor Strategy Combination");
    println!("------------------------------------");

    // Basic multi-factor strategy
    let basic_multi_config = StrategyConfig::new()
        .with_parameter("min_signal_strength", 0.5)
        .with_parameter("min_confidence", 0.6);

    let basic_multi_strategy = MultiFactorStrategy::new(basic_multi_config);
    let basic_multi_signals = basic_multi_strategy.generate_enhanced_signals(&data)?;
    analyze_strategy_signals("Basic Multi-Factor Strategy", &basic_multi_signals);

    // Advanced multi-factor strategy with higher thresholds
    let advanced_multi_config = StrategyConfig::new()
        .with_parameter("min_signal_strength", 0.7)
        .with_parameter("min_confidence", 0.8)
        .with_parameter("short_ma_period", 5)
        .with_parameter("long_ma_period", 20);

    let advanced_multi_strategy = MultiFactorStrategy::new(advanced_multi_config);
    let advanced_multi_signals = advanced_multi_strategy.generate_enhanced_signals(&data)?;
    analyze_strategy_signals("Advanced Multi-Factor Strategy", &advanced_multi_signals);

    // 3. Signal Combination Analysis
    println!("\n3️⃣ Signal Combination Analysis");
    println!("------------------------------");

    // Combine signals using weighted average
    let signal_sources = vec![
        (vwap_signals.as_slice(), 0.4),    // 40% weight to volume analysis
        (pattern_signals.as_slice(), 0.6), // 60% weight to pattern analysis
    ];

    let combined_signals =
        SignalFilter::combine_signals(&signal_sources, CombinationMethod::WeightedAverage)?;
    analyze_strategy_signals("Combined Strategy (Weighted)", &combined_signals);

    // 4. Signal Filtering Analysis
    println!("\n4️⃣ Signal Filtering Analysis");
    println!("----------------------------");

    // Filter by strength
    let high_strength_signals = SignalFilter::by_strength(&combined_signals, 0.7);
    analyze_strategy_signals("High Strength Signals (>0.7)", &high_strength_signals);

    // Filter by confidence
    let high_confidence_signals = SignalFilter::by_confidence(&combined_signals, 0.8);
    analyze_strategy_signals("High Confidence Signals (>0.8)", &high_confidence_signals);

    // 5. Parameter Sensitivity Analysis
    println!("\n5️⃣ Parameter Sensitivity Analysis");
    println!("---------------------------------");

    // Test different strength thresholds
    println!("Signal Count by Strength Threshold:");
    for threshold in [0.3, 0.5, 0.7, 0.9] {
        let filtered = SignalFilter::by_strength(&combined_signals, threshold);
        let buy_count = filtered.iter().filter(|s| s.signal == Signal::Buy).count();
        let sell_count = filtered.iter().filter(|s| s.signal == Signal::Sell).count();
        println!(
            "  Threshold {:.1}: {} signals ({} buy, {} sell)",
            threshold,
            filtered.len(),
            buy_count,
            sell_count
        );
    }

    // Test different confidence thresholds
    println!("\nSignal Count by Confidence Threshold:");
    for threshold in [0.3, 0.5, 0.7, 0.9] {
        let filtered = SignalFilter::by_confidence(&combined_signals, threshold);
        let buy_count = filtered.iter().filter(|s| s.signal == Signal::Buy).count();
        let sell_count = filtered.iter().filter(|s| s.signal == Signal::Sell).count();
        println!(
            "  Threshold {:.1}: {} signals ({} buy, {} sell)",
            threshold,
            filtered.len(),
            buy_count,
            sell_count
        );
    }

    // 6. Signal Quality Analysis
    println!("\n6️⃣ Signal Quality Analysis");
    println!("-------------------------");

    analyze_signal_quality(&data, &combined_signals)?;

    println!("\n✅ Multi-Factor Strategy Analysis Complete!");

    Ok(())
}

/// Generate sample OHLCV data with realistic patterns
fn generate_sample_data() -> NyxsOwlResult<DataFrame> {
    const PERIODS: usize = 100;

    // Use simple PRNG with step-based randomization
    let mut prices = Vec::with_capacity(PERIODS);
    let mut volumes = Vec::with_capacity(PERIODS);

    let mut price = 100.0;
    let volume_base = 50000.0;

    for i in 0..PERIODS {
        // Create price trend with some volatility
        let trend = (i as f64 * 0.02).sin() * 2.0;
        let noise = ((i * 7) % 17) as f64 / 17.0 - 0.5; // Simple pseudo-random
        price += trend + noise;
        price = price.max(50.0); // Prevent negative prices

        prices.push(price);

        // Create volume with spikes
        let volume_multiplier = if i % 15 == 0 { 2.5 } else { 1.0 };
        let volume_noise = ((i * 11) % 13) as f64 / 13.0 * 0.5 + 0.75;
        volumes.push(volume_base * volume_multiplier * volume_noise);
    }

    // Generate OHLC from close prices
    let mut opens = Vec::with_capacity(PERIODS);
    let mut highs = Vec::with_capacity(PERIODS);
    let mut lows = Vec::with_capacity(PERIODS);

    for i in 0..PERIODS {
        let close = prices[i];
        let daily_range = close * 0.02; // 2% daily range

        opens.push(if i == 0 { close } else { prices[i - 1] });
        highs.push(close + daily_range * 0.7);
        lows.push(close - daily_range * 0.5);
    }

    Ok(df! {
        "open" => opens,
        "high" => highs,
        "low" => lows,
        "close" => prices,
        "volume" => volumes,
    }?)
}

/// Analyze signals from a strategy
fn analyze_strategy_signals(strategy_name: &str, signals: &[TechnicalSignal]) {
    let total_signals = signals.len();
    let buy_signals: Vec<_> = signals.iter().filter(|s| s.signal == Signal::Buy).collect();
    let sell_signals: Vec<_> = signals
        .iter()
        .filter(|s| s.signal == Signal::Sell)
        .collect();
    let hold_signals = total_signals - buy_signals.len() - sell_signals.len();

    println!("{}", strategy_name);
    println!("  Total signals: {}", total_signals);
    println!(
        "  Buy signals: {} ({:.1}%)",
        buy_signals.len(),
        buy_signals.len() as f64 / total_signals as f64 * 100.0
    );
    println!(
        "  Sell signals: {} ({:.1}%)",
        sell_signals.len(),
        sell_signals.len() as f64 / total_signals as f64 * 100.0
    );
    println!(
        "  Hold signals: {} ({:.1}%)",
        hold_signals,
        hold_signals as f64 / total_signals as f64 * 100.0
    );

    if !buy_signals.is_empty() {
        let avg_buy_strength =
            buy_signals.iter().map(|s| s.strength).sum::<f64>() / buy_signals.len() as f64;
        let avg_buy_confidence =
            buy_signals.iter().map(|s| s.confidence).sum::<f64>() / buy_signals.len() as f64;
        println!("  Avg buy strength: {:.3}", avg_buy_strength);
        println!("  Avg buy confidence: {:.3}", avg_buy_confidence);
    }

    if !sell_signals.is_empty() {
        let avg_sell_strength =
            sell_signals.iter().map(|s| s.strength).sum::<f64>() / sell_signals.len() as f64;
        let avg_sell_confidence =
            sell_signals.iter().map(|s| s.confidence).sum::<f64>() / sell_signals.len() as f64;
        println!("  Avg sell strength: {:.3}", avg_sell_strength);
        println!("  Avg sell confidence: {:.3}", avg_sell_confidence);
    }

    println!();
}

/// Analyze signal quality against price data
fn analyze_signal_quality(_data: &DataFrame, signals: &[TechnicalSignal]) -> NyxsOwlResult<()> {
    println!("Signal Quality Metrics:");

    // Calculate basic signal metrics
    let buy_count = signals.iter().filter(|s| s.signal == Signal::Buy).count();
    let sell_count = signals.iter().filter(|s| s.signal == Signal::Sell).count();
    let hold_count = signals.iter().filter(|s| s.signal == Signal::Hold).count();

    // Calculate average strength and confidence for trading signals
    let trading_signals: Vec<_> = signals
        .iter()
        .filter(|s| s.signal != Signal::Hold)
        .collect();

    if !trading_signals.is_empty() {
        let avg_strength: f64 =
            trading_signals.iter().map(|s| s.strength).sum::<f64>() / trading_signals.len() as f64;
        let avg_confidence: f64 = trading_signals.iter().map(|s| s.confidence).sum::<f64>()
            / trading_signals.len() as f64;

        println!("  Average signal strength: {:.3}", avg_strength);
        println!("  Average signal confidence: {:.3}", avg_confidence);
        println!(
            "  Overall signal quality: {:.3}",
            (avg_strength + avg_confidence) / 2.0
        );
    }

    // Analyze signal distribution
    let signal_balance = if buy_count + sell_count > 0 {
        1.0 - ((buy_count as f64 - sell_count as f64).abs() / (buy_count + sell_count) as f64)
    } else {
        0.0
    };

    println!("  Signal balance score: {:.3}", signal_balance);
    println!(
        "  Buy/Sell/Hold ratio: {}/{}/{}",
        buy_count, sell_count, hold_count
    );

    Ok(())
}
