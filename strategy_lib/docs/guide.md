# Strategy Library User Guide

This guide provides detailed instructions on how to use the NyxsOwl Strategy Library to create, test, and implement trading strategies based on technical indicators.

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Creating a Strategy](#creating-a-strategy)
4. [Backtesting a Strategy](#backtesting-a-strategy)
5. [Available Strategies](#available-strategies)
6. [Creating Custom Strategies](#creating-custom-strategies)
7. [Advanced Usage](#advanced-usage)

## Introduction

The NyxsOwl Strategy Library provides a framework for implementing and testing trading strategies based on technical indicators from the RusTaLib crate. It offers:

- Pre-built trading strategies for common technical indicators
- A common interface for all strategies
- A backtesting engine to evaluate strategy performance
- Utility functions for signal generation and analysis

## Getting Started

To use the Strategy Library, add it as a dependency to your Cargo.toml:

```toml
[dependencies]
strategy_lib = { path = "/path/to/NyxsOwl/strategy_lib" }
```

Then import the necessary components in your Rust code:

```rust
use polars::prelude::*;
use strategy_lib::strategy::{Strategy, StrategyConfig};
use strategy_lib::backtest::{run_backtest, BacktestConfig};
```

## Creating a Strategy

Each strategy in the library follows a common interface defined by the `Strategy` trait. To create an instance of a strategy, you need to:

1. Choose a strategy (e.g., `MovingAverageCrossover`)
2. Configure the strategy parameters
3. Initialize the strategy with those parameters

Here's an example of creating a Moving Average Crossover strategy:

```rust
use strategy_lib::strategy::trend_following::MovingAverageCrossover;

// Create strategy configuration
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
```

## Backtesting a Strategy

Once you have a strategy, you can backtest it on historical data using the `run_backtest` function:

```rust
// Load historical data
let df = DataFrame::new(vec![
    // Your price data here (date, open, high, low, close, volume, etc.)
])?;

// Configure backtest parameters
let backtest_config = BacktestConfig {
    initial_capital: 10000.0,
    commission: 0.001,   // 0.1%
    slippage: 0.0005,    // 0.05%
    position_size: 0.1,  // 10% of capital per trade
};

// Run the backtest
let results = run_backtest(&strategy, &df, backtest_config)?;

// Analyze the results
println!("Total Return: {:.2}%", results.metrics.total_return * 100.0);
println!("Max Drawdown: {:.2}%", results.metrics.max_drawdown * 100.0);
println!("Sharpe Ratio: {:.2}", results.metrics.sharpe_ratio);
println!("Win Rate: {:.2}%", results.metrics.win_rate * 100.0);
```

## Available Strategies

The library organizes strategies into several categories:

### Trend Following Strategies

- **Moving Average Crossover**: Generates signals when a fast moving average crosses a slow moving average.

### Mean Reversion Strategies

- Coming soon: Bollinger Bands strategy, RSI mean reversion, and more.

### Momentum Strategies

- Coming soon: RSI momentum strategy, MACD strategy, and more.

### Volatility Strategies

- Coming soon: ATR breakout strategy, Bollinger Bands volatility strategy, and more.

### Volume Strategies

- Coming soon: OBV strategy, Volume Price Trend strategy, and more.

### Multi-Indicator Strategies

- Coming soon: Triple Screen strategy, MACD + RSI strategy, and more.

## Creating Custom Strategies

You can create your own custom strategies by implementing the `Strategy` trait:

```rust
use strategy_lib::strategy::{Strategy, StrategyConfig, Signal, StrategyError};

struct MyCustomStrategy {
    // Your strategy fields here
    name: String,
    description: String,
}

impl Strategy for MyCustomStrategy {
    fn new(config: StrategyConfig) -> Self {
        // Parse configuration and initialize your strategy
        Self {
            name: "My Custom Strategy".to_string(),
            description: "Description of my strategy".to_string(),
        }
    }
    
    fn generate_signals(&self, data: &DataFrame) -> Result<Series, StrategyError> {
        // Your signal generation logic here
        // Return a Series with Signal::Buy, Signal::Sell, or Signal::Hold values
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn required_columns(&self) -> Vec<&str> {
        vec!["close", "volume"] // List columns your strategy needs
    }
}
```

## Advanced Usage

### Combining Multiple Strategies

You can combine multiple strategies to create a more complex trading system:

```rust
fn combine_signals(signals1: &Series, signals2: &Series) -> Result<Series, StrategyError> {
    // Your logic for combining signals from multiple strategies
    // For example, only buy when both strategies generate buy signals
}

// Create and run multiple strategies
let strategy1 = MovingAverageCrossover::new(config1);
let strategy2 = RsiStrategy::new(config2);

let signals1 = strategy1.generate_signals(&df)?;
let signals2 = strategy2.generate_signals(&df)?;

// Combine the signals
let combined_signals = combine_signals(&signals1, &signals2)?;
```

### Position Sizing

The backtesting engine supports different position sizing strategies:

```rust
let backtest_config = BacktestConfig {
    initial_capital: 10000.0,
    commission: 0.001,
    slippage: 0.0005,
    position_size: 0.1, // 10% of capital per trade
};
```

You can implement more sophisticated position sizing by modifying the backtest engine or post-processing the backtest results.

### Extending the Library

The Strategy Library is designed to be easily extended. You can:

1. Add new strategy categories
2. Implement new strategies based on different indicators
3. Enhance the backtesting engine with additional features
4. Create custom metrics for strategy evaluation

See the source code for examples of how to implement these extensions. 