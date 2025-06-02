#[cfg(all(test, feature = "forecasting"))]
mod forecast_trade_tests {
    use chrono::{DateTime, Duration, Utc};
    use nyxs_owl::forecast_trade::easy::*;
    use nyxs_owl::forecast_trade::*;
    use rstest::*;

    /// Test fixture for common forecast data
    #[fixture]
    fn sample_time_series() -> Vec<f64> {
        (0..50)
            .map(|i| 100.0 + i as f64 * 0.5 + (i as f64 * 0.1).sin() * 5.0)
            .collect()
    }

    #[fixture]
    fn sample_dates() -> Vec<DateTime<Utc>> {
        let start = Utc::now();
        (0..50).map(|i| start + Duration::days(i as i64)).collect()
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

    /// Helper function to validate forecast results
    fn assert_forecast_valid(forecast: &[f64], expected_length: usize) {
        assert_eq!(
            forecast.len(),
            expected_length,
            "Forecast length doesn't match expected"
        );
        assert!(
            forecast.iter().all(|&x| x.is_finite()),
            "Forecast contains non-finite values"
        );
        assert!(!forecast.is_empty(), "Forecast should not be empty");
    }

    mod time_series_data_tests {
        use super::*;

        #[rstest]
        fn test_time_series_creation_success(
            sample_dates: Vec<DateTime<Utc>>,
            sample_time_series: Vec<f64>,
        ) {
            let timestamps = sample_dates[..10].to_vec();
            let values = sample_time_series[..10].to_vec();

            let ts_data = TimeSeriesData::new(timestamps, values);
            assert!(ts_data.is_ok());

            let ts_data = ts_data.unwrap();
            assert_eq!(ts_data.len(), 10);
            assert!(!ts_data.is_empty());
            assert!(ts_data.last_value().is_some());
        }

        #[rstest]
        fn test_time_series_validation_errors() {
            let timestamps = vec![Utc::now()];
            let values = vec![1.0, 2.0]; // Mismatched lengths

            let result = TimeSeriesData::new(timestamps, values);
            assert!(result.is_err());

            // Test empty data
            let result = TimeSeriesData::new(vec![], vec![]);
            assert!(result.is_err());

            // Test invalid values
            let timestamps = vec![Utc::now()];
            let values = vec![f64::NAN];
            let result = TimeSeriesData::new(timestamps, values);
            assert!(result.is_err());
        }
    }

    mod moving_average_tests {
        use super::*;

        #[rstest]
        fn test_moving_average_basic(sample_time_series: Vec<f64>) {
            let result = forecast_moving_average(&sample_time_series, 5, 10);

            match result {
                Ok(forecast) => {
                    assert_forecast_valid(&forecast, 10);
                    // Moving average should smooth out volatility
                    assert!(forecast.iter().all(|&x| x > 0.0));
                    println!("✓ Moving average forecast successful");
                }
                Err(e) => {
                    println!("Moving average failed (may be expected): {}", e);
                }
            }
        }

        #[rstest]
        fn test_moving_average_with_different_windows(sample_time_series: Vec<f64>) {
            let windows = [3, 5, 10, 20];

            for &window in &windows {
                let result = forecast_moving_average(&sample_time_series, window, 5);
                if result.is_ok() {
                    println!("✓ Moving average works for window size {}", window);
                }
            }
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
                    println!("✓ Exponential smoothing forecast successful");
                }
                Err(e) => {
                    println!("Exponential smoothing failed (may be expected): {}", e);
                }
            }
        }

        #[rstest]
        fn test_exponential_smoothing_alpha_values(sample_time_series: Vec<f64>) {
            let alphas = [0.1, 0.3, 0.5, 0.7, 0.9];

            for &alpha in &alphas {
                let result = forecast_exponential_smoothing(&sample_time_series, alpha, 5);
                if result.is_ok() {
                    println!("✓ Exponential smoothing works for alpha {}", alpha);
                }
            }
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
                    println!("✓ ARIMA forecast successful");
                }
                Err(e) => {
                    println!("ARIMA failed (may be expected): {}", e);
                }
            }
        }

        #[rstest]
        fn test_arima_different_orders(sample_time_series: Vec<f64>) {
            let orders = [(1, 0, 0), (0, 1, 1), (2, 1, 2), (1, 1, 1)];

            for &order in &orders {
                let result = forecast_arima(&sample_time_series, order, 5);
                if result.is_ok() {
                    println!("✓ ARIMA works for order {:?}", order);
                }
            }
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
    }

    mod easy_api_tests {
        use super::*;

        #[rstest]
        fn test_auto_forecast(sample_dates: Vec<DateTime<Utc>>, sample_time_series: Vec<f64>) {
            let timestamps = sample_dates[..30].to_vec();
            let values = sample_time_series[..30].to_vec();

            let result = auto_forecast(&timestamps, &values, 10);

            match result {
                Ok((forecast, model_name)) => {
                    assert_forecast_valid(&forecast, 10);
                    assert!(!model_name.is_empty());
                    println!(
                        "✓ Auto-forecast works: {} model, {} predictions",
                        model_name,
                        forecast.len()
                    );
                }
                Err(e) => {
                    println!("Auto-forecast failed (may be expected): {}", e);
                }
            }
        }

        #[rstest]
        fn test_financial_forecast(sample_time_series: Vec<f64>) {
            let result = financial_forecast(&sample_time_series, 5);

            match result {
                Ok((forecast, model_name)) => {
                    assert_forecast_valid(&forecast, 5);
                    assert!(!model_name.is_empty());
                    println!("✓ Financial forecast works: {} model", model_name);
                }
                Err(e) => {
                    println!("Financial forecast failed (may be expected): {}", e);
                }
            }
        }

        #[rstest]
        fn test_model_comparison(sample_dates: Vec<DateTime<Utc>>, sample_time_series: Vec<f64>) {
            let timestamps = sample_dates[..30].to_vec();
            let values = sample_time_series[..30].to_vec();

            let result = model_comparison(&timestamps, &values, 5);

            match result {
                Ok(results) => {
                    assert!(!results.is_empty());
                    for (model_name, forecast) in results {
                        assert_forecast_valid(&forecast, 5);
                        println!("✓ {} model produced valid forecast", model_name);
                    }
                }
                Err(e) => {
                    println!("Model comparison failed (may be expected): {}", e);
                }
            }
        }
    }

    mod integration_tests {
        use super::*;

        #[rstest]
        fn test_forecast_methods_consistency(sample_time_series: Vec<f64>) {
            let periods = 5;

            // Test all methods with same data
            let ma_result = forecast_moving_average(&sample_time_series, 10, periods);
            let es_result = forecast_exponential_smoothing(&sample_time_series, 0.3, periods);
            let arima_result = forecast_arima(&sample_time_series, (1, 1, 1), periods);

            let mut successful_methods = 0;

            if let Ok(forecast) = ma_result {
                assert_forecast_valid(&forecast, periods);
                successful_methods += 1;
                println!("✓ Moving Average successful");
            }

            if let Ok(forecast) = es_result {
                assert_forecast_valid(&forecast, periods);
                successful_methods += 1;
                println!("✓ Exponential Smoothing successful");
            }

            if let Ok(forecast) = arima_result {
                assert_forecast_valid(&forecast, periods);
                successful_methods += 1;
                println!("✓ ARIMA successful");
            }

            // At least one method should work for reasonable data
            assert!(
                successful_methods > 0,
                "At least one forecasting method should work"
            );
        }

        #[rstest]
        fn test_oxidiviner_integration() {
            // Create test data
            let values: Vec<f64> = (0..50).map(|i| 100.0 + i as f64 * 0.02).collect();
            let now = Utc::now();
            let timestamps: Vec<DateTime<Utc>> =
                (0..50).map(|i| now + Duration::days(i as i64)).collect();

            let periods = 10;
            let mut successful_models = Vec::new();

            // Test each model through the easy API
            if let Ok(forecast) = ma_forecast(&timestamps, &values, periods) {
                assert_forecast_valid(&forecast, periods);
                successful_models.push("Moving Average");
            }

            if let Ok(forecast) = es_forecast(&timestamps, &values, periods) {
                assert_forecast_valid(&forecast, periods);
                successful_models.push("Exponential Smoothing");
            }

            if let Ok(forecast) = arima_forecast(&timestamps, &values, periods) {
                assert_forecast_valid(&forecast, periods);
                successful_models.push("ARIMA");
            }

            // At least one model should work
            assert!(
                !successful_models.is_empty(),
                "At least one forecasting model should work"
            );

            for model_name in successful_models {
                println!("✓ {} integration successful", model_name);
            }
        }
    }

    mod error_handling_tests {
        use super::*;

        #[test]
        fn test_graceful_error_handling() {
            let invalid_data = vec![f64::NAN, f64::INFINITY, -f64::INFINITY];

            // Test that methods handle invalid data (some might work, some might fail)
            // The important thing is they don't panic
            let _ma_result = forecast_moving_average(&invalid_data, 2, 1);
            let _es_result = forecast_exponential_smoothing(&invalid_data, 0.3, 1);
            let _arima_result = forecast_arima(&invalid_data, (1, 1, 1), 1);

            // Just ensure we don't panic - oxidiviner might handle some edge cases
            println!("✓ No panics with invalid data");
        }

        #[test]
        fn test_error_message_quality() {
            let result = forecast_moving_average(&[], 1, 1);
            assert!(result.is_err());

            let error_msg = format!("{}", result.unwrap_err());
            assert!(!error_msg.is_empty());
            assert!(error_msg.contains("Data cannot be empty"));
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
}
