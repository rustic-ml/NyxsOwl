use assert_approx_eq::assert_approx_eq;
use forecast_trade::data::TimeSeriesData;
use forecast_trade::models::arima::ARIMA;
use forecast_trade::models::exponential_smoothing::ExponentialSmoothing;
use forecast_trade::models::moving_average::MovingAverage;
use forecast_trade::models::{ForecastModel, ForecastResult};

fn create_test_data() -> TimeSeriesData {
    let dates = vec![
        "2023-01-01",
        "2023-01-02",
        "2023-01-03",
        "2023-01-04",
        "2023-01-05",
    ]
    .into_iter()
    .map(|s| s.parse().unwrap())
    .collect();

    let values = vec![100.0, 102.0, 104.0, 103.0, 105.0];

    TimeSeriesData::new(dates, values).unwrap()
}

#[test]
fn test_exponential_smoothing() {
    let data = create_test_data();
    let model = ExponentialSmoothing::new(0.7).unwrap();

    // Train the model
    let trained_model = model.train(&data).unwrap();

    // Forecast future values
    let forecast = trained_model.forecast(3).unwrap();

    assert_eq!(forecast.horizons(), 3);
    assert!(!forecast.values().is_empty());

    // Test prediction accuracy
    let predicted = trained_model.predict(&data).unwrap();
    assert_eq!(predicted.len(), data.len());

    // Calculate error metrics
    let mse = predicted.mean_squared_error(&data).unwrap();
    assert!(mse >= 0.0);
}

#[test]
fn test_moving_average() {
    let data = create_test_data();
    let model = MovingAverage::new(3).unwrap();

    // Train the model
    let trained_model = model.train(&data).unwrap();

    // Forecast future values
    let forecast = trained_model.forecast(2).unwrap();

    assert_eq!(forecast.horizons(), 2);
    assert!(!forecast.values().is_empty());

    // Verify forecast values are reasonable
    let values = forecast.values();
    for value in values {
        assert!(*value > 100.0 && *value < 110.0);
    }
}

#[test]
fn test_arima_model() {
    let data = create_test_data();
    let model = ARIMA::new(1, 0, 1).unwrap();

    // Train the model
    let trained_model = model.train(&data).unwrap();

    // Forecast future values
    let forecast = trained_model.forecast(1).unwrap();

    assert_eq!(forecast.horizons(), 1);
    assert!(!forecast.values().is_empty());

    // Test confidence intervals
    let intervals = forecast.confidence_intervals(0.95).unwrap();
    assert_eq!(intervals.len(), forecast.horizons());

    for (lower, upper) in intervals {
        assert!(lower < upper);
    }
}

#[test]
fn test_forecast_result_operations() {
    let values = vec![105.0, 106.0, 107.0];
    let forecast = ForecastResult::new(values.clone(), 3).unwrap();

    assert_eq!(forecast.horizons(), 3);
    assert_eq!(forecast.values(), &values);

    // Test serialization methods
    let json = forecast.to_json().unwrap();
    assert!(!json.is_empty());

    // Test comparison methods
    let actual = vec![106.0, 107.0, 108.0];
    let error = forecast.mean_absolute_error(&actual).unwrap();
    assert_approx_eq!(error, 1.0);
}

#[test]
fn test_model_parameter_validation() {
    // Test invalid parameters
    let result = ExponentialSmoothing::new(1.5);
    assert!(result.is_err());

    let result = MovingAverage::new(0);
    assert!(result.is_err());

    let result = ARIMA::new(-1, 0, 1);
    assert!(result.is_err());
}
