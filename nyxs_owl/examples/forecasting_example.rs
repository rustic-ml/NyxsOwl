//! # Forecasting Example
//!
//! This example demonstrates the forecasting capabilities of NyxsOwl using OxiDiviner.
//! Shows how to perform time series forecasting for financial data.

use chrono::{DateTime, Duration, Utc};
use nyxs_owl::forecast_trade::easy::*;
use nyxs_owl::forecast_trade::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦉 NyxsOwl Forecasting Example");
    println!("==================================");

    // Generate sample financial data (e.g., stock prices)
    let start_date = Utc::now() - Duration::days(60);
    let timestamps: Vec<DateTime<Utc>> = (0..60)
        .map(|i| start_date + Duration::days(i as i64))
        .collect();

    // Create realistic stock price data with trend and volatility
    let mut prices = Vec::new();
    let base_price = 100.0;
    for i in 0..60 {
        let trend = i as f64 * 0.2; // Upward trend
        let volatility = (i as f64 * 0.1).sin() * 3.0; // Some volatility
        let noise = (i as f64 * 0.05).cos() * 1.5; // Additional noise
        prices.push(base_price + trend + volatility + noise);
    }

    println!("\n📊 Sample Data:");
    println!(
        "Generated {} price points from {} days ago to today",
        prices.len(),
        60
    );
    println!(
        "Price range: ${:.2} - ${:.2}",
        prices.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    );

    // 1. Quick Financial Forecast (automatic model selection)
    println!("\n🚀 1. Auto Financial Forecast:");
    match financial_forecast(&prices, 10) {
        Ok((forecast, model_name)) => {
            println!("   ✅ Model: {}", model_name);
            println!(
                "   📈 10-day forecast: ${:.2} - ${:.2}",
                forecast.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                forecast.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
            );
            for (i, &price) in forecast.iter().enumerate() {
                println!("      Day {}: ${:.2}", i + 1, price);
            }
        }
        Err(e) => println!("   ❌ Auto forecast failed: {}", e),
    }

    // 2. Manual forecasting methods
    println!("\n🔬 2. Individual Forecasting Methods:");

    // Moving Average
    println!("\n   📊 Moving Average:");
    match forecast_moving_average(&prices, 10, 5) {
        Ok(forecast) => {
            println!(
                "      ✅ 5-day MA forecast: {:?}",
                forecast
                    .iter()
                    .map(|&x| format!("${:.2}", x))
                    .collect::<Vec<_>>()
            );
        }
        Err(e) => println!("      ❌ MA failed: {}", e),
    }

    // Exponential Smoothing
    println!("\n   📈 Exponential Smoothing (α=0.3):");
    match forecast_exponential_smoothing(&prices, 0.3, 5) {
        Ok(forecast) => {
            println!(
                "      ✅ 5-day ES forecast: {:?}",
                forecast
                    .iter()
                    .map(|&x| format!("${:.2}", x))
                    .collect::<Vec<_>>()
            );
        }
        Err(e) => println!("      ❌ ES failed: {}", e),
    }

    // ARIMA
    println!("\n   🎯 ARIMA(1,1,1):");
    match forecast_arima(&prices, (1, 1, 1), 5) {
        Ok(forecast) => {
            println!(
                "      ✅ 5-day ARIMA forecast: {:?}",
                forecast
                    .iter()
                    .map(|&x| format!("${:.2}", x))
                    .collect::<Vec<_>>()
            );
        }
        Err(e) => println!("      ❌ ARIMA failed: {}", e),
    }

    // 3. Model Comparison
    println!("\n⚖️  3. Model Comparison:");
    match model_comparison(&timestamps, &prices, 7) {
        Ok(results) => {
            println!("   Successfully compared {} models:", results.len());
            for (model_name, forecast) in results {
                let avg_price = forecast.iter().sum::<f64>() / forecast.len() as f64;
                println!(
                    "      {} - Avg 7-day forecast: ${:.2}",
                    model_name, avg_price
                );
            }
        }
        Err(e) => println!("   ❌ Model comparison failed: {}", e),
    }

    // 4. Using Easy API with timestamps
    println!("\n🎯 4. Easy API with Timestamps:");
    match auto_forecast(&timestamps, &prices, 5) {
        Ok((forecast, model_name)) => {
            println!("   ✅ Selected Model: {}", model_name);
            println!("   📅 5-day detailed forecast:");
            let forecast_start = *timestamps.last().unwrap() + Duration::days(1);
            for (i, &price) in forecast.iter().enumerate() {
                let forecast_date = forecast_start + Duration::days(i as i64);
                println!("      {} - ${:.2}", forecast_date.format("%Y-%m-%d"), price);
            }
        }
        Err(e) => println!("   ❌ Easy API failed: {}", e),
    }

    // 5. Time Series Data Structure
    println!("\n📋 5. Time Series Data Structure:");
    match TimeSeriesData::new(timestamps.clone(), prices.clone()) {
        Ok(ts_data) => {
            println!("   ✅ Time series created successfully");
            println!("   📊 Length: {} data points", ts_data.len());
            println!(
                "   📈 Last value: ${:.2}",
                ts_data.last_value().unwrap_or(0.0)
            );
            println!(
                "   📉 Recent 5 values: {:?}",
                ts_data
                    .recent_values(5)
                    .iter()
                    .map(|&x| format!("${:.2}", x))
                    .collect::<Vec<_>>()
            );
        }
        Err(e) => println!("   ❌ Time series creation failed: {}", e),
    }

    println!("\n🎉 Forecasting example completed!");
    println!("\n💡 Note: Forecasting accuracy depends on data quality and market conditions.");
    println!("   Always combine multiple models and validate results before making decisions.");

    Ok(())
}
