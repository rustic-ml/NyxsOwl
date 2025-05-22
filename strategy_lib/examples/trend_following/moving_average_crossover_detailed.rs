//! # Moving Average Crossover Strategy Example
//! 
//! This example demonstrates the Moving Average Crossover strategy with real-world data.
//! It shows how to:
//! 
//! - Load historical price data from a CSV file
//! - Configure the strategy with different parameters
//! - Backtest the strategy
//! - Analyze and visualize the results

use polars::prelude::*;
use std::fs::File;
use std::path::Path;
use strategy_lib::strategy::{Strategy, StrategyConfig};
use strategy_lib::strategy::trend_following::MovingAverageCrossover;
use strategy_lib::backtest::{run_backtest, BacktestConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Moving Average Crossover Strategy Example");
    println!("=========================================\n");
    
    // Create sample price data - in a real application, you would load this from a file
    let dates = date_range_vec("2023-01-01", 60, Duration::parse("1d")?)?;
    let dates = Series::new("date", dates);
    
    let close_prices = Series::new(
        "close",
        &[
            100.0, 101.0, 102.0, 103.0, 105.0, 104.0, 106.0, 107.0, 109.0, 108.0,
            107.0, 109.0, 111.0, 114.0, 113.0, 116.0, 119.0, 120.0, 119.0, 117.0,
            118.0, 120.0, 123.0, 122.0, 120.0, 118.0, 119.0, 121.0, 124.0, 125.0,
            127.0, 129.0, 130.0, 132.0, 129.0, 128.0, 127.0, 126.0, 128.0, 130.0,
            133.0, 135.0, 137.0, 140.0, 139.0, 137.0, 136.0, 138.0, 140.0, 142.0,
            144.0, 145.0, 147.0, 149.0, 150.0, 152.0, 153.0, 155.0, 154.0, 156.0,
        ],
    );
    
    // Create a DataFrame with price data
    let mut df = DataFrame::new(vec![dates, close_prices.clone()])?;
    
    // Add high, low, and volume columns for completeness
    let high = close_prices.clone() * 1.01;  // 1% higher than close
    let low = close_prices.clone() * 0.99;   // 1% lower than close
    let volume = Series::new("volume", vec![100000; 60]);  // Constant volume for simplicity
    
    df.with_column(high.rename("high"))?;
    df.with_column(low.rename("low"))?;
    df.with_column(volume)?;
    
    println!("Data Sample:");
    println!("{}", df.head(Some(5)));
    println!();

    // Set up multiple configurations to compare
    let configs = vec![
        ("Fast SMA", create_strategy_config("sma", 5, 20, "close")),
        ("Medium SMA", create_strategy_config("sma", 10, 30, "close")),
        ("Slow SMA", create_strategy_config("sma", 20, 50, "close")),
        ("Fast EMA", create_strategy_config("ema", 5, 20, "close")),
        ("Medium EMA", create_strategy_config("ema", 10, 30, "close")),
        ("Slow EMA", create_strategy_config("ema", 20, 50, "close")),
    ];
    
    // Standard backtest configuration
    let backtest_config = BacktestConfig {
        initial_capital: 10000.0,
        commission: 0.001,   // 0.1%
        slippage: 0.0005,    // 0.05%
        position_size: 0.1,  // 10%
    };
    
    println!("Backtest Results:");
    println!("{:<15} {:<15} {:<15} {:<15} {:<15}", 
             "Strategy", "Total Return", "Max Drawdown", "Sharpe Ratio", "Win Rate");
    println!("{:-<75}", "");
    
    // Run backtests for each configuration
    for (name, config) in configs {
        let strategy = MovingAverageCrossover::new(config);
        let results = run_backtest(&strategy, &df, backtest_config.clone())?;
        
        // Print results
        println!("{:<15} {:<15.2}% {:<15.2}% {:<15.2} {:<15.2}%",
                 name,
                 results.metrics.total_return * 100.0,
                 results.metrics.max_drawdown * 100.0,
                 results.metrics.sharpe_ratio,
                 results.metrics.win_rate * 100.0);
    }
    
    println!("\nDetailed Strategy Information:");
    // Create one specific strategy for detailed info
    let medium_ema_config = create_strategy_config("ema", 10, 30, "close");
    let strategy = MovingAverageCrossover::new(medium_ema_config);
    
    println!("Strategy: {}", strategy.name());
    println!("Description: {}", strategy.description());
    println!("Required columns: {:?}", strategy.required_columns());
    
    Ok(())
}

/// Helper function to create date range vectors
fn date_range_vec(start_date: &str, days: usize, duration: Duration) -> Result<Vec<Date32>, PolarsError> {
    let start = Date32::from_str(start_date)?;
    let mut dates = Vec::with_capacity(days);
    
    for i in 0..days {
        dates.push(start + Duration::new((i as i64) * duration.nanoseconds(), TimeUnit::Nanoseconds));
    }
    
    Ok(dates)
}

/// Helper function to create strategy configurations
fn create_strategy_config(ma_type: &str, fast_period: u32, slow_period: u32, price_col: &str) -> StrategyConfig {
    let parameters = StructChunked::new(
        "params",
        &[
            Series::new("fast_period", [fast_period]),
            Series::new("slow_period", [slow_period]),
            Series::new("ma_type", [ma_type]),
            Series::new("price_col", [price_col]),
        ],
    ).unwrap();
    
    StrategyConfig {
        parameters: parameters.into_series(),
    }
} 