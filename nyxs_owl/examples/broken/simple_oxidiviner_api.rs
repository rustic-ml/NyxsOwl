//! Simple OxiDiviner API Example
//!
//! This example demonstrates how to use the simplified OxiDiviner API
//! for quick forecasting without dealing with complex adapter patterns.

use chrono::{DateTime, TimeZone, Utc};
use forecast_trade::error::Result;
use forecast_trade::models::oxidiviner::easy;

fn main() -> Result<()> {
    println!("=== Simple OxiDiviner API Example ===\n");

    // Create sample time series data
    let dates: Vec<DateTime<Utc>> = (0..30)
        .map(|i| Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap() + chrono::Duration::days(i))
        .collect();

    // Generate some sample price data with trend and noise
    let values: Vec<f64> = (0..30)
        .map(|i| 100.0 + (i as f64) * 0.5 + (i as f64 * 0.1).sin() * 5.0)
        .collect();

    println!("Sample data created: {} data points", values.len());
    println!(
        "Price range: {:.2} - {:.2}",
        values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    );

    // 1. Quick ARIMA forecast
    println!("\n--- ARIMA Forecast ---");
    match easy::arima_forecast(dates.clone(), values.clone(), 5) {
        Ok(forecast) => {
            println!("ARIMA forecast for next 5 periods: {:?}", forecast);
        }
        Err(e) => println!("ARIMA forecast failed: {}", e),
    }

    // 2. Quick Moving Average forecast
    println!("\n--- Moving Average Forecast ---");
    match easy::ma_forecast(dates.clone(), values.clone(), 5, Some(7)) {
        Ok(forecast) => {
            println!("MA(7) forecast for next 5 periods: {:?}", forecast);
        }
        Err(e) => println!("MA forecast failed: {}", e),
    }

    // 3. Quick Exponential Smoothing forecast
    println!("\n--- Exponential Smoothing Forecast ---");
    match easy::es_forecast(dates.clone(), values.clone(), 5, Some(0.3)) {
        Ok(forecast) => {
            println!("ES(α=0.3) forecast for next 5 periods: {:?}", forecast);
        }
        Err(e) => println!("ES forecast failed: {}", e),
    }

    // 4. Automatic model selection
    println!("\n--- Automatic Model Selection ---");
    match easy::auto_forecast(dates.clone(), values.clone(), 5) {
        Ok((forecast, model_name)) => {
            println!("Best model: {}", model_name);
            println!("Auto forecast for next 5 periods: {:?}", forecast);
        }
        Err(e) => println!("Auto forecast failed: {}", e),
    }

    // 5. Compare multiple forecasts
    println!("\n--- Model Comparison ---");
    let models = vec![
        (
            "ARIMA",
            easy::arima_forecast(dates.clone(), values.clone(), 3),
        ),
        (
            "MA(5)",
            easy::ma_forecast(dates.clone(), values.clone(), 3, Some(5)),
        ),
        (
            "ES",
            easy::es_forecast(dates.clone(), values.clone(), 3, Some(0.2)),
        ),
    ];

    for (name, result) in models {
        match result {
            Ok(forecast) => println!("{}: {:?}", name, forecast),
            Err(e) => println!("{}: Failed - {}", name, e),
        }
    }

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
