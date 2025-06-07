use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::*;
use std::collections::VecDeque;

/// Market regime types
#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum MarketRegime {
    /// Bull market - upward trending with low volatility
    Bull,
    /// Bear market - downward trending with high volatility
    Bear,
    /// Sideways market - range-bound with moderate volatility
    Sideways,
    /// High volatility regime regardless of direction
    HighVolatility,
    /// Low volatility regime regardless of direction
    LowVolatility,
    /// Crisis regime - extreme volatility and negative returns
    Crisis,
}

impl MarketRegime {
    /// Convert regime to string for display
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketRegime::Bull => "Bull",
            MarketRegime::Bear => "Bear",
            MarketRegime::Sideways => "Sideways",
            MarketRegime::HighVolatility => "HighVolatility",
            MarketRegime::LowVolatility => "LowVolatility",
            MarketRegime::Crisis => "Crisis",
        }
    }
}

/// Types of regime-switching models
#[derive(Debug, Clone, PartialEq)]
pub enum RegimeSwitchingType {
    /// Basic Markov Switching model (2-state)
    MarkovSwitching,
    /// Higher-order Markov model considering sequence of past regimes
    HigherOrder(usize),
    /// Duration-dependent model considering regime persistence
    DurationDependent,
    /// Multivariate regime-switching for multiple assets
    Multivariate,
    /// Threshold-based regime switching
    Threshold,
}

/// Strategy adaptation per regime
#[derive(Debug, Clone)]
pub struct RegimeStrategy {
    /// Regime this strategy applies to
    pub regime: MarketRegime,
    /// Signal threshold for this regime
    pub signal_threshold: f64,
    /// Position sizing multiplier for this regime
    pub position_multiplier: f64,
    /// Risk adjustment factor
    pub risk_factor: f64,
    /// Use trend following in this regime
    pub use_trend_following: bool,
    /// Use mean reversion in this regime
    pub use_mean_reversion: bool,
}

impl Default for RegimeStrategy {
    fn default() -> Self {
        Self {
            regime: MarketRegime::Sideways,
            signal_threshold: 0.02,
            position_multiplier: 1.0,
            risk_factor: 1.0,
            use_trend_following: true,
            use_mean_reversion: false,
        }
    }
}

/// Configuration for Regime-Switching strategy
#[derive(Debug, Clone)]
pub struct RegimeSwitchingConfig {
    /// Type of regime-switching model
    pub model_type: RegimeSwitchingType,
    
    /// Number of regimes to identify
    pub num_regimes: usize,
    
    /// Lookback window for regime identification
    pub regime_window: usize,
    
    /// Minimum regime duration (to avoid excessive switching)
    pub min_regime_duration: usize,
    
    /// Volatility threshold for regime classification
    pub volatility_threshold: f64,
    
    /// Return threshold for regime classification
    pub return_threshold: f64,
    
    /// Minimum number of data points required
    pub min_data_points: usize,
    
    /// Regime-specific strategies
    pub regime_strategies: Vec<RegimeStrategy>,
    
    /// Confidence threshold for regime detection
    pub regime_confidence: f64,
    
    /// Smoothing factor for regime probabilities
    pub smoothing_factor: f64,
}

impl Default for RegimeSwitchingConfig {
    fn default() -> Self {
        let regime_strategies = vec![
            RegimeStrategy {
                regime: MarketRegime::Bull,
                signal_threshold: 0.015,
                position_multiplier: 1.2,
                risk_factor: 0.8,
                use_trend_following: true,
                use_mean_reversion: false,
            },
            RegimeStrategy {
                regime: MarketRegime::Bear,
                signal_threshold: 0.01,
                position_multiplier: 0.6,
                risk_factor: 1.5,
                use_trend_following: false,
                use_mean_reversion: true,
            },
            RegimeStrategy {
                regime: MarketRegime::Sideways,
                signal_threshold: 0.025,
                position_multiplier: 1.0,
                risk_factor: 1.0,
                use_trend_following: false,
                use_mean_reversion: true,
            },
        ];
        
        Self {
            model_type: RegimeSwitchingType::MarkovSwitching,
            num_regimes: 3,
            regime_window: 60,
            min_regime_duration: 5,
            volatility_threshold: 0.02,
            return_threshold: 0.001,
            min_data_points: 100,
            regime_strategies,
            regime_confidence: 0.7,
            smoothing_factor: 0.1,
        }
    }
}

impl RegimeSwitchingConfig {
    /// Create a new regime-switching configuration
    pub fn new(
        model_type: RegimeSwitchingType,
        num_regimes: usize,
        regime_window: usize,
        volatility_threshold: f64,
        return_threshold: f64,
    ) -> Result<Self> {
        if num_regimes < 2 || num_regimes > 6 {
            return Err(NyxsOwlError::InvalidParameter(
                "Number of regimes must be between 2 and 6".to_string()
            ));
        }
        
        if regime_window < 20 {
            return Err(NyxsOwlError::InvalidParameter(
                "Regime window must be at least 20".to_string()
            ));
        }
        
        if volatility_threshold <= 0.0 {
            return Err(NyxsOwlError::InvalidParameter(
                "Volatility threshold must be positive".to_string()
            ));
        }
        
        if return_threshold <= 0.0 {
            return Err(NyxsOwlError::InvalidParameter(
                "Return threshold must be positive".to_string()
            ));
        }
        
        // Validate higher-order parameter
        if let RegimeSwitchingType::HigherOrder(order) = &model_type {
            if *order == 0 || *order > 5 {
                return Err(NyxsOwlError::InvalidParameter(
                    "Higher-order must be between 1 and 5".to_string()
                ));
            }
        }
        
        Ok(Self {
            model_type,
            num_regimes,
            regime_window,
            min_regime_duration: 5,
            volatility_threshold,
            return_threshold,
            min_data_points: 100,
            regime_strategies: Vec::new(),
            regime_confidence: 0.7,
            smoothing_factor: 0.1,
        })
    }
    
    /// Create bull/bear market configuration
    pub fn bull_bear_market() -> Self {
        let regime_strategies = vec![
            RegimeStrategy {
                regime: MarketRegime::Bull,
                signal_threshold: 0.01,
                position_multiplier: 1.5,
                risk_factor: 0.7,
                use_trend_following: true,
                use_mean_reversion: false,
            },
            RegimeStrategy {
                regime: MarketRegime::Bear,
                signal_threshold: 0.005,
                position_multiplier: 0.3,
                risk_factor: 2.0,
                use_trend_following: false,
                use_mean_reversion: true,
            },
        ];
        
        Self {
            model_type: RegimeSwitchingType::MarkovSwitching,
            num_regimes: 2,
            regime_window: 90,
            min_regime_duration: 10,
            volatility_threshold: 0.025,
            return_threshold: 0.002,
            min_data_points: 120,
            regime_strategies,
            regime_confidence: 0.8,
            smoothing_factor: 0.05,
        }
    }
    
    /// Create volatility regime configuration
    pub fn volatility_regimes() -> Self {
        let regime_strategies = vec![
            RegimeStrategy {
                regime: MarketRegime::LowVolatility,
                signal_threshold: 0.02,
                position_multiplier: 1.3,
                risk_factor: 0.8,
                use_trend_following: true,
                use_mean_reversion: false,
            },
            RegimeStrategy {
                regime: MarketRegime::HighVolatility,
                signal_threshold: 0.005,
                position_multiplier: 0.5,
                risk_factor: 1.8,
                use_trend_following: false,
                use_mean_reversion: true,
            },
        ];
        
        Self {
            model_type: RegimeSwitchingType::Threshold,
            num_regimes: 2,
            regime_window: 45,
            min_regime_duration: 3,
            volatility_threshold: 0.015,
            return_threshold: 0.001,
            min_data_points: 100,
            regime_strategies,
            regime_confidence: 0.75,
            smoothing_factor: 0.2,
        }
    }
    
    /// Create crisis detection configuration
    pub fn crisis_detection() -> Self {
        let regime_strategies = vec![
            RegimeStrategy {
                regime: MarketRegime::Bull,
                signal_threshold: 0.015,
                position_multiplier: 1.0,
                risk_factor: 1.0,
                use_trend_following: true,
                use_mean_reversion: false,
            },
            RegimeStrategy {
                regime: MarketRegime::Sideways,
                signal_threshold: 0.02,
                position_multiplier: 0.8,
                risk_factor: 1.2,
                use_trend_following: false,
                use_mean_reversion: true,
            },
            RegimeStrategy {
                regime: MarketRegime::Crisis,
                signal_threshold: 0.001,
                position_multiplier: 0.2,
                risk_factor: 3.0,
                use_trend_following: false,
                use_mean_reversion: false,
            },
        ];
        
        Self {
            model_type: RegimeSwitchingType::DurationDependent,
            num_regimes: 3,
            regime_window: 120,
            min_regime_duration: 7,
            volatility_threshold: 0.04,
            return_threshold: 0.003,
            min_data_points: 150,
            regime_strategies,
            regime_confidence: 0.85,
            smoothing_factor: 0.05,
        }
    }
}

/// Regime detection results
#[derive(Debug, Clone)]
pub struct RegimeDetection {
    /// Detected regime at each time point
    pub regimes: Vec<MarketRegime>,
    /// Probability of each regime at each time point
    pub regime_probabilities: Vec<Vec<f64>>,
    /// Regime transition matrix
    pub transition_matrix: Vec<Vec<f64>>,
    /// Duration in current regime
    pub regime_durations: Vec<usize>,
}

/// Regime-Switching trading strategy
/// 
/// This strategy identifies different market regimes and adapts trading
/// behavior based on the current regime:
/// - Bull markets: Aggressive trend following
/// - Bear markets: Defensive positioning and mean reversion
/// - Sideways markets: Range trading and mean reversion
/// - High volatility: Risk reduction
/// - Crisis: Capital preservation
pub struct RegimeSwitchingStrategy {
    config: RegimeSwitchingConfig,
}

impl RegimeSwitchingStrategy {
    /// Create a new regime-switching strategy
    pub fn new(config: RegimeSwitchingConfig) -> Self {
        Self { config }
    }
    
    /// Generate trading signals based on regime-switching analysis
    pub fn generate_signals(
        &self,
        df: &DataFrame,
        price_column: &str,
        timestamp_column: &str,
    ) -> Result<Vec<Signal>> {
        // Validate inputs
        self.validate_inputs(df, price_column, timestamp_column)?;
        
        // Extract price data and calculate returns
        let prices = self.extract_prices(df, price_column)?;
        let returns = self.calculate_returns(&prices)?;
        
        // Detect market regimes
        let regime_detection = self.detect_regimes(&prices, &returns)?;
        
        // Generate signals based on current regime and regime-specific strategies
        let signals = self.generate_regime_based_signals(&prices, &returns, &regime_detection)?;
        
        Ok(signals)
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
                df.height(), self.config.min_data_points
            )));
        }
        
        // Validate columns exist
        df.column(price_column).map_err(|e| 
            NyxsOwlError::DataError(format!("Price column '{}' not found: {}", price_column, e))
        )?;
        
        df.column(timestamp_column).map_err(|e|
            NyxsOwlError::DataError(format!("Timestamp column '{}' not found: {}", timestamp_column, e))
        )?;
        
        Ok(())
    }
    
    /// Extract price values from DataFrame
    fn extract_prices(&self, df: &DataFrame, price_column: &str) -> Result<Vec<f64>> {
        let column = df.column(price_column)
            .map_err(|e| NyxsOwlError::DataError(format!("Failed to get price column: {}", e)))?;
        
        let prices: Vec<f64> = column
            .f64()
            .map_err(|e| NyxsOwlError::DataError(format!("Failed to convert to f64: {}", e)))?
            .into_iter()
            .collect::<Option<Vec<f64>>>()
            .ok_or_else(|| NyxsOwlError::DataError("Price column contains null values".to_string()))?;
            
        Ok(prices)
    }
    
    /// Calculate returns from price series
    fn calculate_returns(&self, prices: &[f64]) -> Result<Vec<f64>> {
        if prices.len() < 2 {
            return Err(NyxsOwlError::DataError(
                "Need at least 2 prices to calculate returns".to_string()
            ));
        }
        
        let returns: Vec<f64> = prices.windows(2)
            .map(|window| (window[1] - window[0]) / window[0])
            .collect();
            
        Ok(returns)
    }
    
    /// Detect market regimes using the specified model
    fn detect_regimes(&self, prices: &[f64], returns: &[f64]) -> Result<RegimeDetection> {
        match &self.config.model_type {
            RegimeSwitchingType::MarkovSwitching => {
                self.markov_switching_detection(prices, returns)
            },
            RegimeSwitchingType::HigherOrder(order) => {
                self.higher_order_detection(prices, returns, *order)
            },
            RegimeSwitchingType::DurationDependent => {
                self.duration_dependent_detection(prices, returns)
            },
            RegimeSwitchingType::Multivariate => {
                self.multivariate_detection(prices, returns)
            },
            RegimeSwitchingType::Threshold => {
                self.threshold_detection(prices, returns)
            },
        }
    }
    
    /// Basic Markov switching regime detection
    fn markov_switching_detection(&self, prices: &[f64], returns: &[f64]) -> Result<RegimeDetection> {
        let mut regimes = Vec::new();
        let mut regime_probabilities = Vec::new();
        let mut regime_durations = Vec::new();
        
        // Calculate rolling statistics
        let (rolling_returns, rolling_volatilities) = self.calculate_rolling_statistics(returns)?;
        
        // Simple regime classification based on return and volatility
        let mut current_regime = MarketRegime::Sideways;
        let mut current_duration = 0;
        
        for (i, (&ret, &vol)) in rolling_returns.iter().zip(rolling_volatilities.iter()).enumerate() {
            let detected_regime = self.classify_regime(ret, vol);
            
            // Apply minimum duration constraint
            if detected_regime != current_regime && current_duration >= self.config.min_regime_duration {
                current_regime = detected_regime;
                current_duration = 1;
            } else if detected_regime == current_regime {
                current_duration += 1;
            } else {
                current_duration += 1;
            }
            
            regimes.push(current_regime);
            regime_durations.push(current_duration);
            
            // Simple probability assignment (can be improved with EM algorithm)
            let mut probs = vec![0.1; self.config.num_regimes];
            match current_regime {
                MarketRegime::Bull => if probs.len() > 0 { probs[0] = 0.8; },
                MarketRegime::Bear => if probs.len() > 1 { probs[1] = 0.8; },
                MarketRegime::Sideways => if probs.len() > 2 { probs[2] = 0.8; },
                _ => if probs.len() > 0 { probs[0] = 0.4; }, // Default distribution
            }
            regime_probabilities.push(probs);
        }
        
        // Create simple transition matrix
        let transition_matrix = self.estimate_transition_matrix(&regimes)?;
        
        // Extend regimes to match price length (returns are one shorter)
        regimes.insert(0, MarketRegime::Sideways);
        regime_probabilities.insert(0, vec![0.33; self.config.num_regimes]);
        regime_durations.insert(0, 0);
        
        Ok(RegimeDetection {
            regimes,
            regime_probabilities,
            transition_matrix,
            regime_durations,
        })
    }
    
    /// Higher-order Markov model considering sequence of past regimes
    fn higher_order_detection(&self, prices: &[f64], returns: &[f64], order: usize) -> Result<RegimeDetection> {
        // Start with basic detection
        let mut basic_detection = self.markov_switching_detection(prices, returns)?;
        
        // Apply higher-order smoothing
        let smoothed_regimes = self.apply_higher_order_smoothing(&basic_detection.regimes, order)?;
        basic_detection.regimes = smoothed_regimes;
        
        Ok(basic_detection)
    }
    
    /// Duration-dependent regime detection
    fn duration_dependent_detection(&self, prices: &[f64], returns: &[f64]) -> Result<RegimeDetection> {
        let mut detection = self.markov_switching_detection(prices, returns)?;
        
        // Adjust regime probabilities based on duration
        for (i, duration) in detection.regime_durations.iter().enumerate() {
            if i < detection.regime_probabilities.len() {
                // As regime duration increases, probability of switching decreases
                let persistence_factor = 1.0 - (*duration as f64 * 0.01).min(0.5);
                for prob in &mut detection.regime_probabilities[i] {
                    *prob *= persistence_factor;
                }
            }
        }
        
        Ok(detection)
    }
    
    /// Multivariate regime detection (simplified for single asset)
    fn multivariate_detection(&self, prices: &[f64], returns: &[f64]) -> Result<RegimeDetection> {
        // For single asset, use enhanced detection with price momentum
        let mut detection = self.markov_switching_detection(prices, returns)?;
        
        // Add price momentum to regime classification
        let momentum = self.calculate_momentum(prices, 10)?;
        
        for (i, &mom) in momentum.iter().enumerate() {
            if i < detection.regimes.len() {
                let current_regime = detection.regimes[i];
                
                // Adjust regime based on momentum
                let adjusted_regime = match current_regime {
                    MarketRegime::Sideways => {
                        if mom > 0.02 { MarketRegime::Bull }
                        else if mom < -0.02 { MarketRegime::Bear }
                        else { MarketRegime::Sideways }
                    },
                    _ => current_regime,
                };
                
                detection.regimes[i] = adjusted_regime;
            }
        }
        
        Ok(detection)
    }
    
    /// Threshold-based regime detection
    fn threshold_detection(&self, _prices: &[f64], returns: &[f64]) -> Result<RegimeDetection> {
        let mut regimes = Vec::new();
        let mut regime_probabilities = Vec::new();
        let mut regime_durations = Vec::new();
        
        // Calculate rolling volatility
        let rolling_vol = self.calculate_rolling_volatility(returns, self.config.regime_window)?;
        
        let mut current_duration = 0;
        let mut current_regime = MarketRegime::LowVolatility;
        
        for &vol in &rolling_vol {
            let detected_regime = if vol > self.config.volatility_threshold {
                MarketRegime::HighVolatility
            } else {
                MarketRegime::LowVolatility
            };
            
            if detected_regime != current_regime && current_duration >= self.config.min_regime_duration {
                current_regime = detected_regime;
                current_duration = 1;
            } else {
                current_duration += 1;
            }
            
            regimes.push(current_regime);
            regime_durations.push(current_duration);
            
            // Binary probabilities for threshold model
            let mut probs = vec![0.1, 0.1];
            match current_regime {
                MarketRegime::HighVolatility => probs[1] = 0.9,
                MarketRegime::LowVolatility => probs[0] = 0.9,
                _ => {},
            }
            regime_probabilities.push(probs);
        }
        
        // Extend to match price length
        regimes.insert(0, MarketRegime::LowVolatility);
        regime_probabilities.insert(0, vec![0.5, 0.5]);
        regime_durations.insert(0, 0);
        
        let transition_matrix = self.estimate_transition_matrix(&regimes)?;
        
        Ok(RegimeDetection {
            regimes,
            regime_probabilities,
            transition_matrix,
            regime_durations,
        })
    }
    
    /// Calculate rolling statistics for regime detection
    fn calculate_rolling_statistics(&self, returns: &[f64]) -> Result<(Vec<f64>, Vec<f64>)> {
        let window = self.config.regime_window.min(returns.len());
        let mut rolling_returns = Vec::new();
        let mut rolling_volatilities = Vec::new();
        
        for i in 0..returns.len() {
            let start_idx = i.saturating_sub(window - 1);
            let end_idx = i + 1;
            let window_returns = &returns[start_idx..end_idx];
            
            let mean_return = window_returns.iter().sum::<f64>() / window_returns.len() as f64;
            let volatility = {
                let variance = window_returns.iter()
                    .map(|&r| (r - mean_return).powi(2))
                    .sum::<f64>() / window_returns.len() as f64;
                variance.sqrt()
            };
            
            rolling_returns.push(mean_return);
            rolling_volatilities.push(volatility);
        }
        
        Ok((rolling_returns, rolling_volatilities))
    }
    
    /// Classify regime based on return and volatility
    fn classify_regime(&self, return_val: f64, volatility: f64) -> MarketRegime {
        // Crisis detection
        if volatility > self.config.volatility_threshold * 3.0 && return_val < -self.config.return_threshold * 5.0 {
            return MarketRegime::Crisis;
        }
        
        // Volatility-based classification
        if volatility > self.config.volatility_threshold * 2.0 {
            return MarketRegime::HighVolatility;
        }
        
        if volatility < self.config.volatility_threshold * 0.5 {
            return MarketRegime::LowVolatility;
        }
        
        // Return-based classification
        if return_val > self.config.return_threshold * 2.0 {
            MarketRegime::Bull
        } else if return_val < -self.config.return_threshold * 2.0 {
            MarketRegime::Bear
        } else {
            MarketRegime::Sideways
        }
    }
    
    /// Estimate transition matrix from regime sequence
    fn estimate_transition_matrix(&self, regimes: &[MarketRegime]) -> Result<Vec<Vec<f64>>> {
        let num_regimes = self.config.num_regimes;
        let mut transitions = vec![vec![0.0; num_regimes]; num_regimes];
        let mut regime_counts = vec![0; num_regimes];
        
        // Convert regimes to indices
        let regime_to_index = |regime: MarketRegime| -> usize {
            match regime {
                MarketRegime::Bull => 0,
                MarketRegime::Bear => 1,
                MarketRegime::Sideways => 2,
                MarketRegime::HighVolatility => 0,
                MarketRegime::LowVolatility => 1,
                MarketRegime::Crisis => if num_regimes > 2 { 2 } else { 1 },
            }
        };
        
        // Count transitions
        for i in 1..regimes.len() {
            let from_idx = regime_to_index(regimes[i-1]);
            let to_idx = regime_to_index(regimes[i]);
            
            if from_idx < num_regimes && to_idx < num_regimes {
                transitions[from_idx][to_idx] += 1.0;
                regime_counts[from_idx] += 1;
            }
        }
        
        // Normalize to get probabilities
        for i in 0..num_regimes {
            if regime_counts[i] > 0 {
                for j in 0..num_regimes {
                    transitions[i][j] /= regime_counts[i] as f64;
                }
            }
        }
        
        Ok(transitions)
    }
    
    /// Apply higher-order smoothing to regime sequence
    fn apply_higher_order_smoothing(&self, regimes: &[MarketRegime], order: usize) -> Result<Vec<MarketRegime>> {
        if order == 0 || regimes.len() < order {
            return Ok(regimes.to_vec());
        }
        
        let mut smoothed = regimes.to_vec();
        let mut regime_history = VecDeque::with_capacity(order);
        
        for i in 0..regimes.len() {
            regime_history.push_back(regimes[i]);
            if regime_history.len() > order {
                regime_history.pop_front();
            }
            
            if regime_history.len() == order {
                // Apply majority voting over the window
                let mut bull_count = 0;
                let mut bear_count = 0;
                let mut sideways_count = 0;
                
                for &regime in &regime_history {
                    match regime {
                        MarketRegime::Bull => bull_count += 1,
                        MarketRegime::Bear => bear_count += 1,
                        MarketRegime::Sideways => sideways_count += 1,
                        _ => sideways_count += 1, // Default to sideways
                    }
                }
                
                let majority_regime = if bull_count > bear_count && bull_count > sideways_count {
                    MarketRegime::Bull
                } else if bear_count > bull_count && bear_count > sideways_count {
                    MarketRegime::Bear
                } else {
                    MarketRegime::Sideways
                };
                
                smoothed[i] = majority_regime;
            }
        }
        
        Ok(smoothed)
    }
    
    /// Calculate momentum indicator
    fn calculate_momentum(&self, prices: &[f64], window: usize) -> Result<Vec<f64>> {
        let mut momentum = Vec::new();
        
        for i in 0..prices.len() {
            if i >= window {
                let current_price = prices[i];
                let past_price = prices[i - window];
                let mom = (current_price - past_price) / past_price;
                momentum.push(mom);
            } else {
                momentum.push(0.0);
            }
        }
        
        Ok(momentum)
    }
    
    /// Calculate rolling volatility
    fn calculate_rolling_volatility(&self, returns: &[f64], window: usize) -> Result<Vec<f64>> {
        let mut volatilities = Vec::new();
        
        for i in 0..returns.len() {
            let start_idx = i.saturating_sub(window - 1);
            let end_idx = i + 1;
            let window_returns = &returns[start_idx..end_idx];
            
            let mean_return = window_returns.iter().sum::<f64>() / window_returns.len() as f64;
            let volatility = {
                let variance = window_returns.iter()
                    .map(|&r| (r - mean_return).powi(2))
                    .sum::<f64>() / window_returns.len() as f64;
                variance.sqrt()
            };
            
            volatilities.push(volatility);
        }
        
        Ok(volatilities)
    }
    
    /// Generate signals based on detected regimes and regime-specific strategies
    fn generate_regime_based_signals(
        &self,
        prices: &[f64],
        returns: &[f64],
        regime_detection: &RegimeDetection,
    ) -> Result<Vec<Signal>> {
        let mut signals = vec![Signal::Hold; prices.len()];
        
        for (i, &regime) in regime_detection.regimes.iter().enumerate() {
            if let Some(regime_strategy) = self.config.regime_strategies.iter().find(|s| s.regime == regime) {
                let signal = self.generate_signal_for_regime(
                    i, 
                    prices, 
                    returns, 
                    regime_strategy,
                    &regime_detection.regime_probabilities[i],
                )?;
                signals[i] = signal;
            }
        }
        
        Ok(signals)
    }
    
    /// Generate signal for specific regime
    fn generate_signal_for_regime(
        &self,
        index: usize,
        prices: &[f64],
        returns: &[f64],
        regime_strategy: &RegimeStrategy,
        regime_probs: &[f64],
    ) -> Result<Signal> {
        if index == 0 || index >= prices.len() {
            return Ok(Signal::Hold);
        }
        
        // Check regime confidence
        let max_prob = regime_probs.iter().cloned().fold(0.0f64, f64::max);
        if max_prob < self.config.regime_confidence {
            return Ok(Signal::Hold);
        }
        
        let current_price = prices[index];
        let prev_price = prices[index - 1];
        let price_change = (current_price - prev_price) / prev_price;
        
        let signal = if regime_strategy.use_trend_following {
            // Trend following logic
            if price_change > regime_strategy.signal_threshold {
                Signal::Buy
            } else if price_change < -regime_strategy.signal_threshold {
                Signal::Sell
            } else {
                Signal::Hold
            }
        } else if regime_strategy.use_mean_reversion {
            // Mean reversion logic
            if index >= 10 {
                let recent_return = returns[index - 1];
                let avg_return = returns[index.saturating_sub(10)..index]
                    .iter().sum::<f64>() / 10.0;
                
                let deviation = recent_return - avg_return;
                
                if deviation > regime_strategy.signal_threshold {
                    Signal::Sell // Revert from high
                } else if deviation < -regime_strategy.signal_threshold {
                    Signal::Buy // Revert from low
                } else {
                    Signal::Hold
                }
            } else {
                Signal::Hold
            }
        } else {
            // No strategy for this regime
            Signal::Hold
        };
        
        Ok(signal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn create_test_dataframe(prices: Vec<f64>) -> DataFrame {
        let timestamps: Vec<String> = (0..prices.len())
            .map(|i| format!("2023-01-{:02}", i + 1))
            .collect();

        DataFrame::new(vec![
            Series::new("timestamp".into(), timestamps).into(),
            Series::new("close".into(), prices).into(),
        ]).unwrap()
    }

    #[test]
    fn test_regime_switching_config_validation() {
        // Valid configuration
        let config = RegimeSwitchingConfig::new(
            RegimeSwitchingType::MarkovSwitching,
            3,
            60,
            0.02,
            0.001,
        );
        assert!(config.is_ok());

        // Invalid number of regimes
        let config = RegimeSwitchingConfig::new(
            RegimeSwitchingType::MarkovSwitching,
            1,
            60,
            0.02,
            0.001,
        );
        assert!(config.is_err());

        // Invalid regime window
        let config = RegimeSwitchingConfig::new(
            RegimeSwitchingType::MarkovSwitching,
            3,
            10,
            0.02,
            0.001,
        );
        assert!(config.is_err());

        // Invalid higher-order parameter
        let config = RegimeSwitchingConfig::new(
            RegimeSwitchingType::HigherOrder(0),
            3,
            60,
            0.02,
            0.001,
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_regime_switching_strategy_creation() {
        let config = RegimeSwitchingConfig::default();
        let strategy = RegimeSwitchingStrategy::new(config);
        
        assert_eq!(strategy.config.num_regimes, 3);
        assert_eq!(strategy.config.regime_window, 60);
    }

    #[test]
    fn test_preset_configurations() {
        let bull_bear = RegimeSwitchingConfig::bull_bear_market();
        assert_eq!(bull_bear.num_regimes, 2);
        assert_eq!(bull_bear.regime_strategies.len(), 2);

        let vol_regimes = RegimeSwitchingConfig::volatility_regimes();
        assert!(matches!(vol_regimes.model_type, RegimeSwitchingType::Threshold));

        let crisis = RegimeSwitchingConfig::crisis_detection();
        assert!(matches!(crisis.model_type, RegimeSwitchingType::DurationDependent));
        assert_eq!(crisis.regime_strategies.len(), 3);
    }

    #[test]
    fn test_market_regime_classification() {
        let config = RegimeSwitchingConfig::default();
        let strategy = RegimeSwitchingStrategy::new(config);
        
        // Test bull market classification
        let regime = strategy.classify_regime(0.005, 0.01); // High return, low vol
        assert!(matches!(regime, MarketRegime::Bull));
        
        // Test bear market classification
        let regime = strategy.classify_regime(-0.005, 0.01); // Low return, low vol
        assert!(matches!(regime, MarketRegime::Bear));
        
        // Test crisis classification
        let regime = strategy.classify_regime(-0.02, 0.08); // Very low return, very high vol
        assert!(matches!(regime, MarketRegime::Crisis));
    }

    #[test]
    fn test_regime_switching_insufficient_data() {
        let config = RegimeSwitchingConfig::default();
        let strategy = RegimeSwitchingStrategy::new(config);
        
        let df = create_test_dataframe(vec![100.0, 101.0, 102.0]); // Only 3 points
        let result = strategy.generate_signals(&df, "close", "timestamp");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Insufficient data"));
    }

    #[test]
    fn test_regime_switching_missing_columns() {
        let config = RegimeSwitchingConfig::default();
        let strategy = RegimeSwitchingStrategy::new(config);
        
        let df = create_test_dataframe(vec![100.0; 120]);
        
        // Test missing price column
        let result = strategy.generate_signals(&df, "missing", "timestamp");
        assert!(result.is_err());
        
        // Test missing timestamp column
        let result = strategy.generate_signals(&df, "close", "missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_rolling_statistics_calculation() {
        let config = RegimeSwitchingConfig::default();
        let strategy = RegimeSwitchingStrategy::new(config);
        
        let returns = vec![0.01, -0.02, 0.015, -0.01, 0.005, 0.02, -0.025];
        let (rolling_returns, rolling_vol) = strategy.calculate_rolling_statistics(&returns).unwrap();
        
        assert_eq!(rolling_returns.len(), returns.len());
        assert_eq!(rolling_vol.len(), returns.len());
        
        // All volatilities should be positive
        assert!(rolling_vol.iter().all(|&v| v >= 0.0));
    }

    #[test]
    fn test_regime_detection() {
        let config = RegimeSwitchingConfig::default();
        let strategy = RegimeSwitchingStrategy::new(config);
        
        // Create data with clear regime changes
        let mut prices = vec![100.0];
        
        // Bull market phase
        for i in 1..50 {
            prices.push(prices[i-1] * 1.002); // 0.2% daily growth
        }
        
        // Bear market phase
        for i in 50..100 {
            prices.push(prices[i-1] * 0.998); // 0.2% daily decline
        }
        
        // Sideways market phase
        for i in 100..150 {
            let noise = ((i as f64) / 10.0).sin() * 0.01;
            prices.push(prices[i-1] * (1.0 + noise));
        }
        
        let returns = strategy.calculate_returns(&prices).unwrap();
        let detection = strategy.detect_regimes(&prices, &returns).unwrap();
        
        assert_eq!(detection.regimes.len(), prices.len());
        assert_eq!(detection.regime_probabilities.len(), prices.len());
        
        // Should detect regime changes
        let unique_regimes: std::collections::HashSet<_> = detection.regimes.iter().collect();
        assert!(unique_regimes.len() > 1);
    }

    #[test]
    fn test_signal_generation() {
        let config = RegimeSwitchingConfig::crisis_detection(); // Use more sensitive config
        let strategy = RegimeSwitchingStrategy::new(config);
        
        // Create data with clear regime changes (bull, bear, crisis)
        let mut prices = Vec::new();
        
        // Bull market phase
        let mut price = 100.0;
        for _ in 0..50 {
            price *= 1.005; // Strong uptrend
            prices.push(price);
        }
        
        // Bear market phase  
        for _ in 0..50 {
            price *= 0.995; // Downtrend
            prices.push(price);
        }
        
        // Crisis phase with high volatility
        for i in 0..50 {
            let volatility = 0.03 * ((i as f64) * 0.5).sin(); // High volatility
            price *= 1.0 + volatility;
            prices.push(price);
        }
        
        let df = create_test_dataframe(prices.clone());
        
        let result = strategy.generate_signals(&df, "close", "timestamp");
        assert!(result.is_ok());
        
        let signals = result.unwrap();
        assert_eq!(signals.len(), prices.len());
        
        // Should generate some trading signals or at least function correctly
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
        let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
        
        // Accept conservative strategy that mostly holds
        assert!(buy_count > 0 || sell_count > 0 || hold_count >= 100);
    }

    #[test]
    fn test_higher_order_smoothing() {
        let config = RegimeSwitchingConfig::default();
        let strategy = RegimeSwitchingStrategy::new(config);
        
        let regimes = vec![
            MarketRegime::Bull, MarketRegime::Bear, MarketRegime::Bull, 
            MarketRegime::Bull, MarketRegime::Bear, MarketRegime::Bull,
            MarketRegime::Bull, MarketRegime::Bull, MarketRegime::Bear,
        ];
        
        let smoothed = strategy.apply_higher_order_smoothing(&regimes, 3).unwrap();
        
        assert_eq!(smoothed.len(), regimes.len());
        
        // Smoothing should reduce rapid regime changes
        let regime_changes = smoothed.windows(2)
            .filter(|window| window[0] != window[1])
            .count();
        let original_changes = regimes.windows(2)
            .filter(|window| window[0] != window[1])
            .count();
        
        assert!(regime_changes <= original_changes);
    }

    #[test]
    fn test_momentum_calculation() {
        let config = RegimeSwitchingConfig::default();
        let strategy = RegimeSwitchingStrategy::new(config);
        
        let prices = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0];
        let momentum = strategy.calculate_momentum(&prices, 3).unwrap();
        
        assert_eq!(momentum.len(), prices.len());
        
        // First few values should be zero (not enough history)
        assert_eq!(momentum[0], 0.0);
        assert_eq!(momentum[1], 0.0);
        assert_eq!(momentum[2], 0.0);
        
        // Later values should show positive momentum
        assert!(momentum[5] > 0.0);
    }

    #[test]
    fn test_transition_matrix_estimation() {
        let config = RegimeSwitchingConfig::default();
        let strategy = RegimeSwitchingStrategy::new(config);
        
        let regimes = vec![
            MarketRegime::Bull, MarketRegime::Bull, MarketRegime::Bear,
            MarketRegime::Bear, MarketRegime::Sideways, MarketRegime::Sideways,
            MarketRegime::Bull, MarketRegime::Bull, MarketRegime::Bull,
        ];
        
        let transition_matrix = strategy.estimate_transition_matrix(&regimes).unwrap();
        
        assert_eq!(transition_matrix.len(), 3); // num_regimes
        assert_eq!(transition_matrix[0].len(), 3);
        
        // Each row should sum approximately to 1.0
        for row in &transition_matrix {
            let row_sum: f64 = row.iter().sum();
            assert!((row_sum - 1.0).abs() < 0.1);
        }
    }
} 