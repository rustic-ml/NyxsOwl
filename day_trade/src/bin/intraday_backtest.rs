use chrono::{Duration, TimeZone, Timelike, Utc};
use day_trade::{
    BollingerBandsStrategy, IntradayTradingStrategy, MinuteOhlcv, OhlcvData, Signal, TradeError,
    VwapStrategy,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};

// Function to generate synthetic intraday data for testing
fn generate_test_data() -> Vec<MinuteOhlcv> {
    let mut data = Vec::new();
    // Create data for a 5-day period
    for day in 0..5 {
        // Base timestamp for market open (9:30 AM)
        let base_date = Utc.with_ymd_and_hms(2023, 6, 5 + day, 9, 30, 0).unwrap();

        // Starting price randomly varies each day
        let mut price = 100.0 + (day as f64 * 2.0) + (day % 3) as f64;

        // Generate minute data for each trading day (9:30 AM to 4:00 PM = 390 minutes)
        for minute in 0..390 {
            let timestamp = base_date + Duration::minutes(minute);
            let hour = timestamp.hour();
            let min = timestamp.minute();

            // Create price patterns that mimic real market behavior:
            // - Higher volatility at open and close
            // - Midday lull
            // - Trending patterns

            let mut price_change = 0.0;
            let mut volume_multiplier = 1.0;

            // Opening volatility (9:30-10:30)
            if hour == 9 || (hour == 10 && min < 30) {
                price_change = ((minute % 15) as f64 / 15.0).sin() * 0.3;
                volume_multiplier = 1.5;
            }
            // Midday lull (11:30-1:30)
            else if (hour == 11 && min >= 30) || hour == 12 || (hour == 13 && min <= 30) {
                price_change = ((minute % 20) as f64 / 20.0).sin() * 0.1;
                volume_multiplier = 0.7;
            }
            // Closing volatility (3:00-4:00)
            else if hour >= 15 {
                price_change = ((minute % 10) as f64 / 10.0).sin() * 0.25;
                volume_multiplier = 1.3;
            }
            // Normal trading
            else {
                price_change = ((minute % 30) as f64 / 30.0).sin() * 0.2;
            }

            // Add day-specific trend
            if day % 3 == 0 {
                // Uptrend day
                price_change += 0.01;
            } else if day % 3 == 1 {
                // Downtrend day
                price_change -= 0.01;
            }
            // day % 3 == 2 is a sideways day, no additional change

            // Apply the change
            price *= 1.0 + price_change;

            // Add some randomness
            price += (minute % 5) as f64 * 0.05 - 0.125;

            // Ensure price stays positive
            price = price.max(50.0);

            // Calculate reasonable high/low prices
            let high = price * (1.0 + 0.001 * ((minute % 3) as f64));
            let low = price * (1.0 - 0.001 * ((minute % 4) as f64));

            // Calculate a realistic volume
            let base_volume = 1000u64;
            let volume = (base_volume as f64 * volume_multiplier) as u64
                + ((minute % 5) * 50) as u64
                + if minute % 15 == 0 { 500 } else { 0 };

            data.push(MinuteOhlcv {
                timestamp,
                data: OhlcvData {
                    open: price,
                    high,
                    low,
                    close: price,
                    volume,
                },
            });
        }
    }

    data
}

// Run backtest for a given strategy and data
fn run_backtest<T: IntradayTradingStrategy>(
    strategy: &T,
    data: &[MinuteOhlcv],
    name: &str,
) -> Result<(), TradeError> {
    // Generate signals
    let signals = strategy.generate_signals(data)?;

    // Calculate performance
    let performance = strategy.calculate_performance(data, &signals)?;

    // Print results
    println!("Strategy: {}", name);
    println!("  Performance: {:.2}%", performance);

    // Count signal types
    let mut signal_counts = HashMap::new();
    for signal in &signals {
        *signal_counts.entry(signal).or_insert(0) += 1;
    }

    println!("  Signal counts:");
    println!(
        "    Buy:  {}",
        signal_counts.get(&Signal::Buy).unwrap_or(&0)
    );
    println!(
        "    Sell: {}",
        signal_counts.get(&Signal::Sell).unwrap_or(&0)
    );
    println!(
        "    Hold: {}",
        signal_counts.get(&Signal::Hold).unwrap_or(&0)
    );

    // Export signals to CSV for visualization
    if let Err(e) = export_signals_to_csv(data, &signals, name) {
        println!("Warning: Failed to export signals to CSV: {}", e);
    }

    Ok(())
}

// Export signals to CSV for visualization
fn export_signals_to_csv(
    data: &[MinuteOhlcv],
    signals: &[Signal],
    strategy_name: &str,
) -> Result<(), io::Error> {
    let filename = format!(
        "{}_signals.csv",
        strategy_name.to_lowercase().replace(" ", "_")
    );
    let file = File::create(&filename)?;
    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(writer, "timestamp,open,high,low,close,volume,signal")?;

    // Write data rows
    for (i, point) in data.iter().enumerate() {
        let signal_str = match signals[i] {
            Signal::Buy => "buy",
            Signal::Sell => "sell",
            Signal::Hold => "hold",
        };

        writeln!(
            writer,
            "{},{},{},{},{},{},{}",
            point.timestamp.format("%Y-%m-%d %H:%M:%S"),
            point.data.open,
            point.data.high,
            point.data.low,
            point.data.close,
            point.data.volume,
            signal_str
        )?;
    }

    println!("  Exported signals to {}", filename);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Intraday Trading Strategy Backtest");
    println!("==================================");

    // Generate test data
    let data = generate_test_data();
    println!(
        "Generated {} minutes of test data across {} days",
        data.len(),
        data.len() / 390
    );

    // Create various strategy instances
    let vwap_mean_reversion = VwapStrategy::mean_reversion();
    let vwap_trend_following = VwapStrategy::trend_following();

    let bb_mean_reversion = BollingerBandsStrategy::mean_reversion();
    let bb_breakout = BollingerBandsStrategy::volatility_breakout();

    // Run backtests
    run_backtest(&vwap_mean_reversion, &data, "VWAP Mean Reversion")?;
    run_backtest(&vwap_trend_following, &data, "VWAP Trend Following")?;
    run_backtest(&bb_mean_reversion, &data, "Bollinger Bands Mean Reversion")?;
    run_backtest(&bb_breakout, &data, "Bollinger Bands Volatility Breakout")?;

    println!("\nBacktests complete. Results saved to CSV files for visualization.");
    println!("You can import these files into a charting tool or spreadsheet for analysis.");

    Ok(())
}
