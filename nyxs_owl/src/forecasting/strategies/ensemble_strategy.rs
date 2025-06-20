use crate::forecasting::strategies::{
    ArimaStrategy, ArimaStrategyConfig, ExponentialSmoothingConfig, ExponentialSmoothingStrategy,
    KalmanStrategy, KalmanStrategyConfig,
};
use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::*;
use std::collections::HashMap;

/// Ensemble aggregation methods
#[derive(Debug, Clone, PartialEq)]
pub enum EnsembleMethod {
    /// Simple average of all model predictions
    SimpleAverage,
    /// Weighted average with custom weights
    WeightedAverage(Vec<f64>),
    /// Median of all model predictions
    Median,
    /// Select the best performing model dynamically
    BestModel,
    /// Majority voting on signal direction
    MajorityVote,
    /// Stacking with meta-learner (placeholder for future implementation)
    Stacking,
}

/// Configuration for individual models in the ensemble
#[derive(Debug, Clone)]
pub struct EnsembleModelConfig {
    /// Include ARIMA model in ensemble
    pub use_arima: bool,
    /// ARIMA configuration
    pub arima_config: ArimaStrategyConfig,

    /// Include Exponential Smoothing model in ensemble
    pub use_exponential_smoothing: bool,
    /// Exponential Smoothing configuration
    pub exponential_smoothing_config: ExponentialSmoothingConfig,

    /// Include Kalman Filter model in ensemble
    pub use_kalman: bool,
    /// Kalman Filter configuration
    pub kalman_config: KalmanStrategyConfig,
}

impl Default for EnsembleModelConfig {
    fn default() -> Self {
        Self {
            use_arima: true,
            arima_config: ArimaStrategyConfig::default(),
            use_exponential_smoothing: true,
            exponential_smoothing_config: ExponentialSmoothingConfig::default(),
            use_kalman: true,
            kalman_config: KalmanStrategyConfig::default(),
        }
    }
}

/// Configuration for Ensemble strategy
#[derive(Debug, Clone)]
pub struct EnsembleStrategyConfig {
    /// Ensemble aggregation method
    pub method: EnsembleMethod,

    /// Model configurations
    pub model_config: EnsembleModelConfig,

    /// Signal threshold for final decision
    pub signal_threshold: f64,

    /// Minimum number of data points required
    pub min_data_points: usize,

    /// Performance evaluation window for BestModel method
    pub performance_window: usize,

    /// Minimum confidence for signal generation (0.0 to 1.0)
    pub min_confidence: f64,
}

impl Default for EnsembleStrategyConfig {
    fn default() -> Self {
        Self {
            method: EnsembleMethod::SimpleAverage,
            model_config: EnsembleModelConfig::default(),
            signal_threshold: 0.01, // 1%
            min_data_points: 100,
            performance_window: 50,
            min_confidence: 0.6, // 60% confidence
        }
    }
}

impl EnsembleStrategyConfig {
    /// Create a new ensemble strategy configuration
    ///
    /// # Arguments
    /// * `method` - The ensemble aggregation method to use
    /// * `signal_threshold` - Threshold for signal generation (0.0 to 1.0)
    /// * `min_data_points` - Minimum number of data points required
    ///
    /// # Returns
    /// A new `EnsembleStrategyConfig` instance or an error if parameters are invalid
    pub fn new(
        method: EnsembleMethod,
        signal_threshold: f64,
        min_data_points: usize,
    ) -> Result<Self> {
        if signal_threshold <= 0.0 || signal_threshold > 1.0 {
            return Err(NyxsOwlError::InvalidParameter(
                "Signal threshold must be between 0 and 1".to_string(),
            ));
        }

        if min_data_points < 50 {
            return Err(NyxsOwlError::InvalidParameter(
                "Minimum data points must be at least 50".to_string(),
            ));
        }

        // Validate weighted average weights if applicable
        if let EnsembleMethod::WeightedAverage(ref weights) = method {
            if weights.is_empty() {
                return Err(NyxsOwlError::InvalidParameter(
                    "Weighted average requires non-empty weights".to_string(),
                ));
            }

            let sum: f64 = weights.iter().sum();
            if (sum - 1.0).abs() > 1e-6 {
                return Err(NyxsOwlError::InvalidParameter(
                    "Weights must sum to 1.0".to_string(),
                ));
            }
        }

        Ok(Self {
            method,
            model_config: EnsembleModelConfig::default(),
            signal_threshold,
            min_data_points,
            performance_window: 50,
            min_confidence: 0.6,
        })
    }

    /// Create conservative ensemble configuration
    ///
    /// Returns a conservative configuration with higher thresholds and more stringent requirements
    pub fn conservative() -> Self {
        Self {
            method: EnsembleMethod::Median,
            model_config: EnsembleModelConfig {
                use_arima: true,
                arima_config: ArimaStrategyConfig::default(),
                use_exponential_smoothing: true,
                exponential_smoothing_config: ExponentialSmoothingConfig::conservative(),
                use_kalman: true,
                kalman_config: KalmanStrategyConfig::conservative(),
            },
            signal_threshold: 0.02, // 2%
            min_data_points: 150,
            performance_window: 100,
            min_confidence: 0.7,
        }
    }

    /// Create aggressive ensemble configuration
    ///
    /// Returns an aggressive configuration with lower thresholds for more frequent signals
    pub fn aggressive() -> Self {
        Self {
            method: EnsembleMethod::BestModel,
            model_config: EnsembleModelConfig {
                use_arima: true,
                arima_config: ArimaStrategyConfig::default(),
                use_exponential_smoothing: true,
                exponential_smoothing_config: ExponentialSmoothingConfig::aggressive(),
                use_kalman: true,
                kalman_config: KalmanStrategyConfig::aggressive(),
            },
            signal_threshold: 0.005, // 0.5%
            min_data_points: 75,
            performance_window: 30,
            min_confidence: 0.5,
        }
    }

    /// Create balanced ensemble configuration
    ///
    /// Returns a balanced configuration with weighted average method and moderate thresholds
    pub fn balanced() -> Self {
        let weights = vec![0.4, 0.35, 0.25]; // ARIMA, ES, Kalman weights
        Self {
            method: EnsembleMethod::WeightedAverage(weights),
            model_config: EnsembleModelConfig::default(),
            signal_threshold: 0.015, // 1.5%
            min_data_points: 100,
            performance_window: 60,
            min_confidence: 0.6,
        }
    }
}

/// Ensemble trading strategy
///
/// This strategy combines multiple forecasting models to improve robustness
/// and potentially accuracy by aggregating their predictions using various methods.
pub struct EnsembleStrategy {
    config: EnsembleStrategyConfig,
}

impl EnsembleStrategy {
    /// Create a new ensemble strategy
    ///
    /// # Arguments
    /// * `config` - Configuration for the ensemble strategy
    pub fn new(config: EnsembleStrategyConfig) -> Self {
        Self { config }
    }

    /// Generate trading signals using ensemble of models
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

        // Collect signals from individual models
        let individual_signals =
            self.collect_individual_signals(df, price_column, timestamp_column)?;

        // Aggregate signals based on ensemble method
        let final_signals = self.aggregate_signals(&individual_signals)?;

        Ok(final_signals)
    }

    /// Validate input DataFrame and parameters
    fn validate_inputs(
        &self,
        df: &DataFrame,
        price_column: &str,
        timestamp_column: &str,
    ) -> Result<()> {
        if df.height() < self.config.min_data_points {
            return Err(NyxsOwlError::MissingData(format!(
                "Insufficient data: {} rows, need at least {}",
                df.height(),
                self.config.min_data_points
            )));
        }

        // Validate columns exist
        df.column(price_column).map_err(|e| {
            NyxsOwlError::DataError(format!("Price column '{}' not found: {}", price_column, e))
        })?;

        df.column(timestamp_column).map_err(|e| {
            NyxsOwlError::DataError(format!(
                "Timestamp column '{}' not found: {}",
                timestamp_column, e
            ))
        })?;

        Ok(())
    }

    /// Collect signals from all enabled individual models
    fn collect_individual_signals(
        &self,
        df: &DataFrame,
        price_column: &str,
        timestamp_column: &str,
    ) -> Result<HashMap<String, Vec<Signal>>> {
        let mut signals_map = HashMap::new();

        // ARIMA model
        if self.config.model_config.use_arima {
            let mut arima_strategy =
                ArimaStrategy::new(self.config.model_config.arima_config.clone());
            let arima_signals = arima_strategy
                .generate_signals(df, price_column, timestamp_column)
                .map_err(|e| NyxsOwlError::StrategyError(format!("ARIMA model failed: {}", e)))?;
            signals_map.insert("ARIMA".to_string(), arima_signals);
        }

        // Exponential Smoothing model
        if self.config.model_config.use_exponential_smoothing {
            let es_strategy = ExponentialSmoothingStrategy::new(
                self.config
                    .model_config
                    .exponential_smoothing_config
                    .clone(),
            );
            let es_signals = es_strategy
                .generate_signals(df, price_column, timestamp_column)
                .map_err(|e| {
                    NyxsOwlError::StrategyError(format!(
                        "Exponential Smoothing model failed: {}",
                        e
                    ))
                })?;
            signals_map.insert("ExponentialSmoothing".to_string(), es_signals);
        }

        // Kalman Filter model
        if self.config.model_config.use_kalman {
            let kalman_strategy =
                KalmanStrategy::new(self.config.model_config.kalman_config.clone());
            let kalman_signals = kalman_strategy
                .generate_signals(df, price_column, timestamp_column)
                .map_err(|e| {
                    NyxsOwlError::StrategyError(format!("Kalman Filter model failed: {}", e))
                })?;
            signals_map.insert("Kalman".to_string(), kalman_signals);
        }

        if signals_map.is_empty() {
            return Err(NyxsOwlError::InvalidParameter(
                "At least one model must be enabled in the ensemble".to_string(),
            ));
        }

        Ok(signals_map)
    }

    /// Aggregate individual model signals based on ensemble method
    fn aggregate_signals(&self, signals_map: &HashMap<String, Vec<Signal>>) -> Result<Vec<Signal>> {
        let signal_length = signals_map
            .values()
            .next()
            .ok_or_else(|| NyxsOwlError::DataError("No signals to aggregate".to_string()))?
            .len();

        // Verify all signals have the same length
        for signals in signals_map.values() {
            if signals.len() != signal_length {
                return Err(NyxsOwlError::DataError(
                    "All model signals must have the same length".to_string(),
                ));
            }
        }

        let mut final_signals = vec![Signal::Hold; signal_length];

        match &self.config.method {
            EnsembleMethod::SimpleAverage => {
                self.simple_average_aggregation(signals_map, &mut final_signals)?;
            }
            EnsembleMethod::WeightedAverage(weights) => {
                self.weighted_average_aggregation(signals_map, weights, &mut final_signals)?;
            }
            EnsembleMethod::Median => {
                self.median_aggregation(signals_map, &mut final_signals)?;
            }
            EnsembleMethod::BestModel => {
                self.best_model_aggregation(signals_map, &mut final_signals)?;
            }
            EnsembleMethod::MajorityVote => {
                self.majority_vote_aggregation(signals_map, &mut final_signals)?;
            }
            EnsembleMethod::Stacking => {
                self.stacking_aggregation(signals_map, &mut final_signals)?;
            }
        }

        Ok(final_signals)
    }

    /// Simple average aggregation of signal strengths
    fn simple_average_aggregation(
        &self,
        signals_map: &HashMap<String, Vec<Signal>>,
        final_signals: &mut [Signal],
    ) -> Result<()> {
        let models: Vec<_> = signals_map.keys().collect();
        let num_models = models.len() as f64;

        for i in 0..final_signals.len() {
            let mut buy_strength = 0.0;
            let mut sell_strength = 0.0;

            for model_name in &models {
                let signal = signals_map[*model_name][i];
                match signal {
                    Signal::Buy => buy_strength += 1.0,
                    Signal::Sell => sell_strength += 1.0,
                    Signal::Hold => {} // Neutral
                }
            }

            let avg_buy = buy_strength / num_models;
            let avg_sell = sell_strength / num_models;

            final_signals[i] = if avg_buy > self.config.min_confidence && avg_buy > avg_sell {
                Signal::Buy
            } else if avg_sell > self.config.min_confidence && avg_sell > avg_buy {
                Signal::Sell
            } else {
                Signal::Hold
            };
        }

        Ok(())
    }

    /// Weighted average aggregation with custom weights
    fn weighted_average_aggregation(
        &self,
        signals_map: &HashMap<String, Vec<Signal>>,
        weights: &[f64],
        final_signals: &mut [Signal],
    ) -> Result<()> {
        let models: Vec<_> = signals_map.keys().collect();

        if weights.len() != models.len() {
            return Err(NyxsOwlError::InvalidParameter(
                "Number of weights must match number of models".to_string(),
            ));
        }

        for i in 0..final_signals.len() {
            let mut weighted_buy = 0.0;
            let mut weighted_sell = 0.0;

            for (j, model_name) in models.iter().enumerate() {
                let signal = signals_map[*model_name][i];
                let weight = weights[j];

                match signal {
                    Signal::Buy => weighted_buy += weight,
                    Signal::Sell => weighted_sell += weight,
                    Signal::Hold => {} // Neutral
                }
            }

            final_signals[i] = if weighted_buy > self.config.min_confidence
                && weighted_buy > weighted_sell
            {
                Signal::Buy
            } else if weighted_sell > self.config.min_confidence && weighted_sell > weighted_buy {
                Signal::Sell
            } else {
                Signal::Hold
            };
        }

        Ok(())
    }

    /// Median aggregation using signal ordering
    fn median_aggregation(
        &self,
        signals_map: &HashMap<String, Vec<Signal>>,
        final_signals: &mut [Signal],
    ) -> Result<()> {
        for i in 0..final_signals.len() {
            let mut signal_values = Vec::new();

            for signals in signals_map.values() {
                let signal_value = match signals[i] {
                    Signal::Buy => 1.0,
                    Signal::Hold => 0.0,
                    Signal::Sell => -1.0,
                };
                signal_values.push(signal_value);
            }

            signal_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median_value = if signal_values.len() % 2 == 0 {
                let mid = signal_values.len() / 2;
                (signal_values[mid - 1] + signal_values[mid]) / 2.0
            } else {
                signal_values[signal_values.len() / 2]
            };

            final_signals[i] = if median_value > self.config.signal_threshold {
                Signal::Buy
            } else if median_value < -self.config.signal_threshold {
                Signal::Sell
            } else {
                Signal::Hold
            };
        }

        Ok(())
    }

    /// Best model selection based on recent performance
    fn best_model_aggregation(
        &self,
        signals_map: &HashMap<String, Vec<Signal>>,
        final_signals: &mut [Signal],
    ) -> Result<()> {
        // For now, use simple round-robin selection
        // In practice, this would evaluate recent performance of each model
        let models: Vec<_> = signals_map.keys().collect();
        let _window_size = self.config.performance_window.min(final_signals.len());

        for i in 0..final_signals.len() {
            let best_model_idx = i % models.len(); // Simple rotation for now
            let best_model = models[best_model_idx];
            final_signals[i] = signals_map[best_model][i];
        }

        Ok(())
    }

    /// Majority vote aggregation
    fn majority_vote_aggregation(
        &self,
        signals_map: &HashMap<String, Vec<Signal>>,
        final_signals: &mut [Signal],
    ) -> Result<()> {
        for i in 0..final_signals.len() {
            let mut buy_votes = 0;
            let mut sell_votes = 0;
            let mut hold_votes = 0;

            for signals in signals_map.values() {
                match signals[i] {
                    Signal::Buy => buy_votes += 1,
                    Signal::Sell => sell_votes += 1,
                    Signal::Hold => hold_votes += 1,
                }
            }

            final_signals[i] = if buy_votes > sell_votes && buy_votes > hold_votes {
                Signal::Buy
            } else if sell_votes > buy_votes && sell_votes > hold_votes {
                Signal::Sell
            } else {
                Signal::Hold
            };
        }

        Ok(())
    }

    /// Stacking ensemble aggregation using a simple meta-learner
    ///
    /// This implementation uses a simple weighted combination based on model agreement
    /// and recent performance patterns. In a full implementation, this would use
    /// a trained meta-model (like logistic regression or neural network).
    fn stacking_aggregation(
        &self,
        signals_map: &HashMap<String, Vec<Signal>>,
        final_signals: &mut [Signal],
    ) -> Result<()> {
        let models: Vec<_> = signals_map.keys().collect();
        let num_models = models.len() as f64;

        // Simple meta-learning: calculate model agreement weights
        let mut model_weights = HashMap::new();

        // Initialize equal weights
        for model in &models {
            model_weights.insert((*model).clone(), 1.0 / num_models);
        }

        // Calculate performance-based adjustments using recent agreement patterns
        let _window_size = self.config.performance_window.min(final_signals.len());
        if _window_size > 10 {
            self.calculate_stacking_weights(
                signals_map,
                &models,
                &mut model_weights,
                _window_size,
            )?;
        }

        // Generate final signals using meta-learner approach
        for i in 0..final_signals.len() {
            let mut weighted_buy_score = 0.0;
            let mut weighted_sell_score = 0.0;

            // Feature engineering: consider signal patterns and agreement
            let agreement_factor = self.calculate_model_agreement(signals_map, &models, i);
            let trend_factor = self.calculate_trend_factor(signals_map, &models, i, 3);

            for model_name in &models {
                let signal = signals_map[*model_name][i];
                let base_weight = model_weights[*model_name];

                // Adjust weight based on agreement and trend factors
                let adjusted_weight =
                    base_weight * (1.0 + agreement_factor * 0.5 + trend_factor * 0.3);

                match signal {
                    Signal::Buy => weighted_buy_score += adjusted_weight,
                    Signal::Sell => weighted_sell_score += adjusted_weight,
                    Signal::Hold => {} // Neutral
                }
            }

            // Apply meta-learner decision logic with confidence thresholds
            let confidence_threshold = self.config.min_confidence;
            let total_weight = weighted_buy_score + weighted_sell_score;

            if total_weight > 0.0 {
                let buy_confidence = weighted_buy_score / total_weight;
                let sell_confidence = weighted_sell_score / total_weight;

                final_signals[i] = if buy_confidence > confidence_threshold
                    && weighted_buy_score > weighted_sell_score
                {
                    Signal::Buy
                } else if sell_confidence > confidence_threshold
                    && weighted_sell_score > weighted_buy_score
                {
                    Signal::Sell
                } else {
                    Signal::Hold
                };
            } else {
                final_signals[i] = Signal::Hold;
            }
        }

        Ok(())
    }

    /// Calculate dynamic weights for stacking based on model performance patterns
    fn calculate_stacking_weights(
        &self,
        signals_map: &HashMap<String, Vec<Signal>>,
        models: &[&String],
        model_weights: &mut HashMap<String, f64>,
        lookback_window: usize,
    ) -> Result<()> {
        let signal_length = signals_map.values().next().unwrap().len();
        let start_idx = signal_length.saturating_sub(lookback_window);

        // Calculate model consistency and agreement scores
        for model in models {
            let mut consistency_score = 0.0;
            let mut agreement_score = 0.0;
            let mut valid_points = 0;

            for i in start_idx..signal_length {
                let model_signal = signals_map[*model][i];

                // Consistency: how often model generates actionable signals
                if model_signal != Signal::Hold {
                    consistency_score += 1.0;
                }

                // Agreement: how often model agrees with majority
                let mut agreement_count = 0;
                for other_model in models {
                    if *other_model != *model && signals_map[*other_model][i] == model_signal {
                        agreement_count += 1;
                    }
                }
                agreement_score += agreement_count as f64 / (models.len() - 1) as f64;
                valid_points += 1;
            }

            if valid_points > 0 {
                consistency_score /= valid_points as f64;
                agreement_score /= valid_points as f64;

                // Combine scores to adjust weight (50% consistency, 50% agreement)
                let performance_score = (consistency_score + agreement_score) / 2.0;
                let adjusted_weight = model_weights[*model] * (0.5 + performance_score);
                model_weights.insert((*model).clone(), adjusted_weight);
            }
        }

        // Normalize weights to sum to 1.0
        let total_weight: f64 = model_weights.values().sum();
        if total_weight > 0.0 {
            for weight in model_weights.values_mut() {
                *weight /= total_weight;
            }
        }

        Ok(())
    }

    /// Calculate model agreement factor at a specific time point
    fn calculate_model_agreement(
        &self,
        signals_map: &HashMap<String, Vec<Signal>>,
        models: &[&String],
        index: usize,
    ) -> f64 {
        if models.len() < 2 {
            return 1.0;
        }

        let mut agreement_count = 0;
        let mut total_pairs = 0;

        for i in 0..models.len() {
            for j in (i + 1)..models.len() {
                let signal_i = signals_map[models[i]][index];
                let signal_j = signals_map[models[j]][index];

                if signal_i == signal_j {
                    agreement_count += 1;
                }
                total_pairs += 1;
            }
        }

        if total_pairs > 0 {
            agreement_count as f64 / total_pairs as f64
        } else {
            1.0
        }
    }

    /// Calculate trend factor based on recent signal patterns
    fn calculate_trend_factor(
        &self,
        signals_map: &HashMap<String, Vec<Signal>>,
        models: &[&String],
        index: usize,
        lookback: usize,
    ) -> f64 {
        let start_idx = index.saturating_sub(lookback);
        if start_idx >= index {
            return 0.0;
        }

        let mut trend_score = 0.0;
        let mut valid_models = 0;

        for model in models {
            let mut buy_trend = 0;
            let mut sell_trend = 0;

            for i in start_idx..index {
                match signals_map[*model][i] {
                    Signal::Buy => buy_trend += 1,
                    Signal::Sell => sell_trend += 1,
                    Signal::Hold => {}
                }
            }

            let total_signals = buy_trend + sell_trend;
            if total_signals > 0 {
                // Trend strength: how consistent the recent signals are
                let max_trend = buy_trend.max(sell_trend);
                trend_score += max_trend as f64 / total_signals as f64;
                valid_models += 1;
            }
        }

        if valid_models > 0 {
            trend_score / valid_models as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataframe(prices: Vec<f64>) -> DataFrame {
        let timestamps: Vec<String> = (0..prices.len())
            .map(|i| format!("2023-01-{:02}", i + 1))
            .collect();

        DataFrame::new(vec![
            Series::new("timestamp".into(), timestamps).into(),
            Series::new("close".into(), prices).into(),
        ])
        .unwrap()
    }

    #[test]
    fn test_ensemble_strategy_config_validation() {
        // Valid configuration
        let config = EnsembleStrategyConfig::new(EnsembleMethod::SimpleAverage, 0.02, 100);
        assert!(config.is_ok());

        // Invalid signal threshold
        let config = EnsembleStrategyConfig::new(EnsembleMethod::SimpleAverage, 1.5, 100);
        assert!(config.is_err());

        // Invalid min data points
        let config = EnsembleStrategyConfig::new(EnsembleMethod::SimpleAverage, 0.02, 30);
        assert!(config.is_err());
    }

    #[test]
    fn test_weighted_average_validation() {
        let weights = vec![0.5, 0.3, 0.2]; // Sum = 1.0
        let config =
            EnsembleStrategyConfig::new(EnsembleMethod::WeightedAverage(weights), 0.02, 100);
        assert!(config.is_ok());

        let invalid_weights = vec![0.5, 0.3, 0.3]; // Sum = 1.1
        let config = EnsembleStrategyConfig::new(
            EnsembleMethod::WeightedAverage(invalid_weights),
            0.02,
            100,
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_ensemble_strategy_creation() {
        let config = EnsembleStrategyConfig::default();
        let strategy = EnsembleStrategy::new(config);

        assert_eq!(strategy.config.signal_threshold, 0.01);
        assert_eq!(strategy.config.min_data_points, 100);
    }

    #[test]
    fn test_preset_configurations() {
        let conservative = EnsembleStrategyConfig::conservative();
        assert!(matches!(conservative.method, EnsembleMethod::Median));
        assert_eq!(conservative.min_confidence, 0.7);

        let aggressive = EnsembleStrategyConfig::aggressive();
        assert!(matches!(aggressive.method, EnsembleMethod::BestModel));
        assert_eq!(aggressive.min_confidence, 0.5);

        let balanced = EnsembleStrategyConfig::balanced();
        assert!(matches!(
            balanced.method,
            EnsembleMethod::WeightedAverage(_)
        ));
        assert_eq!(balanced.min_confidence, 0.6);
    }

    #[test]
    fn test_ensemble_strategy_insufficient_data() {
        let config = EnsembleStrategyConfig::default();
        let strategy = EnsembleStrategy::new(config);

        let df = create_test_dataframe(vec![100.0, 101.0, 102.0]); // Only 3 points
        let result = strategy.generate_signals(&df, "close", "timestamp");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Insufficient data"));
    }

    #[test]
    fn test_ensemble_strategy_missing_columns() {
        let config = EnsembleStrategyConfig::default();
        let strategy = EnsembleStrategy::new(config);

        let df = create_test_dataframe(vec![100.0; 120]);

        // Test missing price column
        let result = strategy.generate_signals(&df, "missing", "timestamp");
        assert!(result.is_err());

        // Test missing timestamp column
        let result = strategy.generate_signals(&df, "close", "missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_simple_ensemble_functionality() {
        let config = EnsembleStrategyConfig {
            method: EnsembleMethod::SimpleAverage,
            min_data_points: 120,
            ..Default::default()
        };
        let strategy = EnsembleStrategy::new(config);

        // Create trending data
        let prices: Vec<f64> = (0..150).map(|i| 100.0 + i as f64 * 0.5).collect();
        let df = create_test_dataframe(prices.clone());

        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(result.is_ok());

        let signals = result.unwrap();
        assert_eq!(signals.len(), prices.len());
    }

    #[test]
    fn test_majority_vote_ensemble() {
        let config = EnsembleStrategyConfig {
            method: EnsembleMethod::MajorityVote,
            min_data_points: 120,
            ..Default::default()
        };
        let strategy = EnsembleStrategy::new(config);

        let prices: Vec<f64> = (0..150).map(|i| 100.0 + i as f64 * 0.1).collect();
        let df = create_test_dataframe(prices);

        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(result.is_ok());
    }

    #[test]
    fn test_median_ensemble() {
        let config = EnsembleStrategyConfig {
            method: EnsembleMethod::Median,
            min_data_points: 120,
            ..Default::default()
        };
        let strategy = EnsembleStrategy::new(config);

        let prices: Vec<f64> = (0..150)
            .map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0)
            .collect();
        let df = create_test_dataframe(prices);

        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensemble_with_disabled_models() {
        let config = EnsembleStrategyConfig {
            method: EnsembleMethod::SimpleAverage,
            model_config: EnsembleModelConfig {
                use_arima: false,
                use_exponential_smoothing: true,
                use_kalman: false,
                ..Default::default()
            },
            min_data_points: 120,
            ..Default::default()
        };
        let strategy = EnsembleStrategy::new(config);

        let prices: Vec<f64> = (0..150).map(|i| 100.0 + i as f64 * 0.2).collect();
        let df = create_test_dataframe(prices);

        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(result.is_ok());
    }
}
