# NyxsOwl Strategy Library

A library of trading strategies built on top of the RusTaLib technical indicators library. This library is a subcrate within the NyxsOwl workspace.

## Features

- **Strategy Interface**: Common interface for all trading strategies
- **Backtesting Engine**: Framework for testing strategies on historical data
- **Predefined Strategies**: Ready-to-use strategies based on technical indicators
- **Utility Functions**: Helper functions for strategy development

## Strategy Categories

The library organizes strategies into several categories:

- **Trend-Following Strategies**: Strategies that aim to capture market trends
- **Mean-Reversion Strategies**: Strategies that capitalize on price returning to the mean
- **Momentum Strategies**: Strategies that trade based on market momentum
- **Volatility Strategies**: Strategies that trade based on market volatility
- **Volume Strategies**: Strategies that use volume as a primary signal
- **Multi-Indicator Strategies**: Strategies that combine multiple indicators

## Current Implementation

- [x] Strategy trait definition
- [x] Backtesting engine framework
- [x] Utility functions for signal processing
- [x] Moving Average Crossover strategy
- [ ] RSI strategy
- [ ] Bollinger Bands strategy
- [ ] MACD strategy
- [ ] Volume-based strategies

## Usage

To use the library, add it as a dependency to your Cargo.toml:

```toml
[dependencies]
strategy_lib = { path = "/path/to/NyxsOwl/strategy_lib" }
```

Then, you can use the library in your code:

```rust
use polars::prelude::*;
use strategy_lib::strategy::{Strategy, StrategyConfig};
use strategy_lib::strategy::trend_following::MovingAverageCrossover;
use strategy_lib::backtest::{run_backtest, BacktestConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create or load price data
    let df = DataFrame::new(vec![
        // Your price data here
    ])?;
    
    // Configure the strategy
    let config = StrategyConfig {
        parameters: Series::new("params".into(), [0i32]), // Simple config for now
    };
    
    // Create the strategy
    let strategy = MovingAverageCrossover::new(config);
    
    // Run the backtest
    let results = run_backtest(&strategy, &df, BacktestConfig::default())?;
    
    // Analyze the results
    println!("Backtest results: {:?}", results.metrics);
    
    Ok(())
}
```

## Documentation

See the [docs](docs/) directory for more detailed documentation:

- [User Guide](docs/guide.md): Comprehensive guide for using the library
- [API Reference](docs/index.md): Reference for all library components
- [Examples](docs/examples/): Example code demonstrating various strategies

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT License

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 