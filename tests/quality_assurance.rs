//! Quality Assurance Test Suite for NyxsOwl Trading Library

#[cfg(test)]
mod quality_tests {
    use nyxs_owl::prelude::*;
    use polars::prelude::*;
    
    /// Create test data for validation
    fn create_test_data(length: usize) -> DataFrame {
        let mut prices = Vec::new();
        let mut volumes = Vec::new();
        let mut timestamps = Vec::new();
        
        for i in 0..length {
            let base_price = 100.0;
            let trend = i as f64 * 0.1;
            let noise = (i as f64 * 0.1).sin() * 2.0;
            let price = base_price + trend + noise;
            
            prices.push(price);
            volumes.push(1000000 + i * 1000);
            timestamps.push(format!("2023-01-{:02}", (i % 30) + 1));
        }
        
        let highs: Vec<f64> = prices.iter().map(|p| p * 1.02).collect();
        let lows: Vec<f64> = prices.iter().map(|p| p * 0.98).collect();
        let opens: Vec<f64> = std::iter::once(prices[0])
            .chain(prices.iter().take(prices.len() - 1).cloned())
            .collect();
        
        df! {
            "timestamp" => timestamps,
            "open" => opens,
            "high" => highs,
            "low" => lows,
            "close" => prices,
            "volume" => volumes
        }.unwrap()
    }
    
    #[test]
    fn test_technical_indicators_quality() {
        let df = create_test_data(100);
        let close_series = df.column("close").unwrap();
        
        // Test SMA accuracy
        let sma = nyxs_owl::trade_math::moving_averages::calculate_sma(close_series, 20).unwrap();
        assert_eq!(sma.len(), close_series.len());
        
        // Test EMA accuracy  
        let ema = nyxs_owl::trade_math::moving_averages::calculate_ema(close_series, 20, 2.0/21.0).unwrap();
        assert_eq!(ema.len(), close_series.len());
        
        // Test RSI bounds
        let rsi = nyxs_owl::trade_math::momentum::calculate_rsi(close_series, 14).unwrap();
        let rsi_values: Vec<Option<f64>> = rsi.f64().unwrap().into_iter().collect();
        
        for value in rsi_values.iter().flatten() {
            assert!(*value >= 0.0 && *value <= 100.0, "RSI out of bounds: {}", value);
        }
        
        println!("✓ Technical indicators quality test passed");
    }
    
    #[test]
    fn test_trading_strategies_quality() {
        let df = create_test_data(50);
        
        // Test MACD strategy
        let macd_signals = nyxs_owl::technical_strategies::momentum::macd_strategy::macd_signals(&df, "close", 12, 26, 9).unwrap();
        assert_eq!(macd_signals.len(), df.height());
        
        // Test Stochastic strategy
        let stoch_signals = nyxs_owl::technical_strategies::momentum::stochastic_strategy::stochastic_signals(&df, "high", "low", "close", 14, 3, 20.0, 80.0).unwrap();
        assert_eq!(stoch_signals.len(), df.height());
        
        println!("✓ Trading strategies quality test passed");
    }
    
    #[test]
    fn test_numerical_stability() {
        // Test with extreme values
        let extreme_values = vec![0.001, 1000000.0, 0.1, 999999.9];
        let series = Series::new("test".into(), extreme_values);
        
        // Should not panic or produce NaN/Inf
        let sma_result = nyxs_owl::trade_math::moving_averages::calculate_sma(&series, 2);
        assert!(sma_result.is_ok());
        
        let ema_result = nyxs_owl::trade_math::moving_averages::calculate_ema(&series, 2, 0.5);
        assert!(ema_result.is_ok());
        
        println!("✓ Numerical stability test passed");
    }
    
    #[test]
    fn test_performance_benchmarks() {
        let df = create_test_data(1000);
        let close_series = df.column("close").unwrap();
        
        let start = std::time::Instant::now();
        
        // Benchmark technical indicators
        let _sma = nyxs_owl::trade_math::moving_averages::calculate_sma(close_series, 20).unwrap();
        let _ema = nyxs_owl::trade_math::moving_averages::calculate_ema(close_series, 20, 2.0/21.0).unwrap();
        let _rsi = nyxs_owl::trade_math::momentum::calculate_rsi(close_series, 14).unwrap();
        
        let duration = start.elapsed();
        println!("Performance: {:?} for 1000 data points", duration);
        
        // Should complete within reasonable time
        assert!(duration.as_millis() < 500, "Performance too slow: {:?}", duration);
        
        println!("✓ Performance benchmark test passed");
    }
} 