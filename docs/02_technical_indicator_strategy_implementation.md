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

### Unified Configuration API

NyxsOwl v0.7.4 provides a unified configuration system that works across both technical strategies and forecasting strategies through the `ConfigExtractor` trait:

```rust
use nyxs_owl::technical_strategies::ConfigExtractor;

// The ConfigExtractor trait provides safe access to configuration values
// regardless of whether forecasting features are enabled
pub trait ConfigExtractor {
    fn get_int_safe(&self, key: &str) -> Option<i64>;
    fn get_float_safe(&self, key: &str) -> Option<f64>;
    fn get_bool_safe(&self, key: &str) -> Option<bool>;
    fn get_string_safe(&self, key: &str) -> Option<&str>;
}

// Usage in strategies
impl TechnicalStrategy for SMAStrategy {
    fn generate_signals(&mut self, df: &DataFrame) -> Result<Vec<Signal>, NyxsOwlError> {
        // Safely extract configuration values using the unified API
        let fast_period = self.config.get_int_safe("fast_period").unwrap_or(10);
        let slow_period = self.config.get_int_safe("slow_period").unwrap_or(30);
        let threshold = self.config.get_float_safe("signal_threshold").unwrap_or(0.01);
        
        // Strategy implementation...
    }
}
```

This unified API handles the differences between:
- `common::StrategyConfig` (returns `Option<T>`)
- `forecasting::StrategyConfig` (returns `Result<T, _>`)

The `ConfigExtractor` trait automatically converts `Result<T, _>` to `Option<T>` using `.ok()` when the forecasting feature is enabled.

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

### Enhanced RSI Strategy

```rust
use nyxs_owl::trade_math::oscillators::RelativeStrengthIndex;

#[derive(Debug, Clone)]
pub struct EnhancedRSIConfig {
    pub period: usize,
    pub oversold_threshold: f64,
    pub overbought_threshold: f64,
    pub divergence_lookback: usize,
    pub volume_confirmation: bool,
}

impl Default for EnhancedRSIConfig {
    fn default() -> Self {
        Self {
            period: 14,
            oversold_threshold: 30.0,
            overbought_threshold: 70.0,
            divergence_lookback: 20,
            volume_confirmation: true,
        }
    }
}

pub struct EnhancedRSIStrategy {
    config: EnhancedRSIConfig,
    rsi: RelativeStrengthIndex,
    price_history: Vec<f64>,
    rsi_history: Vec<f64>,
    volume_history: Vec<f64>,
}

impl TechnicalStrategy for EnhancedRSIStrategy {
    type Config = EnhancedRSIConfig;
    
    fn new(config: Self::Config) -> Result<Self, NyxsOwlError> {
        Ok(Self {
            rsi: RelativeStrengthIndex::new(config.period)?,
            price_history: Vec::new(),
            rsi_history: Vec::new(),
            volume_history: Vec::new(),
            config,
        })
    }
    
    fn generate_signals(&mut self, df: &DataFrame) -> Result<Vec<Signal>, NyxsOwlError> {
        let mut signals = Vec::new();
        
        let prices = df.column("close")?.f64()?.into_no_null_iter();
        let volumes = df.column("volume")?.f64()?.into_no_null_iter();
        let timestamps = df.column("timestamp")?.datetime()?.into_no_null_iter();
        
        for ((price, volume), timestamp) in prices.zip(volumes).zip(timestamps) {
            // Update RSI
            self.rsi.update(price)?;
            
            // Store history for divergence analysis
            self.price_history.push(price);
            self.volume_history.push(volume);
            
            if let Some(rsi_value) = self.rsi.value() {
                self.rsi_history.push(rsi_value);
                
                // Generate signals
                if let Some(signal) = self.evaluate_rsi_signals(price, rsi_value, volume, timestamp)? {
                    signals.push(signal);
                }
            }
            
            // Maintain history size
            if self.price_history.len() > self.config.divergence_lookback {
                self.price_history.remove(0);
                self.rsi_history.remove(0);
                self.volume_history.remove(0);
            }
        }
        
        Ok(signals)
    }
    
    fn update_indicators(&mut self, price: f64, volume: Option<f64>) -> Result<(), NyxsOwlError> {
        self.rsi.update(price)?;
        
        self.price_history.push(price);
        if let Some(vol) = volume {
            self.volume_history.push(vol);
        }
        
        if let Some(rsi_value) = self.rsi.value() {
            self.rsi_history.push(rsi_value);
        }
        
        // Maintain history size
        if self.price_history.len() > self.config.divergence_lookback {
            self.price_history.remove(0);
            self.rsi_history.remove(0);
            self.volume_history.remove(0);
        }
        
        Ok(())
    }
    
    fn get_strategy_name(&self) -> &'static str {
        "Enhanced_RSI"
    }
}

impl EnhancedRSIStrategy {
    fn evaluate_rsi_signals(
        &self,
        price: f64,
        rsi: f64,
        volume: f64,
        timestamp_ns: i64,
    ) -> Result<Option<Signal>, NyxsOwlError> {
        let datetime = DateTime::from_timestamp_nanos(timestamp_ns);
        let mut metadata = HashMap::new();
        metadata.insert("rsi".to_string(), rsi);
        metadata.insert("volume".to_string(), volume);
        
        // Basic RSI signals
        let (signal_type, strength) = if rsi < self.config.oversold_threshold {
            (SignalType::Buy, (self.config.oversold_threshold - rsi) / self.config.oversold_threshold)
        } else if rsi > self.config.overbought_threshold {
            (SignalType::Sell, (rsi - self.config.overbought_threshold) / (100.0 - self.config.overbought_threshold))
        } else {
            return Ok(None); // No signal in neutral zone
        };
        
        // Volume confirmation
        if self.config.volume_confirmation {
            let avg_volume = self.volume_history.iter().sum::<f64>() / self.volume_history.len() as f64;
            if volume < avg_volume * 0.8 {
                // Weak volume, reduce signal strength
                metadata.insert("volume_confirmation".to_string(), 0.5);
            } else {
                metadata.insert("volume_confirmation".to_string(), 1.0);
            }
        }
        
        // Divergence detection
        if let Some(divergence_strength) = self.detect_divergence(price, rsi)? {
            metadata.insert("divergence_strength".to_string(), divergence_strength);
        }
        
        Ok(Some(Signal {
            timestamp: datetime,
            signal_type,
            strength,
            price,
            metadata,
        }))
    }
    
    fn detect_divergence(&self, current_price: f64, current_rsi: f64) -> Result<Option<f64>, NyxsOwlError> {
        if self.price_history.len() < 10 || self.rsi_history.len() < 10 {
            return Ok(None);
        }
        
        // Simple divergence detection
        let price_trend = current_price - self.price_history[0];
        let rsi_trend = current_rsi - self.rsi_history[0];
        
        // Bullish divergence: price making lower lows, RSI making higher lows
        if price_trend < 0.0 && rsi_trend > 0.0 {
            return Ok(Some(rsi_trend.abs()));
        }
        
        // Bearish divergence: price making higher highs, RSI making lower highs
        if price_trend > 0.0 && rsi_trend < 0.0 {
            return Ok(Some(-rsi_trend.abs()));
        }
        
        Ok(None)
    }
}
```

## Multi-Indicator Systems

### Multi-Factor Strategy

```rust
use nyxs_owl::technical_strategies::multi_factor::MultiFactorStrategy;

#[derive(Debug, Clone)]
pub struct MultiFactorConfig {
    pub factors: Vec<FactorConfig>,
    pub weights: Vec<f64>,
    pub signal_threshold: f64,
    pub min_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct FactorConfig {
    pub name: String,
    pub indicator_type: IndicatorType,
    pub parameters: HashMap<String, f64>,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub enum IndicatorType {
    RSI { period: usize },
    MACD { fast: usize, slow: usize, signal: usize },
    BollingerBands { period: usize, std_dev: f64 },
    SMA { period: usize },
    EMA { period: usize },
}

impl Default for MultiFactorConfig {
    fn default() -> Self {
        Self {
            factors: vec![
                FactorConfig {
                    name: "RSI".to_string(),
                    indicator_type: IndicatorType::RSI { period: 14 },
                    parameters: HashMap::new(),
                    weight: 0.3,
                },
                FactorConfig {
                    name: "MACD".to_string(),
                    indicator_type: IndicatorType::MACD { fast: 12, slow: 26, signal: 9 },
                    parameters: HashMap::new(),
                    weight: 0.4,
                },
                FactorConfig {
                    name: "Bollinger".to_string(),
                    indicator_type: IndicatorType::BollingerBands { period: 20, std_dev: 2.0 },
                    parameters: HashMap::new(),
                    weight: 0.3,
                },
            ],
            weights: vec![0.3, 0.4, 0.3],
            signal_threshold: 0.02,
            min_confidence: 0.7,
        }
    }
}

pub struct MultiFactorStrategy {
    config: MultiFactorConfig,
    indicators: HashMap<String, Box<dyn Indicator>>,
}

impl TechnicalStrategy for MultiFactorStrategy {
    type Config = MultiFactorConfig;
    
    fn new(config: Self::Config) -> Result<Self, NyxsOwlError> {
        let mut indicators = HashMap::new();
        
        for factor in &config.factors {
            let indicator = Self::create_indicator(&factor.indicator_type)?;
            indicators.insert(factor.name.clone(), indicator);
        }
        
        Ok(Self { config, indicators })
    }
    
    fn generate_signals(&mut self, df: &DataFrame) -> Result<Vec<Signal>, NyxsOwlError> {
        let mut signals = Vec::new();
        
        let prices = df.column("close")?.f64()?.into_no_null_iter();
        let timestamps = df.column("timestamp")?.datetime()?.into_no_null_iter();
        
        for (price, timestamp) in prices.zip(timestamps) {
            // Update all indicators
            for indicator in self.indicators.values_mut() {
                indicator.update(price)?;
            }
            
            // Calculate composite signal
            if let Some(signal) = self.calculate_composite_signal(price, timestamp)? {
                signals.push(signal);
            }
        }
        
        Ok(signals)
    }
    
    fn update_indicators(&mut self, price: f64, _volume: Option<f64>) -> Result<(), NyxsOwlError> {
        for indicator in self.indicators.values_mut() {
            indicator.update(price)?;
        }
        Ok(())
    }
    
    fn get_strategy_name(&self) -> &'static str {
        "Multi_Factor"
    }
}

impl MultiFactorStrategy {
    fn create_indicator(indicator_type: &IndicatorType) -> Result<Box<dyn Indicator>, NyxsOwlError> {
        match indicator_type {
            IndicatorType::RSI { period } => {
                Ok(Box::new(RelativeStrengthIndex::new(*period)?))
            },
            IndicatorType::MACD { fast, slow, signal } => {
                Ok(Box::new(MACD::new(*fast, *slow, *signal)?))
            },
            IndicatorType::BollingerBands { period, std_dev } => {
                Ok(Box::new(BollingerBands::new(*period, *std_dev)?))
            },
            IndicatorType::SMA { period } => {
                Ok(Box::new(SimpleMovingAverage::new(*period)?))
            },
            IndicatorType::EMA { period } => {
                Ok(Box::new(ExponentialMovingAverage::new(*period)?))
            },
        }
    }
    
    fn calculate_composite_signal(
        &self,
        price: f64,
        timestamp_ns: i64,
    ) -> Result<Option<Signal>, NyxsOwlError> {
        let datetime = DateTime::from_timestamp_nanos(timestamp_ns);
        let mut composite_score = 0.0;
        let mut total_weight = 0.0;
        let mut metadata = HashMap::new();
        
        // Calculate weighted score from all factors
        for (i, factor) in self.config.factors.iter().enumerate() {
            if let Some(indicator) = self.indicators.get(&factor.name) {
                if let Some(value) = indicator.value() {
                    let normalized_value = self.normalize_indicator_value(&factor.name, value)?;
                    let weighted_value = normalized_value * factor.weight;
                    composite_score += weighted_value;
                    total_weight += factor.weight;
                    
                    metadata.insert(format!("{}_value", factor.name), value);
                    metadata.insert(format!("{}_normalized", factor.name), normalized_value);
                }
            }
        }
        
        if total_weight == 0.0 {
            return Ok(None);
        }
        
        // Normalize composite score
        composite_score /= total_weight;
        metadata.insert("composite_score".to_string(), composite_score);
        
        // Generate signal based on composite score
        let (signal_type, strength) = if composite_score > self.config.signal_threshold {
            (SignalType::Buy, composite_score)
        } else if composite_score < -self.config.signal_threshold {
            (SignalType::Sell, composite_score.abs())
        } else {
            return Ok(None);
        };
        
        Ok(Some(Signal {
            timestamp: datetime,
            signal_type,
            strength,
            price,
            metadata,
        }))
    }
    
    fn normalize_indicator_value(&self, indicator_name: &str, value: f64) -> Result<f64, NyxsOwlError> {
        // Normalize different indicators to a common scale (-1 to 1)
        match indicator_name {
            "RSI" => Ok((value - 50.0) / 50.0), // RSI: 0-100 -> -1 to 1
            "MACD" => Ok(value.signum() * value.abs().min(1.0)), // MACD: clip to -1 to 1
            "Bollinger" => Ok(value), // Already normalized
            _ => Ok(value),
        }
    }
}

// Trait for indicators
trait Indicator {
    fn update(&mut self, price: f64) -> Result<(), NyxsOwlError>;
    fn value(&self) -> Option<f64>;
}
```

## Signal Generation Framework

### Signal Aggregation

```rust
pub struct SignalAggregator {
    aggregation_window: usize,
    signal_history: Vec<Signal>,
}

impl SignalAggregator {
    pub fn new(window: usize) -> Self {
        Self {
            aggregation_window: window,
            signal_history: Vec::new(),
        }
    }
    
    pub fn add_signal(&mut self, signal: Signal) {
        self.signal_history.push(signal);
        
        // Maintain window size
        if self.signal_history.len() > self.aggregation_window {
            self.signal_history.remove(0);
        }
    }
    
    pub fn get_aggregated_signal(&self) -> Option<Signal> {
        if self.signal_history.is_empty() {
            return None;
        }
        
        // Calculate weighted average of recent signals
        let total_weight: f64 = self.signal_history.iter()
            .enumerate()
            .map(|(i, _)| (i + 1) as f64)
            .sum();
        
        let weighted_buy_strength: f64 = self.signal_history.iter()
            .enumerate()
            .filter(|(_, s)| matches!(s.signal_type, SignalType::Buy | SignalType::StrongBuy))
            .map(|(i, s)| s.strength * (i + 1) as f64)
            .sum();
        
        let weighted_sell_strength: f64 = self.signal_history.iter()
            .enumerate()
            .filter(|(_, s)| matches!(s.signal_type, SignalType::Sell | SignalType::StrongSell))
            .map(|(i, s)| s.strength * (i + 1) as f64)
            .sum();
        
        let net_strength = (weighted_buy_strength - weighted_sell_strength) / total_weight;
        
        let signal_type = if net_strength > 0.1 {
            SignalType::Buy
        } else if net_strength < -0.1 {
            SignalType::Sell
        } else {
            SignalType::Hold
        };
        
        let latest_signal = &self.signal_history[self.signal_history.len() - 1];
        
        Some(Signal {
            timestamp: latest_signal.timestamp,
            signal_type,
            strength: net_strength.abs(),
            price: latest_signal.price,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("aggregated_signals".to_string(), self.signal_history.len() as f64);
                meta.insert("net_strength".to_string(), net_strength);
                meta
            },
        })
    }
}
```

## Strategy Optimization

### Performance Monitoring

```rust
use std::time::Instant;

pub struct StrategyPerformanceMonitor {
    strategy_name: String,
    start_time: Instant,
    signal_count: usize,
    execution_times: Vec<f64>,
}

impl StrategyPerformanceMonitor {
    pub fn new(strategy_name: String) -> Self {
        Self {
            strategy_name,
            start_time: Instant::now(),
            signal_count: 0,
            execution_times: Vec::new(),
        }
    }
    
    pub fn record_execution(&mut self, execution_time: f64, signals_generated: usize) {
        self.execution_times.push(execution_time);
        self.signal_count += signals_generated;
    }
    
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        let total_time = self.start_time.elapsed().as_secs_f64();
        let avg_execution_time = self.execution_times.iter().sum::<f64>() / self.execution_times.len() as f64;
        let signals_per_second = self.signal_count as f64 / total_time;
        
        PerformanceMetrics {
            strategy_name: self.strategy_name.clone(),
            total_executions: self.execution_times.len(),
            total_signals: self.signal_count,
            avg_execution_time,
            signals_per_second,
            total_runtime: total_time,
        }
    }
}

#[derive(Debug)]
pub struct PerformanceMetrics {
    pub strategy_name: String,
    pub total_executions: usize,
    pub total_signals: usize,
    pub avg_execution_time: f64,
    pub signals_per_second: f64,
    pub total_runtime: f64,
}
```

## Best Practices

### 1. Configuration Management

```rust
use std::env;

fn load_strategy_config() -> MultiFactorConfig {
    let signal_threshold = env::var("SIGNAL_THRESHOLD")
        .unwrap_or_else(|_| "0.02".to_string())
        .parse::<f64>()
        .unwrap_or(0.02);
    
    let min_confidence = env::var("MIN_CONFIDENCE")
        .unwrap_or_else(|_| "0.7".to_string())
        .parse::<f64>()
        .unwrap_or(0.7);
    
    MultiFactorConfig {
        signal_threshold,
        min_confidence,
        ..MultiFactorConfig::default()
    }
}
```

### 2. Error Handling and Validation

```rust
fn robust_signal_generation(
    strategy: &mut dyn TechnicalStrategy,
    df: &DataFrame,
) -> Result<Vec<Signal>, NyxsOwlError> {
    // Validate data requirements
    if df.height() < 50 {
        return Err(NyxsOwlError::DataError("Insufficient data for strategy".to_string()));
    }
    
    // Check required columns
    let required_columns = vec!["close", "timestamp"];
    for col in required_columns {
        if !df.get_column_names().contains(&col) {
            return Err(NyxsOwlError::DataError(
                format!("Required column '{}' not found", col)
            ));
        }
    }
    
    // Generate signals with error handling
    match strategy.generate_signals(df) {
        Ok(signals) => {
            log::info!("Generated {} signals", signals.len());
            Ok(signals)
        },
        Err(NyxsOwlError::DataError(msg)) => {
            log::warn!("Data error in signal generation: {}", msg);
            Ok(Vec::new())
        },
        Err(e) => Err(e),
    }
}
```

### 3. Memory Optimization

```rust
// Use memory-efficient data structures for large datasets
pub struct MemoryOptimizedStrategy {
    config: StrategyConfig,
    // Use circular buffers for history
    price_buffer: CircularBuffer<f64>,
    indicator_buffer: CircularBuffer<f64>,
}

impl MemoryOptimizedStrategy {
    pub fn new(config: StrategyConfig, buffer_size: usize) -> Self {
        Self {
            config,
            price_buffer: CircularBuffer::new(buffer_size),
            indicator_buffer: CircularBuffer::new(buffer_size),
        }
    }
    
    pub fn update_with_memory_optimization(&mut self, price: f64) -> Result<(), NyxsOwlError> {
        // Add to circular buffer (automatically removes oldest if full)
        self.price_buffer.push(price);
        
        // Calculate indicator value
        let indicator_value = self.calculate_indicator()?;
        self.indicator_buffer.push(indicator_value);
        
        Ok(())
    }
}

// Simple circular buffer implementation
struct CircularBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    head: usize,
    size: usize,
}

impl<T: Clone + Default> CircularBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![T::default(); capacity],
            capacity,
            head: 0,
            size: 0,
        }
    }
    
    fn push(&mut self, item: T) {
        self.buffer[self.head] = item;
        self.head = (self.head + 1) % self.capacity;
        if self.size < self.capacity {
            self.size += 1;
        }
    }
    
    fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.size).map(move |i| {
            let index = (self.head + i) % self.capacity;
            &self.buffer[index]
        })
    }
}
```

### 4. Real-time Processing

```rust
use tokio::sync::mpsc;

pub struct RealTimeStrategyProcessor {
    strategy: Box<dyn TechnicalStrategy>,
    signal_sender: mpsc::Sender<Signal>,
    performance_monitor: StrategyPerformanceMonitor,
}

impl RealTimeStrategyProcessor {
    pub fn new(
        strategy: Box<dyn TechnicalStrategy>,
        signal_sender: mpsc::Sender<Signal>,
    ) -> Self {
        Self {
            performance_monitor: StrategyPerformanceMonitor::new(
                strategy.get_strategy_name().to_string()
            ),
            strategy,
            signal_sender,
        }
    }
    
    pub async fn process_tick(&mut self, price: f64, volume: Option<f64>) -> Result<(), NyxsOwlError> {
        let start_time = Instant::now();
        
        // Update strategy indicators
        self.strategy.update_indicators(price, volume)?;
        
        // Generate signals (if any)
        // Note: In real-time, we'd typically use a sliding window approach
        // rather than full DataFrame processing
        
        let execution_time = start_time.elapsed().as_secs_f64();
        self.performance_monitor.record_execution(execution_time, 0);
        
        Ok(())
    }
    
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_monitor.get_performance_metrics()
    }
}
```

---

*Last updated: December 2024 | Version: 0.7.4 | Status: Production Ready* 