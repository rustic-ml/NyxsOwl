# Technical Indicator Strategy Implementation Guide

## Overview

This guide provides comprehensive implementation details for NyxsOwl's technical indicator strategies. NyxsOwl integrates with its trade_math module to provide production-ready technical analysis capabilities with advanced signal generation and strategy automation.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Moving Average Strategies](#moving-average-strategies)
3. [Momentum Indicator Strategies](#momentum-indicator-strategies)
4. [Volatility Indicator Strategies](#volatility-indicator-strategies)
5. [Multi-Indicator Systems](#multi-indicator-systems)
6. [Signal Generation Framework](#signal-generation-framework)
7. [Strategy Optimization](#strategy-optimization)
8. [Best Practices](#best-practices)

## Architecture Overview

### Core Strategy Pattern

All technical indicator strategies implement the `TechnicalStrategy` trait:

```rust
use nyxs_owl::trade_math::*;
use polars::prelude::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub trait TechnicalStrategy {
    type Config;
    
    fn new(config: Self::Config) -> Result<Self, NyxsOwlError> where Self: Sized;
    fn generate_signals(&mut self, df: &DataFrame) -> Result<Vec<Signal>, NyxsOwlError>;
    fn update_indicators(&mut self, price: f64, volume: Option<f64>) -> Result<(), NyxsOwlError>;
    fn get_strategy_name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct Signal {
    pub timestamp: DateTime<Utc>,
    pub signal_type: SignalType,
    pub strength: f64,
    pub price: f64,
    pub metadata: HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
    StrongBuy,
    StrongSell,
}
```

## Moving Average Strategies

### Simple Moving Average Crossover Strategy

```rust
use nyxs_owl::trade_math::moving_averages::SimpleMovingAverage;

#[derive(Debug, Clone)]
pub struct SMAStrategyConfig {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_threshold: f64,
}

impl Default for SMAStrategyConfig {
    fn default() -> Self {
        Self {
            fast_period: 10,
            slow_period: 30,
            signal_threshold: 0.01, // 1% minimum difference
        }
    }
}

pub struct SMAStrategy {
    config: SMAStrategyConfig,
    fast_sma: SimpleMovingAverage,
    slow_sma: SimpleMovingAverage,
}

impl TechnicalStrategy for SMAStrategy {
    type Config = SMAStrategyConfig;
    
    fn new(config: Self::Config) -> Result<Self, NyxsOwlError> {
        Ok(Self {
            fast_sma: SimpleMovingAverage::new(config.fast_period)?,
            slow_sma: SimpleMovingAverage::new(config.slow_period)?,
            config,
        })
    }
    
    fn generate_signals(&mut self, df: &DataFrame) -> Result<Vec<Signal>, NyxsOwlError> {
        let mut signals = Vec::new();
        
        let prices = df.column("close")?.f64()?.into_no_null_iter();
        let timestamps = df.column("timestamp")?.datetime()?.into_no_null_iter();
        
        for (price, timestamp) in prices.zip(timestamps) {
            // Update indicators
            self.fast_sma.update(price)?;
            self.slow_sma.update(price)?;
            
            // Generate signals when both SMAs have values
            if let (Some(fast), Some(slow)) = (self.fast_sma.value(), self.slow_sma.value()) {
                let signal = self.evaluate_crossover(fast, slow, price, timestamp)?;
                if signal.signal_type != SignalType::Hold {
                    signals.push(signal);
                }
            }
        }
        
        Ok(signals)
    }
    
    fn update_indicators(&mut self, price: f64, _volume: Option<f64>) -> Result<(), NyxsOwlError> {
        self.fast_sma.update(price)?;
        self.slow_sma.update(price)?;
        Ok(())
    }
    
    fn get_strategy_name(&self) -> &'static str {
        "SMA_Crossover"
    }
}

impl SMAStrategy {
    fn evaluate_crossover(
        &self, 
        fast: f64, 
        slow: f64, 
        price: f64, 
        timestamp_ns: i64
    ) -> Result<Signal, NyxsOwlError> {
        // Convert nanoseconds to DateTime
        let datetime = DateTime::from_timestamp_nanos(timestamp_ns);
        
        let diff_pct = (fast - slow) / slow;
        
        let (signal_type, strength) = if diff_pct > self.config.signal_threshold {
            // Golden Cross: Fast MA crosses above Slow MA
            (SignalType::Buy, diff_pct)
        } else if diff_pct < -self.config.signal_threshold {
            // Death Cross: Fast MA crosses below Slow MA
            (SignalType::Sell, diff_pct.abs())
        } else {
            (SignalType::Hold, 0.0)
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("fast_sma".to_string(), fast);
        metadata.insert("slow_sma".to_string(), slow);
        metadata.insert("diff_pct".to_string(), diff_pct);
        
        Ok(Signal {
            timestamp: datetime,
            signal_type,
            strength,
            price,
            metadata,
        })
    }
}
```

### Complete Implementation Example

```rust
use nyxs_owl::trade_math::{moving_averages::*, oscillators::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load market data
    let df = LazyFrame::scan_csv("data/AAPL_daily.csv", ScanArgsCSV::default())?
        .collect()?;
    
    // Create SMA crossover strategy
    let config = SMAStrategyConfig {
        fast_period: 20,
        slow_period: 50,
        signal_threshold: 0.005, // 0.5% threshold
    };
    
    let mut strategy = SMAStrategy::new(config)?;
    
    // Generate signals
    let signals = strategy.generate_signals(&df)?;
    
    println!("Generated {} signals", signals.len());
    for signal in signals.iter().take(10) {
        println!("{:?}", signal);
    }
    
    Ok(())
}
```

## Momentum Indicator Strategies

### RSI Strategy Implementation

```rust
use nyxs_owl::trade_math::oscillators::RelativeStrengthIndex;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RSIStrategyConfig {
    pub period: usize,
    pub overbought_threshold: f64,
    pub oversold_threshold: f64,
    pub enable_divergence: bool,
}

impl Default for RSIStrategyConfig {
    fn default() -> Self {
        Self {
            period: 14,
            overbought_threshold: 70.0,
            oversold_threshold: 30.0,
            enable_divergence: true,
        }
    }
}

pub struct RSIStrategy {
    config: RSIStrategyConfig,
    rsi: RelativeStrengthIndex,
    price_history: VecDeque<f64>,
    rsi_history: VecDeque<f64>,
}

impl TechnicalStrategy for RSIStrategy {
    type Config = RSIStrategyConfig;
    
    fn new(config: Self::Config) -> Result<Self, NyxsOwlError> {
        Ok(Self {
            rsi: RelativeStrengthIndex::new(config.period)?,
            price_history: VecDeque::with_capacity(100),
            rsi_history: VecDeque::with_capacity(100),
            config,
        })
    }
    
    fn generate_signals(&mut self, df: &DataFrame) -> Result<Vec<Signal>, NyxsOwlError> {
        let mut signals = Vec::new();
        
        let prices = df.column("close")?.f64()?.into_no_null_iter();
        let timestamps = df.column("timestamp")?.datetime()?.into_no_null_iter();
        
        for (price, timestamp) in prices.zip(timestamps) {
            // Update RSI
            self.rsi.update(price)?;
            
            // Store history
            self.price_history.push_back(price);
            if let Some(rsi_value) = self.rsi.value() {
                self.rsi_history.push_back(rsi_value);
                
                // Generate signal
                let signal = self.evaluate_rsi_signal(price, rsi_value, timestamp)?;
                if signal.signal_type != SignalType::Hold {
                    signals.push(signal);
                }
            }
            
            // Maintain history size
            if self.price_history.len() > 100 {
                self.price_history.pop_front();
                self.rsi_history.pop_front();
            }
        }
        
        Ok(signals)
    }
    
    fn update_indicators(&mut self, price: f64, _volume: Option<f64>) -> Result<(), NyxsOwlError> {
        self.rsi.update(price)?;
        Ok(())
    }
    
    fn get_strategy_name(&self) -> &'static str {
        "RSI_Strategy"
    }
}

impl RSIStrategy {
    fn evaluate_rsi_signal(
        &self, 
        price: f64, 
        rsi: f64, 
        timestamp_ns: i64
    ) -> Result<Signal, NyxsOwlError> {
        let datetime = DateTime::from_timestamp_nanos(timestamp_ns);
        
        let (signal_type, strength) = if rsi <= self.config.oversold_threshold {
            let strength = (self.config.oversold_threshold - rsi) / self.config.oversold_threshold;
            (SignalType::Buy, strength)
        } else if rsi >= self.config.overbought_threshold {
            let strength = (rsi - self.config.overbought_threshold) / (100.0 - self.config.overbought_threshold);
            (SignalType::Sell, strength)
        } else {
            (SignalType::Hold, 0.0)
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("rsi".to_string(), rsi);
        metadata.insert("overbought_threshold".to_string(), self.config.overbought_threshold);
        metadata.insert("oversold_threshold".to_string(), self.config.oversold_threshold);
        
        Ok(Signal {
            timestamp: datetime,
            signal_type,
            strength,
            price,
            metadata,
        })
    }
}
```

## Multi-Indicator Systems

### Confluence Strategy

```rust
use nyxs_owl::trade_math::{moving_averages::*, oscillators::*, volatility::*};

pub struct ConfluenceStrategy {
    // Multiple indicators
    sma_20: SimpleMovingAverage,
    sma_50: SimpleMovingAverage,
    rsi: RelativeStrengthIndex,
    bb: BollingerBands,
    
    // Configuration
    min_confluence_score: f64,
    indicator_weights: HashMap<String, f64>,
}

impl ConfluenceStrategy {
    pub fn new() -> Result<Self, NyxsOwlError> {
        let mut weights = HashMap::new();
        weights.insert("sma_trend".to_string(), 0.3);
        weights.insert("rsi_momentum".to_string(), 0.3);
        weights.insert("bb_position".to_string(), 0.4);
        
        Ok(Self {
            sma_20: SimpleMovingAverage::new(20)?,
            sma_50: SimpleMovingAverage::new(50)?,
            rsi: RelativeStrengthIndex::new(14)?,
            bb: BollingerBands::new(20, 2.0)?,
            min_confluence_score: 0.6,
            indicator_weights: weights,
        })
    }
    
    fn calculate_confluence_score(&self, price: f64) -> Result<f64, NyxsOwlError> {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        
        // SMA trend component
        if let (Some(sma_20), Some(sma_50)) = (self.sma_20.value(), self.sma_50.value()) {
            let trend_score = if sma_20 > sma_50 { 1.0 } else { -1.0 };
            let weight = self.indicator_weights["sma_trend"];
            total_score += trend_score * weight;
            total_weight += weight;
        }
        
        // RSI momentum component
        if let Some(rsi) = self.rsi.value() {
            let momentum_score = match rsi {
                r if r > 70.0 => -1.0,
                r if r < 30.0 => 1.0,
                r if r > 50.0 => 0.5,
                _ => -0.5,
            };
            let weight = self.indicator_weights["rsi_momentum"];
            total_score += momentum_score * weight;
            total_weight += weight;
        }
        
        // Bollinger Bands position component
        if let Some((upper, middle, lower)) = self.bb.value() {
            let bb_score = if price < lower {
                1.0 // Oversold, bullish
            } else if price > upper {
                -1.0 // Overbought, bearish
            } else {
                0.0 // Neutral
            };
            let weight = self.indicator_weights["bb_position"];
            total_score += bb_score * weight;
            total_weight += weight;
        }
        
        Ok(if total_weight > 0.0 { total_score / total_weight } else { 0.0 })
    }
}

impl TechnicalStrategy for ConfluenceStrategy {
    type Config = ();
    
    fn new(_config: Self::Config) -> Result<Self, NyxsOwlError> {
        ConfluenceStrategy::new()
    }
    
    fn generate_signals(&mut self, df: &DataFrame) -> Result<Vec<Signal>, NyxsOwlError> {
        let mut signals = Vec::new();
        
        let prices = df.column("close")?.f64()?.into_no_null_iter();
        let timestamps = df.column("timestamp")?.datetime()?.into_no_null_iter();
        
        for (price, timestamp) in prices.zip(timestamps) {
            // Update all indicators
            self.sma_20.update(price)?;
            self.sma_50.update(price)?;
            self.rsi.update(price)?;
            self.bb.update(price)?;
            
            // Calculate confluence score
            let confluence_score = self.calculate_confluence_score(price)?;
            
            if confluence_score.abs() >= self.min_confluence_score {
                let (signal_type, strength) = if confluence_score > 0.0 {
                    (SignalType::Buy, confluence_score)
                } else {
                    (SignalType::Sell, confluence_score.abs())
                };
                
                let datetime = DateTime::from_timestamp_nanos(timestamp);
                let mut metadata = HashMap::new();
                metadata.insert("confluence_score".to_string(), confluence_score);
                
                if let Some(rsi) = self.rsi.value() {
                    metadata.insert("rsi".to_string(), rsi);
                }
                
                signals.push(Signal {
                    timestamp: datetime,
                    signal_type,
                    strength,
                    price,
                    metadata,
                });
            }
        }
        
        Ok(signals)
    }
    
    fn update_indicators(&mut self, price: f64, _volume: Option<f64>) -> Result<(), NyxsOwlError> {
        self.sma_20.update(price)?;
        self.sma_50.update(price)?;
        self.rsi.update(price)?;
        self.bb.update(price)?;
        Ok(())
    }
    
    fn get_strategy_name(&self) -> &'static str {
        "Confluence_Strategy"
    }
}
```

## Best Practices

### 1. Strategy Factory Pattern

```rust
pub enum StrategyType {
    SMA,
    RSI,
    Confluence,
}

pub struct StrategyFactory;

impl StrategyFactory {
    pub fn create_strategy(
        strategy_type: StrategyType,
    ) -> Result<Box<dyn TechnicalStrategy<Config = ()>>, NyxsOwlError> {
        match strategy_type {
            StrategyType::SMA => {
                let config = SMAStrategyConfig::default();
                Ok(Box::new(SMAStrategy::new(config)?))
            },
            StrategyType::RSI => {
                let config = RSIStrategyConfig::default();
                Ok(Box::new(RSIStrategy::new(config)?))
            },
            StrategyType::Confluence => {
                Ok(Box::new(ConfluenceStrategy::new(())?))
            },
        }
    }
}
```

### 2. Performance Monitoring

```rust
use std::time::Instant;

pub struct StrategyMonitor {
    strategy_name: String,
    signal_count: usize,
    start_time: Instant,
    performance_metrics: Vec<f64>,
}

impl StrategyMonitor {
    pub fn new(strategy_name: String) -> Self {
        Self {
            strategy_name,
            signal_count: 0,
            start_time: Instant::now(),
            performance_metrics: Vec::new(),
        }
    }
    
    pub fn record_signals(&mut self, signals: &[Signal]) {
        self.signal_count += signals.len();
        
        // Record signal strength distribution
        for signal in signals {
            self.performance_metrics.push(signal.strength);
        }
    }
    
    pub fn get_performance_report(&self) -> PerformanceReport {
        let elapsed = self.start_time.elapsed();
        let signals_per_second = self.signal_count as f64 / elapsed.as_secs_f64();
        
        let avg_strength = if self.performance_metrics.is_empty() {
            0.0
        } else {
            self.performance_metrics.iter().sum::<f64>() / self.performance_metrics.len() as f64
        };
        
        PerformanceReport {
            strategy_name: self.strategy_name.clone(),
            total_signals: self.signal_count,
            elapsed_time: elapsed,
            signals_per_second,
            average_signal_strength: avg_strength,
        }
    }
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub strategy_name: String,
    pub total_signals: usize,
    pub elapsed_time: std::time::Duration,
    pub signals_per_second: f64,
    pub average_signal_strength: f64,
}
```

### 3. Real-time Usage Example

```rust
use tokio;
use nyxs_owl::trade_math::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize strategy
    let mut strategy = ConfluenceStrategy::new()?;
    let mut monitor = StrategyMonitor::new("Confluence".to_string());
    
    // Simulate real-time data processing
    let price_stream = simulate_price_stream().await;
    
    for price_update in price_stream {
        // Update indicators
        strategy.update_indicators(price_update.price, Some(price_update.volume))?;
        
        // Generate signal if conditions are met
        let mock_df = create_single_row_dataframe(price_update)?;
        let signals = strategy.generate_signals(&mock_df)?;
        
        if !signals.is_empty() {
            monitor.record_signals(&signals);
            println!("Generated {} signals at price {}", signals.len(), price_update.price);
            
            for signal in signals {
                process_trading_signal(signal).await?;
            }
        }
    }
    
    // Print performance report
    let report = monitor.get_performance_report();
    println!("{:#?}", report);
    
    Ok(())
}

async fn process_trading_signal(signal: Signal) -> Result<(), Box<dyn std::error::Error>> {
    // Implementation for processing trading signals
    // This could include:
    // - Position sizing
    // - Risk management
    // - Order execution
    // - Portfolio management
    
    match signal.signal_type {
        SignalType::Buy | SignalType::StrongBuy => {
            println!("Executing BUY order at {} with strength {:.2}", 
                     signal.price, signal.strength);
        },
        SignalType::Sell | SignalType::StrongSell => {
            println!("Executing SELL order at {} with strength {:.2}", 
                     signal.price, signal.strength);
        },
        SignalType::Hold => {
            // No action needed
        }
    }
    
    Ok(())
}
```

## Conclusion

NyxsOwl's technical indicator strategies provide a robust foundation for algorithmic trading systems. The modular architecture allows for easy composition of multiple indicators while maintaining performance and reliability.

**Key Benefits**:
- ✅ Production-ready implementations
- ✅ Flexible configuration system
- ✅ Real-time signal generation
- ✅ Built-in performance monitoring
- ✅ Easy integration with backtesting

For backtesting these strategies, see the backtesting integration guide and usage documentation. 