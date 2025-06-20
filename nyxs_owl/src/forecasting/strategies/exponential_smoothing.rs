use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for Exponential Smoothing forecasting strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExponentialSmoothingConfig {
    /// Alpha parameter for level smoothing (0.0 to 1.0)
    pub alpha: f64,
    /// Beta parameter for trend smoothing (0.0 to 1.0, None for simple exponential smoothing)
    pub beta: Option<f64>,
    /// Gamma parameter for seasonal smoothing (0.0 to 1.0, None for non-seasonal)
    pub gamma: Option<f64>,
    /// Number of seasonal periods (e.g., 12 for monthly data with yearly seasonality)
    pub seasonal_periods: Option<usize>,
    /// Forecast horizon (number of periods ahead to predict)
    pub forecast_horizon: usize,
    /// Minimum percentage change threshold to generate signals
    pub threshold: f64,
    /// Minimum number of data points required for forecasting
    pub min_data_points: usize,
    /// Rolling window size for generating forecasts
    pub window_size: usize,
}

impl Default for ExponentialSmoothingConfig {
    fn default() -> Self {
        Self {
            alpha: 0.3,
            beta: Some(0.1),
            gamma: None,
            seasonal_periods: None,
            forecast_horizon: 1,
            threshold: 0.02, // 2%
            min_data_points: 20,
            window_size: 50,
        }
    }
}

impl ExponentialSmoothingConfig {
    /// Create a new configuration with validation
    ///
    /// # Arguments
    /// * `alpha` - Level smoothing parameter (0.0 to 1.0)
    /// * `beta` - Trend smoothing parameter (0.0 to 1.0, None for simple smoothing)
    /// * `gamma` - Seasonal smoothing parameter (0.0 to 1.0, None for non-seasonal)
    /// * `seasonal_periods` - Number of seasonal periods (e.g., 12 for monthly data)
    /// * `forecast_horizon` - Number of periods ahead to predict
    /// * `threshold` - Minimum percentage change threshold for signals
    /// * `min_data_points` - Minimum data points required
    /// * `window_size` - Rolling window size for forecasts
    ///
    /// # Returns
    /// A new `ExponentialSmoothingConfig` instance or an error if parameters are invalid
    pub fn new(
        alpha: f64,
        beta: Option<f64>,
        gamma: Option<f64>,
        seasonal_periods: Option<usize>,
        forecast_horizon: usize,
        threshold: f64,
        min_data_points: usize,
        window_size: usize,
    ) -> Result<Self> {
        // Validate alpha
        if !(0.0..=1.0).contains(&alpha) {
            return Err(NyxsOwlError::InvalidParameter(
                "Alpha must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Validate beta if provided
        if let Some(b) = beta {
            if !(0.0..=1.0).contains(&b) {
                return Err(NyxsOwlError::InvalidParameter(
                    "Beta must be between 0.0 and 1.0".to_string(),
                ));
            }
        }

        // Validate gamma if provided
        if let Some(g) = gamma {
            if !(0.0..=1.0).contains(&g) {
                return Err(NyxsOwlError::InvalidParameter(
                    "Gamma must be between 0.0 and 1.0".to_string(),
                ));
            }
        }

        // Validate seasonal periods
        if let Some(sp) = seasonal_periods {
            if sp < 2 {
                return Err(NyxsOwlError::InvalidParameter(
                    "Seasonal periods must be at least 2".to_string(),
                ));
            }
        }

        // Validate other parameters
        if forecast_horizon == 0 {
            return Err(NyxsOwlError::InvalidParameter(
                "Forecast horizon must be greater than 0".to_string(),
            ));
        }

        if threshold < 0.0 {
            return Err(NyxsOwlError::InvalidParameter(
                "Threshold must be non-negative".to_string(),
            ));
        }

        if min_data_points < 10 {
            return Err(NyxsOwlError::InvalidParameter(
                "Minimum data points must be at least 10".to_string(),
            ));
        }

        if window_size < min_data_points {
            return Err(NyxsOwlError::InvalidParameter(
                "Window size must be at least as large as minimum data points".to_string(),
            ));
        }

        Ok(Self {
            alpha,
            beta,
            gamma,
            seasonal_periods,
            forecast_horizon,
            threshold,
            min_data_points,
            window_size,
        })
    }

    /// Create conservative configuration (lower alpha, no trend/seasonality)
    ///
    /// Returns a conservative configuration with lower alpha values and higher thresholds
    pub fn conservative() -> Self {
        Self {
            alpha: 0.1,
            beta: None,
            gamma: None,
            seasonal_periods: None,
            forecast_horizon: 1,
            threshold: 0.03, // 3%
            min_data_points: 30,
            window_size: 100,
        }
    }

    /// Create moderate configuration (with trend but no seasonality)
    ///
    /// Returns a moderate configuration with balanced parameters
    pub fn moderate() -> Self {
        Self {
            alpha: 0.3,
            beta: Some(0.1),
            gamma: None,
            seasonal_periods: None,
            forecast_horizon: 1,
            threshold: 0.02, // 2%
            min_data_points: 20,
            window_size: 50,
        }
    }

    /// Create aggressive configuration (higher alpha, with trend)
    ///
    /// Returns an aggressive configuration with higher alpha values and lower thresholds
    pub fn aggressive() -> Self {
        Self {
            alpha: 0.5,
            beta: Some(0.3),
            gamma: None,
            seasonal_periods: None,
            forecast_horizon: 1,
            threshold: 0.015, // 1.5%
            min_data_points: 15,
            window_size: 30,
        }
    }

    /// Create seasonal configuration (with trend and seasonality)
    ///
    /// # Arguments
    /// * `seasonal_periods` - Number of seasonal periods (e.g., 12 for monthly data)
    ///
    /// # Returns
    /// A seasonal configuration with trend and seasonality parameters
    pub fn seasonal(seasonal_periods: usize) -> Result<Self> {
        Self::new(
            0.3,
            Some(0.1),
            Some(0.1),
            Some(seasonal_periods),
            1,
            0.02,
            seasonal_periods * 3, // Need at least 3 full seasons
            seasonal_periods * 5,
        )
    }
}

/// Exponential Smoothing forecasting strategy
///
/// This strategy uses exponential smoothing methods to forecast future prices
/// and generate trading signals based on the forecasted values.
pub struct ExponentialSmoothingStrategy {
    config: ExponentialSmoothingConfig,
}

impl ExponentialSmoothingStrategy {
    /// Create a new Exponential Smoothing strategy
    ///
    /// # Arguments
    /// * `config` - Configuration for the exponential smoothing strategy
    pub fn new(config: ExponentialSmoothingConfig) -> Self {
        Self { config }
    }

    /// Generate trading signals based on Exponential Smoothing forecasts
    ///
    /// # Arguments
    /// * `df` - Input DataFrame containing price and timestamp columns
    /// * `price_column` - Name of the price column
    /// * `timestamp_column` - Name of the timestamp column
    ///
    /// # Returns
    /// A vector of trading signals (`Signal`) for each row in the DataFrame
    pub fn generate_signals(
        &self,
        df: &DataFrame,
        price_column: &str,
        timestamp_column: &str,
    ) -> Result<Vec<Signal>> {
        // Validate inputs
        self.validate_inputs(df, price_column, timestamp_column)?;

        // Extract price data
        let prices = self.extract_prices(df, price_column)?;
        let timestamps = self.extract_timestamps(df, timestamp_column)?;

        // Generate forecasts using rolling window approach
        let signals = self.generate_rolling_forecasts(&prices, &timestamps)?;

        Ok(signals)
    }

    // Private helper methods
    fn validate_inputs(&self, df: &DataFrame, price_col: &str, timestamp_col: &str) -> Result<()> {
        if df.height() < self.config.min_data_points {
            return Err(NyxsOwlError::MissingData(format!(
                "Insufficient data: {} rows, need at least {}",
                df.height(),
                self.config.min_data_points
            )));
        }

        // Validate columns exist
        df.column(price_col).map_err(|e| {
            NyxsOwlError::DataError(format!("Price column '{}' not found: {}", price_col, e))
        })?;

        df.column(timestamp_col).map_err(|e| {
            NyxsOwlError::DataError(format!(
                "Timestamp column '{}' not found: {}",
                timestamp_col, e
            ))
        })?;

        Ok(())
    }

    fn extract_prices(&self, df: &DataFrame, price_col: &str) -> Result<Vec<f64>> {
        let price_series = df
            .column(price_col)
            .map_err(|e| NyxsOwlError::DataError(format!("Failed to get price column: {}", e)))?;

        match price_series.dtype() {
            DataType::Float64 => {
                let prices: Vec<f64> = price_series
                    .f64()
                    .map_err(|e| NyxsOwlError::DataError(format!("Failed to cast to f64: {}", e)))?
                    .into_no_null_iter()
                    .collect();
                Ok(prices)
            }
            DataType::Float32 => {
                let prices: Vec<f64> = price_series
                    .f32()
                    .map_err(|e| NyxsOwlError::DataError(format!("Failed to cast to f32: {}", e)))?
                    .into_no_null_iter()
                    .map(|x| x as f64)
                    .collect();
                Ok(prices)
            }
            _ => Err(NyxsOwlError::DataError(format!(
                "Price column must be numeric, found: {:?}",
                price_series.dtype()
            ))),
        }
    }

    fn extract_timestamps(&self, df: &DataFrame, timestamp_col: &str) -> Result<Vec<String>> {
        let timestamp_series = df.column(timestamp_col).map_err(|e| {
            NyxsOwlError::DataError(format!("Failed to get timestamp column: {}", e))
        })?;

        // Handle different timestamp column types
        match timestamp_series.dtype() {
            DataType::String => {
                let timestamps: Vec<String> = timestamp_series
                    .str()
                    .map_err(|e| {
                        NyxsOwlError::DataError(format!(
                            "Failed to cast timestamp to string: {}",
                            e
                        ))
                    })?
                    .into_no_null_iter()
                    .map(|s| s.to_string())
                    .collect();
                Ok(timestamps)
            }
            DataType::Datetime(_, _) => {
                // Convert datetime to string
                let timestamps: Vec<String> = (0..timestamp_series.len())
                    .map(|i| format!("timestamp_{}", i))
                    .collect();
                Ok(timestamps)
            }
            _ => {
                // Fallback: generate sequential timestamps
                let timestamps: Vec<String> = (0..timestamp_series.len())
                    .map(|i| format!("timestamp_{}", i))
                    .collect();
                Ok(timestamps)
            }
        }
    }

    fn generate_rolling_forecasts(
        &self,
        prices: &[f64],
        _timestamps: &[String],
    ) -> Result<Vec<Signal>> {
        let mut signals = Vec::with_capacity(prices.len());
        let window_size = self.config.window_size;

        // For the first window_size points, we can't generate forecasts
        for _ in 0..window_size {
            signals.push(Signal::Hold);
        }

        // Generate forecasts using rolling window
        for i in window_size..prices.len() {
            let window_data = &prices[i - window_size..i];
            let current_price = prices[i];

            match self.generate_exponential_smoothing_forecast(window_data) {
                Ok(forecast) => {
                    let signal = self.forecast_to_signal(current_price, forecast);
                    signals.push(signal);
                }
                Err(_) => {
                    signals.push(Signal::Hold);
                }
            }
        }

        Ok(signals)
    }

    /// Simple exponential smoothing forecast
    fn simple_exponential_smoothing(&self, data: &[f64]) -> Result<f64> {
        if data.is_empty() {
            return Err(NyxsOwlError::DataError("No data provided".to_string()));
        }

        let mut level = data[0];

        for &value in data.iter().skip(1) {
            level = self.config.alpha * value + (1.0 - self.config.alpha) * level;
        }

        Ok(level)
    }

    /// Double exponential smoothing (Holt's method) forecast
    fn double_exponential_smoothing(&self, data: &[f64], beta: f64) -> Result<f64> {
        if data.len() < 2 {
            return Err(NyxsOwlError::DataError(
                "Need at least 2 data points for trend".to_string(),
            ));
        }

        let mut level = data[0];
        let mut trend = data[1] - data[0];

        for &value in data.iter().skip(1) {
            let prev_level = level;
            level = self.config.alpha * value + (1.0 - self.config.alpha) * (level + trend);
            trend = beta * (level - prev_level) + (1.0 - beta) * trend;
        }

        // Forecast one step ahead
        Ok(level + trend * self.config.forecast_horizon as f64)
    }

    /// Triple exponential smoothing (Holt-Winters) forecast
    fn triple_exponential_smoothing(
        &self,
        data: &[f64],
        beta: f64,
        gamma: f64,
        seasonal_periods: usize,
    ) -> Result<f64> {
        if data.len() < seasonal_periods * 2 {
            return Err(NyxsOwlError::DataError(format!(
                "Need at least {} data points for seasonality",
                seasonal_periods * 2
            )));
        }

        // Initialize seasonal factors
        let mut seasonal = vec![0.0; seasonal_periods];
        for i in 0..seasonal_periods {
            let mut sum = 0.0;
            let mut count = 0;
            for j in (i..data.len()).step_by(seasonal_periods) {
                sum += data[j];
                count += 1;
            }
            seasonal[i] = if count > 0 { sum / count as f64 } else { 1.0 };
        }

        // Normalize seasonal factors
        let seasonal_sum: f64 = seasonal.iter().sum();
        let seasonal_avg = seasonal_sum / seasonal_periods as f64;
        for s in &mut seasonal {
            *s /= seasonal_avg;
        }

        let mut level = data[0] / seasonal[0];
        let mut trend = 0.0;

        for (t, &value) in data.iter().enumerate().skip(1) {
            let season_idx = t % seasonal_periods;
            let prev_level = level;

            level = self.config.alpha * (value / seasonal[season_idx])
                + (1.0 - self.config.alpha) * (level + trend);
            trend = beta * (level - prev_level) + (1.0 - beta) * trend;
            seasonal[season_idx] = gamma * (value / level) + (1.0 - gamma) * seasonal[season_idx];
        }

        // Forecast
        let forecast_season_idx =
            (data.len() + self.config.forecast_horizon - 1) % seasonal_periods;
        Ok((level + trend * self.config.forecast_horizon as f64) * seasonal[forecast_season_idx])
    }

    /// Generate forecast using appropriate exponential smoothing method
    fn generate_exponential_smoothing_forecast(&self, data: &[f64]) -> Result<f64> {
        match (
            self.config.beta,
            self.config.gamma,
            self.config.seasonal_periods,
        ) {
            (None, None, None) => {
                // Simple exponential smoothing
                self.simple_exponential_smoothing(data)
            }
            (Some(beta), None, None) => {
                // Double exponential smoothing (Holt's method)
                self.double_exponential_smoothing(data, beta)
            }
            (Some(beta), Some(gamma), Some(seasonal_periods)) => {
                // Triple exponential smoothing (Holt-Winters)
                self.triple_exponential_smoothing(data, beta, gamma, seasonal_periods)
            }
            _ => Err(NyxsOwlError::InvalidParameter(
                "Invalid combination of smoothing parameters".to_string(),
            )),
        }
    }

    fn forecast_to_signal(&self, current_price: f64, forecast: f64) -> Signal {
        let price_change = (forecast - current_price) / current_price;

        if price_change > self.config.threshold {
            Signal::Buy
        } else if price_change < -self.config.threshold {
            Signal::Sell
        } else {
            Signal::Hold
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data(len: usize) -> PolarsResult<DataFrame> {
        let timestamps: Vec<String> = (0..len)
            .map(|i| format!("2023-01-{:02} 09:30:00", (i % 30) + 1))
            .collect();

        let prices: Vec<f64> = (0..len)
            .map(|i| 100.0 + (i as f64 * 0.1) + (i as f64 * 0.1).sin() * 5.0)
            .collect();

        df! {
            "timestamp" => timestamps,
            "close" => prices,
        }
    }

    #[test]
    fn test_exponential_smoothing_config_validation() {
        // Valid configuration
        let config = ExponentialSmoothingConfig::new(0.3, Some(0.1), None, None, 1, 0.02, 20, 50);
        assert!(config.is_ok());

        // Invalid alpha
        let config = ExponentialSmoothingConfig::new(1.5, Some(0.1), None, None, 1, 0.02, 20, 50);
        assert!(config.is_err());

        // Invalid beta
        let config = ExponentialSmoothingConfig::new(0.3, Some(1.5), None, None, 1, 0.02, 20, 50);
        assert!(config.is_err());

        // Invalid seasonal periods
        let config =
            ExponentialSmoothingConfig::new(0.3, Some(0.1), Some(0.1), Some(1), 1, 0.02, 20, 50);
        assert!(config.is_err());
    }

    #[test]
    fn test_exponential_smoothing_strategy_creation() {
        let config = ExponentialSmoothingConfig::default();
        let strategy = ExponentialSmoothingStrategy::new(config);
        assert_eq!(strategy.config.alpha, 0.3);
    }

    #[test]
    fn test_exponential_smoothing_insufficient_data() {
        let config = ExponentialSmoothingConfig::default();
        let strategy = ExponentialSmoothingStrategy::new(config);

        let df = create_test_data(10).unwrap(); // Only 10 points, need 20
        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(matches!(result, Err(NyxsOwlError::MissingData(_))));
    }

    #[test]
    fn test_exponential_smoothing_missing_columns() {
        let config = ExponentialSmoothingConfig::default();
        let strategy = ExponentialSmoothingStrategy::new(config);

        let df = create_test_data(100).unwrap();

        // Test missing price column
        let result = strategy.generate_signals(&df, "missing_price", "timestamp");
        assert!(matches!(result, Err(NyxsOwlError::DataError(_))));

        // Test missing timestamp column
        let result = strategy.generate_signals(&df, "close", "missing_timestamp");
        assert!(matches!(result, Err(NyxsOwlError::DataError(_))));
    }

    #[test]
    fn test_simple_exponential_smoothing() {
        let config = ExponentialSmoothingConfig {
            alpha: 0.3,
            beta: None,
            gamma: None,
            seasonal_periods: None,
            forecast_horizon: 1,
            threshold: 0.01,
            min_data_points: 10,
            window_size: 20,
        };
        let strategy = ExponentialSmoothingStrategy::new(config);

        let df = create_test_data(100).unwrap();
        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(result.is_ok());

        let signals = result.unwrap();
        assert_eq!(signals.len(), df.height());

        // Verify signals are valid
        for signal in &signals {
            assert!(matches!(signal, Signal::Buy | Signal::Sell | Signal::Hold));
        }
    }

    #[test]
    fn test_double_exponential_smoothing() {
        let config = ExponentialSmoothingConfig {
            alpha: 0.3,
            beta: Some(0.1),
            gamma: None,
            seasonal_periods: None,
            forecast_horizon: 1,
            threshold: 0.01,
            min_data_points: 10,
            window_size: 20,
        };
        let strategy = ExponentialSmoothingStrategy::new(config);

        let df = create_test_data(100).unwrap();
        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(result.is_ok());

        let signals = result.unwrap();
        assert!(!signals.is_empty());
    }

    #[test]
    fn test_exponential_smoothing_preset_configs() {
        // Test conservative config
        let conservative = ExponentialSmoothingConfig::conservative();
        assert_eq!(conservative.alpha, 0.1);
        assert_eq!(conservative.threshold, 0.03);

        // Test moderate config
        let moderate = ExponentialSmoothingConfig::moderate();
        assert_eq!(moderate.alpha, 0.3);
        assert!(moderate.beta.is_some());

        // Test aggressive config
        let aggressive = ExponentialSmoothingConfig::aggressive();
        assert_eq!(aggressive.alpha, 0.5);
        assert_eq!(aggressive.threshold, 0.015);

        // Test seasonal config
        let seasonal = ExponentialSmoothingConfig::seasonal(12);
        assert!(seasonal.is_ok());
        let seasonal = seasonal.unwrap();
        assert!(seasonal.gamma.is_some());
        assert_eq!(seasonal.seasonal_periods, Some(12));
    }

    #[test]
    fn test_exponential_smoothing_edge_cases() {
        let config = ExponentialSmoothingConfig::default();
        let strategy = ExponentialSmoothingStrategy::new(config);

        // Test with constant prices
        let df = df! {
            "timestamp" => (0..60).map(|i| format!("2023-01-{:02}", i + 1)).collect::<Vec<_>>(),
            "close" => vec![100.0; 60],
        }
        .unwrap();

        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(result.is_ok());

        // Should generate mostly hold signals for constant prices
        let signals = result.unwrap();
        let hold_signals = signals.iter().filter(|&&s| s == Signal::Hold).count();
        assert!(hold_signals > signals.len() / 2); // Most signals should be Hold
    }
}
