use polars::prelude::*;
use strategy_lib::strategy::trend_following::MovingAverageCrossover;
/// # Import Styles Example
///
/// This example demonstrates the three different ways to import and use the strategy library.
// Option 1: Direct imports from the root (most concise)
use strategy_lib::{run_backtest, BacktestConfig, Strategy, StrategyConfig};

fn example_direct_imports() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 1: Direct imports from root");

    // Create sample price data
    let close = Series::new("close".into(), &[100.0, 101.0, 99.0, 102.0, 103.0]);
    let df = DataFrame::new(vec![close.into()])?;

    // Configure the strategy
    let config = StrategyConfig {
        parameters: Series::new("params".into(), [0i32]), // Simple config
    };

    // Create the strategy
    let strategy = MovingAverageCrossover::new(config);
    println!("Strategy: {}", strategy.name());

    // Run the backtest
    let results = run_backtest(&strategy, &df, BacktestConfig::default())?;
    println!("Total return: {:.2}%", results.metrics.total_return * 100.0);

    Ok(())
}

// Option 2: Using the prelude module (recommended)
use strategy_lib::prelude::*;

fn example_using_prelude() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nExample 2: Using the prelude");

    // Create sample price data
    let close = Series::new("close".into(), &[100.0, 101.0, 99.0, 102.0, 103.0]);
    let df = DataFrame::new(vec![close.into()])?;

    // Create the strategy (the prelude brings in all necessary types)
    let config = StrategyConfig {
        parameters: Series::new("params".into(), [0i32]), // Simple config
    };

    let strategy = MovingAverageCrossover::new(config);
    println!("Strategy: {}", strategy.name());

    // Run the backtest
    let results = run_backtest(&strategy, &df, BacktestConfig::default())?;
    println!("Max drawdown: {:.2}%", results.metrics.max_drawdown * 100.0);

    Ok(())
}

// Option 3: Using the nested module structure
use strategy_lib::backtest;
use strategy_lib::strategy;

fn example_using_nested_modules() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nExample 3: Using nested modules");

    // Create sample price data
    let close = Series::new("close".into(), &[100.0, 101.0, 99.0, 102.0, 103.0]);
    let df = DataFrame::new(vec![close.into()])?;

    // Create the strategy
    let config = strategy::StrategyConfig {
        parameters: Series::new("params".into(), [0i32]), // Simple config
    };

    // Using the nested module structure
    let strategy = strategy::trend_following::MovingAverageCrossover::new(config);
    println!("Strategy: {}", strategy.name());

    // Run the backtest
    let backtest_config = backtest::BacktestConfig::default();
    let results = backtest::run_backtest(&strategy, &df, backtest_config)?;
    println!("Win rate: {:.2}%", results.metrics.win_rate * 100.0);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    example_direct_imports()?;
    example_using_prelude()?;
    example_using_nested_modules()?;
    Ok(())
}
