# Forecast Trade

A Rust library for financial time series forecasting and trading strategy development. This crate uses the OxiDiviner libraries for core mathematical functionality and provides a trader-focused API.

## Features

- Time series data handling (OHLCV data)
- Forecasting models (Exponential Smoothing, Moving Average, ARIMA)
- Trading strategies (Mean Reversion, Trend Following, Volatility Breakout)
- Strategy backtesting with performance metrics
- Volatility analysis and forecasting
- Support for both daily and minute-level data

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
forecast_trade = { path = "/path/to/forecast_trade" }
```

## Quick Start

```rust
use forecast_trade::data::DataLoader;
use forecast_trade::models::exponential_smoothing::ExponentialSmoothing;
use forecast_trade::strategies::mean_reversion::MeanReversionStrategy;
use forecast_trade::strategies::{ForecastStrategy, TimeGranularity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load data from CSV
    let data = DataLoader::from_csv("path/to/data.csv")?;
    
    // Create a forecasting model
    let model = ExponentialSmoothing::new(0.7)?;
    
    // Create a trading strategy for daily data
    let strategy = MeanReversionStrategy::new_with_granularity(
        model,
        2.0, // Threshold
        TimeGranularity::Daily
    )?;
    
    // Generate trading signals
    let signals = strategy.generate_signals(&data)?;
    
    // Run backtest
    let results = strategy.backtest(&data, 10000.0)?;
    println!("Final balance: ${:.2}", results.final_balance);
    println!("Total trades: {}", results.total_trades);
    println!("Win rate: {:.1}%", results.win_rate * 100.0);
    
    Ok(())
}
```

## Working with Different Time Granularities

The library supports both daily and minute-level data. You can create strategies optimized for different time frames:

```rust
// For daily data
let daily_strategy = MeanReversionStrategy::new_with_granularity(
    model.clone(),
    2.0,
    TimeGranularity::Daily
)?;

// For minute data
let minute_strategy = MeanReversionStrategy::new_with_granularity(
    model.clone(),
    1.5, // Lower threshold for minute data (more sensitive)
    TimeGranularity::Minute
)?;
```

The strategies automatically adjust parameters based on the time granularity:

1. **Parameter Scaling**: Window sizes, momentum thresholds, and other parameters are automatically scaled.
2. **Transaction Costs**: Backtests use different commission and slippage models based on granularity.
3. **Direct Integration**: You can work directly with `day_trade::DailyOhlcv` and `minute_trade::MinuteOhlcv` types:

```rust
// Convert TimeSeriesData to daily format
let daily_ohlcv = time_series.to_daily_ohlcv()?;

// Generate signals directly from daily data
let signals = strategy.generate_signals_daily(&daily_ohlcv)?;

// Convert TimeSeriesData to minute format
let minute_ohlcv = time_series.to_minute_ohlcv()?;

// Generate signals directly from minute data
let signals = strategy.generate_signals_minute(&minute_ohlcv)?;
```

## Example Strategies

### Mean Reversion

This strategy generates buy signals when the price falls significantly below its expected value, and sell signals when it rises significantly above it.

```rust
let strategy = MeanReversionStrategy::new_with_granularity(
    ExponentialSmoothing::new(0.7)?,
    2.0, // Threshold in standard deviations
    TimeGranularity::Daily
)?;
```

### Trend Following

This strategy generates buy signals when the price is trending upward and sell signals when it is trending downward.

```rust
let strategy = TrendFollowingStrategy::new_with_granularity(
    ExponentialSmoothing::new(0.8)?,
    20,  // Window size
    TimeGranularity::Daily
)?;
```

### Volatility Breakout

This strategy generates buy signals when the price breaks out above a volatility band and sell signals when it breaks below.

```rust
let strategy = VolatilityBreakoutStrategy::new_with_granularity(
    ExponentialSmoothing::new(0.5)?,
    1.5, // Volatility multiplier
    TimeGranularity::Daily
)?;
```

## Running Examples

```bash
# Run the daily vs minute strategy example
cargo run --example daily_vs_minute_strategy
```

## License

This project is licensed under the MIT License.

# Forecast Trade - Time Granularity Implementation Guide

## Overview of Changes

We've implemented support for both daily and minute-level data in the trading strategies. Here's a summary of the changes:

### 1. Added TimeGranularity Enum
```rust
/// Time granularity for strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeGranularity {
    /// Daily data
    Daily,
    /// Minute data
    Minute,
}
```

### 2. Implemented Parameter Scaling Based on Granularity

For each strategy, we've added configuration parameters that automatically adjust based on the time granularity:

```rust
// Example from MeanReversionStrategy
let lookback_period = match time_granularity {
    TimeGranularity::Daily => 20,  // 20 days
    TimeGranularity::Minute => 60, // 60 minutes
};
```

### 3. Added Conversion Methods for OHLCV Data Types

Added methods to directly work with both daily and minute OHLCV data:

```rust
/// Generate signals with daily OHLCV data
fn generate_signals_daily(&self, data: &[day_trade::DailyOhlcv]) -> Result<Vec<TradingSignal>> {
    let time_series = self.convert_daily_to_time_series(data)?;
    self.generate_signals(&time_series)
}

/// Generate signals with minute OHLCV data
fn generate_signals_minute(&self, data: &[minute_trade::MinuteOhlcv]) -> Result<Vec<TradingSignal>> {
    let time_series = self.convert_minute_to_time_series(data)?;
    self.generate_signals(&time_series)
}
```

### 4. Adjusted Transaction Costs for Different Timeframes

Added realistic transaction cost modeling based on time granularity:

```rust
// Default parameters based on time granularity
let (commission_rate, slippage) = match self.time_granularity() {
    TimeGranularity::Daily => (0.001, 0.0005),  // 0.1% commission, 0.05% slippage for daily
    TimeGranularity::Minute => (0.0005, 0.001), // 0.05% commission, 0.1% slippage for minute
};
```

## Using the Time Granularity Feature

### Creating Strategies with Different Granularities

```rust
// For daily data
let daily_strategy = MeanReversionStrategy::new_with_granularity(
    model.clone(),
    2.0, // Threshold in standard deviations
    TimeGranularity::Daily
)?;

// For minute data
let minute_strategy = MeanReversionStrategy::new_with_granularity(
    model.clone(),
    1.5, // Lower threshold for minute data (more sensitive)
    TimeGranularity::Minute
)?;
```

### Working Directly with OHLCV Data Types

```rust
// Generate signals from daily OHLCV data
let daily_signals = strategy.generate_signals_daily(&daily_ohlcv)?;

// Generate signals from minute OHLCV data
let minute_signals = strategy.generate_signals_minute(&minute_ohlcv)?;

// Run backtest with daily data
let daily_results = strategy.backtest_daily(&daily_ohlcv, 10000.0)?;

// Run backtest with minute data
let minute_results = strategy.backtest_minute(&minute_ohlcv, 10000.0)?;
```

## Implementation Considerations

### 1. Parameter Tuning for Different Timeframes

Different timeframes require different parameter settings:
- **Daily data**: Generally uses wider windows, higher thresholds
- **Minute data**: Uses shorter windows, tighter thresholds, is more noise-sensitive

### 2. Transaction Cost Modeling

Minute-level trading typically has:
- Lower percentage commissions (due to higher volume)
- Higher slippage (due to less liquidity/faster execution)

### 3. Strategy Considerations

- **Mean reversion** behaves differently on minute vs. daily timeframes
- **Trend following** needs shorter lookback periods for minute data
- **Volatility breakout** signals are more frequent on minute timeframes

## Current Status

The implementation is in progress and still has compilation errors. The core architectural changes have been made, but several components need to be updated to match the new interfaces. 