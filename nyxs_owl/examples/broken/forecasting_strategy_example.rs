use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use day_trade::ForecastingStrategy;
use day_trade::RealtimeTradingStrategy;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

/// Helper function to load historical price data from a CSV file
fn load_price_data(file_path: &Path) -> io::Result<Vec<(DateTime<Utc>, f64, f64, f64, f64, f64)>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut data = Vec::new();

    // Skip header line
    for line in reader.lines().skip(1) {
        let line = line?;
        let fields: Vec<&str> = line.split(',').collect();

        if fields.len() >= 6 {
            // Parse timestamp (format: "2022-08-22 04:00:00 UTC")
            let timestamp_str = fields[0];
            let naive_datetime = NaiveDateTime::parse_from_str(
                timestamp_str.trim_end_matches(" UTC"),
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap_or_else(|_| NaiveDateTime::from_timestamp_opt(0, 0).unwrap());

            let timestamp = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);

            // Parse OHLCV
            let open = f64::from_str(fields[1]).unwrap_or(0.0);
            let high = f64::from_str(fields[2]).unwrap_or(0.0);
            let low = f64::from_str(fields[3]).unwrap_or(0.0);
            let close = f64::from_str(fields[4]).unwrap_or(0.0);
            let volume = f64::from_str(fields[5]).unwrap_or(0.0);

            data.push((timestamp, open, high, low, close, volume));
        }
    }

    // Sort by date (oldest first)
    data.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(data)
}

fn main() -> io::Result<()> {
    println!("Forecasting Strategy Example with AAPL Data");
    println!("===========================================\n");

    // Create the forecasting strategy
    let mut strategy = ForecastingStrategy::new().unwrap();

    // Load the AAPL price data from CSV
    let csv_path = Path::new("examples/csv/AAPL_daily_ohlcv.csv");
    let test_data = match load_price_data(&csv_path) {
        Ok(data) => data,
        Err(e) => {
            println!("Error loading CSV data: {}", e);
            return Err(e);
        }
    };

    println!("Loaded {} days of AAPL price data", test_data.len());
    println!(
        "Date range: {} to {}",
        test_data.first().unwrap().0.format("%Y-%m-%d"),
        test_data.last().unwrap().0.format("%Y-%m-%d")
    );
    println!("\nRunning strategy backtest...");

    // A simple structure to track our positions and performance
    let mut position = 0; // -1 = short, 0 = no position, 1 = long
    let mut cash = 10000.0;
    let mut shares = 0.0;
    let mut trades = 0;

    // For tracking buy & hold performance
    let initial_price = test_data[0].4; // First closing price
    let initial_shares = cash / initial_price;

    println!("\nDATE       | PRICE  | SIGNAL | POSITION | CASH    | EQUITY  | B&H EQUITY");
    println!("-----------|--------|--------|----------|---------|---------|----------");

    for (i, (timestamp, open, high, low, close, volume)) in test_data.iter().enumerate() {
        // Update the strategy with the latest candle
        strategy
            .update(*timestamp, *open, *high, *low, *close, *volume)
            .unwrap();

        // Only generate signals once we have enough data (after 15 days)
        if i < 15 {
            continue;
        }

        // Generate the trading signal
        let signal = strategy.generate_signal().unwrap();

        // Simple trading logic
        if signal >= 1 && position <= 0 {
            // Buy signal
            if position == -1 {
                // Cover short
                cash += shares * close;
                trades += 1;
            }

            // Then go long
            shares = cash / close;
            cash = 0.0;
            position = 1;
            trades += 1;
        } else if signal <= -1 && position >= 0 {
            // Sell signal
            if position == 1 {
                // Sell long
                cash += shares * close;
                trades += 1;
            }

            // Then go short
            shares = -(cash / close);
            cash = cash * 2.0; // Reserve cash for covering
            position = -1;
            trades += 1;
        }

        // Calculate current equity and buy & hold strategy
        let equity = cash + (shares * close);
        let buy_and_hold_value = initial_shares * close;

        // Print position and performance every 30 days (about monthly) and last day
        if i % 30 == 0 || i == test_data.len() - 1 {
            let position_str = match position {
                -1 => "SHORT",
                0 => "NONE",
                1 => "LONG",
                _ => "UNKNOWN",
            };

            let signal_str = match signal {
                -2 => "STRONG SELL",
                -1 => "SELL",
                0 => "HOLD",
                1 => "BUY",
                2 => "STRONG BUY",
                _ => "UNKNOWN",
            };

            println!(
                "{} | {:6.2} | {:10} | {:8} | {:7.0} | {:7.0} | {:7.0}",
                timestamp.format("%Y-%m-%d"),
                close,
                signal_str,
                position_str,
                cash,
                equity,
                buy_and_hold_value
            );
        }
    }

    // Calculate final performance
    let final_price = test_data.last().unwrap().4;
    let strategy_final_equity = cash + (shares * final_price);
    let buy_and_hold_final = 10000.0 * (final_price / initial_price);

    println!("\nPerformance Summary:");
    println!("-------------------");
    println!("Initial Capital: $10,000.00");
    println!("Final Equity: ${:.2}", strategy_final_equity);
    println!(
        "Total Return: {:.2}%",
        (strategy_final_equity - 10000.0) / 100.0
    );
    println!(
        "Buy & Hold Return: {:.2}%",
        (buy_and_hold_final - 10000.0) / 100.0
    );
    println!("Number of Trades: {}", trades);

    // Calculate outperformance
    let outperformance = strategy_final_equity - buy_and_hold_final;
    println!(
        "Outperformance vs Buy & Hold: ${:.2} ({:.2}%)",
        outperformance,
        outperformance / 10000.0 * 100.0
    );

    Ok(())
}
