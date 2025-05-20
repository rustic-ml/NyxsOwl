# Minute Trade

A Rust library for implementing intraday trading strategies using minute-by-minute OHLCV (Open, High, Low, Close, Volume) data spanning multiple days.

## Features

- **High-Frequency Trading Strategies**: Specialized for minute-level analysis
- **Multiple Strategy Categories**: Momentum, mean reversion, volatility, pattern recognition, and more
- **Performance Evaluation**: Comprehensive metrics for strategy evaluation 
- **Efficient Implementation**: Optimize for fast backtesting on large datasets
- **Customizable Framework**: Easily extend with your own strategies

## Strategy Categories

The library includes seven categories of intraday trading strategies:

### Momentum Strategies

Capitalize on price movement continuation:

- **Scalping Strategy**: Ultra-short term trades capturing small price movements
- **Momentum Breakout Strategy**: Trades breakouts with volume confirmation

### Mean Reversion Strategies

Trade on the assumption that prices revert to the mean:

- **Statistical Arbitrage Strategy**: Exploits price divergence between correlated assets
- **Mean Reversion Oscillator Strategy**: Uses oversold/overbought indicators to find reversal points

### Volatility Strategies

Trade based on market volatility patterns:

- **Volatility Breakout Strategy**: Enters trades after periods of low volatility
- **Bollinger Band Contraction Strategy**: Trades volatility expansions after contractions

### Pattern Recognition Strategies

Identify and trade chart patterns:

- **Chart Pattern Strategy**: Identifies common chart patterns (flags, triangles, etc.)
- **Support/Resistance Strategy**: Trades bounces and breakouts from key levels

### Time-Based Strategies

Trade based on specific times of the day:

- **Time of Day Strategy**: Trades specific time periods with historical edge
- **Session Transition Strategy**: Trades market opens, closes, and session overlaps

### Statistical Strategies

Use statistical methods to find trading opportunities:

- **Regression Strategy**: Uses linear regression for mean reversion
- **Z-Score Strategy**: Uses statistical deviation for entry and exit

### Volume-Based Strategies

Analyze volume patterns for trading signals:

- **Volume Profile Strategy**: Trades based on volume distribution at price levels
- **Relative Volume Strategy**: Trades unusual volume spikes

## Usage Example

```rust
use minute_trade::{ScalpingStrategy, IntradayStrategy};
use minute_trade::utils::load_minute_data;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load minute-by-minute data
    let data = load_minute_data("AAPL_minute_data.csv")?;
    
    // Create a scalping strategy with 5-minute lookback and 0.1% threshold
    let strategy = ScalpingStrategy::new(5, 0.1)?;
    
    // Generate trading signals
    let signals = strategy.generate_signals(&data)?;
    
    // Calculate performance
    let performance = strategy.calculate_performance(&data, &signals)?;
    println!("Strategy performance: {}%", performance);
    
    Ok(())
}
```

## Comparing Multiple Strategies

The library makes it easy to compare different strategies:

```rust
use minute_trade::{
    ScalpingStrategy, MomentumBreakoutStrategy,
    IntradayStrategy, Signal
};
use minute_trade::utils::{generate_minute_data, calculate_detailed_performance};

// Create multiple strategies
let strategies: Vec<Box<dyn IntradayStrategy>> = vec![
    Box::new(ScalpingStrategy::new(5, 0.1)?),
    Box::new(MomentumBreakoutStrategy::new(30, 1.5)?),
];

// Generate test data
let data = generate_minute_data(5, 390, 100.0, 0.02, 0.0001);

// Test each strategy
for strategy in &strategies {
    let signals = strategy.generate_signals(&data)?;
    let performance = calculate_detailed_performance(&data, &signals, 10000.0, 0.05)?;
    println!("{}: {:.2}% return, Sharpe: {:.2}", 
             strategy.name(), performance.total_return, performance.sharpe_ratio);
}
```

## Creating Your Own Strategy

To implement a custom strategy, implement the `IntradayStrategy` trait:

```rust
use minute_trade::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};
use minute_trade::utils::calculate_basic_performance;

struct MyCustomStrategy {
    name: String,
    parameter: f64,
}

impl MyCustomStrategy {
    pub fn new(parameter: f64) -> Result<Self, String> {
        if parameter <= 0.0 {
            return Err("Parameter must be positive".to_string());
        }
        
        Ok(Self {
            name: format!("My Custom Strategy ({})", parameter),
            parameter,
        })
    }
}

impl IntradayStrategy for MyCustomStrategy {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        let mut signals = Vec::with_capacity(data.len());
        
        // Your custom signal generation logic here
        for i in 0..data.len() {
            // Example: Buy if close > open, Sell if close < open, otherwise Hold
            let candle = &data[i].data;
            let signal = if candle.close > candle.open {
                Signal::Buy
            } else if candle.close < candle.open {
                Signal::Sell
            } else {
                Signal::Hold
            };
            
            signals.push(signal);
        }
        
        Ok(signals)
    }
    
    fn calculate_performance(&self, data: &[MinuteOhlcv], signals: &[Signal]) -> Result<f64, TradeError> {
        calculate_basic_performance(data, signals, 10000.0, 0.05)
    }
}
```

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
minute_trade = "0.1.0"
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions. 