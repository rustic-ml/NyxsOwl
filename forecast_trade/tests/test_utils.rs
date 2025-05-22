use forecast_trade::data::TimeSeriesData;
use forecast_trade::utils::{
    csv_serializer, date_parser, exponential_moving_average, moving_average, normalize_data,
    standardize_data,
};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

#[test]
fn test_normalize_data() {
    let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];

    // Test normalization to 0-1 range
    let normalized = normalize_data(&data).unwrap();

    assert_eq!(normalized.len(), data.len());
    assert_eq!(normalized[0], 0.0);
    assert_eq!(normalized[4], 1.0);
    assert!(normalized[2] > 0.45 && normalized[2] < 0.55);

    // Test with custom range
    let custom_normalized = normalize_data_range(&data, -1.0, 1.0).unwrap();

    assert_eq!(custom_normalized.len(), data.len());
    assert_eq!(custom_normalized[0], -1.0);
    assert_eq!(custom_normalized[4], 1.0);
}

#[test]
fn test_standardize_data() {
    let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];

    // Test standardization (z-score)
    let standardized = standardize_data(&data).unwrap();

    assert_eq!(standardized.len(), data.len());

    // Verify mean is approximately 0
    let mean: f64 = standardized.iter().sum::<f64>() / standardized.len() as f64;
    assert!(mean.abs() < 1e-10);

    // Verify standard deviation is approximately 1
    let variance: f64 =
        standardized.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / standardized.len() as f64;
    let std_dev = variance.sqrt();
    assert!((std_dev - 1.0).abs() < 1e-10);
}

#[test]
fn test_moving_average() {
    let data = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0];

    // Test simple moving average with window 3
    let ma = moving_average(&data, 3).unwrap();

    assert_eq!(ma.len(), data.len() - 2); // Window size - 1
    assert_eq!(ma[0], 20.0); // (10 + 20 + 30) / 3
    assert_eq!(ma[4], 60.0); // (50 + 60 + 70) / 3

    // Test with window larger than data
    let result = moving_average(&data, 10);
    assert!(result.is_err());
}

#[test]
fn test_exponential_moving_average() {
    let data = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0];

    // Test EMA with alpha=0.5
    let ema = exponential_moving_average(&data, 0.5).unwrap();

    assert_eq!(ema.len(), data.len());
    assert_eq!(ema[0], data[0]); // First value is the same

    // Check calculated values
    // EMA = prev_EMA + alpha * (current - prev_EMA)
    let expected_ema1 = 10.0 + 0.5 * (20.0 - 10.0); // 15.0
    assert_eq!(ema[1], expected_ema1);

    // Test with invalid alpha
    let result = exponential_moving_average(&data, 1.5);
    assert!(result.is_err());
}

#[test]
fn test_date_parser() {
    // Test various date formats
    let iso_date = "2023-01-15";
    let parsed_iso = date_parser::parse_date(iso_date).unwrap();
    assert_eq!(parsed_iso.to_string(), "2023-01-15 00:00:00 UTC");

    let us_date = "01/15/2023";
    let parsed_us = date_parser::parse_date(us_date).unwrap();
    assert_eq!(parsed_us.to_string(), "2023-01-15 00:00:00 UTC");

    let datetime = "2023-01-15T14:30:45";
    let parsed_datetime = date_parser::parse_date(datetime).unwrap();
    assert_eq!(parsed_datetime.to_string(), "2023-01-15 14:30:45 UTC");

    // Test invalid date
    let invalid_date = "not-a-date";
    let result = date_parser::parse_date(invalid_date);
    assert!(result.is_err());
}

#[test]
fn test_csv_serializer() {
    // Create test data
    let dates = vec!["2023-01-01", "2023-01-02", "2023-01-03"]
        .into_iter()
        .map(|s| date_parser::parse_date(s).unwrap())
        .collect();

    let values = vec![100.0, 102.0, 104.0];
    let data = TimeSeriesData::new(dates, values).unwrap();

    // Create a temporary file
    let mut file = NamedTempFile::new().unwrap();
    let path = file.path().to_str().unwrap();

    // Serialize to CSV
    csv_serializer::write_to_csv(&data, path).unwrap();

    // Read back and verify
    let read_data = csv_serializer::read_from_csv(path).unwrap();

    assert_eq!(read_data.len(), data.len());

    // Test with invalid path
    let result = csv_serializer::read_from_csv("/nonexistent/path.csv");
    assert!(result.is_err());
}
