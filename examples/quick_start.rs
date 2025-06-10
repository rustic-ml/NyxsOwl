//! # NyxsOwl Quick Start Example
//!
//! This example demonstrates the simplest way to get started with NyxsOwl
//! for basic technical analysis.

use nyxs_owl::prelude::*;
use nyxs_owl::trade_math::momentum::calculate_rsi;
use nyxs_owl::trade_math::moving_averages::{calculate_ema, calculate_sma};
use nyxs_owl::trade_math::volatility::calculate_bollinger_bands;
use polars::prelude::*;

fn main() -> Result<()> {
    println!("🦉 NyxsOwl Quick Start Example");
    println!("================================");

    // Sample price data (e.g., closing prices for the last 10 days)
    let prices_vec = vec![
        100.0, 102.0, 101.5, 103.0, 104.5, 106.0, 105.5, 107.0, 108.5, 107.0,
    ];

    println!("\n📊 Price Data: {:?}", prices_vec);

    // Convert to Polars Series for NyxsOwl functions
    let prices = Series::new("close".into(), &prices_vec);

    // Calculate Simple Moving Average (SMA)
    println!("\n📈 Technical Indicators:");
    println!("------------------------");

    if let Ok(sma_series) = calculate_sma(&prices, 3) {
        // Extract values to Vec for display
        let sma_values: Vec<Option<f64>> = sma_series.f64().unwrap().into_iter().collect();
        println!("SMA(3): {:?}", sma_values);
        if let Some(latest_sma) = sma_values.iter().filter_map(|&x| x).next_back() {
            println!("Latest SMA: {:.2}", latest_sma);
        }
    }

    // Calculate Exponential Moving Average (EMA) - using smoothing factor of 2.0
    if let Ok(ema_series) = calculate_ema(&prices, 3, 2.0) {
        let ema_values: Vec<Option<f64>> = ema_series.f64().unwrap().into_iter().collect();
        println!("EMA(3): {:?}", ema_values);
        if let Some(latest_ema) = ema_values.iter().filter_map(|&x| x).next_back() {
            println!("Latest EMA: {:.2}", latest_ema);
        }
    }

    // Calculate RSI (Relative Strength Index)
    if let Ok(rsi_series) = calculate_rsi(&prices, 5) {
        let rsi_values: Vec<Option<f64>> = rsi_series.f64().unwrap().into_iter().collect();
        println!("RSI(5): {:?}", rsi_values);
        if let Some(latest_rsi) = rsi_values.iter().filter_map(|&x| x).next_back() {
            println!("Latest RSI: {:.2}", latest_rsi);

            // Simple trading signal based on RSI
            if latest_rsi > 70.0 {
                println!("🔴 RSI Signal: OVERBOUGHT - Consider selling");
            } else if latest_rsi < 30.0 {
                println!("🟢 RSI Signal: OVERSOLD - Consider buying");
            } else {
                println!("🟡 RSI Signal: NEUTRAL - Hold position");
            }
        }
    }

    // Calculate Bollinger Bands
    if let Ok((upper_band, middle_band, lower_band)) = calculate_bollinger_bands(&prices, 5, 2.0) {
        let upper_values: Vec<Option<f64>> = upper_band.f64().unwrap().into_iter().collect();
        let middle_values: Vec<Option<f64>> = middle_band.f64().unwrap().into_iter().collect();
        let lower_values: Vec<Option<f64>> = lower_band.f64().unwrap().into_iter().collect();

        if let (Some(upper), Some(middle), Some(lower)) = (
            upper_values.iter().filter_map(|&x| x).next_back(),
            middle_values.iter().filter_map(|&x| x).next_back(),
            lower_values.iter().filter_map(|&x| x).next_back(),
        ) {
            println!("\n📊 Bollinger Bands (5, 2.0):");
            println!("Upper Band:  {:.2}", upper);
            println!("Middle Band: {:.2}", middle);
            println!("Lower Band:  {:.2}", lower);

            let current_price = *prices_vec.last().unwrap();
            println!("Current Price: {:.2}", current_price);

            // Simple trading signal based on Bollinger Bands
            if current_price > upper {
                println!("🔴 BB Signal: Price above upper band - Consider selling");
            } else if current_price < lower {
                println!("🟢 BB Signal: Price below lower band - Consider buying");
            } else {
                println!("🟡 BB Signal: Price within bands - Hold position");
            }
        }
    }

    // Show forecasting capability
    println!("\n🔮 Forecasting Capabilities:");
    println!("-----------------------------");
    println!("✅ ARIMA Strategy - Advanced time series forecasting");
    println!("✅ Exponential Smoothing - Trend and seasonality analysis");
    println!("✅ Kalman Filter - Dynamic state estimation");
    println!("✅ Ensemble Methods - Multiple strategy combination");
    println!("✅ GARCH Models - Volatility forecasting");
    println!("✅ Copula Analysis - Multi-asset dependency modeling");
    println!("✅ Regime Switching - Market state detection");

    println!("\n✅ Quick start completed successfully!");
    println!("💡 Try running other examples to see forecasting in action:");
    println!("   cargo run --example arima_strategy_example --features forecasting");

    Ok(())
}
