use nyxs_owl::prelude::*;
use nyxs_owl::technical_strategies::prelude::*;
use polars::prelude::*;

fn main() -> NyxsOwlResult<()> {
    println!("🎯 NyxsOwl VWAP Strategy Example");
    println!("════════════════════════════════");
    println!();

    // Create sample OHLCV data for demonstration
    let sample_data = create_sample_ohlcv_data()?;
    println!(
        "📊 Created sample OHLCV data with {} rows",
        sample_data.height()
    );

    // Display first few rows of the data
    println!("\n📈 Sample Data (first 5 rows):");
    println!("{}", sample_data.head(Some(5)));

    // Example 1: Basic VWAP Strategy
    println!("\n🔍 Example 1: Basic VWAP Strategy");
    println!("───────────────────────────────────");

    let basic_config = StrategyConfig::new()
        .with_parameter("signal_threshold", 0.02) // 2% threshold
        .with_parameter("min_volume_ratio", 1.5) // 1.5x average volume
        .with_parameter("min_data_points", 10); // Minimum 10 data points

    let vwap_strategy = VWAPStrategy::new(basic_config);

    // Generate signals
    let signals = vwap_strategy.generate_enhanced_signals(&sample_data)?;
    println!("Generated {} VWAP signals", signals.len());

    // Display signal details
    display_signal_summary(&signals);

    // Example 2: Aggressive VWAP Strategy (Lower Thresholds)
    println!("\n🚀 Example 2: Aggressive VWAP Strategy");
    println!("─────────────────────────────────────");

    let aggressive_config = StrategyConfig::new()
        .with_parameter("signal_threshold", 0.01) // 1% threshold (more sensitive)
        .with_parameter("min_volume_ratio", 1.2) // 1.2x average volume
        .with_parameter("min_data_points", 5); // Lower minimum

    let aggressive_vwap = VWAPStrategy::new(aggressive_config);
    let aggressive_signals = aggressive_vwap.generate_enhanced_signals(&sample_data)?;

    println!(
        "Generated {} aggressive VWAP signals",
        aggressive_signals.len()
    );
    display_signal_summary(&aggressive_signals);

    // Example 3: Conservative VWAP Strategy (Higher Thresholds)
    println!("\n🛡️ Example 3: Conservative VWAP Strategy");
    println!("─────────────────────────────────────");

    let conservative_config = StrategyConfig::new()
        .with_parameter("signal_threshold", 0.05) // 5% threshold (less sensitive)
        .with_parameter("min_volume_ratio", 2.0) // 2x average volume
        .with_parameter("min_data_points", 20); // Higher minimum

    let conservative_vwap = VWAPStrategy::new(conservative_config);
    let conservative_signals = conservative_vwap.generate_enhanced_signals(&sample_data)?;

    println!(
        "Generated {} conservative VWAP signals",
        conservative_signals.len()
    );
    display_signal_summary(&conservative_signals);

    // Example 4: Analyze VWAP Indicator Values
    println!("\n📊 Example 4: VWAP Indicator Analysis");
    println!("────────────────────────────────────");

    let indicators = vwap_strategy.get_indicator_values(&sample_data)?;

    if let Some(vwap_series) = indicators.get("vwap") {
        let vwap_values: Vec<f64> = vwap_series.f64()?.into_iter().flatten().collect();

        if !vwap_values.is_empty() {
            println!("VWAP Values:");
            println!("  Latest: {:.4}", vwap_values.last().unwrap_or(&0.0));
            println!(
                "  Average: {:.4}",
                vwap_values.iter().sum::<f64>() / vwap_values.len() as f64
            );
            println!(
                "  Min: {:.4}",
                vwap_values.iter().fold(f64::INFINITY, |a, &b| a.min(b))
            );
            println!(
                "  Max: {:.4}",
                vwap_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
            );
        }
    }

    if let Some(obv_series) = indicators.get("obv") {
        let obv_values: Vec<f64> = obv_series.f64()?.into_iter().flatten().collect();

        if !obv_values.is_empty() {
            println!("\nOBV (On-Balance Volume) Values:");
            println!("  Latest: {:.0}", obv_values.last().unwrap_or(&0.0));
            println!(
                "  Trend: {}",
                if obv_values.len() >= 2 {
                    let latest = obv_values.last().unwrap();
                    let previous = obv_values[obv_values.len() - 2];
                    if latest > &previous {
                        "📈 Rising"
                    } else if latest < &previous {
                        "📉 Falling"
                    } else {
                        "➡️ Flat"
                    }
                } else {
                    "N/A"
                }
            );
        }
    }

    // Example 5: Parameter Optimization Demo
    println!("\n⚙️ Example 5: Parameter Sensitivity Analysis");
    println!("───────────────────────────────────────────");

    analyze_parameter_sensitivity(&sample_data)?;

    println!("\n✅ VWAP Strategy examples completed successfully!");
    println!("\n💡 Key Takeaways:");
    println!("   • VWAP combines price and volume analysis");
    println!("   • Lower thresholds = more signals (but more noise)");
    println!("   • Higher thresholds = fewer, higher-quality signals");
    println!("   • Volume confirmation is crucial for VWAP signals");
    println!("   • Best used in conjunction with other indicators");

    Ok(())
}

fn create_sample_ohlcv_data() -> PolarsResult<DataFrame> {
    // Create realistic OHLCV data
    let n_points = 50;
    let mut open_prices = Vec::new();
    let mut high_prices = Vec::new();
    let mut low_prices = Vec::new();
    let mut close_prices = Vec::new();
    let mut volumes = Vec::new();

    let mut price = 100.0;
    let mut rng_state = 12345u64; // Simple PRNG state

    for _i in 0..n_points {
        // Simple PRNG
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let rand1 = (rng_state as f64) / (u64::MAX as f64);

        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let rand2 = (rng_state as f64) / (u64::MAX as f64);

        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let rand3 = (rng_state as f64) / (u64::MAX as f64);

        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let rand4 = (rng_state as f64) / (u64::MAX as f64);

        // Create realistic OHLC movement
        let daily_return = (rand1 - 0.5) * 0.05; // +/- 2.5% daily moves
        let volatility = 0.02; // 2% intraday volatility

        let open = price;
        let close = price * (1.0 + daily_return);
        let high = open.max(close) * (1.0 + volatility * rand2);
        let low = open.min(close) * (1.0 - volatility * rand3);

        // Volume with some correlation to price movement
        let base_volume = 1000000.0;
        let volume_multiplier = 1.0 + (daily_return.abs() * 5.0) + (rand4 - 0.5) * 0.5;
        let volume = (base_volume * volume_multiplier) as i64;

        open_prices.push(open);
        high_prices.push(high);
        low_prices.push(low);
        close_prices.push(close);
        volumes.push(volume);

        price = close;
    }

    df! {
        "open" => &open_prices,
        "high" => &high_prices,
        "low" => &low_prices,
        "close" => &close_prices,
        "volume" => &volumes,
    }
}

fn display_signal_summary(signals: &[TechnicalSignal]) {
    let buy_count = signals.iter().filter(|s| s.signal == Signal::Buy).count();
    let sell_count = signals.iter().filter(|s| s.signal == Signal::Sell).count();
    let hold_count = signals.iter().filter(|s| s.signal == Signal::Hold).count();

    println!("Signal Distribution:");
    println!("  📈 Buy signals:  {}", buy_count);
    println!("  📉 Sell signals: {}", sell_count);
    println!("  ⏸️  Hold signals: {}", hold_count);

    if !signals.is_empty() {
        let avg_strength: f64 =
            signals.iter().map(|s| s.strength.abs()).sum::<f64>() / signals.len() as f64;
        let avg_confidence: f64 =
            signals.iter().map(|s| s.confidence).sum::<f64>() / signals.len() as f64;

        println!("Signal Quality:");
        println!("  💪 Average strength: {:.3}", avg_strength);
        println!("  🎯 Average confidence: {:.3}", avg_confidence);

        // Show a few example signals with metadata
        println!("\nSample Signals:");
        for (i, signal) in signals.iter().take(3).enumerate() {
            println!(
                "  Signal {}: {:?} (strength: {:.3}, confidence: {:.3})",
                i + 1,
                signal.signal,
                signal.strength,
                signal.confidence
            );

            if let Some(vwap) = signal.metadata.get("vwap") {
                println!("    VWAP: {:.4}", vwap);
            }
            if let Some(volume_ratio) = signal.metadata.get("volume_ratio") {
                println!("    Volume Ratio: {:.2}x", volume_ratio);
            }
        }
    }
}

fn analyze_parameter_sensitivity(data: &DataFrame) -> NyxsOwlResult<()> {
    let thresholds = vec![0.01, 0.02, 0.03, 0.05];
    let volume_ratios = vec![1.2, 1.5, 2.0];

    println!("Analyzing parameter sensitivity...");
    println!(
        "{:<12} {:<12} {:<12} {:<12}",
        "Threshold", "Vol Ratio", "Signals", "Avg Strength"
    );
    println!("{}", "─".repeat(50));

    for &threshold in &thresholds {
        for &vol_ratio in &volume_ratios {
            let config = StrategyConfig::new()
                .with_parameter("signal_threshold", threshold)
                .with_parameter("min_volume_ratio", vol_ratio)
                .with_parameter("min_data_points", 5);

            let strategy = VWAPStrategy::new(config);

            if let Ok(signals) = strategy.generate_enhanced_signals(data) {
                let signal_count = signals.len();
                let avg_strength = if signal_count > 0 {
                    signals.iter().map(|s| s.strength.abs()).sum::<f64>() / signal_count as f64
                } else {
                    0.0
                };

                println!(
                    "{:<12.3} {:<12.1} {:<12} {:<12.3}",
                    threshold, vol_ratio, signal_count, avg_strength
                );
            }
        }
    }

    Ok(())
}
