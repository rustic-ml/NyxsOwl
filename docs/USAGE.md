# NyxsOwl Usage Guide

## Overview

This guide provides comprehensive usage instructions for NyxsOwl's forecasting capabilities, including practical examples, configuration options, and best practices for building adaptive trading strategies with OxiDiviner 1.2.0.

## Table of Contents

1. [Quick Start](#quick-start)
2. [OxiDiviner 1.2.0 Adaptive Features](#oxidiviner-120-adaptive-features)
3. [Strategy Configuration Guide](#strategy-configuration-guide)
4. [Practical Examples](#practical-examples)
5. [Backtesting & Performance Analysis](#backtesting--performance-analysis)
6. [Best Practices](#best-practices)

## Quick Start

### Installation

Add NyxsOwl to your `Cargo.toml`:

```toml
[dependencies]
nyxs_owl = { path = ".", features = ["forecasting"] }
oxidiviner = { version = "1.2.0", features = ["adaptive"] }
polars = { version = "0.47.0", features = ["lazy", "csv"] }
```

### Basic Usage Example

```rust
use nyxs_owl::forecasting::strategies::{ArimaStrategy, ArimaStrategyConfig};
use polars::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load market data
    let df = LazyFrame::scan_csv("examples/csv/AAPL_daily.csv", ScanArgsCSV::default())?
        .collect()?;
    
    // Create strategy with adaptive features
    let config = ArimaStrategyConfig {
        model_selection: true,      // Auto-select optimal parameters
        dynamic_threshold: true,    // Volatility-based thresholds
        regime_detection: true,     // Market regime awareness
        ..ArimaStrategyConfig::default()
    };
    
    let mut strategy = ArimaStrategy::new(config);
    
    // Generate trading signals
    let signals = strategy.generate_signals(&df, "close", "timestamp")?;
    
    println!("Generated {} signals", signals.len());
    for signal in signals.iter().take(5) {
        println!("{:?}", signal);
    }
    
    Ok(())
}
```

## OxiDiviner 1.2.0 Adaptive Features

### Key Enhancements

#### 🔄 **Dynamic Model Selection & Parameter Adaptation**
- **Automatic Order Selection**: Models automatically determine optimal parameters
- **Adaptive Parameter Tuning**: Real-time adjustment based on performance and volatility
- **Regime-Aware Forecasting**: Models detect and adapt to different market regimes

#### 📊 **Enhanced Ensemble Intelligence**
- **Dynamic Model Weighting**: Ensemble weights automatically adjust based on forecast accuracy
- **Performance-Based Selection**: Models selected dynamically based on rolling performance
- **Meta-Learning Integration**: Advanced stacking approaches for optimal combination

#### 🛡️ **Robust Data Quality & Preprocessing**
- **Intelligent Outlier Detection**: Multiple detection methods with adaptive thresholds
- **Missing Data Handling**: Smart interpolation strategies
- **Numerical Stability Enhancements**: Advanced handling of edge cases

### Enhanced ARIMA with Adaptive Features

```rust
use nyxs_owl::forecasting::strategies::{ArimaStrategy, ArimaStrategyConfig};

let config = ArimaStrategyConfig {
    // Enable adaptive features
    model_selection: true,      // Automatic (p,d,q) order selection
    dynamic_threshold: true,    // Volatility-based signal thresholds
    outlier_detection: true,    // IQR-based outlier detection
    regime_detection: true,     // Market regime identification
    adaptive_refit: true,       // Performance-based model refitting
    
    // Adaptive parameters
    volatility_lookback: 30,    // Rolling volatility window
    volatility_multiplier: 2.0, // Threshold scaling factor
    
    // Traditional ARIMA parameters (fallback)
    p: 1, d: 1, q: 1,
    threshold: 0.02,
    min_data_points: 50,
    
    ..ArimaStrategyConfig::default()
};

let mut strategy = ArimaStrategy::new(config);
let signals = strategy.generate_signals(&df, "close", "timestamp")?;

// Check current market regime
if let Some(regime) = strategy.get_current_regime() {
    println!("Current market regime: {:?}", regime);
}
```

### NEW: Adaptive Ensemble Strategy

```rust
use nyxs_owl::forecasting::strategies::{AdaptiveEnsembleStrategy, AdaptiveEnsembleConfig, ModelType};

let config = AdaptiveEnsembleConfig {
    // Enable adaptive features
    adaptive_weighting: true,   // Performance-based model weighting
    regime_detection: true,     // Regime-aware parameter adjustment
    quality_monitoring: true,   // Real-time quality tracking
    
    // Performance tracking
    performance_window: 50,     // Rolling performance window
    weight_decay_factor: 0.95,  // Weight decay for old performance
    quality_threshold: 0.6,     // Quality degradation threshold
    
    // Model ensemble
    models: vec![
        ModelType::ARIMA,
        ModelType::ExponentialSmoothing,
        ModelType::KalmanFilter,
    ],
    
    ..AdaptiveEnsembleConfig::default()
};

let mut strategy = AdaptiveEnsembleStrategy::new(config);
let signals = strategy.generate_signals(&df, "close", "timestamp")?;

// Monitor ensemble performance
let model_weights = strategy.get_current_weights();
println!("Current model weights: {:?}", model_weights);
```

## Strategy Configuration Guide

### Available Forecasting Strategies

#### 1. Enhanced ARIMA Strategy

**Best for**: Trending markets, time series with clear patterns

```rust
let config = ArimaStrategyConfig {
    // Traditional parameters
    p: 1,                      // Auto-regressive order
    d: 1,                      // Differencing order
    q: 1,                      // Moving average order
    threshold: 0.02,           // 2% signal threshold
    
    // Enhanced adaptive features
    model_selection: true,     // Auto (p,d,q) selection
    dynamic_threshold: true,   // Adaptive thresholds
    outlier_detection: true,   // Data cleaning
    regime_detection: true,    // Market regime awareness
    adaptive_refit: true,      // Performance-based refitting
    
    // Configuration parameters
    volatility_lookback: 30,   // Volatility calculation window
    volatility_multiplier: 2.0, // Threshold scaling
    min_data_points: 50,       // Minimum data for fitting
    
    ..ArimaStrategyConfig::default()
};
```

#### 2. Adaptive Ensemble Strategy

**Best for**: Robust forecasting, combining multiple models

```rust
let config = AdaptiveEnsembleConfig {
    // Adaptive features
    adaptive_weighting: true,   // Dynamic model weighting
    regime_detection: true,     // Regime-aware adaptation
    quality_monitoring: true,   // Real-time quality tracking
    
    // Performance tracking
    performance_window: 50,     // Rolling performance window
    weight_decay_factor: 0.95,  // Weight decay factor
    min_model_weight: 0.05,     // Minimum model weight
    quality_threshold: 0.6,     // Quality degradation threshold
    
    // Model ensemble
    models: vec![
        ModelType::ARIMA,
        ModelType::ExponentialSmoothing,
        ModelType::KalmanFilter,
    ],
    
    // Signal generation
    threshold: 0.02,
    min_models_agreement: 2,    // Minimum models for signal
    
    ..AdaptiveEnsembleConfig::default()
};
```

## Practical Examples

### Example 1: Basic Trend Following

```rust
use nyxs_owl::forecasting::strategies::{ArimaStrategy, ArimaStrategyConfig};
use nyxs_owl::forecasting::backtest::{backtest_strategy, BacktestConfig};

fn trend_following_example() -> Result<(), Box<dyn std::error::Error>> {
    // Load data
    let df = LazyFrame::scan_csv("examples/csv/AAPL_daily.csv", ScanArgsCSV::default())?
        .collect()?;
    
    // Configure enhanced ARIMA for trend following
    let config = ArimaStrategyConfig {
        // Enable adaptive features for better trend detection
        model_selection: true,      // Auto-select best (p,d,q)
        dynamic_threshold: true,    // Adapt to volatility
        regime_detection: true,     // Detect trending vs ranging
        
        // Trend-following parameters
        p: 2, d: 1, q: 1,          // Favor AR terms for trends
        threshold: 0.015,          // 1.5% signal threshold
        volatility_multiplier: 1.5, // Conservative scaling
        
        ..ArimaStrategyConfig::default()
    };
    
    let mut strategy = ArimaStrategy::new(config);
    let signals = strategy.generate_signals(&df, "close", "timestamp")?;
    
    // Backtest the strategy
    let backtest_config = BacktestConfig {
        initial_capital: 10000.0,
        commission: 0.001,         // 0.1% commission
        slippage: 0.0005,          // 0.05% slippage
        max_position_size: 1.0,    // 100% allocation
    };
    
    let performance = backtest_strategy(&strategy, &df, backtest_config)?;
    
    // Display results
    println!("=== Trend Following Strategy Results ===");
    println!("Total Return: {:.2}%", performance.total_return * 100.0);
    println!("Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("Win Rate: {:.1}%", performance.win_rate * 100.0);
    
    Ok(())
}
```

### Example 2: Multi-Model Ensemble Strategy

```rust
use nyxs_owl::forecasting::strategies::{AdaptiveEnsembleStrategy, AdaptiveEnsembleConfig, ModelType};

fn ensemble_strategy_example() -> Result<(), Box<dyn std::error::Error>> {
    let df = LazyFrame::scan_csv("examples/csv/AAPL_daily.csv", ScanArgsCSV::default())?
        .collect()?;
    
    // Configure adaptive ensemble
    let config = AdaptiveEnsembleConfig {
        // Enable all adaptive features
        adaptive_weighting: true,
        regime_detection: true,
        quality_monitoring: true,
        
        // Performance tracking
        performance_window: 30,     // 30-day performance window
        weight_decay_factor: 0.9,   // Emphasize recent performance
        quality_threshold: 0.7,     // Quality degradation alert
        
        // Model ensemble
        models: vec![
            ModelType::ARIMA,          // Trend detection
            ModelType::ExponentialSmoothing, // Seasonality
            ModelType::KalmanFilter,   // Noise filtering
        ],
        
        // Signal generation
        threshold: 0.02,
        min_models_agreement: 2,    // Require 2/3 models
        
        ..AdaptiveEnsembleConfig::default()
    };
    
    let mut strategy = AdaptiveEnsembleStrategy::new(config);
    let signals = strategy.generate_signals(&df, "close", "timestamp")?;
    
    // Monitor ensemble health
    println!("=== Ensemble Strategy Monitoring ===");
    
    if let Some(regime) = strategy.get_current_regime() {
        println!("Current Market Regime: {:?}", regime);
    }
    
    let weights = strategy.get_current_weights();
    println!("Model Weights:");
    for (model, weight) in weights.iter() {
        println!("  {:?}: {:.3}", model, weight);
    }
    
    let quality = strategy.get_ensemble_quality();
    println!("Ensemble Quality Score: {:.3}", quality);
    
    // Check for alerts
    let alerts = strategy.get_degradation_alerts();
    if !alerts.is_empty() {
        println!("⚠️ Performance Alerts:");
        for alert in alerts {
            println!("  {}", alert);
        }
    }
    
    Ok(())
}
```

## Backtesting & Performance Analysis

### Backtesting Configuration

```rust
use nyxs_owl::forecasting::backtest::{BacktestConfig, PerformanceMetrics};

let backtest_config = BacktestConfig {
    // Capital management
    initial_capital: 100000.0,  // $100k starting capital
    max_position_size: 0.95,    // Maximum 95% allocation
    
    // Transaction costs
    commission: 0.001,          // 0.1% commission per trade
    slippage: 0.0005,          // 0.05% slippage
    
    // Risk management
    stop_loss: Some(0.05),      // 5% stop loss
    take_profit: Some(0.15),    // 15% take profit
    max_drawdown_limit: Some(0.20), // 20% maximum drawdown
    
    ..BacktestConfig::default()
};
```

### Performance Analysis

```rust
fn analyze_strategy_performance(
    strategy: &mut dyn ForecastingStrategy,
    data: &DataFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    let backtest_config = BacktestConfig {
        initial_capital: 100000.0,
        commission: 0.001,
        slippage: 0.0005,
        ..BacktestConfig::default()
    };
    
    // Run backtest
    let performance = backtest_strategy(strategy, data, backtest_config)?;
    
    // Display comprehensive analysis
    println!("=== Strategy Performance Analysis ===");
    println!();
    
    println!("📈 Return Metrics:");
    println!("  Total Return:      {:>8.2}%", performance.total_return * 100.0);
    println!("  Annualized Return: {:>8.2}%", performance.annualized_return * 100.0);
    println!();
    
    println!("⚖️ Risk Metrics:");
    println!("  Sharpe Ratio:      {:>8.3}", performance.sharpe_ratio);
    println!("  Maximum Drawdown:  {:>8.2}%", performance.max_drawdown * 100.0);
    println!();
    
    println!("📊 Trade Statistics:");
    println!("  Total Trades:      {:>8}", performance.total_trades);
    println!("  Win Rate:          {:>8.1}%", performance.win_rate * 100.0);
    println!("  Profit Factor:     {:>8.3}", performance.profit_factor);
    
    Ok(())
}
```

## Best Practices

### 1. Data Quality & Preprocessing

```rust
// Always validate and clean your data
fn preprocess_market_data(df: &DataFrame) -> Result<DataFrame, NyxsOwlError> {
    df.clone()
        .lazy()
        // Remove weekends and holidays
        .filter(col("timestamp").dt().weekday().lt(lit(6)))
        // Handle missing values
        .with_columns([
            col("close").fill_null(col("close").forward_fill()),
            col("volume").fill_null(lit(0)),
        ])
        // Remove extreme outliers (> 3 sigma)
        .filter(
            (col("close").log() - col("close").log().mean())
            .abs()
            .lt(col("close").log().std() * lit(3.0))
        )
        // Ensure chronological order
        .sort("timestamp", Default::default())
        .collect()
        .map_err(|e| NyxsOwlError::DataError(e.to_string()))
}
```

### 2. Strategy Configuration Guidelines

#### Start Conservative
```rust
// Begin with conservative parameters
let conservative_config = ArimaStrategyConfig {
    threshold: 0.03,           // Higher threshold = fewer trades
    volatility_multiplier: 1.0, // No volatility scaling initially
    min_data_points: 100,      // Require more data for robustness
    
    // Enable adaptive features gradually
    model_selection: true,     // Start with auto model selection
    dynamic_threshold: false,  // Add later after validation
    regime_detection: false,   // Add after understanding base behavior
    
    ..ArimaStrategyConfig::default()
};
```

### 3. Risk Management

#### Position Sizing
```rust
fn calculate_position_size(
    signal_strength: f64,
    current_volatility: f64,
    max_risk_per_trade: f64,
) -> f64 {
    // Kelly criterion with volatility adjustment
    let base_size = signal_strength * max_risk_per_trade;
    let volatility_adjusted = base_size / (1.0 + current_volatility * 5.0);
    
    // Cap at maximum position size
    volatility_adjusted.min(0.2) // Maximum 20% per position
}
```

### 4. Performance Monitoring

```rust
// Real-time strategy monitoring
struct StrategyMonitor {
    performance_history: Vec<PerformanceMetrics>,
    alert_thresholds: AlertThresholds,
}

impl StrategyMonitor {
    fn check_alerts(&self) -> Vec<String> {
        let mut alerts = Vec::new();
        
        if let Some(latest) = self.performance_history.last() {
            if latest.max_drawdown > 0.15 {
                alerts.push(format!(
                    "⚠️ High drawdown: {:.2}%",
                    latest.max_drawdown * 100.0
                ));
            }
            
            if latest.sharpe_ratio < 0.5 {
                alerts.push(format!(
                    "⚠️ Low Sharpe ratio: {:.3}",
                    latest.sharpe_ratio
                ));
            }
        }
        
        alerts
    }
}
```

## Migration from Previous Versions

### Upgrading to OxiDiviner 1.2.0

1. **Update Dependencies**:
```toml
# Update in Cargo.toml
oxidiviner = { version = "1.2.0", features = ["adaptive"] }
```

2. **Enable Adaptive Features**:
```rust
// Old configuration
let old_config = ArimaStrategyConfig {
    p: 1, d: 1, q: 1,
    threshold: 0.02,
    ..ArimaStrategyConfig::default()
};

// New configuration with adaptive features
let new_config = ArimaStrategyConfig {
    // Keep existing parameters
    p: 1, d: 1, q: 1,
    threshold: 0.02,
    
    // Add adaptive enhancements
    model_selection: true,     // Auto parameter selection
    dynamic_threshold: true,   // Volatility-based thresholds
    outlier_detection: true,   // Data cleaning
    regime_detection: true,    // Market regime awareness
    adaptive_refit: true,      // Performance-based refitting
    
    ..ArimaStrategyConfig::default()
};
```

3. **Utilize New Features**:
```rust
let mut strategy = ArimaStrategy::new(new_config);
let signals = strategy.generate_signals(&df, "close", "timestamp")?;

// NEW: Monitor regime and performance
if let Some(regime) = strategy.get_current_regime() {
    println!("Current regime: {:?}", regime);
}

let alerts = strategy.get_degradation_alerts();
if !alerts.is_empty() {
    println!("Performance alerts: {:?}", alerts);
}
```

### Backward Compatibility

All existing configurations continue to work unchanged. New adaptive features are opt-in via configuration flags, ensuring smooth migration without breaking existing code.

## Example: Complete Trading System

```rust
use nyxs_owl::forecasting::strategies::{AdaptiveEnsembleStrategy, AdaptiveEnsembleConfig};
use nyxs_owl::forecasting::backtest::{backtest_strategy, BacktestConfig};

fn complete_trading_system() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load and preprocess data
    let raw_data = LazyFrame::scan_csv("examples/csv/AAPL_daily.csv", ScanArgsCSV::default())?
        .collect()?;
    let clean_data = preprocess_market_data(&raw_data)?;
    
    // 2. Configure adaptive ensemble strategy
    let strategy_config = AdaptiveEnsembleConfig {
        adaptive_weighting: true,
        regime_detection: true,
        quality_monitoring: true,
        performance_window: 30,
        models: vec![
            ModelType::ARIMA,
            ModelType::ExponentialSmoothing,
            ModelType::KalmanFilter,
        ],
        ..AdaptiveEnsembleConfig::default()
    };
    
    // 3. Create and run strategy
    let mut strategy = AdaptiveEnsembleStrategy::new(strategy_config);
    let signals = strategy.generate_signals(&clean_data, "close", "timestamp")?;
    
    // 4. Configure backtesting with risk management
    let backtest_config = BacktestConfig {
        initial_capital: 100000.0,
        commission: 0.001,
        slippage: 0.0005,
        stop_loss: Some(0.05),
        take_profit: Some(0.15),
        max_drawdown_limit: Some(0.20),
        ..BacktestConfig::default()
    };
    
    // 5. Run comprehensive backtest
    let performance = backtest_strategy(&strategy, &clean_data, backtest_config)?;
    
    // 6. Analyze and report results
    println!("=== Complete Trading System Results ===");
    println!("Total Return: {:.2}%", performance.total_return * 100.0);
    println!("Sharpe Ratio: {:.3}", performance.sharpe_ratio);
    println!("Max Drawdown: {:.2}%", performance.max_drawdown * 100.0);
    println!("Win Rate: {:.1}%", performance.win_rate * 100.0);
    println!("Total Trades: {}", performance.total_trades);
    
    // 7. Monitor strategy health
    if let Some(regime) = strategy.get_current_regime() {
        println!("Current Market Regime: {:?}", regime);
    }
    
    let quality_score = strategy.get_ensemble_quality();
    println!("Strategy Quality Score: {:.3}", quality_score);
    
    Ok(())
}
```

This usage guide provides comprehensive examples and best practices for leveraging NyxsOwl's enhanced forecasting capabilities with OxiDiviner 1.2.0's adaptive features. 