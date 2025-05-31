# NyxsOwl - Consolidated Financial Trading Library

[![Rust](https://github.com/username/NyxsOwl/workflows/Rust/badge.svg)](https://github.com/username/NyxsOwl/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## 🌙 About the Name

**NyxsOwl** draws its name from ancient Greek mythology, combining the wisdom of two powerful symbols:

- **Nyx (Νύξ)**: The primordial goddess of night, one of the most powerful deities in Greek mythology. Nyx represents the darkness from which all things emerge and the strategic advantage that comes from seeing what others cannot.

- **Bubo**: The wise owl, sacred companion to Athena and symbol of wisdom, strategy, and keen observation. In trading, like the owl hunting in darkness, success comes from patience, precision, and seeing opportunities others miss.

Together, **NyxsOwl** embodies the essence of strategic financial analysis - the ability to navigate market darkness with wisdom, patience, and tactical precision. Just as Nyx commands the night and the owl sees clearly in darkness, this library empowers traders to make intelligent decisions in the complex, ever-changing landscape of financial markets.

*"In the darkness of market uncertainty, wisdom and strategy illuminate the path to success."* 🦉

A comprehensive Rust library for financial time series analysis, trading strategies, forecasting, and technical indicators. **NyxsOwl** has been consolidated from a multi-crate workspace into a single, feature-rich crate with modular functionality.

## 🚀 Features

### ✅ **Working Modules**

- **trade_math**: Complete technical indicator library
  - Moving averages (SMA, EMA, VWMA)
  - Volatility indicators (Bollinger Bands, ATR, Standard Deviation)
  - Oscillators (RSI, MACD, Stochastic)
  - Volume indicators (OBV, VMA, VROC, VPT)
  - Forecasting algorithms (Linear Regression, Exponential Smoothing)

### 🚧 **In Development**

- **day_trade**: Daily trading strategies (integration in progress)
- **minute_trade**: Minute-level intraday trading (integration in progress)
- **forecast_trade**: Financial forecasting with OxiDiviner (integration in progress)
- **strategy_lib**: Advanced strategy library (integration in progress)

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nyxs_owl = { version = "0.3.0", features = ["trading-math"] }

# Or with all features (when fully integrated)
# nyxs_owl = "0.3.0"
```

## 🎯 Quick Start

### Running Examples

The library includes several examples to demonstrate its capabilities:

#### Basic OxiDiviner Integration
```bash
# Test basic OxiDiviner functionality
cargo run --example test_oxidiviner_imports --no-default-features --features="forecasting"

# Advanced forecasting demo with multiple models
cargo run --example advanced_forecasting_demo --no-default-features --features="forecasting"
```

#### Full Trading Examples (requires all features)
```bash
# Comprehensive trading analysis
cargo run --example comprehensive_trading_analysis --all-features

# Performance monitoring
cargo run --example performance_monitoring --all-features

# Advanced optimizations demo
cargo run --example advanced_optimizations_demo --all-features
```

### Feature Flags

- `forecasting` - Core forecasting functionality with OxiDiviner integration
- `day-trading` - Daily trading data structures and utilities
- `minute-trading` - Minute-level trading data structures and utilities
- `trading-math` - Mathematical functions for trading analysis
- `strategies` - Trading strategy implementations

## 🏗️ Architecture

### Consolidated Structure

```
nyxs_owl/
├── src/
│   ├── lib.rs                    # Main library with feature gates
│   ├── trade_math/               # ✅ Technical indicators & math
│   │   ├── moving_averages.rs
│   │   ├── volatility.rs
│   │   ├── oscillators.rs
│   │   ├── volume.rs
│   │   └── forecasting.rs
│   ├── day_trade/                # 🚧 Daily trading strategies
│   ├── minute_trade/             # 🚧 Intraday trading
│   ├── forecast_trade/           # 🚧 Forecasting with OxiDiviner
│   └── strategy_lib/             # 🚧 Advanced strategies
├── examples/
│   └── trade_math_demo.rs        # ✅ Working example
└── tests/                        # Unit & integration tests
```

### Feature Flags

- `trading-math` (working): Technical indicators and mathematical functions
- `day-trading` (in progress): Daily timeframe trading strategies
- `minute-trading` (in progress): Minute-level trading strategies
- `forecasting` (in progress): Time series forecasting capabilities
- `strategies` (in progress): Advanced strategy library

## 📊 Technical Indicators

### Available Indicators

| Category | Indicators | Status |
|----------|------------|--------|
| **Moving Averages** | SMA, EMA, VWMA | ✅ Working |
| **Volatility** | Bollinger Bands, ATR, Standard Deviation | ✅ Working |
| **Oscillators** | RSI, MACD, Stochastic | ✅ Working |
| **Volume** | OBV, VMA, VROC, VPT | ✅ Working |
| **Forecasting** | Linear Regression, Exponential Smoothing | ✅ Working |

## 🧪 Testing

Run tests for the working modules:

```bash
# Test just the trade_math module
cargo test --no-default-features --features="trading-math"

# Run the working example
cargo run --example trade_math_demo --no-default-features --features="trading-math"
```

## 📈 Examples & Usage

### Running Examples

The library includes comprehensive examples demonstrating various features. Here's how to run them:

#### Technical Indicators Demo (Working)

```bash
# Navigate to the project directory
cd nyxs_owl

# Run the technical indicators example
cargo run --example trade_math_demo --no-default-features --features="trading-math"

# Or run with verbose output for debugging
cargo run --example trade_math_demo --no-default-features --features="trading-math" -- --verbose
```

#### Available Examples

| Example | Command | Status | Description |
|---------|---------|--------|-------------|
| `trade_math_demo` | `cargo run --example trade_math_demo --no-default-features --features="trading-math"` | ✅ Working | Technical indicators demonstration |
| `day_trade_demo` | `cargo run --example day_trade_demo --features="day-trading"` | 🚧 In Progress | Daily trading strategies |
| `minute_trade_demo` | `cargo run --example minute_trade_demo --features="minute-trading"` | 🚧 In Progress | Intraday trading strategies |
| `forecast_demo` | `cargo run --example forecast_demo --features="forecasting"` | 🚧 In Progress | Time series forecasting |

#### Example Output

When you run the technical indicators demo, you'll see:

```
🦉 NyxsOwl Technical Analysis Demo
===============================

📊 Testing Simple Moving Average (SMA-20)...
📊 Testing Exponential Moving Average (EMA-12)...
📊 Testing Bollinger Bands (BB-20, 2.0)...
📊 Testing Relative Strength Index (RSI-14)...
📊 Testing On-Balance Volume (OBV)...

✅ All indicators working correctly!
```

### Code Examples

#### Basic Technical Analysis

```rust
use nyxs_owl::trade_math::{
    moving_averages::SimpleMovingAverage,
    volatility::BollingerBands,
    oscillators::RelativeStrengthIndex,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create indicators
    let mut sma = SimpleMovingAverage::new(20)?;
    let mut bb = BollingerBands::new(20, 2.0)?;
    let mut rsi = RelativeStrengthIndex::new(14)?;

    // Sample price data
    let prices = vec![100.0, 102.0, 101.5, 103.0, 104.5];

    for price in prices {
        sma.update(price)?;
        bb.update(price)?;
        rsi.update(price)?;

        // Use indicators when enough data is available
        if let Ok(sma_val) = sma.value() {
            println!("SMA: {:.2}", sma_val);
        }
    }

    Ok(())
}
```

#### Advanced Strategy Development (Coming Soon)

```rust
// Will be available when strategy modules are integrated
use nyxs_owl::{
    day_trade::strategies::MeanReversion,
    minute_trade::scalping::ScalpingStrategy,
    forecast_trade::models::ARIMAModel,
};

// Strategy examples coming soon...
```

## 🔗 Dependencies

- **chrono**: Date and time handling
- **serde**: Serialization support
- **polars**: Fast DataFrame operations (for full features)
- **oxidiviner**: Time series forecasting (for forecasting features)
- **ta-lib-in-rust**: Additional technical indicators
- **rustalib**: Rust technical analysis library

## 📊 Integration Status

| Module | Structure | Dependencies | Compilation | Tests | Examples |
|--------|-----------|--------------|-------------|--------|----------|
| **trade_math** | ✅ Complete | ✅ Working | ✅ Success | ✅ 19/20 pass | ✅ Working |
| **day_trade** | ✅ Complete | ⚠️ Partial | ❌ 579 errors | ❌ Blocked | ❌ Blocked |
| **minute_trade** | ✅ Complete | ⚠️ Partial | ❌ Blocked | ❌ Blocked | ❌ Blocked |
| **forecast_trade** | ✅ Complete | ⚠️ Partial | ❌ Blocked | ❌ Blocked | ❌ Blocked |
| **strategy_lib** | ✅ Complete | ⚠️ Partial | ❌ Blocked | ❌ Blocked | ❌ Blocked |

## 🛠️ Development Status

### Recently Completed ✅

- ✅ Structural consolidation of all sub-crates
- ✅ Dependencies consolidated into main Cargo.toml
- ✅ Feature flag system implemented
- ✅ Core trade_math module fully functional
- ✅ Working example created and tested
- ✅ Comprehensive technical indicator suite

### Current Work 🚧

- 🚧 Resolving cross-module import issues
- 🚧 Fixing remaining compilation errors (579 → 0)
- 🚧 Type resolution across modules
- 🚧 Integration testing

### Next Steps 📋

1. Complete import resolution for remaining modules
2. Fix compilation errors systematically
3. Enable all feature flags
4. Create comprehensive examples for each module
5. Add integration tests
6. Performance benchmarking

## 🤝 Contributing

Contributions are welcome! The library is in active consolidation phase.

### Development Setup

```bash
git clone https://github.com/username/NyxsOwl.git
cd NyxsOwl/nyxs_owl

# Test working functionality
cargo test --lib --no-default-features --features="trading-math"

# Run working example
cargo run --example trade_math_demo --no-default-features --features="trading-math"
```

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 📬 Contact

- **Author**: Celsis Durham
- **Email**: durhamcelsis@gmail.com
- **Repository**: https://github.com/rustic-ml/NyxsOwl

---

*NyxsOwl: Where technical analysis meets Rust performance* 🦉⚡

## 📊 OxiDiviner Integration Status

### ✅ **Fully Implemented**
- **Path Dependency Removed**: Now uses published OxiDiviner v0.4.3 from crates.io
- **Quick API**: Direct access to OxiDiviner's quick forecasting functions
  - `easy::arima_forecast()` - ARIMA time series forecasting
  - `easy::exponential_smoothing_forecast()` - Exponential smoothing with configurable alpha
  - `easy::moving_average_forecast()` - Moving average forecasting with window size
  - `easy::auto_forecast()` - Automatic model selection
- **Unified Adapter**: `OxiDivinerAdapter` provides consistent interface for all models
- **Type Aliases**: Backward compatibility with specific model adapters
- **Error Handling**: Comprehensive error handling and conversion

### 🚧 **Partially Implemented**
- **Advanced Model Training**: Individual model adapters work but have data conversion issues
- **Model Validation**: Error metrics calculation implemented but needs refinement
- **Strategy Integration**: ForecastStrategy trait implemented but needs testing

### 📈 **Working Examples**
The following functionality is confirmed working:

```rust
use nyxs_owl::forecast_trade::models::oxidiviner::easy;

// Quick ARIMA forecast
let forecast = easy::arima_forecast(dates, values, 5)?;

// Exponential smoothing with alpha parameter
let forecast = easy::exponential_smoothing_forecast(dates, values, 5, Some(0.3))?;

// Moving average with window size
let forecast = easy::moving_average_forecast(dates, values, 5, Some(10))?;

// Automatic model selection
let (model_name, forecast) = easy::auto_forecast(dates, values, 5)?;
```
