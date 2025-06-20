use crate::simple_types::{NyxsOwlError, Result, Signal};
use polars::prelude::*;
use std::collections::HashMap;

/// Types of copula models supported
#[derive(Debug, Clone, PartialEq)]
pub enum CopulaType {
    /// Gaussian copula with correlation matrix
    Gaussian,
    /// Student-t copula with degrees of freedom
    StudentT(f64),
    /// Clayton copula (lower tail dependence)
    Clayton(f64),
    /// Gumbel copula (upper tail dependence)
    Gumbel(f64),
    /// Frank copula (symmetric dependence)
    Frank(f64),
}

/// Strategy types for copula-based trading
#[derive(Debug, Clone, PartialEq)]
pub enum CopulaStrategyType {
    /// Pairs trading based on correlation deviations
    PairsTrading,
    /// Statistical arbitrage across multiple assets
    StatisticalArbitrage,
    /// Portfolio optimization using dependency structure
    PortfolioOptimization,
    /// Risk management based on tail dependencies
    RiskManagement,
}

/// Configuration for Copula strategy
#[derive(Debug, Clone)]
pub struct CopulaStrategyConfig {
    /// Type of copula model to use
    pub copula_type: CopulaType,
    /// Strategy type
    pub strategy_type: CopulaStrategyType,
    /// Asset pairs or groups to analyze
    pub asset_pairs: Vec<(String, String)>,
    /// Lookback window for correlation estimation
    pub lookback_window: usize,
    /// Threshold for correlation deviation signals
    pub correlation_threshold: f64,
    /// Signal threshold for trading decisions
    pub signal_threshold: f64,
    /// Minimum number of data points required
    pub min_data_points: usize,
    /// Rolling window for dynamic correlation
    pub rolling_window: usize,
    /// Confidence level for tail dependence
    pub confidence_level: f64,
    /// Risk adjustment factor
    pub risk_adjustment: f64,
}

impl Default for CopulaStrategyConfig {
    fn default() -> Self {
        Self {
            copula_type: CopulaType::Gaussian,
            strategy_type: CopulaStrategyType::PairsTrading,
            asset_pairs: vec![("ASSET1".to_string(), "ASSET2".to_string())],
            lookback_window: 60,
            correlation_threshold: 0.7,
            signal_threshold: 0.02,
            min_data_points: 100,
            rolling_window: 30,
            confidence_level: 0.95,
            risk_adjustment: 1.0,
        }
    }
}

impl CopulaStrategyConfig {
    /// Create a new copula strategy configuration
    pub fn new(
        copula_type: CopulaType,
        strategy_type: CopulaStrategyType,
        asset_pairs: Vec<(String, String)>,
        correlation_threshold: f64,
        signal_threshold: f64,
    ) -> Result<Self> {
        if asset_pairs.is_empty() {
            return Err(NyxsOwlError::InvalidParameter(
                "At least one asset pair must be specified".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&correlation_threshold) {
            return Err(NyxsOwlError::InvalidParameter(
                "Correlation threshold must be between 0 and 1".to_string(),
            ));
        }

        if signal_threshold <= 0.0 || signal_threshold > 1.0 {
            return Err(NyxsOwlError::InvalidParameter(
                "Signal threshold must be between 0 and 1".to_string(),
            ));
        }

        // Validate copula parameters
        match &copula_type {
            CopulaType::StudentT(_df) => {
                if *_df <= 0.0 {
                    return Err(NyxsOwlError::InvalidParameter(
                        "Student-t degrees of freedom must be positive".to_string(),
                    ));
                }
            }
            CopulaType::Clayton(alpha) => {
                if *alpha <= 0.0 {
                    return Err(NyxsOwlError::InvalidParameter(
                        "Clayton alpha parameter must be positive".to_string(),
                    ));
                }
            }
            CopulaType::Gumbel(alpha) => {
                if *alpha < 1.0 {
                    return Err(NyxsOwlError::InvalidParameter(
                        "Gumbel alpha parameter must be >= 1".to_string(),
                    ));
                }
            }
            _ => {} // No additional validation needed
        }

        Ok(Self {
            copula_type,
            strategy_type,
            asset_pairs,
            correlation_threshold,
            signal_threshold,
            lookback_window: 60,
            min_data_points: 100,
            rolling_window: 30,
            confidence_level: 0.95,
            risk_adjustment: 1.0,
        })
    }

    /// Create pairs trading configuration
    pub fn pairs_trading(asset1: &str, asset2: &str) -> Self {
        Self {
            copula_type: CopulaType::Gaussian,
            strategy_type: CopulaStrategyType::PairsTrading,
            asset_pairs: vec![(asset1.to_string(), asset2.to_string())],
            correlation_threshold: 0.8,
            signal_threshold: 0.015,
            lookback_window: 90,
            min_data_points: 120,
            rolling_window: 45,
            confidence_level: 0.95,
            risk_adjustment: 1.0,
        }
    }

    /// Create statistical arbitrage configuration
    pub fn statistical_arbitrage(asset_pairs: Vec<(String, String)>) -> Self {
        Self {
            copula_type: CopulaType::StudentT(5.0),
            strategy_type: CopulaStrategyType::StatisticalArbitrage,
            asset_pairs,
            correlation_threshold: 0.6,
            signal_threshold: 0.01,
            lookback_window: 120,
            min_data_points: 150,
            rolling_window: 60,
            confidence_level: 0.99,
            risk_adjustment: 0.8,
        }
    }

    /// Create portfolio optimization configuration
    pub fn portfolio_optimization(assets: Vec<String>) -> Self {
        let asset_pairs: Vec<(String, String)> = assets
            .iter()
            .enumerate()
            .flat_map(|(i, asset1)| {
                assets
                    .iter()
                    .skip(i + 1)
                    .map(move |asset2| (asset1.clone(), asset2.clone()))
            })
            .collect();

        Self {
            copula_type: CopulaType::Gaussian,
            strategy_type: CopulaStrategyType::PortfolioOptimization,
            asset_pairs,
            correlation_threshold: 0.5,
            signal_threshold: 0.02,
            lookback_window: 150,
            min_data_points: 200,
            rolling_window: 75,
            confidence_level: 0.95,
            risk_adjustment: 1.2,
        }
    }

    /// Create risk management configuration  
    pub fn risk_management(asset_pairs: Vec<(String, String)>) -> Self {
        Self {
            copula_type: CopulaType::Clayton(2.0), // Focus on lower tail dependence
            strategy_type: CopulaStrategyType::RiskManagement,
            asset_pairs,
            correlation_threshold: 0.9,
            signal_threshold: 0.005,
            lookback_window: 200,
            min_data_points: 250,
            rolling_window: 100,
            confidence_level: 0.99,
            risk_adjustment: 0.5, // Very conservative
        }
    }
}

/// Dependency structure analysis results
#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    /// Correlation matrix
    pub correlations: HashMap<(String, String), f64>,
    /// Tail dependencies (lower, upper)
    pub tail_dependencies: HashMap<(String, String), (f64, f64)>,
    /// Copula parameters
    pub copula_parameters: HashMap<String, f64>,
    /// Goodness of fit measures
    pub fit_statistics: HashMap<String, f64>,
}

/// Copula trading strategy
///
/// This strategy uses copula models to capture dependency structures between
/// multiple assets and generates trading signals based on:
/// - Pairs trading when correlations deviate from historical norms
/// - Statistical arbitrage across multiple correlated assets
/// - Portfolio optimization using forecasted dependency structures
/// - Risk management based on tail dependencies
pub struct CopulaStrategy {
    config: CopulaStrategyConfig,
}

impl CopulaStrategy {
    /// Create a new copula strategy
    pub fn new(config: CopulaStrategyConfig) -> Self {
        Self { config }
    }

    /// Generate trading signals based on copula dependency analysis
    ///
    /// # Arguments
    /// * `df` - Input DataFrame containing price and timestamp columns.
    /// * `price_columns` - Names of the price columns for each asset.
    /// * `timestamp_column` - Name of the timestamp column.
    ///
    /// # Returns
    /// A vector of trading signals (`Signal`) for each row in the DataFrame.
    pub fn generate_signals(
        &self,
        df: &DataFrame,
        price_columns: &[String],
        timestamp_column: &str,
    ) -> Result<Vec<Signal>> {
        // Validate inputs
        self.validate_inputs(df, price_columns, timestamp_column)?;

        // Extract price data for all assets
        let price_data = self.extract_multi_asset_prices(df, price_columns)?;

        // Calculate returns for all assets
        let returns_data = self.calculate_multi_asset_returns(&price_data)?;

        // Analyze dependency structure using copulas
        let dependency_analysis = self.analyze_dependencies(&returns_data)?;

        // Generate signals based on strategy type
        let signals = match self.config.strategy_type {
            CopulaStrategyType::PairsTrading => self.generate_pairs_trading_signals(
                &price_data,
                &returns_data,
                &dependency_analysis,
            )?,
            CopulaStrategyType::StatisticalArbitrage => {
                self.generate_stat_arb_signals(&price_data, &returns_data, &dependency_analysis)?
            }
            CopulaStrategyType::PortfolioOptimization => {
                self.generate_portfolio_signals(&price_data, &returns_data, &dependency_analysis)?
            }
            CopulaStrategyType::RiskManagement => self.generate_risk_management_signals(
                &price_data,
                &returns_data,
                &dependency_analysis,
            )?,
        };

        Ok(signals)
    }

    /// Validate input DataFrame and parameters
    fn validate_inputs(
        &self,
        df: &DataFrame,
        price_columns: &[String],
        timestamp_column: &str,
    ) -> Result<()> {
        if df.height() < self.config.min_data_points {
            return Err(NyxsOwlError::MissingData(format!(
                "Insufficient data: {} rows, need at least {}",
                df.height(),
                self.config.min_data_points
            )));
        }

        if price_columns.len() < 2 {
            return Err(NyxsOwlError::InvalidParameter(
                "At least 2 price columns required for copula analysis".to_string(),
            ));
        }

        // Validate all price columns exist
        for col in price_columns {
            df.column(col).map_err(|e| {
                NyxsOwlError::DataError(format!("Price column '{}' not found: {}", col, e))
            })?;
        }

        // Validate timestamp column exists
        df.column(timestamp_column).map_err(|e| {
            NyxsOwlError::DataError(format!(
                "Timestamp column '{}' not found: {}",
                timestamp_column, e
            ))
        })?;

        Ok(())
    }

    /// Extract price data for multiple assets
    fn extract_multi_asset_prices(
        &self,
        df: &DataFrame,
        price_columns: &[String],
    ) -> Result<HashMap<String, Vec<f64>>> {
        let mut price_data = HashMap::new();

        for col_name in price_columns {
            let column = df.column(col_name).map_err(|e| {
                NyxsOwlError::DataError(format!("Failed to get column {}: {}", col_name, e))
            })?;

            let prices: Vec<f64> = column
                .f64()
                .map_err(|e| {
                    NyxsOwlError::DataError(format!("Failed to convert {} to f64: {}", col_name, e))
                })?
                .into_iter()
                .collect::<Option<Vec<f64>>>()
                .ok_or_else(|| {
                    NyxsOwlError::DataError(format!("Column {} contains null values", col_name))
                })?;

            price_data.insert(col_name.clone(), prices);
        }

        Ok(price_data)
    }

    /// Calculate returns for multiple assets
    fn calculate_multi_asset_returns(
        &self,
        price_data: &HashMap<String, Vec<f64>>,
    ) -> Result<HashMap<String, Vec<f64>>> {
        let mut returns_data = HashMap::new();

        for (asset, prices) in price_data {
            if prices.len() < 2 {
                return Err(NyxsOwlError::DataError(format!(
                    "Need at least 2 prices for {} to calculate returns",
                    asset
                )));
            }

            let returns: Vec<f64> = prices
                .windows(2)
                .map(|window| (window[1] - window[0]) / window[0])
                .collect();

            returns_data.insert(asset.clone(), returns);
        }

        Ok(returns_data)
    }

    /// Analyze dependency structure using copula models
    fn analyze_dependencies(
        &self,
        returns_data: &HashMap<String, Vec<f64>>,
    ) -> Result<DependencyAnalysis> {
        let mut correlations = HashMap::new();
        let mut tail_dependencies = HashMap::new();
        let mut copula_parameters = HashMap::new();
        let mut fit_statistics = HashMap::new();

        // Analyze each asset pair
        for (asset1, asset2) in &self.config.asset_pairs {
            if let (Some(returns1), Some(returns2)) =
                (returns_data.get(asset1), returns_data.get(asset2))
            {
                // Calculate correlation
                let correlation = self.calculate_correlation(returns1, returns2)?;
                correlations.insert((asset1.clone(), asset2.clone()), correlation);

                // Estimate copula parameters and tail dependencies
                let (copula_param, tail_deps) =
                    self.estimate_copula_parameters(returns1, returns2)?;
                copula_parameters.insert(format!("{}_{}", asset1, asset2), copula_param);
                tail_dependencies.insert((asset1.clone(), asset2.clone()), tail_deps);

                // Calculate goodness of fit
                let fit_stat = self.calculate_copula_fit(returns1, returns2, copula_param)?;
                fit_statistics.insert(format!("{}_{}_fit", asset1, asset2), fit_stat);
            }
        }

        Ok(DependencyAnalysis {
            correlations,
            tail_dependencies,
            copula_parameters,
            fit_statistics,
        })
    }

    /// Calculate Pearson correlation coefficient
    fn calculate_correlation(&self, x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() || x.is_empty() {
            return Err(NyxsOwlError::DataError(
                "Input vectors must have the same non-zero length".to_string(),
            ));
        }

        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;

        for (xi, yi) in x.iter().zip(y.iter()) {
            let dx = xi - mean_x;
            let dy = yi - mean_y;
            sum_xy += dx * dy;
            sum_x2 += dx * dx;
            sum_y2 += dy * dy;
        }

        let correlation = sum_xy / (sum_x2.sqrt() * sum_y2.sqrt());
        Ok(correlation)
    }

    /// Estimate copula parameters (simplified implementation)
    fn estimate_copula_parameters(&self, x: &[f64], y: &[f64]) -> Result<(f64, (f64, f64))> {
        // Convert to ranks (empirical copula)
        let ranks_x = self.convert_to_ranks(x);
        let ranks_y = self.convert_to_ranks(y);

        let copula_param = match &self.config.copula_type {
            CopulaType::Gaussian => {
                // For Gaussian copula, parameter is correlation
                self.calculate_correlation(x, y)?
            }
            CopulaType::StudentT(_df) => {
                // For Student-t copula, use correlation with given df
                self.calculate_correlation(x, y)?
            }
            CopulaType::Clayton(alpha) => *alpha,
            CopulaType::Gumbel(alpha) => *alpha,
            CopulaType::Frank(alpha) => *alpha,
        };

        // Estimate tail dependencies (simplified)
        let lower_tail = self.estimate_lower_tail_dependence(&ranks_x, &ranks_y)?;
        let upper_tail = self.estimate_upper_tail_dependence(&ranks_x, &ranks_y)?;

        Ok((copula_param, (lower_tail, upper_tail)))
    }

    /// Convert data to ranks for empirical copula
    fn convert_to_ranks(&self, data: &[f64]) -> Vec<f64> {
        let mut indexed_data: Vec<(usize, f64)> =
            data.iter().enumerate().map(|(i, &x)| (i, x)).collect();
        indexed_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut ranks = vec![0.0; data.len()];
        for (rank, (original_index, _)) in indexed_data.iter().enumerate() {
            ranks[*original_index] = (rank + 1) as f64 / data.len() as f64;
        }

        ranks
    }

    /// Estimate lower tail dependence
    fn estimate_lower_tail_dependence(&self, u: &[f64], v: &[f64]) -> Result<f64> {
        let threshold = 0.1; // Use bottom 10% for tail dependence
        let mut tail_count = 0;
        let mut u_tail_count = 0;

        for (ui, vi) in u.iter().zip(v.iter()) {
            if *ui <= threshold {
                u_tail_count += 1;
                if *vi <= threshold {
                    tail_count += 1;
                }
            }
        }

        if u_tail_count == 0 {
            return Ok(0.0);
        }

        Ok(tail_count as f64 / u_tail_count as f64)
    }

    /// Estimate upper tail dependence
    fn estimate_upper_tail_dependence(&self, u: &[f64], v: &[f64]) -> Result<f64> {
        let threshold = 0.9; // Use top 10% for tail dependence
        let mut tail_count = 0;
        let mut u_tail_count = 0;

        for (ui, vi) in u.iter().zip(v.iter()) {
            if *ui >= threshold {
                u_tail_count += 1;
                if *vi >= threshold {
                    tail_count += 1;
                }
            }
        }

        if u_tail_count == 0 {
            return Ok(0.0);
        }

        Ok(tail_count as f64 / u_tail_count as f64)
    }

    /// Calculate copula goodness of fit (simplified)
    fn calculate_copula_fit(&self, x: &[f64], y: &[f64], _parameter: f64) -> Result<f64> {
        // Simple fit measure: correlation-based
        let correlation = self.calculate_correlation(x, y)?;
        Ok(correlation.abs()) // Higher absolute correlation = better fit for this simplified measure
    }

    /// Generate pairs trading signals
    fn generate_pairs_trading_signals(
        &self,
        price_data: &HashMap<String, Vec<f64>>,
        _returns_data: &HashMap<String, Vec<f64>>,
        dependency_analysis: &DependencyAnalysis,
    ) -> Result<Vec<Signal>> {
        let data_length = price_data
            .values()
            .next()
            .ok_or_else(|| NyxsOwlError::DataError("No price data available".to_string()))?
            .len();

        let mut signals = vec![Signal::Hold; data_length];

        // For each asset pair, generate pairs trading signals
        for (asset1, asset2) in &self.config.asset_pairs {
            if let (Some(prices1), Some(prices2)) = (price_data.get(asset1), price_data.get(asset2))
            {
                if let Some(&correlation) = dependency_analysis
                    .correlations
                    .get(&(asset1.clone(), asset2.clone()))
                {
                    // Calculate rolling spread and z-score
                    for i in self.config.rolling_window..data_length {
                        let spread = prices1[i] - prices2[i];

                        // Calculate rolling mean and std of spread
                        let window_start = i.saturating_sub(self.config.rolling_window);
                        let spreads: Vec<f64> =
                            (window_start..i).map(|j| prices1[j] - prices2[j]).collect();

                        let mean_spread = spreads.iter().sum::<f64>() / spreads.len() as f64;
                        let std_spread = {
                            let variance = spreads
                                .iter()
                                .map(|&s| (s - mean_spread).powi(2))
                                .sum::<f64>()
                                / spreads.len() as f64;
                            variance.sqrt()
                        };

                        if std_spread > 1e-8 {
                            let z_score = (spread - mean_spread) / std_spread;

                            // Generate signals based on z-score and correlation strength
                            if correlation.abs() > self.config.correlation_threshold {
                                if z_score > 2.0 {
                                    // Spread too high -> sell asset1, buy asset2
                                    signals[i] = Signal::Sell;
                                } else if z_score < -2.0 {
                                    // Spread too low -> buy asset1, sell asset2
                                    signals[i] = Signal::Buy;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(signals)
    }

    /// Generate statistical arbitrage signals
    fn generate_stat_arb_signals(
        &self,
        _price_data: &HashMap<String, Vec<f64>>,
        returns_data: &HashMap<String, Vec<f64>>,
        dependency_analysis: &DependencyAnalysis,
    ) -> Result<Vec<Signal>> {
        let data_length = returns_data
            .values()
            .next()
            .ok_or_else(|| NyxsOwlError::DataError("No returns data available".to_string()))?
            .len();

        let mut signals = vec![Signal::Hold; data_length + 1]; // +1 because returns are one shorter

        // Multi-asset statistical arbitrage based on correlation breakdowns
        for i in self.config.rolling_window..data_length {
            let mut signal_strength = 0.0;
            let mut pair_count = 0;

            for (asset1, asset2) in &self.config.asset_pairs {
                if let (Some(returns1), Some(returns2)) =
                    (returns_data.get(asset1), returns_data.get(asset2))
                {
                    // Calculate recent correlation
                    let window_start = i.saturating_sub(self.config.rolling_window);
                    let recent_returns1 = &returns1[window_start..i];
                    let recent_returns2 = &returns2[window_start..i];

                    if let Ok(recent_corr) =
                        self.calculate_correlation(recent_returns1, recent_returns2)
                    {
                        if let Some(&historical_corr) = dependency_analysis
                            .correlations
                            .get(&(asset1.clone(), asset2.clone()))
                        {
                            let corr_deviation = (recent_corr - historical_corr).abs();

                            if corr_deviation > self.config.signal_threshold {
                                // Correlation breakdown -> arbitrage opportunity
                                signal_strength += corr_deviation;
                                pair_count += 1;
                            }
                        }
                    }
                }
            }

            if pair_count > 0 {
                let avg_signal = signal_strength / pair_count as f64;
                if avg_signal > self.config.correlation_threshold * 0.5 {
                    signals[i + 1] = Signal::Buy; // Generic buy signal for stat arb
                }
            }
        }

        Ok(signals)
    }

    /// Generate portfolio optimization signals
    fn generate_portfolio_signals(
        &self,
        _price_data: &HashMap<String, Vec<f64>>,
        returns_data: &HashMap<String, Vec<f64>>,
        dependency_analysis: &DependencyAnalysis,
    ) -> Result<Vec<Signal>> {
        let data_length = returns_data
            .values()
            .next()
            .ok_or_else(|| NyxsOwlError::DataError("No returns data available".to_string()))?
            .len();

        let mut signals = vec![Signal::Hold; data_length + 1];

        // Portfolio rebalancing based on changing correlations
        for i in self.config.rolling_window..data_length {
            let mut correlation_changes = Vec::new();

            for (asset1, asset2) in &self.config.asset_pairs {
                if let (Some(returns1), Some(returns2)) =
                    (returns_data.get(asset1), returns_data.get(asset2))
                {
                    let window_start = i.saturating_sub(self.config.rolling_window);
                    let recent_returns1 = &returns1[window_start..i];
                    let recent_returns2 = &returns2[window_start..i];

                    if let Ok(recent_corr) =
                        self.calculate_correlation(recent_returns1, recent_returns2)
                    {
                        if let Some(&historical_corr) = dependency_analysis
                            .correlations
                            .get(&(asset1.clone(), asset2.clone()))
                        {
                            correlation_changes.push(recent_corr - historical_corr);
                        }
                    }
                }
            }

            if !correlation_changes.is_empty() {
                let avg_change =
                    correlation_changes.iter().sum::<f64>() / correlation_changes.len() as f64;

                // If correlations are increasing significantly, reduce exposure (sell)
                if avg_change > self.config.signal_threshold {
                    signals[i + 1] = Signal::Sell;
                }
                // If correlations are decreasing significantly, increase diversification (buy)
                else if avg_change < -self.config.signal_threshold {
                    signals[i + 1] = Signal::Buy;
                }
            }
        }

        Ok(signals)
    }

    /// Generate risk management signals
    fn generate_risk_management_signals(
        &self,
        _price_data: &HashMap<String, Vec<f64>>,
        returns_data: &HashMap<String, Vec<f64>>,
        dependency_analysis: &DependencyAnalysis,
    ) -> Result<Vec<Signal>> {
        let data_length = returns_data
            .values()
            .next()
            .ok_or_else(|| NyxsOwlError::DataError("No returns data available".to_string()))?
            .len();

        let mut signals = vec![Signal::Hold; data_length + 1];

        // Risk management based on tail dependencies and extreme correlations
        for i in self.config.rolling_window..data_length {
            let mut risk_signals = Vec::new();

            for (asset1, asset2) in &self.config.asset_pairs {
                if let Some((lower_tail, upper_tail)) = dependency_analysis
                    .tail_dependencies
                    .get(&(asset1.clone(), asset2.clone()))
                {
                    // High tail dependence indicates contagion risk
                    if *lower_tail > 0.3 || *upper_tail > 0.3 {
                        risk_signals.push(1.0); // Risk signal
                    }
                }

                // Check for extreme correlations
                if let (Some(returns1), Some(returns2)) =
                    (returns_data.get(asset1), returns_data.get(asset2))
                {
                    let window_start = i.saturating_sub(self.config.rolling_window);
                    let recent_returns1 = &returns1[window_start..i];
                    let recent_returns2 = &returns2[window_start..i];

                    if let Ok(recent_corr) =
                        self.calculate_correlation(recent_returns1, recent_returns2)
                    {
                        if recent_corr.abs() > 0.95 {
                            risk_signals.push(recent_corr.abs());
                        }
                    }
                }
            }

            if !risk_signals.is_empty() {
                let avg_risk = risk_signals.iter().sum::<f64>() / risk_signals.len() as f64;

                // High risk -> sell signal to reduce exposure
                if avg_risk > self.config.correlation_threshold {
                    signals[i + 1] = Signal::Sell;
                }
            }
        }

        Ok(signals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataframe(prices1: Vec<f64>, prices2: Vec<f64>) -> DataFrame {
        assert_eq!(prices1.len(), prices2.len());
        let timestamps: Vec<String> = (0..prices1.len())
            .map(|i| format!("2023-01-{:02}", i + 1))
            .collect();

        DataFrame::new(vec![
            Series::new("timestamp".into(), timestamps).into(),
            Series::new("asset1".into(), prices1).into(),
            Series::new("asset2".into(), prices2).into(),
        ])
        .unwrap()
    }

    #[test]
    fn test_copula_strategy_config_validation() {
        // Valid configuration
        let config = CopulaStrategyConfig::new(
            CopulaType::Gaussian,
            CopulaStrategyType::PairsTrading,
            vec![("A".to_string(), "B".to_string())],
            0.7,
            0.02,
        );
        assert!(config.is_ok());

        // Empty asset pairs
        let config = CopulaStrategyConfig::new(
            CopulaType::Gaussian,
            CopulaStrategyType::PairsTrading,
            vec![],
            0.7,
            0.02,
        );
        assert!(config.is_err());

        // Invalid correlation threshold
        let config = CopulaStrategyConfig::new(
            CopulaType::Gaussian,
            CopulaStrategyType::PairsTrading,
            vec![("A".to_string(), "B".to_string())],
            1.5,
            0.02,
        );
        assert!(config.is_err());

        // Invalid Student-t degrees of freedom
        let config = CopulaStrategyConfig::new(
            CopulaType::StudentT(-1.0),
            CopulaStrategyType::PairsTrading,
            vec![("A".to_string(), "B".to_string())],
            0.7,
            0.02,
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_copula_strategy_creation() {
        let config = CopulaStrategyConfig::default();
        let strategy = CopulaStrategy::new(config);

        assert_eq!(strategy.config.lookback_window, 60);
        assert_eq!(strategy.config.correlation_threshold, 0.7);
    }

    #[test]
    fn test_preset_configurations() {
        let pairs_config = CopulaStrategyConfig::pairs_trading("AAPL", "MSFT");
        assert!(matches!(
            pairs_config.strategy_type,
            CopulaStrategyType::PairsTrading
        ));
        assert_eq!(pairs_config.asset_pairs.len(), 1);

        let stat_arb_config =
            CopulaStrategyConfig::statistical_arbitrage(vec![("A".to_string(), "B".to_string())]);
        assert!(matches!(
            stat_arb_config.strategy_type,
            CopulaStrategyType::StatisticalArbitrage
        ));

        let portfolio_config = CopulaStrategyConfig::portfolio_optimization(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]);
        assert!(matches!(
            portfolio_config.strategy_type,
            CopulaStrategyType::PortfolioOptimization
        ));
        assert_eq!(portfolio_config.asset_pairs.len(), 3); // 3 choose 2 = 3 pairs

        let risk_config =
            CopulaStrategyConfig::risk_management(vec![("A".to_string(), "B".to_string())]);
        assert!(matches!(
            risk_config.strategy_type,
            CopulaStrategyType::RiskManagement
        ));
    }

    #[test]
    fn test_correlation_calculation() {
        let config = CopulaStrategyConfig::default();
        let strategy = CopulaStrategy::new(config);

        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let correlation = strategy.calculate_correlation(&x, &y).unwrap();
        assert!((correlation - 1.0).abs() < 1e-10); // Perfect positive correlation

        let y_neg = vec![-2.0, -4.0, -6.0, -8.0, -10.0];
        let correlation_neg = strategy.calculate_correlation(&x, &y_neg).unwrap();
        assert!((correlation_neg - (-1.0)).abs() < 1e-10); // Perfect negative correlation
    }

    #[test]
    fn test_copula_strategy_insufficient_data() {
        let config = CopulaStrategyConfig::default();
        let strategy = CopulaStrategy::new(config);

        let df = create_test_dataframe(vec![100.0, 101.0], vec![200.0, 201.0]);
        let price_columns = vec!["asset1".to_string(), "asset2".to_string()];
        let result = strategy.generate_signals(&df, &price_columns, "timestamp");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Insufficient data"));
    }

    #[test]
    fn test_copula_strategy_missing_columns() {
        let config = CopulaStrategyConfig::default();
        let strategy = CopulaStrategy::new(config);

        let df = create_test_dataframe(vec![100.0; 120], vec![200.0; 120]);

        // Test missing price column
        let price_columns = vec!["missing".to_string(), "asset2".to_string()];
        let result = strategy.generate_signals(&df, &price_columns, "timestamp");
        assert!(result.is_err());

        // Test missing timestamp column
        let price_columns = vec!["asset1".to_string(), "asset2".to_string()];
        let result = strategy.generate_signals(&df, &price_columns, "missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_pairs_trading_signals() {
        let config = CopulaStrategyConfig::pairs_trading("asset1", "asset2");
        let strategy = CopulaStrategy::new(config);

        // Create correlated data with mean-reverting spread
        let mut prices1 = vec![100.0];
        let mut prices2 = vec![100.0];

        for i in 1..150 {
            let trend = i as f64 * 0.1;
            let spread_deviation = ((i as f64) / 10.0).sin() * 5.0;

            prices1.push(100.0 + trend + spread_deviation);
            prices2.push(100.0 + trend - spread_deviation); // Opposite spread
        }

        let df = create_test_dataframe(prices1, prices2);
        let price_columns = vec!["asset1".to_string(), "asset2".to_string()];
        let result = strategy.generate_signals(&df, &price_columns, "timestamp");

        assert!(result.is_ok());
        let signals = result.unwrap();
        assert_eq!(signals.len(), 150);

        // Should generate some trading signals for mean-reverting spread
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
        assert!(buy_count > 0 || sell_count > 0);
    }

    #[test]
    fn test_rank_conversion() {
        let config = CopulaStrategyConfig::default();
        let strategy = CopulaStrategy::new(config);

        let data = vec![3.0, 1.0, 4.0, 2.0];
        let ranks = strategy.convert_to_ranks(&data);

        // Expected ranks: 3.0->0.75, 1.0->0.25, 4.0->1.0, 2.0->0.5
        assert!((ranks[0] - 0.75).abs() < 1e-10);
        assert!((ranks[1] - 0.25).abs() < 1e-10);
        assert!((ranks[2] - 1.0).abs() < 1e-10);
        assert!((ranks[3] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_tail_dependence_estimation() {
        let config = CopulaStrategyConfig::default();
        let strategy = CopulaStrategy::new(config);

        // Create uniform data on [0,1]
        let u: Vec<f64> = (0..100).map(|i| (i + 1) as f64 / 101.0).collect();
        let v = u.clone(); // Perfect dependence

        let lower_tail = strategy.estimate_lower_tail_dependence(&u, &v).unwrap();
        let upper_tail = strategy.estimate_upper_tail_dependence(&u, &v).unwrap();

        // With perfect dependence, tail dependencies should be 1.0
        assert!((lower_tail - 1.0).abs() < 0.1);
        assert!((upper_tail - 1.0).abs() < 0.1);
    }
}
