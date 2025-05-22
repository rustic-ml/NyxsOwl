# NyxsOwl Strategy Library Documentation

Welcome to the documentation for the NyxsOwl Strategy Library, a comprehensive framework for creating, testing, and implementing trading strategies based on technical indicators.

## Documentation Index

- [User Guide](guide.md) - Comprehensive guide for using the library
- [API Reference](#api-reference) - Reference for all library components
- [Examples](examples/) - Example code demonstrating various strategies

## Quick Start

To get started with the Strategy Library, add it to your dependencies in Cargo.toml:

```toml
[dependencies]
strategy_lib = { path = "/path/to/NyxsOwl/strategy_lib" }
```

Here's a simple example of creating and testing a Moving Average Crossover strategy:

```rust
use polars::prelude::*;
use strategy_lib::strategy::{Strategy, StrategyConfig};
use strategy_lib::strategy::trend_following::MovingAverageCrossover;
use strategy_lib::backtest::{run_backtest, BacktestConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load or create price data
    let df = DataFrame::new(vec![
        // Your price data here
    ])?;
    
    // Configure the strategy
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
    
    // Run the backtest
    let results = run_backtest(&strategy, &df, BacktestConfig::default())?;
    
    // Print results
    println!("Total Return: {:.2}%", results.metrics.total_return * 100.0);
    
    Ok(())
}
```

## API Reference

The Strategy Library is organized into the following main components:

### Strategy Module

- **Strategy Trait**: The core interface for all trading strategies
- **Strategy Categories**: Organized into trend following, mean reversion, momentum, volatility, volume, and multi-indicator strategies
- **Strategy Signals**: Buy, Sell, and Hold signals

### Backtest Module

- **Backtest Configuration**: Parameters for configuring backtest runs
- **Backtest Results**: Results and metrics from running a backtest
- **Backtest Engine**: Framework for testing strategies on historical data

### Utils Module

- **Signal Processing**: Functions for processing and combining signals
- **Technical Analysis**: Additional technical analysis functions
- **Data Handling**: Functions for handling price data

## Contributing

The Strategy Library is designed to be extended with new strategies and features. If you'd like to contribute:

1. Fork the repository
2. Create a new branch for your feature
3. Add your changes with tests and documentation
4. Submit a pull request

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 