//! # ARIMA Strategy Module
//!
//! Provides an ARIMA (AutoRegressive Integrated Moving Average) based trading strategy.
//!
//! This strategy utilizes ARIMA model forecasts to generate trading signals.
//! The primary implementation, `ArimaStrategy`, performs walk-forward forecasting,
//! refitting the model at each step to predict the subsequent period.
//!
//! ## Features
//! - Configurable ARIMA (p, d, q) parameters.
//! - Support for different strategy types (currently TrendFollowing).
//! - Walk-forward forecasting methodology.
//! - Robust error handling and logging.
//!
//! ## Usage
//! An `ArimaStrategy` is initialized with a `StrategyConfig` containing
//! the necessary parameters: "p", "d", "q", "threshold", and "strategy_type".
//! It requires historical price data (specifically a "close" column) to generate signals.
//!
//! **Important**: This strategy relies on the `forecast_trade` module, which is
//! conditionally compiled with the "forecasting" feature. Ensure this feature
//! is enabled in your `Cargo.toml`.

#![cfg(feature = "forecasting")]

use crate::forecasting::{
    // Signal, // Removed, will use simple_types::Signal
    Strategy,
    StrategyConfig,
    // StrategyError, // Removed, will use simple_types::NyxsOwlError
};
use crate::simple_types::{NyxsOwlError, Result as NyxsOwlResult, Signal}; // Added
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use polars::prelude::*;

/// Enum to define the type of ARIMA strategy application.
///
/// Determines how the ARIMA forecasts are translated into trading signals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArimaStrategyType {
    /// Strategy that follows the trend indicated by ARIMA forecasts.
    /// Buys if the forecast is significantly above the current price,
    /// Sells if the forecast is significantly below.
    TrendFollowing,
    // MeanReversion, // Future implementation
    // VolatilityBreakout, // Future implementation
}

impl ArimaStrategyType {
    /// Parses a string to an `ArimaStrategyType`.
    ///
    /// Accepts "trend_following" (case-insensitive).
    fn from_string(s: &str) -> NyxsOwlResult<Self> {
        // Changed to NyxsOwlResult
        match s.to_lowercase().as_str() {
            "trend_following" | "trendfollowing" => Ok(ArimaStrategyType::TrendFollowing),
            _ => Err(NyxsOwlError::InvalidParameter(format!(
                // Changed to NyxsOwlError
                "Unknown ArimaStrategyType: {}",
                s
            ))),
        }
    }
}

/// An ARIMA (AutoRegressive Integrated Moving Average) based trading strategy.
///
/// This strategy uses ARIMA forecasts to generate trading signals. It currently
/// supports a trend-following approach where signals are generated based on
/// whether the forecast price is significantly above or below the current price.
/// The forecasting is done using a walk-forward methodology, where the model
/// is refit at each step to predict the next period.
///
/// # Parameters
/// Requires the following parameters in `StrategyConfig`:
/// - `p`: The order of the autoregressive (AR) part of the ARIMA model.
/// - `d`: The degree of differencing (I) part of the ARIMA model.
/// - `q`: The order of the moving-average (MA) part of the ARIMA model.
/// - `threshold`: A float representing the percentage difference between current price
///   and forecast price required to trigger a buy or sell signal.
/// - `strategy_type`: A string specifying the strategy logic. Currently, only
///   `"trend_following"` is supported.
pub struct ArimaStrategy {
    config: StrategyConfig,
    arima_p: usize,
    arima_d: usize,
    arima_q: usize,
    threshold: f64,
    strategy_type: ArimaStrategyType,
}

impl Strategy for ArimaStrategy {
    /// Creates a new `ArimaStrategy` instance from a configuration.
    ///
    /// # Panics
    /// Panics if essential parameters (`p`, `d`, `q`, `threshold`, `strategy_type`)
    /// are missing or invalid in the provided `config`. Errors are logged before panic.
    fn new(config: StrategyConfig) -> Self {
        config
            .validate(&["p", "d", "q", "threshold", "strategy_type"])
            .unwrap_or_else(|e| {
                error!(
                    "ARIMA Strategy: Missing required configuration parameters: {}. Required: p, d, q, threshold, strategy_type.", e
                );
                panic!(
                    "ARIMA Strategy: Missing required configuration parameters: {}. Required: p, d, q, threshold, strategy_type.", e
                );
            });

        let p = config.get_int("p").map_or_else(
            |e: NyxsOwlError| {
                error!(
                    "ARIMA Strategy: Error getting 'p' parameter: {}. Assuming missing.",
                    e
                );
                panic!("ARIMA Strategy: Missing or invalid 'p' parameter in configuration.");
            },
            |val| val as usize,
        );

        let d = config.get_int("d").map_or_else(
            |e: NyxsOwlError| {
                error!(
                    "ARIMA Strategy: Error getting 'd' parameter: {}. Assuming missing.",
                    e
                );
                panic!("ARIMA Strategy: Missing or invalid 'd' parameter in configuration.");
            },
            |val| val as usize,
        );

        let q = config.get_int("q").map_or_else(
            |e: NyxsOwlError| {
                error!(
                    "ARIMA Strategy: Error getting 'q' parameter: {}. Assuming missing.",
                    e
                );
                panic!("ARIMA Strategy: Missing or invalid 'q' parameter in configuration.");
            },
            |val| val as usize,
        );

        let threshold = config
            .get_float("threshold")
            .unwrap_or_else(|e: NyxsOwlError| {
                error!(
                    "ARIMA Strategy: Error getting 'threshold' parameter: {}. Assuming missing.",
                    e
                );
                panic!(
                    "ARIMA Strategy: Missing or invalid 'threshold' parameter in configuration."
                );
            });

        let strategy_type_str = config
            .get_string("strategy_type")
            .unwrap_or_else(|e: NyxsOwlError| {
                error!("ARIMA Strategy: Error getting 'strategy_type' parameter: {}. Assuming missing.", e);
                panic!("ARIMA Strategy: Missing or invalid 'strategy_type' parameter in configuration.");
            });

        let strategy_type =
            ArimaStrategyType::from_string(strategy_type_str).unwrap_or_else(|e: NyxsOwlError| {
                error!(
                    "ARIMA Strategy: Invalid 'strategy_type' configured: {}. Error: {}",
                    strategy_type_str, e
                );
                panic!(
                    "ARIMA Strategy: Invalid 'strategy_type' configured: {}. Error: {}",
                    strategy_type_str, e
                );
            });

        info!(
            "ARIMA Strategy initialized with p={}, d={}, q={}, threshold={}, strategy_type={:?}",
            p, d, q, threshold, strategy_type
        );

        ArimaStrategy {
            config: config.clone(),
            arima_p: p,
            arima_d: d,
            arima_q: q,
            threshold,
            strategy_type,
        }
    }

    fn generate_signals(&self, data: &DataFrame) -> NyxsOwlResult<Series> {
        // Changed to NyxsOwlResult
        self.validate_data(data)?; // This now returns NyxsOwlResult
        info!(
            "ARIMA Strategy: Generating signals for {} data points.",
            data.height()
        );

        let close_prices_ca = data.column("close")?.f64()?;
        let timestamps_ca = data.column("timestamp")?.datetime()?;

        let forecasts_series = self.get_arima_forecasts(timestamps_ca, close_prices_ca)?; // Assuming this will also return NyxsOwlResult
        debug!(
            "ARIMA Strategy: Forecasts series obtained: {:?}",
            forecasts_series
        );
        let forecast_prices_ca = forecasts_series.f64()?;

        if close_prices_ca.len() != forecast_prices_ca.len() {
            error!(
                "ARIMA Strategy: Forecast series length ({}) does not match price series length ({}).",
                forecast_prices_ca.len(),
                close_prices_ca.len()
            );
            return Err(NyxsOwlError::StrategyError(
                // Changed to NyxsOwlError::StrategyError
                "Forecast series length does not match price series length.".to_string(),
            ));
        }

        let mut signals = Vec::with_capacity(close_prices_ca.len());

        match self.strategy_type {
            ArimaStrategyType::TrendFollowing => {
                for i in 0..close_prices_ca.len() {
                    if let (Some(current_price), Some(forecast_price)) =
                        (close_prices_ca.get(i), forecast_prices_ca.get(i))
                    {
                        if forecast_price > current_price * (1.0 + self.threshold) {
                            signals.push(Signal::Buy); // Using simple_types::Signal
                        } else if forecast_price < current_price * (1.0 - self.threshold) {
                            signals.push(Signal::Sell); // Using simple_types::Signal
                        } else {
                            signals.push(Signal::Hold); // Using simple_types::Signal
                        }
                    } else {
                        if i >= self.min_data_points() {
                            debug!(
                                "ARIMA Strategy: Holding signal at index {} due to missing current price or forecast. Current: {:?}, Forecast: {:?}.",
                                i, close_prices_ca.get(i), forecast_prices_ca.get(i)
                            );
                        }
                        signals.push(Signal::Hold); // Using simple_types::Signal
                    }
                }
            }
        }

        Ok(Series::new(
            "signal".into(),
            signals
                .into_iter()
                .map(|s| s as i32) // Changed from s.to_int() to s as i32
                .collect::<Vec<i32>>(),
        ))
    }

    fn name(&self) -> &str {
        "ARIMA Strategy"
    }

    fn description(&self) -> &str {
        "A strategy based on ARIMA model forecasts (walk-forward). Supports Trend Following."
    }

    fn required_columns(&self) -> Vec<&str> {
        vec!["timestamp", "close"] // Re-added "timestamp"
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        (self.arima_p + self.arima_d + self.arima_q + 20).max(60)
    }
}

impl ArimaStrategy {
    fn get_arima_forecasts(
        &self,
        timestamps_ca: &DatetimeChunked,
        close_prices_ca: &Float64Chunked,
    ) -> NyxsOwlResult<Series> {
        // Changed to NyxsOwlResult
        let data_values_options_vec: Vec<Option<f64>> = close_prices_ca.into_iter().collect();

        let timestamps_options_vec: Vec<Option<i64>> =
            timestamps_ca.into_iter().collect();

        let series_len = data_values_options_vec.len();
        if series_len == 0 {
            warn!("ARIMA Strategy: Input data for forecasts is empty. Returning empty forecast series.");
            return Ok(Series::new_empty("forecast".into(), &DataType::Float64));
        }
        if series_len != timestamps_options_vec.len() {
            error!("ARIMA Strategy: Mismatch in processed timestamp and close price lengths internally.");
            return Err(NyxsOwlError::StrategyError(
                // Corrected from ValidationError
                "Timestamp and close price vector length mismatch after initial processing."
                    .to_string(),
            ));
        }

        let mut historical_forecasts: Vec<Option<f64>> = vec![None; series_len];
        info!(
            "ARIMA Strategy: Starting walk-forward forecasting for series of length {}. ARIMA(p={}, d={}, q={})",
            series_len, self.arima_p, self.arima_d, self.arima_q
        );

        let initial_fit_window_size = self.min_data_points();
        // oxidiviner::quick::arima_forecast_custom creates TimeSeriesData internally, which needs at least 1 point.
        // The actual model fitting (ARIMAModel::fit) will have more robust requirements based on p,d,q and data length.
        let min_dense_values_for_arima =
            (self.arima_p.max(1) + self.arima_q.max(1) + self.arima_d + 5).max(10);

        if series_len < initial_fit_window_size {
            warn!(
                "ARIMA Strategy: Series length ({}) is less than initial_fit_window_size ({}). No forecasts will be generated.",
                series_len, initial_fit_window_size
            );
            return Ok(Series::new("forecast".into(), historical_forecasts));
        }

        for i in initial_fit_window_size..series_len {
            // Prepare data for the current walk-forward step
            let mut current_timestamps_for_fit: Vec<DateTime<Utc>> = Vec::new();
            let mut current_values_for_fit: Vec<f64> = Vec::new();

            // Collect only non-missing pairs up to index i (exclusive for current observation)
            for k in 0..i {
                if let (Some(ts_opt), Some(val_opt)) = (
                    timestamps_options_vec.get(k),
                    data_values_options_vec.get(k),
                ) {
                    if let (Some(ts), Some(val)) = (ts_opt, val_opt) {
                        // Convert i64 milliseconds to DateTime<Utc>
                        if let Some(dt) = chrono::DateTime::from_timestamp_millis(*ts) {
                            current_timestamps_for_fit.push(dt);
                            current_values_for_fit.push(*val);
                        }
                    }
                }
            }

            if current_values_for_fit.len() < min_dense_values_for_arima {
                warn!(
                    "ARIMA Strategy: Not enough dense data points ({} available, {} required) in window [0..{}] to fit ARIMA for forecast at index {}. Skipping forecast.",
                    current_values_for_fit.len(), min_dense_values_for_arima, i, i
                );
                continue;
            }
            debug!(
                "ARIMA Strategy: Fitting ARIMA for forecast at index {} using {} dense data points from window [0..{}].",
                i, current_values_for_fit.len(), i
            );

            match crate::forecasting::forecast_trade::forecast_arima(
                current_timestamps_for_fit, // Vec<DateTime<Utc>>
                current_values_for_fit,     // Vec<f64>
                (self.arima_p, self.arima_d, self.arima_q),
                1, // Forecast 1 period ahead
            ) {
                Ok(forecast_vec) => {
                    if !forecast_vec.is_empty() {
                        historical_forecasts[i] = Some(forecast_vec[0]);
                        debug!(
                            "ARIMA Strategy: Forecast for index {}: {}",
                            i, forecast_vec[0]
                        );
                    } else {
                        warn!(
                            "ARIMA Strategy: Forecast for index {} returned empty. Setting to None.", i
                        );
                        historical_forecasts[i] = None;
                    }
                }
                Err(e) => {
                    warn!(
                        "ARIMA Strategy: Walk-forward ARIMA forecast failed at index {} with error: {:?}. Setting forecast to None.",
                        i, e
                    );
                    historical_forecasts[i] = None;
                }
            }
        }
        info!(
            "ARIMA Strategy: Walk-forward forecasting completed. Generated {} potential forecasts.",
            historical_forecasts.iter().filter(|f| f.is_some()).count()
        );

        Ok(Series::new("forecast".into(), historical_forecasts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use polars::prelude::{
        DataFrame, DataType, NamedFrom, Series, TimeUnit,
    };

    // Updated to include a timestamp column
    fn create_test_data(len: usize) -> DataFrame {
        let start_naive_dt =
            NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let timestamps_ms: Vec<Option<i64>> = (0..len)
            .map(|i| {
                if i % 15 == 0 && i > 0 {
                    // Introduce some None timestamps to test robustness
                    None
                } else {
                    Some(
                        start_naive_dt.and_utc().timestamp_millis()
                            + (i as i64 * 24 * 60 * 60 * 1000),
                    ) // daily interval
                }
            })
            .collect();

        let close_values: Vec<Option<f64>> = (0..len)
            .map(|i| {
                if i % 10 == 0 && i > 0 {
                    // Introduce some Nones for close values
                    None
                } else {
                    Some(100.0 + (i as f64 * 0.1) - ((i % 10) as f64 * 0.05))
                }
            })
            .collect();

        polars::df![
            "timestamp" => Series::new("timestamp".into(), timestamps_ms).cast(&DataType::Datetime(TimeUnit::Milliseconds, Some("UTC".into()))).unwrap(),
            "close" => Series::new("close".into(), close_values)
        ]
        .unwrap_or_else(|e| panic!("Failed to create test DataFrame: {}", e))
    }

    fn create_config(
        p: i64,
        d: i64,
        q: i64,
        threshold: f64,
        strategy_type: &str,
    ) -> StrategyConfig {
        StrategyConfig::new()
            .with_parameter("p", p)
            .with_parameter("d", d)
            .with_parameter("q", q)
            .with_parameter("threshold", threshold)
            .with_parameter("strategy_type", strategy_type.to_string())
    }

    #[test]
    fn test_arima_strategy_new() {
        let config = create_config(5, 1, 0, 0.01, "trend_following");
        let strategy = ArimaStrategy::new(config.clone());
        assert_eq!(strategy.name(), "ARIMA Strategy");
        assert_eq!(strategy.config().get_int("p").unwrap(), 5);
        assert_eq!(strategy.arima_p, 5);
        assert_eq!(strategy.arima_d, 1);
        assert_eq!(strategy.arima_q, 0);
        assert_eq!(strategy.threshold, 0.01);
        // Check strategy_type comparison if ArimaStrategyType derives PartialEq
        // assert_eq!(strategy.strategy_type, ArimaStrategyType::TrendFollowing);
        assert_eq!(strategy.min_data_points(), 60);
        assert!(strategy.required_columns().contains(&"timestamp")); // Check timestamp IS required
        assert!(strategy.required_columns().contains(&"close"));
    }

    #[test]
    #[should_panic]
    fn test_arima_strategy_new_missing_params() {
        let config = StrategyConfig::new().with_parameter("p", 5i64);
        ArimaStrategy::new(config);
    }

    #[test]
    #[should_panic]
    fn test_arima_strategy_new_invalid_type() {
        let config = create_config(5, 1, 0, 0.01, "invalid_type");
        ArimaStrategy::new(config);
    }

    #[test]
    fn test_arima_strategy_generate_signals_walk_forward_logic() {
        let p = 1;
        let d = 0;
        let q = 0;
        let config = create_config(p, d, q, 0.01, "trend_following");
        let strategy = ArimaStrategy::new(config);
        let min_points = strategy.min_data_points();

        let data_too_short = create_test_data(min_points - 1);
        match strategy.generate_signals(&data_too_short) {
            Ok(signals) => {
                assert_eq!(signals.len(), data_too_short.height());
                let all_hold = signals
                    .i32()
                    .unwrap()
                    .into_iter()
                    .all(|s_opt| s_opt.is_some_and(|s| s == Signal::Hold.to_int()));
                assert!(
                    all_hold,
                    "Expected all Hold signals for data too short ({}) for walk-forward (min_points: {}). Signals: {:?}", 
                    data_too_short.height(), min_points, signals
                );
            }
            Err(crate::forecasting::NyxsOwlError::StrategyError(msg)) => {
                // Expecting strategy error due to min_data_points check in validate_data
                assert!(msg.contains(&format!(
                    "requires at least {} data points, but got",
                    min_points
                )));
            }
            Err(e) => {
                panic!(
                "generate_signals failed unexpectedly for too_short data ({} points, min {}): {:?}",
                 data_too_short.height(), min_points, e
            )
            }
        }

        // Test with data exactly at min_data_points. Should produce all Hold because loop for forecasts won't run.
        let data_just_enough = create_test_data(min_points);
        match strategy.generate_signals(&data_just_enough) {
            Ok(signals) => {
                assert_eq!(signals.len(), data_just_enough.height());
                let all_hold = signals
                    .i32()
                    .unwrap()
                    .into_iter()
                    .all(|s_opt| s_opt.is_some_and(|s| s == Signal::Hold.to_int()));
                assert!(all_hold, "Expected all Hold signals when data length ({}) equals min_data_points ({}). Forecast loop doesn't run. Signals: {:?}", data_just_enough.height(), min_points, signals);
            }
            Err(e) => panic!(
                "generate_signals failed for just_enough data ({} points, min {}): {:?}",
                data_just_enough.height(),
                min_points,
                e
            ),
        }

        // Test with sufficient data to run the forecast loop at least once
        let data_sufficient = create_test_data(min_points + 1);
        match strategy.generate_signals(&data_sufficient) {
            Ok(signals) => {
                assert_eq!(signals.len(), data_sufficient.height());
                // Actual signals depend on OxiDiviner's output, which is mocked here by being None for first few points
                // For a real test, we'd need more predictable mock forecasts or to inspect the None pattern for initial points
                println!(
                    "Signals for {} data points (min_points {}): {:?}",
                    data_sufficient.height(),
                    min_points,
                    signals
                );
                // Example check: the first `min_points` should be Hold
                for i in 0..min_points {
                    assert_eq!(
                        signals
                            .i32()
                            .unwrap()
                            .get(i)
                            .unwrap_or(Signal::Buy.to_int()),
                        Signal::Hold.to_int(),
                        "Signal at index {} was not Hold",
                        i
                    );
                }
                // The last point might have a non-Hold signal if forecast was generated
                // This is hard to assert without knowing mock forecast behavior from OxiDiviner
            }
            Err(e) => panic!(
                "generate_signals failed for sufficient data ({} points, min {}): {:?}",
                data_sufficient.height(),
                min_points,
                e
            ),
        }
    }

    // MockArimaStrategy for testing signal generation logic independently of actual ARIMA calls
    struct MockArimaStrategy {
        config: StrategyConfig,
        arima_p: usize,
        arima_d: usize,
        arima_q: usize,
        threshold: f64,
        strategy_type: ArimaStrategyType,
        mock_forecasts: Series, // This will be a series of Option<f64>
    }

    impl Strategy for MockArimaStrategy {
        fn new(config: StrategyConfig) -> Self {
            let p = config.get_int("p").expect("Missing 'p' parameter") as usize;
            let d = config.get_int("d").expect("Missing 'd' parameter") as usize;
            let q = config.get_int("q").expect("Missing 'q' parameter") as usize;
            let threshold = config
                .get_float("threshold")
                .expect("Missing 'threshold' parameter");
            let strategy_type_str = config
                .get_string("strategy_type")
                .expect("Missing 'strategy_type' parameter");
            let strategy_type = ArimaStrategyType::from_string(strategy_type_str).unwrap();
            MockArimaStrategy {
                config: config.clone(),
                arima_p: p,
                arima_d: d,
                arima_q: q,
                threshold,
                strategy_type,
                // Initialize with an empty or correctly typed placeholder forecast series
                mock_forecasts: Series::new_empty("empty_mock_forecast".into(), &DataType::Float64),
            }
        }

        fn generate_signals(
            &self,
            data: &DataFrame,
        ) -> Result<Series, crate::forecasting::NyxsOwlError> {
            self.validate_data(data)?;
            let close_prices_ca = data.column("close")?.f64()?;
            // Timestamps are present in data but not explicitly used by MockArimaStrategy's signal logic directly,
            // as it uses self.mock_forecasts.
            // let _timestamps_ca = data.column("timestamp")?.datetime()?;

            let forecast_prices_ca = self.mock_forecasts.f64()?;

            if close_prices_ca.len() != forecast_prices_ca.len() {
                return Err(crate::forecasting::NyxsOwlError::IndicatorError(
                    format!("Mock: Forecast series length ({}) does not match price series length ({}).", forecast_prices_ca.len(), close_prices_ca.len()).to_string(),
                ));
            }

            let mut signals = Vec::with_capacity(close_prices_ca.len());
            match self.strategy_type {
                ArimaStrategyType::TrendFollowing => {
                    for i in 0..close_prices_ca.len() {
                        if let (Some(current_price), Some(forecast_price)) =
                            (close_prices_ca.get(i), forecast_prices_ca.get(i))
                        {
                            if forecast_price > current_price * (1.0 + self.threshold) {
                                signals.push(Signal::Buy);
                            } else if forecast_price < current_price * (1.0 - self.threshold) {
                                signals.push(Signal::Sell);
                            } else {
                                signals.push(Signal::Hold);
                            }
                        } else {
                            signals.push(Signal::Hold); // Hold if current price or forecast is None
                        }
                    }
                }
            }
            Ok(Series::new(
                "signal".into(),
                signals
                    .into_iter()
                    .map(|s| s.to_int())
                    .collect::<Vec<i32>>(),
            ))
        }

        fn name(&self) -> &str {
            "Mock ARIMA Strategy"
        }
        fn description(&self) -> &str {
            "Mock for testing ARIMA logic (bypasses internal forecast generation)"
        }
        fn required_columns(&self) -> Vec<&str> {
            vec!["timestamp", "close"] // Mock now also requires timestamp for validate_data consistency
        }
        fn config(&self) -> &StrategyConfig {
            &self.config
        }
        fn min_data_points(&self) -> usize {
            // Mock can have a simpler min_data_points for testing
            (self.arima_p + self.arima_d + self.arima_q + 1).max(2)
        }
    }

    #[test]
    fn test_trend_following_logic_with_mock_forecasts() {
        let config = create_config(1, 0, 0, 0.01, "trend_following");

        // Create test data with timestamps and close prices
        let n_rows = 5;
        let start_naive_dt =
            NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let timestamps_ms: Vec<i64> = (0..n_rows)
            .map(|i| start_naive_dt.and_utc().timestamp_millis() + (i as i64 * 24 * 60 * 60 * 1000))
            .collect();
        let close_prices_options: Vec<Option<f64>> = vec![
            Some(100.0),
            Some(100.05),
            Some(100.1),
            Some(100.15),
            Some(100.2),
        ];
        let data =
            polars::df![
                "timestamp" => Series::new("timestamp".into(), timestamps_ms).cast(&DataType::Datetime(TimeUnit::Milliseconds, Some("UTC".into()))).unwrap(),
                "close" => Series::new("close".into(), close_prices_options)
            ].unwrap();

        // Test Buy signal
        let forecasts_buy_options: Vec<Option<f64>> = vec![
            Some(100.0),
            Some(102.0),
            Some(100.1),
            Some(100.15),
            Some(100.2),
        ];
        let forecasts_buy = Series::new("forecast".into(), forecasts_buy_options);
        let mut strategy_buy = MockArimaStrategy::new(config.clone());
        strategy_buy.mock_forecasts = forecasts_buy;
        let signals_buy = strategy_buy.generate_signals(&data).unwrap();
        assert_eq!(
            signals_buy.i32().unwrap().get(1).unwrap_or_default(),
            Signal::Buy.to_int()
        );

        // Test Sell signal
        let forecasts_sell_options: Vec<Option<f64>> = vec![
            Some(100.0),
            Some(98.0),
            Some(100.1),
            Some(100.15),
            Some(100.2),
        ];
        let forecasts_sell = Series::new("forecast".into(), forecasts_sell_options);
        let mut strategy_sell = MockArimaStrategy::new(config.clone());
        strategy_sell.mock_forecasts = forecasts_sell;
        let signals_sell = strategy_sell.generate_signals(&data).unwrap();
        assert_eq!(
            signals_sell.i32().unwrap().get(1).unwrap_or_default(),
            Signal::Sell.to_int()
        );

        // Test Hold signal
        let forecasts_hold_options: Vec<Option<f64>> = vec![
            Some(100.0),
            Some(100.06),
            Some(100.1),
            Some(100.15),
            Some(100.2),
        ];
        let forecasts_hold = Series::new("forecast".into(), forecasts_hold_options);
        let mut strategy_hold = MockArimaStrategy::new(config.clone());
        strategy_hold.mock_forecasts = forecasts_hold;
        let signals_hold = strategy_hold.generate_signals(&data).unwrap();
        assert_eq!(
            signals_hold.i32().unwrap().get(1).unwrap_or_default(),
            Signal::Hold.to_int()
        );
    }

    #[test]
    fn test_arima_required_columns_validation() {
        let config = create_config(1, 0, 0, 0.01, "trend_following");
        let strategy = ArimaStrategy::new(config);
        let correct_data = create_test_data(strategy.min_data_points());
        assert!(strategy.validate_data(&correct_data).is_ok());

        let incorrect_data_no_close = polars::df!["timestamp" => Series::new("timestamp".into(), vec![0i64]).cast(&DataType::Datetime(TimeUnit::Milliseconds, None)).unwrap()].unwrap();
        match strategy.validate_data(&incorrect_data_no_close) {
            Err(crate::forecasting::NyxsOwlError::MissingData(msg)) => {
                assert!(msg.contains("Column 'close' not found"));
            }
            _ => panic!("Expected MissingData error for 'close'"),
        }

        let incorrect_data_no_timestamp =
            polars::df!["close" => Series::new("close".into(), vec![Some(1.0), Some(2.0)])]
                .unwrap();
        match strategy.validate_data(&incorrect_data_no_timestamp) {
            Err(crate::forecasting::NyxsOwlError::MissingData(msg)) => {
                assert!(msg.contains("Column 'timestamp' not found"));
            }
            _ => panic!("Expected MissingData error for 'timestamp'"),
        }
    }

    #[test]
    fn test_arima_min_data_points_validation() {
        let config = create_config(1, 0, 0, 0.01, "trend_following");
        let strategy = ArimaStrategy::new(config);
        let min_points = strategy.min_data_points();

        let insufficient_data = create_test_data(min_points - 1);
        match strategy.validate_data(&insufficient_data) {
            Err(crate::forecasting::NyxsOwlError::StrategyError(msg)) => {
                assert!(msg.contains(&format!(
                    "requires at least {} data points, but got {}",
                    min_points,
                    min_points - 1
                )));
            }
            _ => panic!(
                "Expected StrategyError for insufficient data ({} rows, min {})",
                min_points - 1,
                min_points
            ),
        }

        let sufficient_data = create_test_data(min_points);
        assert!(strategy.validate_data(&sufficient_data).is_ok());
    }
}
