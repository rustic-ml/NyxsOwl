# OxiDiviner Integration

This module provides the only forecasting models used in the `forecast_trade` crate, leveraging the sophisticated time series algorithms from [OxiDiviner](https://github.com/oxidiviner/oxidiviner).

## Overview

OxiDiviner provides a wide range of specialized time series forecasting models that we leverage through adapter implementations. These adapters:

1. Convert between our `TimeSeriesData` format and OxiDiviner's format
2. Implement our `ForecastModel` trait for OxiDiviner models
3. Add support for time granularity to models for both daily and minute-level trading
4. Handle error conversion between the two systems

## Available Models

### Exponential Smoothing

For mean-reverting price movements and adaptive forecasting.

```rust
use forecast_trade::models::oxidiviner::ExponentialSmoothing;
use forecast_trade::strategies::TimeGranularity;

// Create a daily model
let daily_es = ExponentialSmoothing::new(0.2)?;

// Create a minute model
let minute_es = ExponentialSmoothing::new_minute(0.4)?;

// Create with default parameters for a granularity
let model = ExponentialSmoothing::with_default_params(TimeGranularity::Daily)?;
```

### Moving Average

For trend identification and smoothing noisy data.

```rust
use forecast_trade::models::oxidiviner::MovingAverage;
use forecast_trade::strategies::TimeGranularity;

// Create a daily model (20-day moving average)
let daily_ma = MovingAverage::new(20)?;

// Create a minute model (60-minute moving average)
let minute_ma = MovingAverage::new_minute(60)?;

// Create with default parameters for a granularity
let model = MovingAverage::with_default_params(TimeGranularity::Minute)?;
```

## Using Models with Strategies

All models implement the `ForecastModel` trait, making them compatible with all trading strategies:

```rust
use forecast_trade::models::oxidiviner::ExponentialSmoothing;
use forecast_trade::strategies::mean_reversion::MeanReversionStrategy;

// Create a strategy using an OxiDiviner model
let strategy = MeanReversionStrategy::new(
    ExponentialSmoothing::new(0.2)?,
    2.0,
)?;

// Generate signals
let signals = strategy.generate_signals(&data)?;

// Run backtest
let results = strategy.backtest(&data, 10000.0)?;
```

## Benefits of OxiDiviner Models

OxiDiviner provides several advanced forecasting models that this crate leverages:

1. **Exponential Smoothing**: Simple, Holt's Linear, and Holt-Winters
2. **Moving Average**: Simple and Exponentially Weighted
3. **Autoregressive**: AR, MA, ARMA, and ARIMA models
4. **GARCH**: For volatility forecasting

These models have sophisticated implementations with many options, making them perfect for real-world trading scenarios.

## Examples

See the `examples/oxidiviner_models.rs` file for a complete example of using OxiDiviner models with trading strategies for both daily and minute-level data. 