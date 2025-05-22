use forecast_trade::error::ForecastError;
use std::io;
use std::path::Path;

#[test]
fn test_error_conversion() {
    // Test IO error conversion
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let forecast_error = ForecastError::from(io_error);

    match forecast_error {
        ForecastError::IoError(_) => assert!(true),
        _ => panic!("Expected IoError variant"),
    }

    // Test parse error
    let parse_error = "invalid data".parse::<i32>().unwrap_err();
    let forecast_error = ForecastError::from(parse_error);

    match forecast_error {
        ForecastError::ParseError(_) => assert!(true),
        _ => panic!("Expected ParseError variant"),
    }
}

#[test]
fn test_error_display() {
    // Test display implementation
    let error = ForecastError::InvalidParameter("alpha must be between 0 and 1".to_string());
    let error_string = format!("{}", error);

    assert!(error_string.contains("alpha must be between 0 and 1"));

    // Test with source error
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
    let error = ForecastError::from(io_error);
    let error_string = format!("{}", error);

    assert!(error_string.contains("IO error"));
    assert!(error_string.contains("permission denied"));
}

#[test]
fn test_error_creation() {
    // Test creating different error types
    let data_error = ForecastError::DataError("Empty time series".to_string());
    let model_error = ForecastError::ModelError("Failed to converge".to_string());
    let parameter_error = ForecastError::InvalidParameter("Invalid window size".to_string());

    // Verify they are different types
    assert!(matches!(data_error, ForecastError::DataError(_)));
    assert!(matches!(model_error, ForecastError::ModelError(_)));
    assert!(matches!(
        parameter_error,
        ForecastError::InvalidParameter(_)
    ));

    // Test extracting error messages
    if let ForecastError::DataError(msg) = data_error {
        assert_eq!(msg, "Empty time series");
    } else {
        panic!("Wrong error variant");
    }
}

#[test]
fn test_result_mapping() {
    // Test using map_err with Result
    let result: Result<(), &str> = Err("test error");
    let mapped = result.map_err(|e| ForecastError::Other(e.to_string()));

    assert!(mapped.is_err());
    if let Err(ForecastError::Other(msg)) = mapped {
        assert_eq!(msg, "test error");
    } else {
        panic!("Wrong error variant");
    }

    // Test with a simulated file operation
    let file_result = Path::new("/nonexistent/path").try_exists();
    let mapped = file_result.map_err(ForecastError::from);

    assert!(mapped.is_err());
}
