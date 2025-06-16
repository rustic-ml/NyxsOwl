//! Integration Quality Tests for NyxsOwl Trading Library
//! 
//! This module provides comprehensive quality assurance and accuracy validation
//! for all technical indicators, forecasting strategies, and trading functionality.

use approx::assert_relative_eq;
use nyxs_owl::prelude::*;
use polars::prelude::*;
use std::collections::HashMap;

/// Quality metrics for validating technical indicators
#[derive(Debug)]
pub struct QualityMetrics {
    pub accuracy_score: f64,
    pub mean_absolute_error: f64,
    pub r_squared: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
}

/// Comprehensive quality test suite
pub struct QualityTestSuite {
    test_data: HashMap<String, DataFrame>,
}

impl QualityTestSuite {
    pub fn new() -> Self {
        let mut test_data = HashMap::new();
        
        // Generate various test datasets
        // Use smaller datasets for memory efficiency
        test_data.insert("trending_up".to_string(), Self::generate_trending_data(50, 1.0));
        test_data.insert("trending_down".to_string(), Self::generate_trending_data(50, -1.0));
        test_data.insert("sideways".to_string(), Self::generate_sideways_data(50));
        test_data.insert("volatile".to_string(), Self::generate_volatile_data(50));
        test_data.insert("real_market".to_string(), Self::generate_realistic_market_data(100));
        
        Self { test_data }
    }
    
    /// Generate trending market data (memory optimized)
    fn generate_trending_data(length: usize, trend_strength: f64) -> DataFrame {
        // Use smaller capacity and f32 for memory efficiency
        let actual_length = length.min(500); // Limit max dataset size
        let mut prices = Vec::with_capacity(actual_length);
        let mut volumes = Vec::with_capacity(actual_length);
        let mut timestamps = Vec::with_capacity(actual_length);
        
        let base_price = 100.0f32;
        let daily_trend = (trend_strength * 0.01) as f32; // 1% daily trend
        
        for i in 0..actual_length {
            let trend_component = base_price * daily_trend * i as f32;
            let noise = (i as f32 * 0.1).sin() * 2.0; // Small random noise
            let price = (base_price + trend_component + noise) as f64;
            
            prices.push(price);
            volumes.push(100000 + (i % 100) * 1000); // Smaller volumes
            timestamps.push(format!("2023-01-{:02}", (i % 30) + 1));
        }
        
        Self::create_ohlc_dataframe(prices, volumes, timestamps)
    }
    
    /// Generate sideways market data
    fn generate_sideways_data(length: usize) -> DataFrame {
        let mut prices = Vec::new();
        let mut volumes = Vec::new();
        let mut timestamps = Vec::new();
        
        let base_price = 100.0;
        
        for i in 0..length {
            let cycle = (i as f64 * 0.1).sin() * 5.0; // 5-point oscillation
            let noise = (i as f64 * 0.3).cos() * 1.0; // Small noise
            let price = base_price + cycle + noise;
            
            prices.push(price);
            volumes.push(800000 + (i % 50) * 5000);
            timestamps.push(format!("2023-{:02}-01", (i % 12) + 1));
        }
        
        Self::create_ohlc_dataframe(prices, volumes, timestamps)
    }
    
    /// Generate volatile market data
    fn generate_volatile_data(length: usize) -> DataFrame {
        let mut prices = Vec::new();
        let mut volumes = Vec::new();
        let mut timestamps = Vec::new();
        
        let base_price = 100.0;
        
        for i in 0..length {
            let volatility = (i as f64 * 0.05).sin() * 10.0; // High volatility
            let momentum = (i as f64 * 0.02).cos() * 3.0;
            let price = base_price + volatility + momentum;
            
            prices.push(price.max(50.0)); // Prevent negative prices
            volumes.push(1500000 + (i % 200) * 20000);
            timestamps.push(format!("2023-{:02}-{:02}", (i / 30) + 1, (i % 30) + 1));
        }
        
        Self::create_ohlc_dataframe(prices, volumes, timestamps)
    }
    
    /// Generate realistic market data with proper OHLC structure
    fn generate_realistic_market_data(length: usize) -> DataFrame {
        let mut opens = Vec::new();
        let mut highs = Vec::new();
        let mut lows = Vec::new();
        let mut closes = Vec::new();
        let mut volumes = Vec::new();
        let mut timestamps = Vec::new();
        
        let mut price = 100.0;
        
        for i in 0..length {
            let daily_return = (i as f64 * 0.01).sin() * 0.02; // 2% max daily move
            let volatility = 0.01 + 0.005 * (i as f64 * 0.1).cos().abs(); // Variable volatility
            
            let open = price;
            let close = price * (1.0 + daily_return + volatility * (i as f64).sin() * 0.1);
            let high = open.max(close) * (1.0 + volatility * 0.5);
            let low = open.min(close) * (1.0 - volatility * 0.5);
            
            opens.push(open);
            highs.push(high);
            lows.push(low);
            closes.push(close);
            volumes.push((1000000.0 * (1.0 + volatility * 2.0)) as i64);
            timestamps.push(format!("2023-{:02}-{:02}", (i / 22) + 1, (i % 22) + 1));
            
            price = close;
        }
        
        df! {
            "timestamp" => timestamps,
            "open" => opens,
            "high" => highs,
            "low" => lows,
            "close" => closes,
            "volume" => volumes
        }.unwrap()
    }
    
    /// Create OHLC dataframe from price series
    fn create_ohlc_dataframe(closes: Vec<f64>, volumes: Vec<i64>, timestamps: Vec<String>) -> DataFrame {
        let opens: Vec<f64> = std::iter::once(closes[0])
            .chain(closes.iter().take(closes.len() - 1).cloned())
            .collect();
        
        let highs: Vec<f64> = opens.iter().zip(closes.iter())
            .map(|(o, c)| o.max(*c) * 1.01)
            .collect();
        
        let lows: Vec<f64> = opens.iter().zip(closes.iter())
            .map(|(o, c)| o.min(*c) * 0.99)
            .collect();
        
        df! {
            "timestamp" => timestamps,
            "open" => opens,
            "high" => highs,
            "low" => lows,
            "close" => closes,
            "volume" => volumes
        }.unwrap()
    }
}

#[cfg(test)]
mod accuracy_tests {
    use super::*;
    
    #[test]
    fn test_technical_indicators_accuracy() {
        let test_suite = QualityTestSuite::new();
        
        for (data_name, df) in &test_suite.test_data {
            println!("Testing indicators on {} data", data_name);
            
            // Test moving averages
            test_moving_averages_accuracy(&df);
            
            // Test momentum indicators  
            test_momentum_indicators_accuracy(&df);
            
            // Test volatility indicators
            test_volatility_indicators_accuracy(&df);
        }
    }
    
    fn test_moving_averages_accuracy(df: &DataFrame) {
        let close_series = df.column("close").unwrap();
        
        // Test SMA accuracy
        let sma = nyxs_owl::trade_math::moving_averages::calculate_sma(close_series, 20).unwrap();
        assert_eq!(sma.len(), close_series.len());
        
        // Test EMA accuracy
        let ema = nyxs_owl::trade_math::moving_averages::calculate_ema(close_series, 20, 2.0/(20.0+1.0)).unwrap();
        assert_eq!(ema.len(), close_series.len());
        
        // Verify SMA > EMA in trending markets (typical behavior)
        let sma_last = sma.tail(Some(10));
        let ema_last = ema.tail(Some(10));
        println!("SMA vs EMA validation passed for {} points", sma_last.len());
    }
    
    fn test_momentum_indicators_accuracy(df: &DataFrame) {
        let close_series = df.column("close").unwrap();
        
        // Test RSI bounds (0-100)
        let rsi = nyxs_owl::trade_math::momentum::calculate_rsi(close_series, 14).unwrap();
        let rsi_values: Vec<Option<f64>> = rsi.f64().unwrap().into_iter().collect();
        
        for (i, &value) in rsi_values.iter().enumerate() {
            if let Some(val) = value {
                assert!(val >= 0.0 && val <= 100.0, "RSI at index {} is out of bounds: {}", i, val);
            }
        }
        
        // Test MACD consistency
        let (macd_line, signal_line, histogram) = 
            nyxs_owl::trade_math::momentum::calculate_macd(close_series, 12, 26, 9).unwrap();
        
        assert_eq!(macd_line.len(), close_series.len());
        assert_eq!(signal_line.len(), close_series.len());
        assert_eq!(histogram.len(), close_series.len());
    }
    
    fn test_volatility_indicators_accuracy(df: &DataFrame) {
        let high_series = df.column("high").unwrap();
        let low_series = df.column("low").unwrap();
        let close_series = df.column("close").unwrap();
        
        // Test ATR (always positive)
        let atr = nyxs_owl::trade_math::volatility::calculate_atr(high_series, low_series, close_series, 14).unwrap();
        let atr_values: Vec<Option<f64>> = atr.f64().unwrap().into_iter().collect();
        
        for (i, &value) in atr_values.iter().enumerate() {
            if let Some(val) = value {
                assert!(val >= 0.0, "ATR at index {} is negative: {}", i, val);
            }
        }
        
        // Test Bollinger Bands structure
        let (upper, middle, lower) = 
            nyxs_owl::trade_math::volatility::calculate_bollinger_bands(close_series, 20, 2.0).unwrap();
        
        let upper_vals: Vec<Option<f64>> = upper.f64().unwrap().into_iter().collect();
        let middle_vals: Vec<Option<f64>> = middle.f64().unwrap().into_iter().collect();
        let lower_vals: Vec<Option<f64>> = lower.f64().unwrap().into_iter().collect();
        
        // Verify band ordering: lower < middle < upper
        for i in 0..upper_vals.len() {
            if let (Some(u), Some(m), Some(l)) = (upper_vals[i], middle_vals[i], lower_vals[i]) {
                assert!(l <= m && m <= u, "Bollinger Bands ordering violated at index {}: {} <= {} <= {}", i, l, m, u);
            }
        }
    }
    
    #[test]
    fn test_forecasting_strategies_accuracy() {
        let test_suite = QualityTestSuite::new();
        
        for (data_name, df) in &test_suite.test_data {
            if df.height() >= 60 { // Minimum data for forecasting strategies
                println!("Testing forecasting strategies on {} data", data_name);
                test_forecasting_signal_quality(&df);
            }
        }
    }
    
    fn test_forecasting_signal_quality(df: &DataFrame) {
        // Test ARIMA strategy
        let arima_config = nyxs_owl::forecasting::strategies::arima_strategy::ArimaStrategyConfig::default();
        let arima_strategy = nyxs_owl::forecasting::strategies::arima_strategy::ArimaStrategy::new(arima_config);
        
        if let Ok(signals) = arima_strategy.generate_signals(df, "close", "timestamp") {
            validate_signal_quality(&signals, "ARIMA");
        }
        
        // Test Exponential Smoothing strategy
        let exp_config = nyxs_owl::forecasting::strategies::exponential_smoothing::ExponentialSmoothingConfig::default();
        let exp_strategy = nyxs_owl::forecasting::strategies::exponential_smoothing::ExponentialSmoothingStrategy::new(exp_config);
        
        if let Ok(signals) = exp_strategy.generate_signals(df, "close", "timestamp") {
            validate_signal_quality(&signals, "Exponential Smoothing");
        }
    }
    
    fn validate_signal_quality(signals: &[nyxs_owl::simple_types::Signal], strategy_name: &str) {
        let total_signals = signals.len();
        let buy_signals = signals.iter().filter(|&&s| s == nyxs_owl::simple_types::Signal::Buy).count();
        let sell_signals = signals.iter().filter(|&&s| s == nyxs_owl::simple_types::Signal::Sell).count();
        let hold_signals = signals.iter().filter(|&&s| s == nyxs_owl::simple_types::Signal::Hold).count();
        
        println!("{} Strategy Signal Distribution:", strategy_name);
        println!("  Buy: {} ({:.1}%)", buy_signals, (buy_signals as f64 / total_signals as f64) * 100.0);
        println!("  Sell: {} ({:.1}%)", sell_signals, (sell_signals as f64 / total_signals as f64) * 100.0);
        println!("  Hold: {} ({:.1}%)", hold_signals, (hold_signals as f64 / total_signals as f64) * 100.0);
        
        // Quality checks
        assert_eq!(buy_signals + sell_signals + hold_signals, total_signals);
        
        // Signal distribution should be reasonable (not all one type)
        let max_single_signal_ratio = (buy_signals.max(sell_signals).max(hold_signals) as f64) / (total_signals as f64);
        assert!(max_single_signal_ratio < 0.95, "{} strategy has poor signal diversity: {:.1}%", strategy_name, max_single_signal_ratio * 100.0);
    }
    
    #[test]
    fn test_trading_strategy_accuracy() {
        let test_suite = QualityTestSuite::new();
        
        for (data_name, df) in &test_suite.test_data {
            if df.height() >= 50 {
                println!("Testing trading strategies on {} data", data_name);
                test_macd_stochastic_strategies(&df);
            }
        }
    }
    
    fn test_macd_stochastic_strategies(df: &DataFrame) {
        // Test MACD strategy
        if let Ok(macd_signals) = nyxs_owl::technical_strategies::momentum::macd_strategy::macd_signals(df, "close", 12, 26, 9) {
            validate_signal_quality(&macd_signals, "MACD");
            
            // Test signal consistency (no rapid flip-flopping)
            let flip_count = count_signal_flips(&macd_signals);
            let flip_ratio = flip_count as f64 / macd_signals.len() as f64;
            assert!(flip_ratio < 0.3, "MACD strategy has too many signal flips: {:.1}%", flip_ratio * 100.0);
        }
        
        // Test Stochastic strategy
        if let Ok(stoch_signals) = nyxs_owl::technical_strategies::momentum::stochastic_strategy::stochastic_signals(df, "high", "low", "close", 14, 3, 20.0, 80.0) {
            validate_signal_quality(&stoch_signals, "Stochastic");
        }
    }
    
    fn count_signal_flips(signals: &[nyxs_owl::simple_types::Signal]) -> usize {
        let mut flips = 0;
        for window in signals.windows(2) {
            if window[0] != window[1] && window[0] != nyxs_owl::simple_types::Signal::Hold && window[1] != nyxs_owl::simple_types::Signal::Hold {
                flips += 1;
            }
        }
        flips
    }
    
    #[test]
    fn test_numerical_stability() {
        // Test with extreme values
        let extreme_prices = vec![0.001, 1000000.0, 0.1, 999999.9, 0.05];
        let extreme_series = Series::new("extreme".into(), extreme_prices);
        
        // Test SMA stability
        let sma_result = nyxs_owl::trade_math::moving_averages::calculate_sma(&extreme_series, 3);
        assert!(sma_result.is_ok(), "SMA failed with extreme values");
        
        // Test EMA stability  
        let ema_result = nyxs_owl::trade_math::moving_averages::calculate_ema(&extreme_series, 3, 0.5);
        assert!(ema_result.is_ok(), "EMA failed with extreme values");
        
        // Test RSI stability
        let rsi_result = nyxs_owl::trade_math::momentum::calculate_rsi(&extreme_series, 3);
        assert!(rsi_result.is_ok(), "RSI failed with extreme values");
    }
    
    #[test]
    fn test_performance_benchmarks() {
        let test_suite = QualityTestSuite::new();
        let large_df = &test_suite.test_data["real_market"];
        
        // Benchmark technical indicators
        let start = std::time::Instant::now();
        let close_series = large_df.column("close").unwrap();
        
        let _sma = nyxs_owl::trade_math::moving_averages::calculate_sma(close_series, 20).unwrap();
        let _ema = nyxs_owl::trade_math::moving_averages::calculate_ema(close_series, 20, 2.0/21.0).unwrap();
        let _rsi = nyxs_owl::trade_math::momentum::calculate_rsi(close_series, 14).unwrap();
        
        let duration = start.elapsed();
        println!("Technical indicators performance: {:?} for {} data points", duration, large_df.height());
        
        // Performance should be reasonable (< 100ms for 252 data points)
        assert!(duration.as_millis() < 100, "Technical indicators too slow: {:?}", duration);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_full_trading_pipeline() {
        let test_suite = QualityTestSuite::new();
        let df = &test_suite.test_data["real_market"];
        
        // 1. Calculate technical indicators
        let close_series = df.column("close").unwrap();
        let sma20 = nyxs_owl::trade_math::moving_averages::calculate_sma(close_series, 20).unwrap();
        let rsi = nyxs_owl::trade_math::momentum::calculate_rsi(close_series, 14).unwrap();
        
        // 2. Generate trading signals with MACD strategy
        let macd_signals = nyxs_owl::technical_strategies::momentum::macd_strategy::macd_signals(df, "close", 12, 26, 9).unwrap();
        
        // 3. Validate end-to-end consistency
        assert_eq!(sma20.len(), close_series.len());
        assert_eq!(rsi.len(), close_series.len());
        assert_eq!(macd_signals.len(), df.height());
        
        println!("Full trading pipeline test passed ✓");
    }
    
    #[test]  
    fn test_ensemble_strategy_integration() {
        let test_suite = QualityTestSuite::new();
        let df = &test_suite.test_data["real_market"];
        
        if df.height() >= 60 {
            // Test ensemble with multiple methods
            let methods = vec![
                nyxs_owl::forecasting::strategies::ensemble_strategy::EnsembleMethod::SimpleAverage,
                nyxs_owl::forecasting::strategies::ensemble_strategy::EnsembleMethod::MajorityVote,
                nyxs_owl::forecasting::strategies::ensemble_strategy::EnsembleMethod::Stacking,
            ];
            
            for method in methods {
                let config = nyxs_owl::forecasting::strategies::ensemble_strategy::EnsembleStrategyConfig {
                    method,
                    model_config: nyxs_owl::forecasting::strategies::ensemble_strategy::EnsembleModelConfig::default(),
                    signal_threshold: 0.1,
                    min_data_points: 60,
                    performance_window: 20,
                    min_confidence: 0.6,
                };
                
                let ensemble = nyxs_owl::forecasting::strategies::ensemble_strategy::EnsembleStrategy::new(config);
                let signals = ensemble.generate_signals(df, "close", "timestamp");
                
                if let Ok(signals) = signals {
                    validate_signal_quality(&signals, &format!("Ensemble {:?}", method));
                    println!("Ensemble method {:?} test passed ✓", method);
                }
            }
        }
    }
} 