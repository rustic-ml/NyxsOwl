use nyxs_owl::prelude::*;
use polars::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Load sample OHLCV data
    let df = CsvReader::from_path("examples/csv/AAPL_daily_ohlcv.csv")?
        .infer_schema(None)
        .has_header(true)
        .finish()?;

    // Extract OHLCV columns
    let high = df.column("High")?;
    let low = df.column("Low")?;
    let close = df.column("Close")?;

    // Calculate Chandelier Exit with standard parameters
    let (long_exit, short_exit) = calculate_chandelier_exit(
        high,
        low,
        close,
        22,  // period
        22,  // ATR period
        3.0, // multiplier
    )?;

    // Create a new DataFrame with results
    let mut result_df = df.clone();
    result_df.with_column(long_exit.rename("chandelier_exit_long"))?;
    result_df.with_column(short_exit.rename("chandelier_exit_short"))?;

    // Print the last 10 rows to see the results
    println!("Last 10 rows of data with Chandelier Exit values:");
    println!("{}", result_df.tail(Some(10)));

    // Example of how to use Chandelier Exit for trading signals
    let close_values: Vec<Option<f64>> = close.f64()?.into_iter().collect();
    let long_values: Vec<Option<f64>> = long_exit.f64()?.into_iter().collect();
    let short_values: Vec<Option<f64>> = short_exit.f64()?.into_iter().collect();

    println!("\nAnalyzing trading signals...");
    for i in (22..close_values.len()).rev().take(5) {
        if let (Some(close_val), Some(long_val), Some(short_val)) = 
            (close_values[i], long_values[i], short_values[i]) {
            
            println!("Date: Row {}", i);
            println!("Close: {:.2}", close_val);
            println!("Long Exit: {:.2}", long_val);
            println!("Short Exit: {:.2}", short_val);
            
            if close_val > short_val {
                println!("Signal: Bullish (Price above Short Exit)");
            } else if close_val < long_val {
                println!("Signal: Bearish (Price below Long Exit)");
            } else {
                println!("Signal: Neutral");
            }
            println!();
        }
    }

    Ok(())
} 