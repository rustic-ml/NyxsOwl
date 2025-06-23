//! Comprehensive Technical Indicators Demo
//!
//! This example demonstrates the complete suite of technical indicators available in NyxsOwl.

use nyxs_owl::trade_math::{
    calculate_cci, calculate_mfi, calculate_roc,
    calculate_vroc, calculate_vwap_with_bands, calculate_adl, calculate_cmf,
    calculate_supertrend,
    calculate_fibonacci_retracements, calculate_fibonacci_extensions,
};
use polars::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Comprehensive Technical Indicators Demo");
    println!("==========================================");
    
    // Create sample market data
    let market_data = create_sample_market_data()?;
    println!("📊 Created sample market data with {} data points", market_data.height());
    
    // Demonstrate Advanced Oscillators
    demonstrate_advanced_oscillators(&market_data)?;
    
    // Demonstrate Volume-Based Indicators
    demonstrate_volume_indicators(&market_data)?;
    
    // Demonstrate Advanced Trend Indicators
    demonstrate_trend_indicators(&market_data)?;
    
    // Demonstrate Pattern Recognition
    demonstrate_pattern_recognition(&market_data)?;
    
    println!("\n✅ All demonstrations completed successfully!");
    Ok(())
}

fn create_sample_market_data() -> Result<DataFrame, Box<dyn std::error::Error>> {
    let n_points = 100;
    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    let mut volume = Vec::new();
    
    let mut price = 100.0;
    for i in 0..n_points {
        let trend = (i as f64 * 0.1).sin() * 2.0;
        let volatility = 1.0 + (i as f64 * 0.05).sin() * 0.5;
        
        let change = trend * volatility;
        price += change;
        
        let o = price;
        let h = price + volatility * 0.5;
        let l = price - volatility * 0.5;
        let c = price + change * 0.3;
        let v = 1000.0 + (i as f64 * 10.0).sin() * 200.0;
        
        open.push(o);
        high.push(h);
        low.push(l);
        close.push(c);
        volume.push(v);
    }
    
    let df = df! {
        "open" => open,
        "high" => high,
        "low" => low,
        "close" => close,
        "volume" => volume,
    }?;
    
    Ok(df)
}

fn demonstrate_advanced_oscillators(data: &DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📈 Advanced Oscillators Demo");
    println!("---------------------------");
    
    let high = data.column("high")?.as_series().expect("Failed to get high series");
    let low = data.column("low")?.as_series().expect("Failed to get low series");
    let close = data.column("close")?.as_series().expect("Failed to get close series");
    let volume = data.column("volume")?.as_series().expect("Failed to get volume series");
    
    // CCI (Commodity Channel Index)
    let cci = calculate_cci(&high, &low, &close, 20)?;
    println!("✅ CCI calculated - Latest value: {:.2}", 
        cci.f64()?.get(cci.len() - 1).unwrap_or(0.0));
    
    // MFI (Money Flow Index)
    let mfi = calculate_mfi(&high, &low, &close, &volume, 14)?;
    println!("✅ MFI calculated - Latest value: {:.2}", 
        mfi.f64()?.get(mfi.len() - 1).unwrap_or(0.0));
    
    // ROC (Rate of Change)
    let roc = calculate_roc(&close, 10)?;
    println!("✅ ROC calculated - Latest value: {:.2}%", 
        roc.f64()?.get(roc.len() - 1).unwrap_or(0.0));
    
    Ok(())
}

fn demonstrate_volume_indicators(data: &DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Volume-Based Indicators Demo");
    println!("-------------------------------");
    
    let high = data.column("high")?.as_series().expect("Failed to get high series");
    let low = data.column("low")?.as_series().expect("Failed to get low series");
    let close = data.column("close")?.as_series().expect("Failed to get close series");
    let volume = data.column("volume")?.as_series().expect("Failed to get volume series");
    
    // VROC (Volume Rate of Change)
    let vroc = calculate_vroc(&volume, 25)?;
    println!("✅ VROC calculated - Latest value: {:.2}%", 
        vroc.f64()?.get(vroc.len() - 1).unwrap_or(0.0));
    
    // VWAP with Bands
    let (vwap, vwap_upper, vwap_lower) = calculate_vwap_with_bands(data, 20, 2.0)?;
    println!("✅ VWAP with bands calculated");
    println!("   VWAP: {:.2}", vwap.f64()?.get(vwap.len() - 1).unwrap_or(0.0));
    println!("   Upper Band: {:.2}", vwap_upper.f64()?.get(vwap_upper.len() - 1).unwrap_or(0.0));
    println!("   Lower Band: {:.2}", vwap_lower.f64()?.get(vwap_lower.len() - 1).unwrap_or(0.0));
    
    // ADL (Accumulation/Distribution Line)
    let adl = calculate_adl(&high, &low, &close, &volume)?;
    println!("✅ ADL calculated - Latest value: {:.2}", 
        adl.f64()?.get(adl.len() - 1).unwrap_or(0.0));
    
    // CMF (Chaikin Money Flow)
    let cmf = calculate_cmf(&high, &low, &close, &volume, 20)?;
    println!("✅ CMF calculated - Latest value: {:.3}", 
        cmf.f64()?.get(cmf.len() - 1).unwrap_or(0.0));
    
    Ok(())
}

fn demonstrate_trend_indicators(data: &DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📈 Advanced Trend Indicators Demo");
    println!("----------------------------------");
    
    let high = data.column("high")?.as_series().expect("Failed to get high series");
    let low = data.column("low")?.as_series().expect("Failed to get low series");
    let close = data.column("close")?.as_series().expect("Failed to get close series");
    
    // SuperTrend
    let (supertrend, trend_direction) = calculate_supertrend(&high, &low, &close, 10, 3.0)?;
    println!("✅ SuperTrend calculated");
    println!("   SuperTrend: {:.2}", supertrend.f64()?.get(supertrend.len() - 1).unwrap_or(0.0));
    println!("   Trend Direction: {:.0}", trend_direction.f64()?.get(trend_direction.len() - 1).unwrap_or(0.0));
    
    Ok(())
}

fn demonstrate_pattern_recognition(data: &DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Pattern Recognition Demo");
    println!("---------------------------");
    
    let high = data.column("high")?.as_series().expect("Failed to get high series");
    let low = data.column("low")?.as_series().expect("Failed to get low series");
    
    // Fibonacci Retracements
    let swing_high = high.f64()?.into_no_null_iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
    let swing_low = low.f64()?.into_no_null_iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
    
    let fib_retracements = calculate_fibonacci_retracements(swing_high, swing_low)?;
    println!("✅ Fibonacci Retracements calculated");
    println!("   Swing High: {:.2}, Swing Low: {:.2}", swing_high, swing_low);
    for (level, price) in fib_retracements.iter() {
        println!("   {}: {:.2}", level, price);
    }
    
    // Fibonacci Extensions
    let extension_levels = vec![1.272, 1.618, 2.0, 2.618];
    let fib_extensions = calculate_fibonacci_extensions(swing_high, swing_low, &extension_levels)?;
    println!("✅ Fibonacci Extensions calculated");
    for (level, price) in fib_extensions.iter() {
        println!("   {}: {:.2}", level, price);
    }
    
    Ok(())
} 