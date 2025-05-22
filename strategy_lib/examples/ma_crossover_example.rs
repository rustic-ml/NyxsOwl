use polars::prelude::*;
use strategy_lib::strategy::{Strategy, StrategyConfig};
use strategy_lib::strategy::trend_following::MovingAverageCrossover;
use strategy_lib::backtest::{run_backtest, BacktestConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample price data
    let dates = date_range(
        "2023-01-01", 
        "2023-03-01", 
        Duration::parse("1d")
    )?;
    
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
    let df = DataFrame::new(vec![dates, close_prices.clone()])?;
    
    // Create strategy configuration
    // In a real application, these parameters would come from user input or config file
    let parameters = StructChunked::new(
        "params",
        &[
            Series::new("fast_period", [10u32]),
            Series::new("slow_period", [30u32]),
            Series::new("ma_type", ["ema"]),
            Series::new("price_col", ["close"]),
        ],
    )?;
    
    let strategy_config = StrategyConfig {
        parameters: parameters.into_series(),
    };
    
    // Create the strategy
    let strategy = MovingAverageCrossover::new(strategy_config);
    
    // Configure and run the backtest
    let backtest_config = BacktestConfig {
        initial_capital: 10000.0,
        commission: 0.001,   // 0.1%
        slippage: 0.0005,    // 0.05%
        position_size: 0.1,  // 10%
    };
    
    let results = run_backtest(&strategy, &df, backtest_config)?;
    
    // Print results
    println!("Strategy: {}", strategy.name());
    println!("Description: {}", strategy.description());
    println!("Total Return: {:.2}%", results.metrics.total_return * 100.0);
    println!("Max Drawdown: {:.2}%", results.metrics.max_drawdown * 100.0);
    println!("Sharpe Ratio: {:.2}", results.metrics.sharpe_ratio);
    println!("Win Rate: {:.2}%", results.metrics.win_rate * 100.0);
    
    Ok(())
} 