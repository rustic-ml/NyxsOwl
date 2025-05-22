use forecast_trade::data::{DataLoader, TimeSeriesData};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

#[test]
fn test_data_loader_from_csv() {
    // Create a temporary CSV file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "date,open,high,low,close,volume").unwrap();
    writeln!(file, "2023-01-01,100.0,105.0,98.0,103.0,1000").unwrap();
    writeln!(file, "2023-01-02,103.0,107.0,101.0,106.0,1200").unwrap();
    writeln!(file, "2023-01-03,106.0,110.0,104.0,108.0,1500").unwrap();

    let path = file.path().to_str().unwrap();
    let data = DataLoader::from_csv(path).unwrap();

    assert_eq!(data.len(), 3);
    assert!(!data.is_empty());
}

#[test]
fn test_time_series_data_operations() {
    // Create test data
    let dates = vec!["2023-01-01", "2023-01-02", "2023-01-03"]
        .into_iter()
        .map(|s| s.parse().unwrap())
        .collect();

    let values = vec![100.0, 103.0, 106.0];

    let data = TimeSeriesData::new(dates, values).unwrap();

    // Test operations
    assert_eq!(data.len(), 3);
    assert!(!data.is_empty());

    // Test slicing
    let subset = data.slice(1, 3).unwrap();
    assert_eq!(subset.len(), 2);

    // Test statistical methods
    let mean = data.mean().unwrap();
    assert!(mean > 102.0 && mean < 104.0);

    let std_dev = data.std_dev().unwrap();
    assert!(std_dev > 2.0 && std_dev < 4.0);
}

#[test]
fn test_data_loader_error_handling() {
    // Test with non-existent file
    let result = DataLoader::from_csv("nonexistent_file.csv");
    assert!(result.is_err());

    // Test with invalid data format
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "invalid,csv,format").unwrap();

    let path = file.path().to_str().unwrap();
    let result = DataLoader::from_csv(path);
    assert!(result.is_err());
}
