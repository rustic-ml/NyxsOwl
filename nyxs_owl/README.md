# NyxsOwl

[![crates.io](https://img.shields.io/crates/v/nyxs_owl.svg)](https://crates.io/crates/nyxs_owl)
[![Documentation](https://docs.rs/nyxs_owl/badge.svg)](https://docs.rs/nyxs_owl)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/nyxs_owl.svg)](./LICENSE)

A comprehensive Rust library for trading, forecasting, and financial analysis.

## Features

- **Technical Indicators** - Complete suite of trading indicators (SMA, EMA, RSI, MACD, Bollinger Bands, etc.)
- **Trading Math** - Core mathematical functions for financial analysis
- **Forecasting** - Time series forecasting with OxiDiviner integration
- **Strategy Library** - Advanced trading strategies for multiple timeframes

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nyxs_owl = { version = "0.2.0", features = ["trading-math"] }
```

## Quick Start

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

        if let Ok(sma_val) = sma.value() {
            println!("SMA: {:.2}", sma_val);
        }
    }

    Ok(())
}
```

## Examples

Run the working examples:

```bash
# Technical indicators demo
cargo run --example trade_math_demo --features="trading-math"

# Comprehensive trading analysis
cargo run --example comprehensive_trading_analysis --features="trading-math"
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 