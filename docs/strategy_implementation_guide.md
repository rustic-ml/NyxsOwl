# Strategy Implementation Guide for NyxsOwl

This document outlines the comprehensive strategy implementation approach for NyxsOwl, covering both forecasting-based strategies and technical indicator-based strategies. This serves as the primary reference for code organization, testing standards, and development practices.

## Table of Contents

1. [Dependencies and Version Requirements](#dependencies-and-version-requirements)
2. [Architecture Overview](#architecture-overview)
3. [Module Organization](#module-organization)
4. [Strategy Implementation Patterns](#strategy-implementation-patterns)
5. [Testing Standards](#testing-standards)
6. [Example Requirements](#example-requirements)
7. [Code Coverage Requirements](#code-coverage-requirements)
8. [Data Sources](#data-sources)
9. [Development Workflow](#development-workflow)

## Dependencies and Version Requirements

### Core Dependencies

The project uses the following key dependencies with specific version requirements:

- **Polars**: 0.47.0 for data processing and manipulation (latest stable version compatible with our API usage)
- **Chrono**: 0.4 for date/time handling  
- **Serde**: 1.0 for serialization
- **Thiserror**: 1.0 for error handling
- **OxiDiviner**: 1.1.0 for forecasting models

### Version Compatibility Strategy

**Use the latest stable versions of dependencies**, but note that the current codebase requires migration for modern Polars versions. 

#### Current Compatibility Status

- **Polars 0.47.x - 0.48.x**: Requires significant API migration due to:
  - String type changes (`&str` → `PlSmallStr`)
  - Column vs Series type distinctions
  - Rolling window API changes (need to import `SeriesOpsTime` trait)
  - Method relocations (`abs()`, `rolling_sum()`, etc.)
  - Error type compatibility (`PolarsError` → `NyxsOwlError`)

- **OxiDiviner 1.1.0**: Stable and compatible

#### Migration Requirements

Before implementing additional forecasting strategies, the following migration work is needed:

1. **Type System Updates**:
   - Replace `&str` with `PlSmallStr` throughout the codebase
   - Handle `Column` vs `Series` type differences
   - Update `Series::new()` calls to use proper string types

2. **API Updates**:
   - Import required traits (`SeriesOpsTime`, etc.)
   - Update rolling window function calls
   - Fix method relocations and renames

3. **Error Handling**:
   - Implement `From<PolarsError>` for `NyxsOwlError`
   - Update error conversion patterns throughout

4. **Testing**:
   - Ensure all tests pass after migration
   - Verify examples compile and run correctly

#### Recommended Approach

1. **Phase 1**: Complete the migration to modern Polars API
2. **Phase 2**: Implement additional forecasting strategies with updated API
3. **Phase 3**: Add new features and optimizations

For now, new strategy development should focus on the conceptual implementation patterns while noting the API migration requirements.

### Dependency Management

```toml
[dependencies]
# Core data processing - use latest stable compatible version
polars = { version = "0.47.0", features = ["lazy", "strings", "temporal", "rolling_window", "parquet", "dtype-categorical", "dtype-struct", "dtype-full", "csv", "ewma"] }

# Date/time handling
chrono = { version = "0.4", features = ["serde"] }

# Error handling and serialization
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

# Forecasting models (optional)
oxidiviner = { version = "1.1.0", optional = true }

# Development dependencies
[dev-dependencies]
pretty_assertions = "1.4"
tempfile = "3.0"
```

### Checking for Updates

Before upgrading dependencies:

1. **Check breaking changes**: Review changelog and migration guides
2. **Test incrementally**: Upgrade one major dependency at a time  
3. **Run tests**: Ensure all tests pass after each upgrade
4. **Update documentation**: Note any API changes in this guide

Example check:
```bash
# Check for dependency updates
cargo outdated

# Update to latest compatible versions
cargo update

# Run full test suite
cargo test
```

## Architecture Overview

NyxsOwl implements a dual-track approach to trading strategies:

### Forecasting Strategies (`forecasting/` module)
- Leverage OxiDiviner models for time series forecasting
- Implement strategies based on predicted future values
- Support for ARIMA, Exponential Smoothing, Ensemble Methods, Kalman Filters, GARCH, Copula Models, and Regime-Switching Models

### Technical Indicator Strategies (`technical_strategies/` module)
- Based on traditional technical analysis indicators
- Organized by indicator categories (trend, momentum, volatility, etc.)
- Implement signal generation based on indicator calculations

## Module Organization

### Directory Structure

```
nyxs_owl/src/
├── forecasting/                    # Forecasting-based strategies
│   ├── mod.rs                     # Module declarations and exports
│   ├── forecast_trade.rs          # Core forecasting utilities
│   ├── strategies/                # Strategy implementations
│   │   ├── mod.rs                 
│   │   ├── arima_strategy.rs      # ARIMA-based strategies
│   │   ├── exponential_smoothing_strategy.rs
│   │   ├── ensemble_strategy.rs   
│   │   ├── kalman_strategy.rs     
│   │   ├── garch_strategy.rs      
│   │   ├── copula_strategy.rs     
│   │   └── regime_switching_strategy.rs
│   ├── backtest/                  # Forecasting strategy backtesting
│   │   ├── mod.rs
│   │   └── forecast_backtest.rs
│   └── utils/                     # Forecasting utilities
│       ├── mod.rs
│       └── forecast_utils.rs
├── technical_strategies/           # Technical indicator strategies
│   ├── mod.rs                     # Module declarations  
│   ├── trend/                     # Trend-based strategies
│   │   ├── mod.rs
│   │   ├── adx_di_strategy.rs
│   │   ├── aroon_strategy.rs
│   │   ├── ichimoku_strategy.rs
│   │   ├── psar_strategy.rs
│   │   └── vortex_strategy.rs
│   ├── momentum/                  # Momentum strategies
│   │   ├── mod.rs
│   │   ├── rsi_strategy.rs
│   │   ├── macd_strategy.rs
│   │   ├── roc_strategy.rs
│   │   ├── stochastic_strategy.rs
│   │   └── trix_strategy.rs
│   ├── volatility/               # Volatility strategies
│   │   ├── mod.rs
│   │   ├── bollinger_bands_strategy.rs
│   │   ├── atr_strategy.rs
│   │   └── volatility_breakout_strategy.rs
│   ├── oscillators/              # Oscillator strategies
│   │   ├── mod.rs
│   │   └── williams_r_strategy.rs
│   └── moving_averages/          # Moving average strategies
│       ├── mod.rs
│       ├── sma_crossover_strategy.rs
│       ├── ema_crossover_strategy.rs
│       └── vwap_strategy.rs
└── examples/                     # Strategy examples
    ├── forecasting_examples/     # Forecasting strategy examples
    │   ├── arima_strategy_example.rs
    │   ├── ensemble_strategy_example.rs
    │   └── regime_switching_example.rs
    └── technical_examples/       # Technical strategy examples
        ├── trend_following_example.rs
        ├── momentum_example.rs
        └── mean_reversion_example.rs
```

### Data Sources

All examples must use OHLCV data files located in `examples/csv/`:

#### Daily Data Files
- `AAPL_daily_ohlcv.csv`, `MSFT_daily_ohlcv.csv`, `NVDA_daily_ohlcv.csv`
- `GOOGL_daily_ohlcv.csv`, `AMZN_daily_ohlcv.csv`, `TSLA_daily_ohlcv.csv`
- `META_daily_ohlcv.csv`, `TSM_daily_ohlcv.csv`
- Alternative formats: `daily_data.csv`, `aapl_daily.csv`, etc.

#### Minute Data Files  
- `AAPL_minute_ohlcv.csv`, `MSFT_minute_ohlcv.csv`, `NVDA_minute_ohlcv.csv`
- `GOOGL_minute_ohlcv.csv`, `AMZN_minute_ohlcv.csv`, `TSLA_minute_ohlcv.csv`
- `META_minute_ohlcv.csv`, `TSM_minute_ohlcv.csv`
- Alternative formats: `minute_data.csv`, `aapl_minute.csv`

#### Parquet Files
- All CSV files also available as `.parquet` for faster loading
- Same naming convention with `.parquet` extension

## Strategy Implementation Patterns

### Common Strategy Structure

All strategies should follow this consistent pattern:

```rust
// Example: src/forecasting/strategies/arima_strategy.rs
use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::*;

/// Configuration for ARIMA strategy
#[derive(Debug, Clone)]
pub struct ArimaStrategyConfig {
    pub p: usize,              // AR order
    pub d: usize,              // Integration order  
    pub q: usize,              // MA order
    pub threshold: f64,        // Signal threshold
    pub min_data_points: usize, // Minimum data required
}

impl Default for ArimaStrategyConfig {
    fn default() -> Self {
        Self {
            p: 1,
            d: 1, 
            q: 1,
            threshold: 0.01,
            min_data_points: 60,
        }
    }
}

/// ARIMA-based trading strategy
pub struct ArimaStrategy {
    config: ArimaStrategyConfig,
}

impl ArimaStrategy {
    pub fn new(config: ArimaStrategyConfig) -> Self {
        Self { config }
    }
    
    /// Generate trading signals based on ARIMA forecasts
    pub fn generate_signals(
        &self,
        df: &DataFrame,
        price_column: &str,
        timestamp_column: &str,
    ) -> Result<Vec<Signal>> {
        // Validate inputs
        self.validate_inputs(df, price_column, timestamp_column)?;
        
        // Extract price data
        let prices = self.extract_prices(df, price_column)?;
        let timestamps = self.extract_timestamps(df, timestamp_column)?;
        
        // Generate forecasts
        let forecasts = self.generate_forecasts(&prices, &timestamps)?;
        
        // Convert forecasts to signals
        let signals = self.forecasts_to_signals(&prices, &forecasts)?;
        
        Ok(signals)
    }
    
    // Private helper methods
    fn validate_inputs(&self, df: &DataFrame, price_col: &str, timestamp_col: &str) -> Result<()> {
        if df.height() < self.config.min_data_points {
            return Err(NyxsOwlError::MissingData(format!(
                "Insufficient data: {} rows, need at least {}", 
                df.height(), self.config.min_data_points
            )));
        }
        
        // Validate columns exist
        df.column(price_col).map_err(|e| 
            NyxsOwlError::DataError(format!("Price column '{}' not found: {}", price_col, e))
        )?;
        
        df.column(timestamp_col).map_err(|e|
            NyxsOwlError::DataError(format!("Timestamp column '{}' not found: {}", timestamp_col, e))
        )?;
        
        Ok(())
    }
    
    // Additional helper methods...
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    
    fn create_test_data() -> PolarsResult<DataFrame> {
        // Create synthetic test data
        let timestamps: Vec<String> = (0..100)
            .map(|i| format!("2023-01-{:02} 09:30:00", (i % 30) + 1))
            .collect();
            
        let prices: Vec<f64> = (0..100)
            .map(|i| 100.0 + (i as f64 * 0.1) + (i as f64 * 0.1).sin() * 5.0)
            .collect();
            
        df! {
            "timestamp" => timestamps,
            "close" => prices,
        }
    }
    
    #[test]
    fn test_arima_strategy_creation() {
        let config = ArimaStrategyConfig::default();
        let strategy = ArimaStrategy::new(config);
        assert_eq!(strategy.config.p, 1);
        assert_eq!(strategy.config.d, 1);
        assert_eq!(strategy.config.q, 1);
    }
    
    #[test]
    fn test_insufficient_data() {
        let config = ArimaStrategyConfig {
            min_data_points: 50,
            ..Default::default()
        };
        let strategy = ArimaStrategy::new(config);
        
        // Create insufficient data
        let df = df! {
            "timestamp" => vec!["2023-01-01"; 10],
            "close" => vec![100.0; 10],
        }.unwrap();
        
        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(matches!(result, Err(NyxsOwlError::MissingData(_))));
    }
    
    #[test]
    fn test_missing_columns() {
        let strategy = ArimaStrategy::new(ArimaStrategyConfig::default());
        let df = create_test_data().unwrap();
        
        // Test missing price column
        let result = strategy.generate_signals(&df, "missing_price", "timestamp");
        assert!(matches!(result, Err(NyxsOwlError::DataError(_))));
        
        // Test missing timestamp column
        let result = strategy.generate_signals(&df, "close", "missing_timestamp");
        assert!(matches!(result, Err(NyxsOwlError::DataError(_))));
    }
    
    #[test]
    fn test_signal_generation() {
        let strategy = ArimaStrategy::new(ArimaStrategyConfig::default());
        let df = create_test_data().unwrap();
        
        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(result.is_ok());
        
        let signals = result.unwrap();
        assert_eq!(signals.len(), df.height());
        
        // Verify signals are valid
        for signal in &signals {
            assert!(matches!(signal, Signal::Buy | Signal::Sell | Signal::Hold));
        }
    }
}
```

### Strategy Naming Conventions

#### File Naming
- Use snake_case: `arima_strategy.rs`, `rsi_momentum_strategy.rs`
- Include strategy type in name: `_strategy.rs` suffix
- Group by primary indicator/technique

#### Struct Naming  
- PascalCase for structs: `ArimaStrategy`, `RsiMomentumStrategy`
- Config structs: `ArimaStrategyConfig`
- Use descriptive names that indicate the strategy approach

#### Function Naming
- snake_case for functions: `generate_signals()`, `validate_inputs()`
- Primary public method: `generate_signals()`
- Helper methods: `validate_inputs()`, `extract_prices()`, etc.

### Return Types and Error Handling

All strategy functions must use the crate's unified error system:

```rust
use crate::simple_types::{NyxsOwlError, Result, Signal};

// All strategy methods return Result<T>
pub fn generate_signals(&self, ...) -> Result<Vec<Signal>> {
    // Implementation
}

// Error types to use:
NyxsOwlError::InvalidParameter(msg)  // Invalid configuration
NyxsOwlError::DataError(msg)         // Data parsing/validation issues  
NyxsOwlError::StrategyError(msg)     // Strategy execution errors
NyxsOwlError::MissingData(msg)       // Insufficient data
NyxsOwlError::NotImplemented(msg)    // Placeholder for future features
```

## Testing Standards

### Code Coverage Requirements
- **Minimum 80% code coverage** for all strategy modules
- Each strategy must have comprehensive unit tests
- Integration tests for complex strategies
- Performance benchmarks for computationally intensive strategies

### Test Categories

#### Unit Tests (Required for all strategies)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_strategy_creation() {
        // Test strategy initialization
    }
    
    #[test] 
    fn test_invalid_parameters() {
        // Test parameter validation
    }
    
    #[test]
    fn test_insufficient_data() {
        // Test data length validation
    }
    
    #[test]
    fn test_missing_columns() {
        // Test column validation
    }
    
    #[test]
    fn test_signal_generation() {
        // Test core signal generation logic
    }
    
    #[test]
    fn test_edge_cases() {
        // Test boundary conditions
    }
}
```

#### Integration Tests (For complex strategies)
```rust
// tests/integration/forecasting_strategies.rs
use nyxs_owl::forecasting::strategies::ArimaStrategy;
use nyxs_owl::simple_types::Signal;

#[test]
fn test_arima_strategy_with_real_data() {
    // Test with actual market data from CSV files
}
```

#### Performance Tests
``` rust
#[cfg(test)]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_arima_strategy(c: &mut Criterion) {
        c.bench_function("arima_strategy_1000_points", |b| {
            b.iter(|| {
                // Benchmark strategy execution
            })
        });
    }
    
    criterion_group!(benches, bench_arima_strategy);
    criterion_main!(benches);
}
```

### Test Data Guidelines

#### Synthetic Data Creation
```rust
fn create_test_data(len: usize) -> PolarsResult<DataFrame> {
    let timestamps: Vec<String> = (0..len)
        .map(|i| format!("2023-01-{:02} 09:30:00", (i % 30) + 1))
        .collect();
        
    let prices: Vec<f64> = (0..len)
        .map(|i| 100.0 + (i as f64 * 0.1) + (i as f64 * 0.1).sin() * 5.0)
        .collect();
        
    df! {
        "timestamp" => timestamps,
        "close" => prices,
        "high" => prices.iter().map(|p| p * 1.02).collect::<Vec<_>>(),
        "low" => prices.iter().map(|p| p * 0.98).collect::<Vec<_>>(),
        "open" => prices,
        "volume" => vec![1000i64; len],
    }
}
```

## Example Requirements

### Mandatory Examples

Every strategy **must** have a corresponding example in the `examples/` directory:

#### Forecasting Strategy Examples
- File: `examples/forecasting_examples/{strategy_name}_example.rs`
- Must use real OHLCV data from `examples/csv/`
- Demonstrate configuration options
- Show performance metrics
- Include visualization output (optional)

#### Technical Strategy Examples  
- File: `examples/technical_examples/{strategy_name}_example.rs`
- Must use real OHLCV data from `examples/csv/`
- Show different parameter configurations
- Compare with benchmark (buy-and-hold)
- Include backtesting results

### Example Template

```rust
// examples/forecasting_examples/arima_strategy_example.rs
use nyxs_owl::forecasting::strategies::{ArimaStrategy, ArimaStrategyConfig};
use nyxs_owl::simple_types::{Signal, Result};
use polars::prelude::*;
use std::path::Path;

fn main() -> Result<()> {
    println!("ARIMA Strategy Example");
    println!("======================");
    
    // Load data
    let data_path = "examples/csv/AAPL_daily_ohlcv.csv";
    let df = load_ohlcv_data(data_path)?;
    
    // Configure strategy
    let config = ArimaStrategyConfig {
        p: 2,
        d: 1,
        q: 1,
        threshold: 0.02,
        min_data_points: 100,
    };
    
    // Initialize strategy
    let strategy = ArimaStrategy::new(config);
    
    // Generate signals
    let signals = strategy.generate_signals(&df, "close", "timestamp")?;
    
    // Analyze results
    analyze_signals(&signals);
    
    // Backtest (optional)
    let performance = backtest_strategy(&df, &signals)?;
    print_performance_metrics(&performance);
    
    Ok(())
}

fn load_ohlcv_data(path: &str) -> Result<DataFrame> {
    LazyFrame::scan_csv(path, ScanArgsCSV::default())
        .map_err(|e| NyxsOwlError::DataError(format!("Failed to load CSV: {}", e)))?
        .collect()
        .map_err(|e| NyxsOwlError::DataError(format!("Failed to collect data: {}", e)))
}

fn analyze_signals(signals: &[Signal]) {
    let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
    let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
    
    println!("Signal Analysis:");
    println!("  Buy signals: {}", buy_count);
    println!("  Sell signals: {}", sell_count);
    println!("  Hold signals: {}", hold_count);
    println!("  Total signals: {}", signals.len());
}

// Additional helper functions...
```

### Running Examples

All examples must be runnable with:
```bash
# Forecasting examples
cargo run --example arima_strategy_example --features forecasting

# Technical examples  
cargo run --example rsi_momentum_example

# With specific data file
OHLCV_FILE=NVDA_daily_ohlcv.csv cargo run --example arima_strategy_example --features forecasting
```

## Development Workflow

### 1. Strategy Planning
- Define strategy logic and parameters
- Choose appropriate module (forecasting vs technical)
- Review existing similar strategies for patterns

### 2. Implementation
- Create strategy struct and config
- Implement core `generate_signals()` method
- Add comprehensive input validation
- Follow established naming conventions

### 3. Testing
- Write unit tests for all public methods
- Test edge cases and error conditions
- Achieve minimum 80% code coverage
- Add integration tests if needed

### 4. Example Creation
- Create runnable example using real data
- Document configuration options
- Show performance analysis
- Include comparison with benchmarks

### 5. Documentation
- Add rustdoc comments to all public items
- Update module documentation
- Add strategy to appropriate category guide

### 6. Integration
- Update module `mod.rs` files
- Add exports to prelude if appropriate
- Update `Cargo.toml` features if needed
- Test compilation and examples

## Feature Gates

### Forecasting Strategies
Forecasting strategies require the `forecasting` feature:
```toml
[features]
default = []
forecasting = ["oxidiviner"]
```

Usage:
```rust
#[cfg(feature = "forecasting")]
pub mod forecasting;
```

### Optional Dependencies
Some strategies may require additional dependencies:
```toml
[dependencies]
# Required
polars = { version = "0.35", features = ["lazy", "csv", "temporal"] }
thiserror = "1.0"

# Optional
# This is required for forecasting module.
oxidiviner = { version = "1.1", optional = true }
ta = { version = "0.5", optional = true }
```

## Quality Checklist

Before submitting any strategy implementation, ensure:

- [ ] Follows established file and naming conventions
- [ ] Implements required public interface (`generate_signals()`)
- [ ] Includes comprehensive configuration struct
- [ ] Has proper error handling using `NyxsOwlError`
- [ ] Validates all inputs thoroughly
- [ ] Achieves minimum 80% test coverage
- [ ] Includes unit tests for all major code paths
- [ ] Has runnable example using real OHLCV data
- [ ] Generates meaningful trading signals
- [ ] Documentation includes usage examples
- [ ] Code passes `cargo clippy` without warnings
- [ ] Follows Rust formatting standards (`cargo fmt`)

## Conclusion

This guide establishes the foundation for consistent, high-quality strategy implementation in NyxsOwl. By following these patterns and standards, we ensure:

- **Consistency**: All strategies follow the same patterns
- **Reliability**: Comprehensive testing catches edge cases  
- **Usability**: Clear examples demonstrate real-world usage
- **Maintainability**: Well-organized code is easier to extend
- **Performance**: Benchmarks ensure strategies scale appropriately

For questions or clarifications on this guide, refer to existing strategy implementations or raise an issue for discussion. 