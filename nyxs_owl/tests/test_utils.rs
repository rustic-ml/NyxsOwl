use chrono::{DateTime, Duration, Utc};
use fake::{Fake, Faker};
use rand::Rng;
use std::collections::HashMap;

/// Test data generator for time series data
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate a realistic stock price time series
    pub fn generate_stock_prices(
        count: usize,
        start_price: f64,
        volatility: f64,
    ) -> Vec<(DateTime<Utc>, f64)> {
        let mut rng = rand::thread_rng();
        let mut prices = Vec::with_capacity(count);
        let mut current_price = start_price;
        let start_time = Utc::now() - Duration::days(count as i64);

        for i in 0..count {
            let timestamp = start_time + Duration::days(i as i64);

            // Add some realistic noise and trend
            let change_percent = rng.gen_range(-volatility..volatility);
            current_price *= 1.0 + change_percent;

            // Keep prices positive and realistic
            current_price = current_price.max(1.0);

            prices.push((timestamp, current_price));
        }

        prices
    }

    /// Generate OHLCV data for testing
    pub fn generate_ohlcv_data(count: usize) -> Vec<OHLCVData> {
        let mut rng = rand::thread_rng();
        let mut data = Vec::with_capacity(count);
        let mut base_price = 100.0;
        let start_time = Utc::now() - Duration::days(count as i64);

        for i in 0..count {
            let timestamp = start_time + Duration::days(i as i64);

            // Generate realistic OHLCV data
            let open = base_price;
            let high_mult = rng.gen_range(1.0..1.05);
            let low_mult = rng.gen_range(0.95..1.0);
            let close_mult = rng.gen_range(0.98..1.02);

            let high = open * high_mult;
            let low = open * low_mult;
            let close = open * close_mult;
            let volume = rng.gen_range(10000.0..100000.0);

            data.push(OHLCVData {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            });

            base_price = close;
        }

        data
    }

    /// Generate trading signals for testing
    pub fn generate_trading_signals(count: usize) -> Vec<TradingSignal> {
        let mut signals = Vec::with_capacity(count);
        let start_time = Utc::now() - Duration::days(count as i64);

        for i in 0..count {
            let timestamp = start_time + Duration::hours((i * 4) as i64);
            let signal_type = if i % 3 == 0 {
                SignalType::Buy
            } else if i % 3 == 1 {
                SignalType::Sell
            } else {
                SignalType::Hold
            };

            let strength = Faker.fake::<f64>();
            let price = 100.0 + (i as f64 * 0.5);

            signals.push(TradingSignal {
                timestamp,
                signal_type,
                strength,
                price,
                confidence: strength * 0.8,
            });
        }

        signals
    }

    /// Generate market data for backtesting
    pub fn generate_market_data(days: usize) -> MarketData {
        let ohlcv = Self::generate_ohlcv_data(days);
        let signals = Self::generate_trading_signals(days * 6); // 4 signals per day

        MarketData {
            symbol: "TEST".to_string(),
            ohlcv,
            signals,
            metadata: HashMap::from([
                ("exchange".to_string(), "TEST_EXCHANGE".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OHLCVData {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub struct TradingSignal {
    pub timestamp: DateTime<Utc>,
    pub signal_type: SignalType,
    pub strength: f64,
    pub price: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub ohlcv: Vec<OHLCVData>,
    pub signals: Vec<TradingSignal>,
    pub metadata: HashMap<String, String>,
}

/// Common test assertions and utilities
pub mod assertions {
    use approx::assert_relative_eq;

    /// Assert that a value is within expected bounds
    pub fn assert_in_range(value: f64, min: f64, max: f64) {
        assert!(
            value >= min && value <= max,
            "Value {} not in range [{}, {}]",
            value,
            min,
            max
        );
    }

    /// Assert that two vectors are approximately equal
    pub fn assert_vec_approx_eq(actual: &[f64], expected: &[f64], tolerance: f64) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "Vector lengths differ: {} vs {}",
            actual.len(),
            expected.len()
        );

        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert_relative_eq!(
                a,
                e,
                epsilon = tolerance,
                "Values differ at index {}: {} vs {}",
                i,
                a,
                e
            );
        }
    }

    /// Assert that a forecast result is valid
    pub fn assert_forecast_valid(forecast: &[f64], min_len: usize) {
        assert!(
            forecast.len() >= min_len,
            "Forecast too short: {} < {}",
            forecast.len(),
            min_len
        );

        // Check for NaN or infinite values
        for (i, &value) in forecast.iter().enumerate() {
            assert!(
                value.is_finite(),
                "Invalid forecast value at index {}: {}",
                i,
                value
            );
        }
    }

    /// Assert that trading metrics are reasonable
    pub fn assert_trading_metrics_valid(returns: f64, sharpe_ratio: f64, max_drawdown: f64) {
        assert!(returns.is_finite(), "Returns must be finite");
        assert!(sharpe_ratio.is_finite(), "Sharpe ratio must be finite");
        assert!(max_drawdown >= 0.0, "Max drawdown must be non-negative");
        assert!(max_drawdown <= 1.0, "Max drawdown must be <= 1.0");
    }
}

/// Mock implementations for testing
pub mod mocks {
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        pub DataProvider {}

        impl DataProvider {
            fn get_price_data(&self, symbol: &str, days: usize) -> Result<Vec<f64>, String>;
            fn get_market_data(&self, symbol: &str) -> Result<super::MarketData, String>;
        }
    }

    mock! {
        pub ForecastEngine {}

        impl ForecastEngine {
            fn forecast(&self, data: &[f64], periods: usize) -> Result<Vec<f64>, String>;
            fn get_model_name(&self) -> String;
            fn get_accuracy(&self) -> f64;
        }
    }
}

/// Performance testing utilities
pub mod performance {
    use std::time::{Duration, Instant};

    /// Measure execution time of a function
    pub fn measure_time<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Assert that a function executes within a time limit
    pub fn assert_execution_time<F, R>(f: F, max_duration: Duration) -> R
    where
        F: FnOnce() -> R,
    {
        let (result, duration) = measure_time(f);
        assert!(
            duration <= max_duration,
            "Execution took too long: {:?} > {:?}",
            duration,
            max_duration
        );
        result
    }
}

#[cfg(test)]
mod tests {
    use super::assertions::*;
    use super::*;

    #[test]
    fn test_data_generator_stock_prices() {
        let prices = TestDataGenerator::generate_stock_prices(100, 100.0, 0.02);

        assert_eq!(prices.len(), 100);
        assert!(prices.iter().all(|(_, price)| *price > 0.0));

        // Check that timestamps are in order
        for window in prices.windows(2) {
            assert!(window[0].0 < window[1].0);
        }
    }

    #[test]
    fn test_data_generator_ohlcv() {
        let data = TestDataGenerator::generate_ohlcv_data(50);

        assert_eq!(data.len(), 50);

        for ohlcv in &data {
            // Validate OHLCV constraints
            assert!(ohlcv.high >= ohlcv.open);
            assert!(ohlcv.high >= ohlcv.close);
            assert!(ohlcv.low <= ohlcv.open);
            assert!(ohlcv.low <= ohlcv.close);
            assert!(ohlcv.volume > 0.0);
        }
    }

    #[test]
    fn test_assertions() {
        assert_in_range(5.0, 0.0, 10.0);

        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.001, 2.001, 3.001];
        assert_vec_approx_eq(&vec1, &vec2, 0.01);

        let forecast = vec![100.0, 101.0, 102.0];
        assert_forecast_valid(&forecast, 3);

        assert_trading_metrics_valid(0.15, 1.2, 0.05);
    }
}
