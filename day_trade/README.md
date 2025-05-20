# Day Trade

A Rust library for implementing day trading strategies that work with both daily and minute OHLCV (Open, High, Low, Close, Volume) data.

## Features

- **Organized Strategy Categories:** Strategies are categorized into buy-focused, sell-focused, and hold-focused groups
- **Multiple Technical Indicators:** Supports RSI, MACD, Bollinger Bands, Moving Averages, and more
- **Backtesting Support:** Built-in performance measurement and backtesting tools
- **Flexible Strategy Implementation:** Easily extend the library with your own custom strategies
- **Comprehensive Testing:** All strategies are thoroughly tested with various market conditions

## Strategy Categories

### Buy-Focused Strategies

These strategies primarily identify opportunities to enter long positions:

- **MA Crossover:** Uses moving average crossovers to identify trends
- **MACD Strategy:** Follows momentum using Moving Average Convergence Divergence
- **Breakout Strategy:** Identifies price breakouts through resistance levels
- **Adaptive Moving Average Strategy:** Dynamically adjusts to market volatility

### Sell-Focused Strategies

These strategies identify opportunities to exit positions or enter short positions:

- **RSI Strategy:** Uses the Relative Strength Index to identify overbought conditions
- **Mean Reversion Strategy:** Uses Bollinger Bands to identify statistical extremes
- **Volume-Based Strategy:** Uses volume indicators to confirm price movements

### Hold-Focused and Market-Neutral Strategies

These strategies work well in range-bound markets or take a balanced approach:

- **Bollinger Bands Strategy:** Works in range-bound markets
- **Multi-Indicator (Composite) Strategy:** Combines multiple indicators
- **Dual Timeframe Strategy:** Analyzes multiple timeframes simultaneously
- **Grid Trading Strategy:** Places orders at regular intervals above and below price
- **VWAP Strategy:** Uses Volume-Weighted Average Price for trading decisions
- **Forecasting Strategy:** Uses statistical methods to forecast future prices

## Usage Example

```rust
use day_trade::{MeanReversionStrategy, TradingStrategy};
use day_trade::utils::generate_test_data;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a mean reversion strategy with custom parameters
    let strategy = MeanReversionStrategy::new(20, 2.0, 0.05, 0.95)?;

    // Generate test data
    let data = generate_test_data(100, 100.0, 0.05);
    
    // Generate trading signals
    let signals = strategy.generate_signals(&data)?;
    
    // Calculate performance
    let performance = strategy.calculate_performance(&data, &signals)?;
    println!("Strategy performance: {:.2}%", performance);
    
    Ok(())
}
```

## Comparing Multiple Strategies

You can easily compare multiple strategies against the same data:

```rust
use day_trade::{
    MACrossover, MeanReversionStrategy, GridTradingStrategy, 
    TradingStrategy, Signal
};
use day_trade::utils::{generate_test_data, calculate_basic_performance};

// Create different strategies
let buy_strategy = MACrossover::new(10, 30)?;
let sell_strategy = MeanReversionStrategy::default();
let hold_strategy = GridTradingStrategy::default();

// Generate signals for each strategy
let data = generate_test_data(200, 100.0, 0.05);
let buy_signals = buy_strategy.generate_signals(&data)?;
let sell_signals = sell_strategy.generate_signals(&data)?;
let hold_signals = hold_strategy.generate_signals(&data)?;

// Calculate performance for each strategy
let buy_performance = buy_strategy.calculate_performance(&data, &buy_signals)?;
let sell_performance = sell_strategy.calculate_performance(&data, &sell_signals)?;
let hold_performance = hold_strategy.calculate_performance(&data, &hold_signals)?;

println!("Buy Strategy: {:.2}%", buy_performance);
println!("Sell Strategy: {:.2}%", sell_performance);
println!("Hold Strategy: {:.2}%", hold_performance);
```

## Creating Your Own Strategy

To create a custom strategy, implement the `TradingStrategy` trait:

```rust
use day_trade::{DailyOhlcv, Signal, TradeError, TradingStrategy};
use day_trade::utils::calculate_basic_performance;

struct MyCustomStrategy {
    // Strategy parameters
    period: usize,
}

impl MyCustomStrategy {
    pub fn new(period: usize) -> Result<Self, String> {
        if period < 2 {
            return Err("Period must be at least 2".to_string());
        }
        Ok(Self { period })
    }
}

impl TradingStrategy for MyCustomStrategy {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        // Your signal generation logic here
        let mut signals = Vec::with_capacity(data.len());
        
        // Fill with initial hold signals
        for _ in 0..self.period {
            signals.push(Signal::Hold);
        }
        
        // Generate signals for the rest of the data
        for i in self.period..data.len() {
            // Your custom strategy logic here
            let signal = if data[i].data.close > data[i-1].data.close {
                Signal::Buy
            } else if data[i].data.close < data[i-1].data.close {
                Signal::Sell
            } else {
                Signal::Hold
            };
            
            signals.push(signal);
        }
        
        Ok(signals)
    }
    
    fn calculate_performance(&self, data: &[DailyOhlcv], signals: &[Signal]) -> Result<f64, TradeError> {
        // Use the common performance calculation utility
        calculate_basic_performance(data, signals, 10000.0)
    }
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions. 