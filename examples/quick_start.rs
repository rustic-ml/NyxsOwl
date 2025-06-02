//! # NyxsOwl Quick Start Example
//!
//! This example demonstrates the simplest way to get started with NyxsOwl
//! for basic technical analysis.

use nyxs_owl::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦉 NyxsOwl Quick Start Example");
    println!("================================");

    // Sample price data (e.g., closing prices for the last 10 days)
    let prices = vec![
        100.0, 102.0, 101.5, 103.0, 104.5,
        106.0, 105.5, 107.0, 108.5, 107.0
    ];

    println!("\n📊 Price Data: {:?}", prices);

    // Calculate Simple Moving Average (SMA)
    println!("\n📈 Technical Indicators:");
    println!("------------------------");

    if let Ok(sma_values) = sma(&prices, 3) {
        println!("SMA(3): {:?}", sma_values);
        println!("Latest SMA: {:.2}", sma_values.last().unwrap_or(&0.0));
    }

    // Calculate Exponential Moving Average (EMA)
    if let Ok(ema_values) = ema(&prices, 3) {
        println!("EMA(3): {:?}", ema_values);
        println!("Latest EMA: {:.2}", ema_values.last().unwrap_or(&0.0));
    }

    // Calculate RSI (Relative Strength Index)
    if let Ok(rsi_values) = rsi(&prices, 5) {
        println!("RSI(5): {:?}", rsi_values);
        let latest_rsi = rsi_values.last().unwrap_or(&50.0);
        println!("Latest RSI: {:.2}", latest_rsi);
        
        // Simple trading signal based on RSI
        if *latest_rsi > 70.0 {
            println!("🔴 RSI Signal: OVERBOUGHT - Consider selling");
        } else if *latest_rsi < 30.0 {
            println!("🟢 RSI Signal: OVERSOLD - Consider buying");
        } else {
            println!("🟡 RSI Signal: NEUTRAL - Hold position");
        }
    }

    // Calculate Bollinger Bands
    if let Ok(bb) = bollinger_bands(&prices, 5, 2.0) {
        if let (Some(upper), Some(middle), Some(lower)) = 
            (bb.upper.last(), bb.middle.last(), bb.lower.last()) {
            println!("\n📊 Bollinger Bands (5, 2.0):");
            println!("Upper Band:  {:.2}", upper);
            println!("Middle Band: {:.2}", middle);
            println!("Lower Band:  {:.2}", lower);
            
            let current_price = *prices.last().unwrap();
            println!("Current Price: {:.2}", current_price);
            
            // Simple trading signal based on Bollinger Bands
            if current_price > *upper {
                println!("🔴 BB Signal: Price above upper band - Consider selling");
            } else if current_price < *lower {
                println!("🟢 BB Signal: Price below lower band - Consider buying");
            } else {
                println!("🟡 BB Signal: Price within bands - Hold position");
            }
        }
    }

    println!("\n✅ Quick start completed successfully!");
    println!("💡 Try modifying the price data and periods to see different results.");

    Ok(())
} 