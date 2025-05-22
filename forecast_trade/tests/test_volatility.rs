use forecast_trade::data::TimeSeriesData;
use forecast_trade::utils::date_parser;
use forecast_trade::volatility::{
    exponential_volatility, garch_volatility, historical_volatility, parkinson_volatility,
};

fn create_test_data() -> TimeSeriesData {
    let dates = vec![
        "2023-01-01",
        "2023-01-02",
        "2023-01-03",
        "2023-01-04",
        "2023-01-05",
        "2023-01-06",
        "2023-01-07",
        "2023-01-08",
        "2023-01-09",
        "2023-01-10",
    ]
    .into_iter()
    .map(|s| date_parser::parse_date(s).unwrap())
    .collect();

    // Create a price series with increasing volatility
    let values = vec![
        100.0, 101.0, 100.5, 102.0, 100.0, 103.0, 99.0, 105.0, 98.0, 106.0,
    ];

    TimeSeriesData::new(dates, values).unwrap()
}

fn create_ohlc_data() -> TimeSeriesData {
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

    // Create high-low data for Parkinson volatility
    // Each entry has (open, high, low, close)
    let ohlc_data = vec![
        (100.0, 102.0, 98.0, 101.0),
        (101.0, 104.0, 100.0, 103.0),
        (103.0, 106.0, 101.0, 102.0),
        (102.0, 105.0, 99.0, 100.0),
        (100.0, 103.0, 97.0, 102.0),
    ];

    TimeSeriesData::new_ohlc(dates, ohlc_data).unwrap()
}

#[test]
fn test_historical_volatility() {
    let data = create_test_data();

    // Test with window size of 5
    let vol = historical_volatility(&data, 5).unwrap();

    // Should return volatility for each point after the window
    assert_eq!(vol.len(), data.len() - 5 + 1);

    // Volatility should be positive
    for v in vol {
        assert!(v > 0.0);
    }

    // Test with window size larger than data
    let result = historical_volatility(&data, 20);
    assert!(result.is_err());
}

#[test]
fn test_exponential_volatility() {
    let data = create_test_data();

    // Test with lambda = 0.94 (common value)
    let vol = exponential_volatility(&data, 0.94).unwrap();

    // Should return volatility for each point
    assert_eq!(vol.len(), data.len());

    // Volatility should be increasing (our test data has increasing volatility)
    for i in 1..vol.len() {
        if i > 5 {
            // Allow for initialization period
            assert!(vol[i] > vol[1]);
        }
    }

    // Test with invalid lambda
    let result = exponential_volatility(&data, 1.5);
    assert!(result.is_err());
}

#[test]
fn test_garch_volatility() {
    let data = create_test_data();

    // Test with standard GARCH(1,1) parameters
    let vol = garch_volatility(&data, 0.01, 0.1, 0.89).unwrap();

    // Should return volatility for each point
    assert_eq!(vol.len(), data.len());

    // Volatility should be positive
    for v in vol {
        assert!(v > 0.0);
    }

    // Test with invalid parameters (sum > 1)
    let result = garch_volatility(&data, 0.2, 0.5, 0.5);
    assert!(result.is_err());
}

#[test]
fn test_parkinson_volatility() {
    let data = create_ohlc_data();

    // Test Parkinson volatility
    let vol = parkinson_volatility(&data).unwrap();

    // Should return volatility for each point
    assert_eq!(vol.len(), data.len());

    // Volatility should be positive
    for v in vol {
        assert!(v > 0.0);
    }

    // Test with invalid data (no high-low)
    let regular_data = create_test_data();
    let result = parkinson_volatility(&regular_data);
    assert!(result.is_err());
}

#[test]
fn test_volatility_forecast() {
    let data = create_test_data();

    // Create initial volatility estimates
    let hist_vol = historical_volatility(&data, 5).unwrap();

    // Use GARCH to forecast future volatility
    let forecast =
        forecast_trade::volatility::forecast_volatility(&data, &hist_vol, 3, 0.01, 0.1, 0.89)
            .unwrap();

    // Should return forecast for requested horizon
    assert_eq!(forecast.len(), 3);

    // Forecasted volatility should be positive
    for v in forecast {
        assert!(v > 0.0);
    }
}
