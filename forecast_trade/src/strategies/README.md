# Forecast Trade - Strategies Module

This module contains trading strategies that use forecasting models to generate trading signals.

## Time Granularity Support

A key feature of the strategies in this module is support for multiple time granularities. This allows traders to apply the same strategy concepts to different timeframes with appropriate parameter adjustments.

### TimeGranularity Enum

The `TimeGranularity` enum defines the supported time granularities:

```rust
pub enum TimeGranularity {
    Daily,   // Daily data (OHLCV bars with daily intervals)
    Minute,  // Minute data (OHLCV bars with minute intervals)
}
```

### Parameter Scaling

Each strategy automatically scales its parameters based on the selected time granularity:

1. **Daily Data (default)**
   - Longer lookback periods (e.g., 20-day moving averages)
   - Lower commission rates (typically 0.1%)
   - Lower slippage assumptions (typically 0.05%)
   - Lower volatility thresholds

2. **Minute Data**
   - Shorter lookback periods in absolute terms, but longer in market time (e.g., 60-minute moving averages)
   - Lower commission rates (typically 0.05%)
   - Higher slippage assumptions (typically 0.1%)
   - Higher volatility thresholds to filter out noise

### Using Different Granularities

Each strategy provides methods to create instances with specific time granularities:

```rust
// Create a strategy with default daily parameters
let daily_strategy = MeanReversionStrategy::new(model, threshold)?;

// Create a strategy with minute parameters
let minute_strategy = MeanReversionStrategy::new_with_granularity(
    model, 
    threshold,
    TimeGranularity::Minute
)?;
```

### Transaction Cost Models

Backtesting automatically uses appropriate transaction cost models based on the time granularity:

```rust
// Uses default transaction costs for the given granularity
let backtest_results = strategy.backtest(&data, initial_capital)?;

// Or specify custom transaction costs
let custom_results = strategy.backtest_with_params(
    &data, 
    initial_capital, 
    commission_rate, 
    slippage
)?;
```

## Available Strategies

1. **Mean Reversion Strategy**
   - Detects when prices deviate significantly from their expected value
   - Assumes prices will revert to the mean

2. **Trend Following Strategy**
   - Identifies market momentum and trends
   - Generates signals to trade in the direction of the trend

3. **Volatility Breakout Strategy**
   - Detects significant price movements that break through volatility bands
   - Adapts thresholds based on recent market volatility

## Examples

See the examples directory for detailed examples of how to use these strategies with different time granularities:

- `daily_vs_minute_strategy.rs` - Comparison of strategies on daily vs minute data
- `time_granularity_example.rs` - Detailed example of granularity-specific features 