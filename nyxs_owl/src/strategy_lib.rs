//! Strategy library providing a comprehensive collection of trading strategies
//! 
//! This module provides a unified interface to access all available trading strategies
//! in NyxsOwl, including technical analysis strategies, forecasting strategies,
//! and custom strategy management capabilities.

use crate::common::*;
use crate::technical_strategies::*;
use crate::forecasting::*;
use polars::prelude::*;
use std::collections::HashMap;

/// Strategy registry for managing available strategies
pub struct StrategyRegistry {
    /// Registered technical strategies
    technical_strategies: HashMap<String, Box<dyn TechnicalStrategyFactory>>,
    
    /// Registered forecasting strategies
    forecasting_strategies: HashMap<String, Box<dyn ForecastingStrategyFactory>>,
}

impl StrategyRegistry {
    /// Create a new strategy registry with default strategies
    pub fn new() -> Self {
        let mut registry = Self {
            technical_strategies: HashMap::new(),
            forecasting_strategies: HashMap::new(),
        };
        
        registry.register_default_strategies();
        registry
    }
    
    /// Register default strategies
    fn register_default_strategies(&mut self) {
        // Technical strategies would be registered here
        // This is a placeholder for the actual strategy registration
    }
    
    /// Get list of available technical strategies
    pub fn list_technical_strategies(&self) -> Vec<&String> {
        self.technical_strategies.keys().collect()
    }
    
    /// Get list of available forecasting strategies
    pub fn list_forecasting_strategies(&self) -> Vec<&String> {
        self.forecasting_strategies.keys().collect()
    }
    
    /// Create a technical strategy by name
    pub fn create_technical_strategy(
        &self,
        name: &str,
        config: StrategyConfig,
    ) -> NyxsOwlResult<Box<dyn TechnicalStrategy>> {
        match self.technical_strategies.get(name) {
            Some(factory) => factory.create(config),
            None => Err(NyxsOwlError::StrategyError(
                format!("Technical strategy '{}' not found", name)
            )),
        }
    }
    
    /// Create a forecasting strategy by name
    pub fn create_forecasting_strategy(
        &self,
        name: &str,
        config: HashMap<String, ConfigValue>,
    ) -> NyxsOwlResult<Box<dyn ForecastingStrategy>> {
        match self.forecasting_strategies.get(name) {
            Some(factory) => factory.create(config),
            None => Err(NyxsOwlError::StrategyError(
                format!("Forecasting strategy '{}' not found", name)
            )),
        }
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory trait for creating technical strategies
pub trait TechnicalStrategyFactory: Send + Sync {
    /// Create a new instance of the strategy
    fn create(&self, config: StrategyConfig) -> NyxsOwlResult<Box<dyn TechnicalStrategy>>;
    
    /// Get strategy name
    fn name(&self) -> &str;
    
    /// Get strategy description
    fn description(&self) -> &str;
    
    /// Get default configuration
    fn default_config(&self) -> StrategyConfig;
}

/// Factory trait for creating forecasting strategies
pub trait ForecastingStrategyFactory: Send + Sync {
    /// Create a new instance of the strategy
    fn create(&self, config: HashMap<String, ConfigValue>) -> NyxsOwlResult<Box<dyn ForecastingStrategy>>;
    
    /// Get strategy name
    fn name(&self) -> &str;
    
    /// Get strategy description
    fn description(&self) -> &str;
    
    /// Get default configuration
    fn default_config(&self) -> HashMap<String, ConfigValue>;
}

/// Strategy performance analyzer
pub struct StrategyAnalyzer {
    /// Historical performance data
    performance_history: Vec<PerformanceMetrics>,
    
    /// Configuration used for analysis
    config: AnalysisConfig,
}

/// Configuration for strategy analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Benchmark return for comparison
    pub benchmark_return: Option<f64>,
    
    /// Risk-free rate for Sharpe ratio calculation
    pub risk_free_rate: f64,
    
    /// Analysis period in days
    pub analysis_period: usize,
    
    /// Confidence level for statistical tests
    pub confidence_level: f64,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            benchmark_return: None,
            risk_free_rate: 0.02, // 2% annual risk-free rate
            analysis_period: 252,  // 1 year of trading days
            confidence_level: 0.95,
        }
    }
}

impl StrategyAnalyzer {
    /// Create a new strategy analyzer
    pub fn new(config: AnalysisConfig) -> Self {
        Self {
            performance_history: Vec::new(),
            config,
        }
    }
    
    /// Add performance metrics to the analyzer
    pub fn add_performance(&mut self, metrics: PerformanceMetrics) {
        self.performance_history.push(metrics);
    }
    
    /// Calculate aggregate performance statistics
    pub fn calculate_aggregate_performance(&self) -> NyxsOwlResult<AggregatePerformance> {
        if self.performance_history.is_empty() {
            return Err(NyxsOwlError::DataError(
                "No performance data available for analysis".to_string()
            ));
        }
        
        let total_returns: Vec<f64> = self.performance_history
            .iter()
            .map(|p| p.total_return)
            .collect();
        
        let sharpe_ratios: Vec<f64> = self.performance_history
            .iter()
            .map(|p| p.sharpe_ratio)
            .collect();
        
        let max_drawdowns: Vec<f64> = self.performance_history
            .iter()
            .map(|p| p.max_drawdown)
            .collect();
        
        let win_rates: Vec<f64> = self.performance_history
            .iter()
            .map(|p| p.win_rate)
            .collect();
        
        Ok(AggregatePerformance {
            avg_total_return: Self::mean(&total_returns),
            std_total_return: Self::std_dev(&total_returns),
            avg_sharpe_ratio: Self::mean(&sharpe_ratios),
            std_sharpe_ratio: Self::std_dev(&sharpe_ratios),
            avg_max_drawdown: Self::mean(&max_drawdowns),
            worst_max_drawdown: max_drawdowns.iter().fold(0.0, |a, &b| a.max(b)),
            avg_win_rate: Self::mean(&win_rates),
            consistency_score: Self::calculate_consistency(&total_returns),
            total_trades: self.performance_history.iter().map(|p| p.total_trades).sum(),
            analysis_periods: self.performance_history.len(),
        })
    }
    
    /// Calculate mean of a vector
    fn mean(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }
    
    /// Calculate standard deviation
    fn std_dev(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        
        let mean = Self::mean(values);
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;
        
        variance.sqrt()
    }
    
    /// Calculate consistency score (lower volatility = higher consistency)
    fn calculate_consistency(returns: &[f64]) -> f64 {
        let std_dev = Self::std_dev(returns);
        if std_dev == 0.0 {
            return 1.0;
        }
        
        // Normalize to 0-1 scale where 1 is perfectly consistent
        (-std_dev).exp()
    }
}

/// Aggregate performance metrics across multiple periods
#[derive(Debug, Clone, PartialEq)]
pub struct AggregatePerformance {
    /// Average total return across periods
    pub avg_total_return: f64,
    
    /// Standard deviation of total returns
    pub std_total_return: f64,
    
    /// Average Sharpe ratio
    pub avg_sharpe_ratio: f64,
    
    /// Standard deviation of Sharpe ratios
    pub std_sharpe_ratio: f64,
    
    /// Average maximum drawdown
    pub avg_max_drawdown: f64,
    
    /// Worst maximum drawdown encountered
    pub worst_max_drawdown: f64,
    
    /// Average win rate
    pub avg_win_rate: f64,
    
    /// Consistency score (0-1, higher is more consistent)
    pub consistency_score: f64,
    
    /// Total number of trades across all periods
    pub total_trades: i32,
    
    /// Number of analysis periods
    pub analysis_periods: usize,
}

/// Strategy comparison utilities
pub struct StrategyComparator;

impl StrategyComparator {
    /// Compare two strategies based on multiple criteria
    pub fn compare_strategies(
        strategy1: &AggregatePerformance,
        strategy2: &AggregatePerformance,
    ) -> StrategyComparison {
        let return_score = Self::compare_returns(strategy1.avg_total_return, strategy2.avg_total_return);
        let risk_score = Self::compare_risk(strategy1.avg_max_drawdown, strategy2.avg_max_drawdown);
        let sharpe_score = Self::compare_sharpe(strategy1.avg_sharpe_ratio, strategy2.avg_sharpe_ratio);
        let consistency_score = Self::compare_consistency(
            strategy1.consistency_score,
            strategy2.consistency_score,
        );
        
        let overall_score = (return_score + risk_score + sharpe_score + consistency_score) / 4.0;
        
        StrategyComparison {
            return_advantage: return_score,
            risk_advantage: risk_score,
            sharpe_advantage: sharpe_score,
            consistency_advantage: consistency_score,
            overall_advantage: overall_score,
            recommended_strategy: if overall_score > 0.0 { 1 } else { 2 },
        }
    }
    
    /// Compare returns (higher is better)
    fn compare_returns(return1: f64, return2: f64) -> f64 {
        (return1 - return2) / (return1.abs() + return2.abs() + 1e-8)
    }
    
    /// Compare risk (lower drawdown is better)
    fn compare_risk(drawdown1: f64, drawdown2: f64) -> f64 {
        (drawdown2 - drawdown1) / (drawdown1.abs() + drawdown2.abs() + 1e-8)
    }
    
    /// Compare Sharpe ratios (higher is better)
    fn compare_sharpe(sharpe1: f64, sharpe2: f64) -> f64 {
        (sharpe1 - sharpe2) / (sharpe1.abs() + sharpe2.abs() + 1e-8)
    }
    
    /// Compare consistency (higher is better)
    fn compare_consistency(consistency1: f64, consistency2: f64) -> f64 {
        (consistency1 - consistency2) / (consistency1 + consistency2 + 1e-8)
    }
}

/// Result of strategy comparison
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyComparison {
    /// Return advantage (-1 to 1, positive favors strategy 1)
    pub return_advantage: f64,
    
    /// Risk advantage (-1 to 1, positive favors strategy 1)
    pub risk_advantage: f64,
    
    /// Sharpe ratio advantage (-1 to 1, positive favors strategy 1)
    pub sharpe_advantage: f64,
    
    /// Consistency advantage (-1 to 1, positive favors strategy 1)
    pub consistency_advantage: f64,
    
    /// Overall advantage score (-1 to 1, positive favors strategy 1)
    pub overall_advantage: f64,
    
    /// Recommended strategy (1 or 2)
    pub recommended_strategy: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_registry_creation() {
        let registry = StrategyRegistry::new();
        
        // Initially empty since we haven't registered default strategies yet
        assert_eq!(registry.list_technical_strategies().len(), 0);
        assert_eq!(registry.list_forecasting_strategies().len(), 0);
    }

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();
        assert_eq!(config.risk_free_rate, 0.02);
        assert_eq!(config.analysis_period, 252);
        assert_eq!(config.confidence_level, 0.95);
        assert!(config.benchmark_return.is_none());
    }

    #[test]
    fn test_strategy_analyzer() {
        let config = AnalysisConfig::default();
        let mut analyzer = StrategyAnalyzer::new(config);
        
        // Add some performance metrics
        analyzer.add_performance(PerformanceMetrics {
            total_return: 0.15,
            sharpe_ratio: 1.2,
            max_drawdown: 0.05,
            win_rate: 0.6,
            total_trades: 100,
            avg_trade_return: 0.001,
            volatility: 0.12,
        });
        
        analyzer.add_performance(PerformanceMetrics {
            total_return: 0.18,
            sharpe_ratio: 1.4,
            max_drawdown: 0.08,
            win_rate: 0.65,
            total_trades: 120,
            avg_trade_return: 0.0012,
            volatility: 0.13,
        });
        
        let aggregate = analyzer.calculate_aggregate_performance().unwrap();
        
        assert_eq!(aggregate.analysis_periods, 2);
        assert_eq!(aggregate.total_trades, 220);
        assert!((aggregate.avg_total_return - 0.165).abs() < 1e-10);
        assert!((aggregate.avg_sharpe_ratio - 1.3).abs() < 1e-10);
    }

    #[test]
    fn test_strategy_comparison() {
        let strategy1 = AggregatePerformance {
            avg_total_return: 0.15,
            std_total_return: 0.05,
            avg_sharpe_ratio: 1.2,
            std_sharpe_ratio: 0.1,
            avg_max_drawdown: 0.05,
            worst_max_drawdown: 0.08,
            avg_win_rate: 0.6,
            consistency_score: 0.8,
            total_trades: 1000,
            analysis_periods: 10,
        };
        
        let strategy2 = AggregatePerformance {
            avg_total_return: 0.12,
            std_total_return: 0.03,
            avg_sharpe_ratio: 1.0,
            std_sharpe_ratio: 0.08,
            avg_max_drawdown: 0.03,
            worst_max_drawdown: 0.05,
            avg_win_rate: 0.65,
            consistency_score: 0.9,
            total_trades: 800,
            analysis_periods: 10,
        };
        
        let comparison = StrategyComparator::compare_strategies(&strategy1, &strategy2);
        
        // Strategy 1 should have return and Sharpe advantages
        assert!(comparison.return_advantage > 0.0);
        assert!(comparison.sharpe_advantage > 0.0);
        
        // Strategy 2 should have risk and consistency advantages
        assert!(comparison.risk_advantage < 0.0);
        assert!(comparison.consistency_advantage < 0.0);
    }
} 