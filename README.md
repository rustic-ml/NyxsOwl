# NyxsOwl

[![Build Status](https://github.com/rustic-ml/NyxsOwl/workflows/Rust/badge.svg)](https://github.com/rustic-ml/NyxsOwl/actions)
[![Crates.io](https://img.shields.io/crates/v/nyxs_owl.svg)](https://crates.io/crates/nyxs_owl)
[![Documentation](https://docs.rs/nyxs_owl/badge.svg)](https://docs.rs/nyxs_owl)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

**Production-ready financial analysis library for Rust**

NyxsOwl provides institutional-grade tools for quantitative finance, technical analysis, and algorithmic trading. Built for performance, reliability, and ease of use.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
nyxs_owl = "0.5.0"
```

## Core Features

### 📊 **Technical Analysis**
Production-ready indicators with comprehensive test coverage
```rust
use nyxs_owl::trade_math::*;

let mut rsi = RelativeStrengthIndex::new(14)?;
let mut bb = BollingerBands::new(20, 2.0)?;

for price in prices {
    rsi.update(price)?;
    println!("RSI: {:.2}", rsi.value()?);
}
```

### 🔮 **Time Series Forecasting**
Advanced forecasting with multiple model support
```rust
use nyxs_owl::forecast_trade::easy::*;

let (forecast, model) = auto_forecast(timestamps, values, 10)?;
println!("Best model: {} | Forecast: {:?}", model, forecast);
```

### ⚡ **Strategy Backtesting**
High-performance backtesting engine
```rust
use nyxs_owl::strategy_lib::backtest::*;

let config = BacktestConfig::builder()
    .initial_capital(100_000.0)
    .commission(0.001)
    .build();

let results = run_backtest(data, signals, config)?;
println!("Sharpe Ratio: {:.2}", results.sharpe_ratio);
```

## API Overview

### Technical Indicators
- **Moving Averages**: SMA, EMA, VWMA
- **Volatility**: Bollinger Bands, ATR, Standard Deviation  
- **Oscillators**: RSI, MACD, Stochastic
- **Volume**: OBV, Volume Profile, VROC

### Forecasting Models
- **ARIMA**: Autoregressive integrated moving average
- **Exponential Smoothing**: Simple and double exponential
- **Moving Average**: Various window forecasting
- **Auto Selection**: Automatic best model selection

### Strategy Framework
- **Signal Generation**: Buy/Sell/Hold signal framework
- **Risk Management**: Position sizing and stop losses
- **Performance Analytics**: Sharpe ratio, max drawdown, win rate
- **Backtesting**: Historical strategy validation

## Examples

### Basic Technical Analysis
```rust
use nyxs_owl::trade_math::*;

fn analyze_stock(prices: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
    let mut sma = SimpleMovingAverage::new(20)?;
    let mut rsi = RelativeStrengthIndex::new(14)?;
    
    for &price in prices {
        sma.update(price)?;
        rsi.update(price)?;
        
        let signal = match rsi.value()? {
            x if x > 70.0 => "SELL - Overbought",
            x if x < 30.0 => "BUY - Oversold", 
            _ => "HOLD"
        };
        
        println!("Price: {:.2} | SMA: {:.2} | RSI: {:.2} | Signal: {}", 
                price, sma.value()?, rsi.value()?, signal);
    }
    Ok(())
}
```

### Advanced Forecasting
```rust
use nyxs_owl::forecast_trade::*;
use chrono::{DateTime, Utc, Duration};

fn forecast_prices() -> Result<(), Box<dyn std::error::Error>> {
    // Generate sample data
    let now = Utc::now();
    let timestamps: Vec<DateTime<Utc>> = (0..30)
        .map(|i| now - Duration::days(30 - i))
        .collect();
    let prices: Vec<f64> = (0..30)
        .map(|i| 100.0 + i as f64 + (i as f64 * 0.1).sin() * 5.0)
        .collect();
    
    // Auto-select best model and forecast
    let (forecast, model_name) = easy::auto_forecast(timestamps, prices, 5)?;
    
    println!("Selected model: {}", model_name);
    println!("5-day forecast: {:?}", forecast);
    
    // Compare multiple models
    let comparison = easy::compare_models(&timestamps, &prices, 7)?;
    for (model, avg_forecast) in comparison {
        println!("{}: ${:.2}", model, avg_forecast);
    }
    
    Ok(())
}
```

### Strategy Backtesting
```rust
use nyxs_owl::strategy_lib::{backtest::*, strategy::*};
use polars::prelude::*;

fn backtest_strategy() -> Result<(), Box<dyn std::error::Error>> {
    // Load market data (OHLCV format)
    let data = df! {
        "timestamp" => [1, 2, 3, 4, 5],
        "open" => [100.0, 101.0, 102.0, 101.5, 103.0],
        "high" => [101.0, 102.5, 103.0, 102.0, 104.0],
        "low" => [99.5, 100.5, 101.0, 100.5, 102.0],
        "close" => [100.5, 102.0, 101.5, 103.0, 103.5],
        "volume" => [1000, 1500, 1200, 1800, 1600],
    }?;
    
    // Generate signals (1 = Buy, -1 = Sell, 0 = Hold)
    let signals = Series::new("signal".into(), [1, 0, -1, 0, 1]);
    
    // Configure backtest
    let config = BacktestConfig {
        initial_capital: 100_000.0,
        commission: 0.001,        // 0.1% commission
        slippage: 0.0005,        // 0.05% slippage
        position_size: 1.0,      // 100% of capital
    };
    
    // Run backtest
    let results = run_backtest(&data, &signals, &config)?;
    
    // Display results
    println!("=== Backtest Results ===");
    println!("Total Return: {:.2}%", results.total_return * 100.0);
    println!("Sharpe Ratio: {:.2}", results.sharpe_ratio);
    println!("Max Drawdown: {:.2}%", results.max_drawdown * 100.0);
    println!("Win Rate: {:.2}%", results.win_rate * 100.0);
    println!("Total Trades: {}", results.total_trades);
    
    Ok(())
}
```

## Feature Flags

Control what you include:

```toml
[dependencies]
# Minimal - just technical indicators
nyxs_owl = { version = "0.5.0", default-features = false, features = ["trading-math"] }

# With forecasting
nyxs_owl = { version = "0.5.0", features = ["trading-math", "forecasting"] }

# Full features
nyxs_owl = { version = "0.5.0", features = ["all"] }
```

## Performance

NyxsOwl is built for production environments:

- **Memory Efficient**: Streaming calculations, minimal allocation
- **Fast**: Optimized algorithms with SIMD support where applicable
- **Reliable**: 96%+ test coverage with comprehensive edge case testing
- **Concurrent**: Thread-safe design for parallel processing

## Getting Help

- **Documentation**: [docs.rs/nyxs_owl](https://docs.rs/nyxs_owl)
- **Examples**: See `examples/` directory
- **Issues**: [GitHub Issues](https://github.com/rustic-ml/NyxsOwl/issues)

## License

Licensed under either of:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option. 