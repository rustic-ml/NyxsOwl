use nyxs_owl::prelude::*;
use nyxs_owl::technical_strategies::prelude::*;
use polars::prelude::*;

fn main() -> NyxsOwlResult<()> {
    println!("🕯️ NyxsOwl Candlestick Pattern Strategy Example");
    println!("═══════════════════════════════════════════════");
    println!();

    // Create sample OHLC data with specific patterns
    let sample_data = create_candlestick_test_data()?;
    println!(
        "📊 Created sample OHLC data with {} candlesticks",
        sample_data.height()
    );

    // Display first few rows of the data
    println!("\n📈 Sample Data (first 8 rows):");
    println!("{}", sample_data.head(Some(8)));

    // Example 1: Basic Candlestick Pattern Detection
    println!("\n🔍 Example 1: Basic Pattern Detection");
    println!("────────────────────────────────────");

    let basic_config = StrategyConfig::new()
        .with_parameter("min_body_ratio", 0.3) // 30% body ratio
        .with_parameter("min_shadow_ratio", 0.1) // 10% shadow ratio
        .with_parameter("min_data_points", 2); // Minimum 2 data points

    let pattern_strategy = CandlestickPatternStrategy::new(basic_config);

    // Generate pattern signals
    let signals = pattern_strategy.generate_enhanced_signals(&sample_data)?;
    println!("Generated {} candlestick pattern signals", signals.len());

    // Display pattern analysis
    display_pattern_signals(&signals);

    // Example 2: Conservative Pattern Detection (Stricter Requirements)
    println!("\n🛡️ Example 2: Conservative Pattern Detection");
    println!("────────────────────────────────────────────");

    let conservative_config = StrategyConfig::new()
        .with_parameter("min_body_ratio", 0.5) // 50% body ratio (stricter)
        .with_parameter("min_shadow_ratio", 0.2) // 20% shadow ratio (stricter)
        .with_parameter("min_data_points", 5); // Minimum 5 data points

    let conservative_strategy = CandlestickPatternStrategy::new(conservative_config);
    let conservative_signals = conservative_strategy.generate_enhanced_signals(&sample_data)?;

    println!(
        "Generated {} conservative pattern signals",
        conservative_signals.len()
    );
    display_pattern_signals(&conservative_signals);

    // Example 3: Parameter Sensitivity
    println!("\n⚙️ Example 3: Parameter Sensitivity Analysis");
    println!("────────────────────────────────────────────");

    analyze_pattern_sensitivity(&sample_data)?;

    println!("\n✅ Candlestick Pattern examples completed successfully!");
    println!("\n💡 Key Takeaways:");
    println!("   • Candlestick patterns reveal market sentiment");
    println!("   • Stricter parameters = fewer but higher-quality signals");
    println!("   • Pattern strength indicates reliability");
    println!("   • Best combined with volume and trend confirmation");

    Ok(())
}

fn create_candlestick_test_data() -> PolarsResult<DataFrame> {
    // Create OHLC data with specific candlestick patterns
    let mut open_prices = Vec::new();
    let mut high_prices = Vec::new();
    let mut low_prices = Vec::new();
    let mut close_prices = Vec::new();

    let mut price = 100.0;
    let mut rng_state = 54321u64; // Different seed for pattern variety

    // Generate 30 candlesticks with intentional patterns
    for i in 0..30 {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let rand1 = (rng_state as f64) / (u64::MAX as f64);

        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let rand2 = (rng_state as f64) / (u64::MAX as f64);

        let (open, high, low, close) = if i == 5 || i == 15 || i == 25 {
            // Create bullish engulfing patterns
            let open = price * 0.98; // Gap down open
            let close = price * 1.04; // Strong close up
            let high = close * 1.01;
            let low = open * 0.99;
            (open, high, low, close)
        } else if i == 8 || i == 18 {
            // Create bearish engulfing patterns
            let open = price * 1.02; // Gap up open
            let close = price * 0.96; // Strong close down
            let high = open * 1.01;
            let low = close * 0.99;
            (open, high, low, close)
        } else if i == 10 || i == 20 {
            // Create hammer patterns
            let open = price;
            let close = price * 1.01; // Slight up close
            let high = close * 1.005;
            let low = price * 0.94; // Long lower shadow
            (open, high, low, close)
        } else if i == 12 || i == 22 {
            // Create shooting star patterns
            let open = price;
            let close = price * 0.99; // Slight down close
            let high = price * 1.06; // Long upper shadow
            let low = close * 0.995;
            (open, high, low, close)
        } else {
            // Normal candlesticks
            let daily_return = (rand1 - 0.5) * 0.03; // +/- 1.5% moves
            let volatility = 0.015; // 1.5% intraday volatility

            let open = price;
            let close = price * (1.0 + daily_return);
            let high = open.max(close) * (1.0 + volatility * rand2);
            let low = open.min(close) * (1.0 - volatility * rand2);
            (open, high, low, close)
        };

        open_prices.push(open);
        high_prices.push(high);
        low_prices.push(low);
        close_prices.push(close);

        price = close;
    }

    df! {
        "open" => &open_prices,
        "high" => &high_prices,
        "low" => &low_prices,
        "close" => &close_prices,
    }
}

fn display_pattern_signals(signals: &[TechnicalSignal]) {
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

        // Show example patterns detected
        println!("\nPattern Examples:");
        for (i, signal) in signals.iter().take(3).enumerate() {
            if signal.signal != Signal::Hold {
                println!(
                    "  Pattern {}: {:?} (strength: {:.3}, confidence: {:.3})",
                    i + 1,
                    signal.signal,
                    signal.strength,
                    signal.confidence
                );

                if let Some(pattern_type) = signal.metadata.get("pattern_type") {
                    println!("    Pattern Type Code: {:.0}", pattern_type);
                }
                if let Some(body_ratio) = signal.metadata.get("body_ratio") {
                    println!("    Body Ratio: {:.3}", body_ratio);
                }
            }
        }
    }
}

fn analyze_pattern_sensitivity(data: &DataFrame) -> NyxsOwlResult<()> {
    let body_ratios = vec![0.2, 0.3, 0.4, 0.5];
    let shadow_ratios = vec![0.05, 0.1, 0.15, 0.2];

    println!("Parameter Sensitivity Analysis:");
    println!(
        "{:<12} {:<12} {:<12} {:<12}",
        "Body Ratio", "Shadow Ratio", "Signals", "Avg Strength"
    );
    println!("{}", "─".repeat(50));

    for &body_ratio in &body_ratios {
        for &shadow_ratio in &shadow_ratios {
            let config = StrategyConfig::new()
                .with_parameter("min_body_ratio", body_ratio)
                .with_parameter("min_shadow_ratio", shadow_ratio)
                .with_parameter("min_data_points", 2);

            let strategy = CandlestickPatternStrategy::new(config);

            if let Ok(signals) = strategy.generate_enhanced_signals(data) {
                let signal_count = signals.len();
                let avg_strength = if signal_count > 0 {
                    signals.iter().map(|s| s.strength.abs()).sum::<f64>() / signal_count as f64
                } else {
                    0.0
                };

                println!(
                    "{:<12.2} {:<12.2} {:<12} {:<12.3}",
                    body_ratio, shadow_ratio, signal_count, avg_strength
                );
            }
        }
    }

    Ok(())
}
