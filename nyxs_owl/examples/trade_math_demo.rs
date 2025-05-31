//! Trade Math Demo
//!
//! This example demonstrates the trade_math module functionality
//! which includes various technical indicators and calculations.

use nyxs_owl::trade_math::{
    moving_averages::{ExponentialMovingAverage, SimpleMovingAverage},
    oscillators::RelativeStrengthIndex,
    volatility::BollingerBands,
    volume::OnBalanceVolume,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NyxsOwl Trade Math Demo ===\n");

    // Sample price data
    let prices = vec![
        100.0, 102.0, 101.5, 103.0, 104.5, 103.8, 105.2, 106.1, 105.0, 107.3, 106.8, 108.2, 107.5,
        109.1, 110.0, 108.9, 110.5, 111.2, 110.8, 112.0,
    ];

    let volumes = vec![
        1000.0, 1200.0, 900.0, 1100.0, 1300.0, 950.0, 1150.0, 1250.0, 1050.0, 1400.0, 1300.0,
        1500.0, 1200.0, 1350.0, 1600.0, 1100.0, 1450.0, 1550.0, 1200.0, 1700.0,
    ];

    println!("Sample price data: {:?}\n", &prices[..5]);

    // 1. Simple Moving Average
    println!("1. Simple Moving Average (SMA)");
    let mut sma = SimpleMovingAverage::new(5)?;

    for (i, &price) in prices.iter().enumerate() {
        sma.update(price)?;
        if i >= 4 {
            // After 5 data points
            if let Ok(sma_value) = sma.value() {
                println!(
                    "   Day {}: Price = {:.2}, SMA(5) = {:.2}",
                    i + 1,
                    price,
                    sma_value
                );
            }
        }
    }
    println!();

    // 2. Exponential Moving Average
    println!("2. Exponential Moving Average (EMA)");
    let mut ema = ExponentialMovingAverage::new(5)?;

    for (i, &price) in prices.iter().enumerate() {
        ema.update(price)?;
        if i >= 4 {
            // After 5 data points
            if let Ok(ema_value) = ema.value() {
                println!(
                    "   Day {}: Price = {:.2}, EMA(5) = {:.2}",
                    i + 1,
                    price,
                    ema_value
                );
            }
        }
    }
    println!();

    // 3. Bollinger Bands
    println!("3. Bollinger Bands");
    let mut bb = BollingerBands::new(10, 2.0)?;

    for (i, &price) in prices.iter().enumerate() {
        bb.update(price)?;
        if i >= 9 {
            // After 10 data points
            if let (Ok(middle), Ok(upper), Ok(lower)) =
                (bb.middle_band(), bb.upper_band(), bb.lower_band())
            {
                println!(
                    "   Day {}: Price = {:.2}, BB = [{:.2}, {:.2}, {:.2}]",
                    i + 1,
                    price,
                    lower,
                    middle,
                    upper
                );
            }
        }
    }
    println!();

    // 4. RSI
    println!("4. Relative Strength Index (RSI)");
    let mut rsi = RelativeStrengthIndex::new(14)?;

    for (i, &price) in prices.iter().enumerate() {
        rsi.update(price)?;
        if i >= 14 {
            // After 15 data points
            if let Ok(rsi_value) = rsi.value() {
                println!(
                    "   Day {}: Price = {:.2}, RSI(14) = {:.2}",
                    i + 1,
                    price,
                    rsi_value
                );
            }
        }
    }
    println!();

    // 5. On-Balance Volume
    println!("5. On-Balance Volume (OBV)");
    let mut obv = OnBalanceVolume::new();

    for (i, (&price, &volume)) in prices.iter().zip(volumes.iter()).enumerate() {
        obv.update(price, volume)?;
        if let Ok(obv_value) = obv.value() {
            println!(
                "   Day {}: Price = {:.2}, Volume = {:.0}, OBV = {:.0}",
                i + 1,
                price,
                volume,
                obv_value
            );
        }
    }

    println!("\n=== Demo Complete ===");
    println!("The trade_math module is working correctly!");

    Ok(())
}
