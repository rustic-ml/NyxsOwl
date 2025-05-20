# NyxsOwl

[![crates.io](https://img.shields.io/crates/v/nyxs_owl.svg)](https://crates.io/crates/nyxs_owl)
[![Documentation](https://docs.rs/nyxs_owl/badge.svg)](https://docs.rs/nyxs_owl)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/nyxs_owl.svg)](./LICENSE)

A comprehensive Rust library for stock trading strategies and analysis.

## Features

- **Day trading strategies** - Tools for daily market analysis and trading
- **Minute-level trading** - High-frequency trading strategies for minute-level data
- **Mathematical utilities** - Core calculations for technical analysis and strategy evaluation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nyxs_owl = "0.1.0"
```

## Usage Examples

### Basic Usage

```rust
use nyxs_owl::Owl;

fn main() {
    // Create a new owl with default wisdom level
    let owl = Owl::new("Bubo");
    println!("Owl name: {}", owl.name());
    println!("Wisdom level: {}", owl.wisdom_level());
}
```

### Using Day Trading Strategies

```rust
use nyxs_owl::day_trade::strategies::MovingAverageStrategy;
use nyxs_owl::day_trade::DayTradeStrategy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = nyxs_owl::day_trade::utils::load_daily_data("AAPL.csv")?;
    
    let strategy = MovingAverageStrategy::new(20, 50)?;
    let signals = strategy.generate_signals(&data)?;
    
    let performance = strategy.calculate_performance(&data, &signals)?;
    println!("Strategy performance: {}%", performance);
    
    Ok(())
}
```

### Using Minute Trading Strategies

```rust
use nyxs_owl::minute_trade::ScalpingStrategy;
use nyxs_owl::minute_trade::IntradayStrategy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = nyxs_owl::minute_trade::utils::load_minute_data("AAPL_minute_data.csv")?;
    
    let strategy = ScalpingStrategy::new(5, 0.1)?;
    let signals = strategy.generate_signals(&data)?;
    
    let performance = strategy.calculate_performance(&data, &signals)?;
    println!("Strategy performance: {}%", performance);
    
    Ok(())
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions. 