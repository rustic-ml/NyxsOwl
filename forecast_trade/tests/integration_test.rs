use forecast_trade::models::exponential_smoothing::ExponentialSmoothing;
use forecast_trade::strategies::mean_reversion::MeanReversionStrategy;
use forecast_trade::utils::date_parser;
use forecast_trade::volatility::historical_volatility;
use forecast_trade::{
    DataLoader, ForecastError, ForecastModel, ForecastResult, ForecastStrategy, TimeSeriesData,
};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

// Helper function to create a simple test dataset
fn create_sample_data() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();

    writeln!(file, "date,price").unwrap();
    writeln!(file, "2023-01-01,100.0").unwrap();
    writeln!(file, "2023-01-02,102.0").unwrap();
    writeln!(file, "2023-01-03,101.0").unwrap();
    writeln!(file, "2023-01-04,103.0").unwrap();
    writeln!(file, "2023-01-05,102.0").unwrap();
    writeln!(file, "2023-01-06,104.0").unwrap();
    writeln!(file, "2023-01-07,103.0").unwrap();
    writeln!(file, "2023-01-08,105.0").unwrap();
    writeln!(file, "2023-01-09,104.0").unwrap();
    writeln!(file, "2023-01-10,106.0").unwrap();

    file
}

#[test]
fn test_full_forecast_workflow() {
    // 1. Create sample data file
    let data_file = create_sample_data();
    let file_path = data_file.path().to_str().unwrap();

    // 2. Load data
    let data = DataLoader::from_csv(file_path).unwrap();
    assert_eq!(data.len(), 10);

    // 3. Create and train a forecasting model
    let model = ExponentialSmoothing::new(0.7).unwrap();
    let trained_model = model.train(&data).unwrap();

    // 4. Generate forecast
    let forecast = trained_model.forecast(3).unwrap();
    assert_eq!(forecast.horizons(), 3);

    // 5. Evaluate forecast accuracy on existing data
    let predicted = trained_model.predict(&data).unwrap();
    let mse = predicted.mean_squared_error(&data).unwrap();
    assert!(mse >= 0.0);

    // 6. Calculate volatility
    let volatility = historical_volatility(&data, 5).unwrap();
    assert_eq!(volatility.len(), 6); // 10 - 5 + 1

    // 7. Create and test a trading strategy
    let strategy = MeanReversionStrategy::new(model, 1.5).unwrap();
    let signals = strategy.generate_signals(&data).unwrap();
    assert_eq!(signals.len(), data.len());

    // 8. Run backtest
    let backtest_results = strategy.backtest(&data, 1000.0).unwrap();
    assert!(backtest_results.final_balance > 0.0);

    // 9. Test error handling
    let invalid_path = "/nonexistent/path.csv";
    let result = DataLoader::from_csv(invalid_path);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, ForecastError::IoError(_)));
}

#[test]
fn test_create_custom_strategy() {
    // Create a simple dataset
    let dates = vec![
        "2023-01-01",
        "2023-01-02",
        "2023-01-03",
        "2023-01-04",
        "2023-01-05",
    ]
    .into_iter()
    .map(|s| date_parser::parse_date(s).unwrap())
    .collect();

    let values = vec![100.0, 102.0, 101.0, 103.0, 102.0];
    let data = TimeSeriesData::new(dates, values).unwrap();

    // Create a model
    let model = ExponentialSmoothing::new(0.5).unwrap();

    // Test the combined forecasting and volatility approach
    let trained_model = model.train(&data).unwrap();
    let forecast = trained_model.forecast(2).unwrap();
    let volatility = historical_volatility(&data, 3).unwrap();

    // Verify forecast
    assert_eq!(forecast.horizons(), 2);
    assert!(!forecast.values().is_empty());

    // Verify volatility
    assert_eq!(volatility.len(), 3);
    assert!(volatility.iter().all(|v| *v > 0.0));

    // Create a confidence interval based on volatility
    let forecast_values = forecast.values();
    let confidence_intervals: Vec<(f64, f64)> = forecast_values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            // Use the last volatility value for simplicity
            let vol = volatility[volatility.len() - 1];
            let lower = v - 1.96 * vol;
            let upper = v + 1.96 * vol;
            (lower, upper)
        })
        .collect();

    // Verify confidence intervals
    assert_eq!(confidence_intervals.len(), forecast.horizons());
    for (lower, upper) in confidence_intervals {
        assert!(lower < upper);
    }
}
