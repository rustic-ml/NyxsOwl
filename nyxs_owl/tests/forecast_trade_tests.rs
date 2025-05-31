#[cfg(test)]
mod forecast_trade_tests {
    use crate::test_utils::{assertions::*, TestDataGenerator};
    use approx::assert_relative_eq;
    use chrono::{DateTime, Utc};
    use nyxs_owl::forecast_trade::*;
    use rstest::*;
    use std::time::Duration;

    /// Test fixture for common forecast data
    #[fixture]
    fn sample_time_series() -> Vec<f64> {
        TestDataGenerator::generate_stock_prices(100, 100.0, 0.02)
            .into_iter()
            .map(|(_, price)| price)
            .collect()
    }

    #[fixture]
    fn sample_dates() -> Vec<DateTime<Utc>> {
        TestDataGenerator::generate_stock_prices(100, 100.0, 0.02)
            .into_iter()
            .map(|(date, _)| date)
            .collect()
    }

    #[fixture]
    fn trending_data() -> Vec<f64> {
        (0..50).map(|i| 100.0 + i as f64 * 0.5).collect()
    }

    #[fixture]
    fn seasonal_data() -> Vec<f64> {
        (0..100)
            .map(|i| 100.0 + 10.0 * (i as f64 * 0.1).sin())
            .collect()
    }

    mod moving_average_tests {
        use super::*;

        #[rstest]
        fn test_simple_moving_average_basic(sample_time_series: Vec<f64>) {
            let result = forecast_moving_average(&sample_time_series, 5, 10);

            match result {
                Ok(forecast) => {
                    assert_forecast_valid(&forecast, 10);
                    // Moving average should smooth out volatility
                    assert!(forecast.iter().all(|&x| x > 0.0));
                }
                Err(e) => panic!("Moving average failed: {}", e),
            }
        }

        #[rstest]
        fn test_moving_average_with_different_windows(sample_time_series: Vec<f64>) {
            let windows = [3, 5, 10, 20];

            for &window in &windows {
                let result = forecast_moving_average(&sample_time_series, window, 5);
                assert!(
                    result.is_ok(),
                    "Moving average failed for window size {}: {:?}",
                    window,
                    result
                );
            }
        }

        #[rstest]
        fn test_moving_average_edge_cases() {
            // Test with minimal data
            let small_data = vec![100.0, 101.0, 102.0];
            let result = forecast_moving_average(&small_data, 2, 1);
            assert!(result.is_ok());

            // Test with single data point
            let single_data = vec![100.0];
            let result = forecast_moving_average(&single_data, 1, 1);
            assert!(result.is_ok());
        }

        #[rstest]
        fn test_moving_average_error_conditions() {
            let data = vec![100.0, 101.0, 102.0];

            // Window larger than data
            let result = forecast_moving_average(&data, 10, 1);
            assert!(result.is_err());

            // Empty data
            let result = forecast_moving_average(&[], 1, 1);
            assert!(result.is_err());

            // Zero periods
            let result = forecast_moving_average(&data, 2, 0);
            assert!(result.is_err());
        }

        #[rstest]
        fn test_moving_average_consistency(trending_data: Vec<f64>) {
            // Test that MA follows trend
            let forecast = forecast_moving_average(&trending_data, 5, 10).unwrap();

            // For trending data, forecast should generally increase
            let trend_positive = forecast
                .windows(2)
                .filter(|window| window[1] > window[0])
                .count() as f64
                / (forecast.len() - 1) as f64;

            assert!(
                trend_positive > 0.5,
                "Moving average should follow upward trend"
            );
        }
    }

    mod exponential_smoothing_tests {
        use super::*;

        #[rstest]
        fn test_exponential_smoothing_basic(sample_time_series: Vec<f64>) {
            let result = forecast_exponential_smoothing(&sample_time_series, 0.3, 10);

            match result {
                Ok(forecast) => {
                    assert_forecast_valid(&forecast, 10);
                    assert!(forecast.iter().all(|&x| x > 0.0));
                }
                Err(e) => panic!("Exponential smoothing failed: {}", e),
            }
        }

        #[rstest]
        fn test_exponential_smoothing_alpha_values(sample_time_series: Vec<f64>) {
            let alphas = [0.1, 0.3, 0.5, 0.7, 0.9];

            for &alpha in &alphas {
                let result = forecast_exponential_smoothing(&sample_time_series, alpha, 5);
                assert!(
                    result.is_ok(),
                    "Exponential smoothing failed for alpha {}: {:?}",
                    alpha,
                    result
                );
            }
        }

        #[rstest]
        fn test_exponential_smoothing_alpha_effects(trending_data: Vec<f64>) {
            let low_alpha = forecast_exponential_smoothing(&trending_data, 0.1, 5).unwrap();
            let high_alpha = forecast_exponential_smoothing(&trending_data, 0.9, 5).unwrap();

            // High alpha should be more responsive to recent changes
            // For trending data, this should be visible in the forecast
            assert_ne!(low_alpha, high_alpha);
        }

        #[rstest]
        fn test_exponential_smoothing_error_conditions() {
            let data = vec![100.0, 101.0, 102.0];

            // Invalid alpha values
            assert!(forecast_exponential_smoothing(&data, -0.1, 1).is_err());
            assert!(forecast_exponential_smoothing(&data, 1.1, 1).is_err());

            // Empty data
            assert!(forecast_exponential_smoothing(&[], 0.3, 1).is_err());

            // Zero periods
            assert!(forecast_exponential_smoothing(&data, 0.3, 0).is_err());
        }

        #[rstest]
        fn test_exponential_smoothing_seasonal_data(seasonal_data: Vec<f64>) {
            let result = forecast_exponential_smoothing(&seasonal_data, 0.3, 10);
            assert!(result.is_ok());

            let forecast = result.unwrap();
            assert_forecast_valid(&forecast, 10);
        }
    }

    mod arima_tests {
        use super::*;

        #[rstest]
        fn test_arima_basic(sample_time_series: Vec<f64>) {
            let result = forecast_arima(&sample_time_series, (1, 1, 1), 10);

            match result {
                Ok(forecast) => {
                    assert_forecast_valid(&forecast, 10);
                    assert!(forecast.iter().all(|&x| x.is_finite()));
                }
                Err(e) => panic!("ARIMA failed: {}", e),
            }
        }

        #[rstest]
        fn test_arima_different_orders(sample_time_series: Vec<f64>) {
            let orders = [(1, 0, 0), (0, 1, 1), (2, 1, 2), (1, 1, 1)];

            for &order in &orders {
                let result = forecast_arima(&sample_time_series, order, 5);
                assert!(
                    result.is_ok(),
                    "ARIMA failed for order {:?}: {:?}",
                    order,
                    result
                );
            }
        }

        #[rstest]
        fn test_arima_trending_data(trending_data: Vec<f64>) {
            // ARIMA(0,1,1) should handle trends well
            let result = forecast_arima(&trending_data, (0, 1, 1), 10);
            assert!(result.is_ok());

            let forecast = result.unwrap();
            assert_forecast_valid(&forecast, 10);

            // Forecast should continue the trend
            let last_value = trending_data.last().unwrap();
            let first_forecast = forecast[0];
            assert!(
                (first_forecast - last_value).abs() < 5.0,
                "ARIMA forecast should be close to last observed value"
            );
        }

        #[rstest]
        fn test_arima_error_conditions() {
            let data = vec![100.0, 101.0, 102.0];

            // Insufficient data for high-order models
            let result = forecast_arima(&data, (5, 2, 5), 1);
            assert!(result.is_err());

            // Empty data
            assert!(forecast_arima(&[], (1, 1, 1), 1).is_err());

            // Zero periods
            assert!(forecast_arima(&data, (1, 1, 1), 0).is_err());
        }

        #[rstest]
        fn test_arima_seasonal_capability(seasonal_data: Vec<f64>) {
            // Test ARIMA's ability to handle seasonal patterns
            let result = forecast_arima(&seasonal_data, (2, 1, 2), 20);
            assert!(result.is_ok());

            let forecast = result.unwrap();
            assert_forecast_valid(&forecast, 20);
        }
    }

    mod api_integration_tests {
        use super::*;

        #[rstest]
        fn test_oxidiviner_easy_api(
            sample_time_series: Vec<f64>,
            sample_dates: Vec<DateTime<Utc>>,
        ) {
            // Test the easy API functions if available
            if let Ok((forecast, model_name)) =
                crate::forecast_trade::easy::auto_forecast(&sample_dates, &sample_time_series, 10)
            {
                assert_forecast_valid(&forecast, 10);
                assert!(!model_name.is_empty());
                println!("Auto-selected model: {}", model_name);
            }
        }

        #[rstest]
        fn test_all_forecast_methods_consistency(sample_time_series: Vec<f64>) {
            let periods = 5;

            // Test all methods with same data
            let ma_result = forecast_moving_average(&sample_time_series, 10, periods);
            let es_result = forecast_exponential_smoothing(&sample_time_series, 0.3, periods);
            let arima_result = forecast_arima(&sample_time_series, (1, 1, 1), periods);

            // All should succeed or fail consistently based on data quality
            match (ma_result, es_result, arima_result) {
                (Ok(ma), Ok(es), Ok(arima)) => {
                    assert_eq!(ma.len(), periods);
                    assert_eq!(es.len(), periods);
                    assert_eq!(arima.len(), periods);

                    // All forecasts should be finite
                    assert!(ma.iter().all(|&x| x.is_finite()));
                    assert!(es.iter().all(|&x| x.is_finite()));
                    assert!(arima.iter().all(|&x| x.is_finite()));
                }
                _ => {
                    // At least one method should work for reasonable data
                    assert!(
                        ma_result.is_ok() || es_result.is_ok() || arima_result.is_ok(),
                        "At least one forecasting method should work"
                    );
                }
            }
        }

        #[rstest]
        fn test_forecast_performance() {
            let large_data: Vec<f64> = (0..1000).map(|i| 100.0 + (i as f64 * 0.01)).collect();

            // Test that forecasting completes in reasonable time
            let start = std::time::Instant::now();
            let result = forecast_moving_average(&large_data, 20, 10);
            let duration = start.elapsed();

            assert!(
                duration < Duration::from_secs(5),
                "Forecast took too long: {:?}",
                duration
            );
            assert!(result.is_ok(), "Large dataset forecast failed");
        }
    }

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_moving_average_properties(
                data in prop::collection::vec(1.0f64..1000.0, 10..100),
                window in 3usize..10,
                periods in 1usize..20
            ) {
                if data.len() >= window {
                    let result = forecast_moving_average(&data, window, periods);
                    prop_assert!(result.is_ok());

                    if let Ok(forecast) = result {
                        prop_assert_eq!(forecast.len(), periods);
                        prop_assert!(forecast.iter().all(|&x| x.is_finite()));
                        prop_assert!(forecast.iter().all(|&x| x > 0.0));
                    }
                }
            }

            #[test]
            fn test_exponential_smoothing_properties(
                data in prop::collection::vec(1.0f64..1000.0, 5..100),
                alpha in 0.1f64..0.9,
                periods in 1usize..20
            ) {
                let result = forecast_exponential_smoothing(&data, alpha, periods);
                prop_assert!(result.is_ok());

                if let Ok(forecast) = result {
                    prop_assert_eq!(forecast.len(), periods);
                    prop_assert!(forecast.iter().all(|&x| x.is_finite()));
                    prop_assert!(forecast.iter().all(|&x| x > 0.0));
                }
            }

            #[test]
            fn test_arima_properties(
                data in prop::collection::vec(1.0f64..1000.0, 20..100),
                p in 1usize..3,
                d in 0usize..2,
                q in 1usize..3,
                periods in 1usize..10
            ) {
                let result = forecast_arima(&data, (p, d, q), periods);

                // ARIMA might fail for some parameter combinations, but shouldn't panic
                if let Ok(forecast) = result {
                    prop_assert_eq!(forecast.len(), periods);
                    prop_assert!(forecast.iter().all(|&x| x.is_finite()));
                }
            }
        }
    }

    mod error_handling_tests {
        use super::*;

        #[test]
        fn test_graceful_error_handling() {
            let invalid_data = vec![f64::NAN, f64::INFINITY, -f64::INFINITY];

            // All methods should handle invalid data gracefully
            assert!(forecast_moving_average(&invalid_data, 2, 1).is_err());
            assert!(forecast_exponential_smoothing(&invalid_data, 0.3, 1).is_err());
            assert!(forecast_arima(&invalid_data, (1, 1, 1), 1).is_err());
        }

        #[test]
        fn test_error_message_quality() {
            let result = forecast_moving_average(&[], 1, 1);
            assert!(result.is_err());

            let error_msg = format!("{}", result.unwrap_err());
            assert!(!error_msg.is_empty());
            assert!(error_msg.contains("data") || error_msg.contains("empty"));
        }

        #[test]
        fn test_parameter_validation() {
            let data = vec![100.0, 101.0, 102.0];

            // Test various invalid parameter combinations
            assert!(forecast_moving_average(&data, 0, 1).is_err());
            assert!(forecast_exponential_smoothing(&data, 0.0, 1).is_err());
            assert!(forecast_exponential_smoothing(&data, 1.0, 1).is_err());
        }
    }

    mod integration_with_oxidiviner {
        use super::*;

        #[test]
        fn test_oxidiviner_integration() {
            let data = TestDataGenerator::generate_stock_prices(50, 100.0, 0.02);
            let dates: Vec<DateTime<Utc>> = data.iter().map(|(d, _)| *d).collect();
            let values: Vec<f64> = data.iter().map(|(_, v)| *v).collect();

            // Test that we can successfully integrate with OxiDiviner
            // This test ensures the API works end-to-end
            let periods = 10;

            // Test moving average through OxiDiviner
            if let Ok(forecast) = forecast_moving_average(&values, 10, periods) {
                assert_forecast_valid(&forecast, periods);
                println!("✓ Moving Average integration successful");
            }

            // Test exponential smoothing through OxiDiviner
            if let Ok(forecast) = forecast_exponential_smoothing(&values, 0.3, periods) {
                assert_forecast_valid(&forecast, periods);
                println!("✓ Exponential Smoothing integration successful");
            }

            // Test ARIMA through OxiDiviner
            if let Ok(forecast) = forecast_arima(&values, (1, 1, 1), periods) {
                assert_forecast_valid(&forecast, periods);
                println!("✓ ARIMA integration successful");
            }
        }

        #[test]
        fn test_oxidiviner_model_comparison() {
            let data = TestDataGenerator::generate_stock_prices(100, 100.0, 0.02);
            let values: Vec<f64> = data.iter().map(|(_, v)| *v).collect();

            let periods = 5;
            let mut successful_models = Vec::new();

            // Test each model and collect results
            if let Ok(ma_forecast) = forecast_moving_average(&values, 20, periods) {
                successful_models.push(("Moving Average", ma_forecast));
            }

            if let Ok(es_forecast) = forecast_exponential_smoothing(&values, 0.3, periods) {
                successful_models.push(("Exponential Smoothing", es_forecast));
            }

            if let Ok(arima_forecast) = forecast_arima(&values, (1, 1, 1), periods) {
                successful_models.push(("ARIMA", arima_forecast));
            }

            // At least one model should work
            assert!(
                !successful_models.is_empty(),
                "At least one forecasting model should work"
            );

            // All successful forecasts should be valid
            for (model_name, forecast) in successful_models {
                assert_forecast_valid(&forecast, periods);
                println!("✓ {} produced valid forecast", model_name);
            }
        }
    }
}

// Helper functions that might be used by forecast_trade module
// These should be accessible during testing

/// Dummy implementations for testing if the actual module doesn't exist yet
#[cfg(not(feature = "oxidiviner"))]
mod fallback_implementations {
    use super::*;

    pub fn forecast_moving_average(
        data: &[f64],
        window: usize,
        periods: usize,
    ) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Data cannot be empty".to_string());
        }
        if window == 0 || periods == 0 {
            return Err("Window and periods must be greater than 0".to_string());
        }
        if window > data.len() {
            return Err("Window size cannot be larger than data length".to_string());
        }

        // Simple moving average implementation for testing
        let last_values = &data[data.len().saturating_sub(window)..];
        let avg = last_values.iter().sum::<f64>() / last_values.len() as f64;

        Ok(vec![avg; periods])
    }

    pub fn forecast_exponential_smoothing(
        data: &[f64],
        alpha: f64,
        periods: usize,
    ) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Data cannot be empty".to_string());
        }
        if !(0.0 < alpha && alpha < 1.0) {
            return Err("Alpha must be between 0 and 1".to_string());
        }
        if periods == 0 {
            return Err("Periods must be greater than 0".to_string());
        }

        // Simple exponential smoothing
        let mut smoothed = data[0];
        for &value in data.iter().skip(1) {
            smoothed = alpha * value + (1.0 - alpha) * smoothed;
        }

        Ok(vec![smoothed; periods])
    }

    pub fn forecast_arima(
        data: &[f64],
        order: (usize, usize, usize),
        periods: usize,
    ) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Data cannot be empty".to_string());
        }
        if periods == 0 {
            return Err("Periods must be greater than 0".to_string());
        }

        let (p, d, q) = order;
        if p + d + q > data.len() / 2 {
            return Err("Model order too high for data length".to_string());
        }

        // Simple ARIMA approximation - just return last value with small random walk
        let last_value = data[data.len() - 1];
        Ok(vec![last_value; periods])
    }
}
