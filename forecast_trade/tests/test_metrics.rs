use assert_approx_eq::assert_approx_eq;
use forecast_trade::data::TimeSeriesData;
use forecast_trade::metrics::{
    accuracy_score, f1_score, mean_absolute_error, mean_absolute_percentage_error,
    mean_squared_error, precision_score, recall_score, root_mean_squared_error,
    symmetric_mean_absolute_percentage_error,
};

#[test]
fn test_regression_metrics() {
    let actual = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    let predicted = vec![12.0, 18.0, 33.0, 37.0, 52.0];

    // Test MAE
    let mae = mean_absolute_error(&actual, &predicted);
    assert_approx_eq!(mae, 2.8, 0.01);

    // Test MSE
    let mse = mean_squared_error(&actual, &predicted);
    assert_approx_eq!(mse, 10.0, 0.01);

    // Test RMSE
    let rmse = root_mean_squared_error(&actual, &predicted);
    assert_approx_eq!(rmse, 3.16, 0.01);

    // Test MAPE
    let mape = mean_absolute_percentage_error(&actual, &predicted);
    assert!(mape > 0.0 && mape < 0.15);

    // Test SMAPE
    let smape = symmetric_mean_absolute_percentage_error(&actual, &predicted);
    assert!(smape > 0.0 && smape < 0.15);
}

#[test]
fn test_classification_metrics() {
    let actual = vec![1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0];
    let predicted = vec![1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0];

    // Test accuracy
    let accuracy = accuracy_score(&actual, &predicted);
    assert_approx_eq!(accuracy, 0.75, 0.01);

    // Test precision
    let precision = precision_score(&actual, &predicted);
    assert_approx_eq!(precision, 0.8, 0.01);

    // Test recall
    let recall = recall_score(&actual, &predicted);
    assert_approx_eq!(recall, 0.8, 0.01);

    // Test F1 score
    let f1 = f1_score(&actual, &predicted);
    assert_approx_eq!(f1, 0.8, 0.01);
}

#[test]
fn test_error_handling() {
    // Test with empty vectors
    let empty: Vec<f64> = vec![];
    let actual = vec![1.0, 2.0];

    let result = mean_absolute_error(&empty, &actual);
    assert!(result.is_nan());

    // Test with mismatched lengths
    let actual = vec![1.0, 2.0, 3.0];
    let predicted = vec![1.0, 2.0];

    let result = mean_squared_error(&actual, &predicted);
    assert!(result.is_nan());
}

#[test]
fn test_metrics_with_timeseries_data() {
    // Create two TimeSeriesData objects
    let dates1 = vec!["2023-01-01", "2023-01-02", "2023-01-03"]
        .into_iter()
        .map(|s| s.parse().unwrap())
        .collect();

    let values1 = vec![100.0, 102.0, 104.0];
    let data1 = TimeSeriesData::new(dates1, values1).unwrap();

    let dates2 = vec!["2023-01-01", "2023-01-02", "2023-01-03"]
        .into_iter()
        .map(|s| s.parse().unwrap())
        .collect();

    let values2 = vec![101.0, 103.0, 103.0];
    let data2 = TimeSeriesData::new(dates2, values2).unwrap();

    // Calculate metrics
    let mae = data1.mean_absolute_error(&data2).unwrap();
    assert_approx_eq!(mae, 1.0);

    let mse = data1.mean_squared_error(&data2).unwrap();
    assert_approx_eq!(mse, 1.33, 0.01);
}
