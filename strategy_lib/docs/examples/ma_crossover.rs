use polars::prelude::*;
use strategy_lib::strategy::{Strategy, StrategyConfig};
use strategy_lib::strategy::trend_following::MovingAverageCrossover;
use strategy_lib::backtest::{run_backtest, BacktestConfig};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Step 1: Load or create price data
    // In a real application, you would load historical price data from a CSV or database
    let dates = create_date_range("2023-01-01", 60)?;
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
    
    // Create a complete DataFrame with OHLCV data
    let df = create_ohlcv_dataframe(dates, close_prices)?;
    
    // Step 2: Configure the strategy
    // Create configuration with different parameter sets to compare
    let configs = vec![
        ("Fast SMA (5,20)", create_strategy_config("sma", 5, 20, "close")),
        ("Medium SMA (10,30)", create_strategy_config("sma", 10, 30, "close")),
        ("Fast EMA (5,20)", create_strategy_config("ema", 5, 20, "close")),
        ("Medium EMA (10,30)", create_strategy_config("ema", 10, 30, "close")),
    ];
    
    // Step 3: Backtest each strategy configuration
    println!("{:<20} {:<15} {:<15} {:<15} {:<15}", 
             "Strategy", "Total Return", "Max Drawdown", "Sharpe Ratio", "Win Rate");
    println!("{:-<80}", "");
    
    for (name, config) in configs {
        // Create the strategy
        let strategy = MovingAverageCrossover::new(config);
        
        // Configure the backtest
        let backtest_config = BacktestConfig {
            initial_capital: 10000.0,
            commission: 0.001,   // 0.1%
            slippage: 0.0005,    // 0.05%
            position_size: 0.1,  // 10% of capital per trade
        };
        
        // Run the backtest
        let results = run_backtest(&strategy, &df, backtest_config)?;
        
        // Print the results
        println!("{:<20} {:<15.2}% {:<15.2}% {:<15.2} {:<15.2}%",
                 name,
                 results.metrics.total_return * 100.0,
                 results.metrics.max_drawdown * 100.0,
                 results.metrics.sharpe_ratio,
                 results.metrics.win_rate * 100.0);
    }
    
    // Step 4: Analyze one strategy in detail
    println!("\nDetailed Analysis of Medium EMA (10,30):");
    let medium_ema_config = create_strategy_config("ema", 10, 30, "close");
    let strategy = MovingAverageCrossover::new(medium_ema_config);
    
    // Generate signals
    let signals = strategy.generate_signals(&df)?;
    
    // Create a DataFrame with the signals for analysis
    let mut result_df = df.clone();
    result_df.with_column(signals)?;
    
    // Print some analysis
    println!("Strategy: {}", strategy.name());
    println!("Description: {}", strategy.description());
    println!("First 5 signals:");
    println!("{}", result_df.select(["date", "close", "signal"])?.head(Some(5)));
    
    Ok(())
}

/// Helper function to create a date range Series
fn create_date_range(start_date: &str, days: usize) -> Result<Series, Box<dyn Error>> {
    let start = Date32::from_str(start_date)?;
    let mut dates = Vec::with_capacity(days);
    
    for i in 0..days {
        let days_to_add = Duration::new(i as i64 * 24 * 60 * 60 * 1_000_000_000, TimeUnit::Nanoseconds);
        dates.push(start + days_to_add);
    }
    
    Ok(Series::new("date", dates))
}

/// Helper function to create a complete OHLCV DataFrame from close prices
fn create_ohlcv_dataframe(dates: Series, close_prices: Series) -> Result<DataFrame, Box<dyn Error>> {
    // Create synthetic open, high, low prices based on close
    let close_f64 = close_prices.f64()?;
    let mut open = Vec::with_capacity(close_f64.len());
    let mut high = Vec::with_capacity(close_f64.len());
    let mut low = Vec::with_capacity(close_f64.len());
    
    for (i, &close) in close_f64.into_iter().collect::<Vec<_>>().iter().enumerate() {
        let close_val = close.unwrap_or(0.0);
        
        // Open price is previous close or 0.5% different from close
        let open_val = if i > 0 {
            close_f64.get(i - 1).unwrap_or(Some(close_val * 0.995)).unwrap_or(close_val * 0.995)
        } else {
            close_val * 0.995
        };
        
        // High is 0.5% above the max of open and close
        let high_val = f64::max(open_val, close_val) * 1.005;
        
        // Low is 0.5% below the min of open and close
        let low_val = f64::min(open_val, close_val) * 0.995;
        
        open.push(Some(open_val));
        high.push(Some(high_val));
        low.push(Some(low_val));
    }
    
    // Create volume data (just a placeholder with constant volume)
    let volume = Series::new("volume", vec![100000; close_f64.len()]);
    
    // Create the DataFrame
    DataFrame::new(vec![
        dates,
        Series::new("open", open),
        Series::new("high", high),
        Series::new("low", low),
        close_prices,
        volume,
    ]).map_err(|e| Box::new(e) as Box<dyn Error>)
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