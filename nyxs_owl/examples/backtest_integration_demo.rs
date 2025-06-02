//! # Backtest Integration Demo
//!
//! This example demonstrates how to integrate NyxsOwl strategies with external
//! backtesting frameworks using the new integration utilities.

use chrono::Utc;
use nyxs_owl::strategy_lib::strategy::trend_following::MovingAverageCrossover;
use nyxs_owl::strategy_lib::strategy::utils::create_ma_config;
use nyxs_owl::strategy_lib::{
    integration::{DataConverter, ExternalOHLCV, ExternalSignal, SignalConverter},
    AsyncStrategy, Strategy, StrategyAdapter,
};
use polars::prelude::*;

#[cfg(feature = "async-support")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_demo().await
}

#[cfg(not(feature = "async-support"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NyxsOwl Backtest Integration Demo ===\n");
    println!(
        "Note: Running without async support. Enable 'async-support' feature for full demo.\n"
    );

    // Run synchronous parts of the demo
    run_sync_demo()
}

#[cfg(feature = "async-support")]
async fn run_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NyxsOwl Backtest Integration Demo ===\n");

    // 1. Create sample market data in external format
    let external_data = create_sample_data();
    println!("Created {} data points", external_data.len());

    // 2. Convert external data to NyxsOwl format
    let df = DataConverter::from_ohlcv_vec(&external_data)?;
    println!("Converted to DataFrame with shape: {:?}", df.shape());

    // 3. Create and configure a strategy
    let strategy = MovingAverageCrossover::sma_crossover(10, 20);

    println!("\nStrategy: {}", strategy.name());
    println!("Description: {}", strategy.description());
    println!("Required columns: {:?}", strategy.required_columns());
    println!("Min data points: {}", strategy.min_data_points());

    // 4. Test synchronous strategy execution
    println!("\n=== Synchronous Strategy Execution ===");
    let signals = strategy.generate_signals(&df)?;
    println!("Generated {} signals", signals.len());

    // Display first few signals
    let signal_values = signals.i32()?;
    for i in 0..5.min(signals.len()) {
        if let Some(signal_val) = signal_values.get(i) {
            let signal_type = match signal_val {
                0 => "Hold",
                1 => "Buy",
                2 => "Sell",
                _ => "Unknown",
            };
            println!("  Signal {}: {} ({})", i, signal_val, signal_type);
        }
    }

    // 5. Test async strategy execution
    println!("\n=== Asynchronous Strategy Execution ===");
    let async_strategy = StrategyAdapter::new(strategy.clone());
    let async_signals = async_strategy.generate_signals_async(&df).await?;
    println!("Generated {} async signals", async_signals.len());

    // Verify signals match
    let async_signal_values = async_signals.i32()?;
    let mut matches = 0;
    for i in 0..signals.len().min(async_signals.len()) {
        if signal_values.get(i) == async_signal_values.get(i) {
            matches += 1;
        }
    }
    println!(
        "Signal match rate: {}/{} ({:.1}%)",
        matches,
        signals.len(),
        (matches as f64 / signals.len() as f64) * 100.0
    );

    run_common_demo_parts(&external_data, &df, &strategy)?;

    // Performance comparison with async
    println!("\n=== Performance Comparison ===");

    // Time synchronous execution
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = strategy.generate_signals(&df)?;
    }
    let sync_duration = start.elapsed();
    println!("Synchronous execution (10 runs): {:?}", sync_duration);

    // Time asynchronous execution
    let async_strategy = StrategyAdapter::new(strategy.clone());
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = async_strategy.generate_signals_async(&df).await?;
    }
    let async_duration = start.elapsed();
    println!("Asynchronous execution (10 runs): {:?}", async_duration);

    let overhead = if async_duration > sync_duration {
        ((async_duration.as_nanos() as f64 / sync_duration.as_nanos() as f64) - 1.0) * 100.0
    } else {
        0.0
    };
    println!("Async overhead: {:.1}%", overhead);

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn run_sync_demo() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create sample market data in external format
    let external_data = create_sample_data();
    println!("Created {} data points", external_data.len());

    // 2. Convert external data to NyxsOwl format
    let df = DataConverter::from_ohlcv_vec(&external_data)?;
    println!("Converted to DataFrame with shape: {:?}", df.shape());

    // 3. Create and configure a strategy
    let strategy = MovingAverageCrossover::sma_crossover(10, 20);

    println!("\nStrategy: {}", strategy.name());
    println!("Description: {}", strategy.description());
    println!("Required columns: {:?}", strategy.required_columns());
    println!("Min data points: {}", strategy.min_data_points());

    // 4. Test synchronous strategy execution
    println!("\n=== Synchronous Strategy Execution ===");
    let signals = strategy.generate_signals(&df)?;
    println!("Generated {} signals", signals.len());

    // Display first few signals
    let signal_values = signals.i32()?;
    for i in 0..5.min(signals.len()) {
        if let Some(signal_val) = signal_values.get(i) {
            let signal_type = match signal_val {
                0 => "Hold",
                1 => "Buy",
                2 => "Sell",
                _ => "Unknown",
            };
            println!("  Signal {}: {} ({})", i, signal_val, signal_type);
        }
    }

    run_common_demo_parts(&external_data, &df, &strategy)?;

    // Performance measurement (sync only)
    println!("\n=== Performance Measurement ===");
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = strategy.generate_signals(&df)?;
    }
    let sync_duration = start.elapsed();
    println!("Synchronous execution (10 runs): {:?}", sync_duration);

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn run_common_demo_parts(
    external_data: &[ExternalOHLCV],
    df: &DataFrame,
    strategy: &MovingAverageCrossover,
) -> Result<(), Box<dyn std::error::Error>> {
    // 6. Demonstrate data conversion round-trip
    println!("\n=== Data Conversion Round-trip ===");
    let converted_back = DataConverter::to_ohlcv_vec(df)?;
    println!(
        "Round-trip conversion: {} -> DataFrame -> {}",
        external_data.len(),
        converted_back.len()
    );

    // Verify data integrity
    let mut price_matches = 0;
    for (original, converted) in external_data.iter().zip(converted_back.iter()) {
        if (original.close - converted.close).abs() < 0.001 {
            price_matches += 1;
        }
    }
    println!(
        "Price accuracy: {}/{} ({:.1}%)",
        price_matches,
        external_data.len(),
        (price_matches as f64 / external_data.len() as f64) * 100.0
    );

    // 7. Demonstrate signal conversion
    println!("\n=== Signal Conversion ===");
    let sample_signals = vec![
        nyxs_owl::strategy_lib::Signal::Buy,
        nyxs_owl::strategy_lib::Signal::Hold,
        nyxs_owl::strategy_lib::Signal::Sell,
    ];

    for (i, &signal) in sample_signals.iter().enumerate() {
        let external_signal =
            SignalConverter::to_external_signal(&signal, 100.0 + i as f64, Utc::now());
        let converted_back = SignalConverter::from_external_signal(&external_signal);

        println!(
            "  Signal conversion: {:?} -> {:?} -> {:?}",
            signal, external_signal.signal_type, converted_back
        );
    }

    // 8. Strategy metadata
    println!("\n=== Strategy Metadata ===");
    let metadata = strategy.metadata();
    println!("Name: {}", metadata.name);
    println!("Version: {}", metadata.version);
    println!("Supports realtime: {}", metadata.supports_realtime);
    println!("Required columns: {:?}", metadata.required_columns);

    // 9. Configuration validation
    println!("\n=== Configuration Validation ===");
    let config = strategy.config();
    println!("Fast period: {}", config.get_int("fast_period")?);
    println!("Slow period: {}", config.get_int("slow_period")?);
    println!("MA type: {}", config.get_string("ma_type")?);

    // Test validation
    let required_params = ["fast_period", "slow_period", "ma_type"];
    match config.validate(&required_params) {
        Ok(()) => println!("✓ Configuration validation passed"),
        Err(e) => println!("✗ Configuration validation failed: {}", e),
    }

    Ok(())
}

/// Create sample market data for testing
fn create_sample_data() -> Vec<ExternalOHLCV> {
    let mut data = Vec::new();
    let base_time = Utc::now() - chrono::Duration::days(100);

    // Generate trending data with some noise
    for i in 0..100 {
        let timestamp = base_time + chrono::Duration::days(i);
        let trend = 100.0 + (i as f64 * 0.5);
        let noise = (i as f64 * 0.1).sin() * 2.0;
        let price = trend + noise;

        data.push(ExternalOHLCV {
            timestamp,
            open: price - 0.5,
            high: price + 1.0,
            low: price - 1.0,
            close: price,
            volume: 1000.0 + (i as f64 * 10.0),
        });
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_data_creation() {
        let data = create_sample_data();
        assert_eq!(data.len(), 100);

        // Verify data is properly structured
        for ohlcv in &data {
            assert!(ohlcv.high >= ohlcv.low);
            assert!(ohlcv.high >= ohlcv.open);
            assert!(ohlcv.high >= ohlcv.close);
            assert!(ohlcv.low <= ohlcv.open);
            assert!(ohlcv.low <= ohlcv.close);
            assert!(ohlcv.volume > 0.0);
        }
    }

    #[test]
    fn test_data_conversion() {
        let external_data = create_sample_data();
        let df = DataConverter::from_ohlcv_vec(&external_data).unwrap();

        assert_eq!(df.height(), external_data.len());
        assert_eq!(df.width(), 6); // timestamp, open, high, low, close, volume

        let converted_back = DataConverter::to_ohlcv_vec(&df).unwrap();
        assert_eq!(converted_back.len(), external_data.len());
    }

    #[test]
    fn test_strategy_execution() {
        let external_data = create_sample_data();
        let df = DataConverter::from_ohlcv_vec(&external_data).unwrap();
        let strategy = MovingAverageCrossover::sma_crossover(5, 10);

        let signals = strategy.generate_signals(&df).unwrap();
        assert_eq!(signals.len(), external_data.len());

        // Verify signals are valid
        let signal_values = signals.i32().unwrap();
        for i in 0..signals.len() {
            if let Some(val) = signal_values.get(i) {
                assert!(val >= 0 && val <= 2);
            }
        }
    }

    #[cfg(feature = "async-support")]
    #[tokio::test]
    async fn test_async_strategy() {
        let external_data = create_sample_data();
        let df = DataConverter::from_ohlcv_vec(&external_data).unwrap();
        let strategy = MovingAverageCrossover::sma_crossover(5, 10);
        let async_strategy = StrategyAdapter::new(strategy);

        let signals = async_strategy.generate_signals_async(&df).await.unwrap();
        assert_eq!(signals.len(), external_data.len());
    }
}
