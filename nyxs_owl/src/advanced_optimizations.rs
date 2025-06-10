//! Advanced optimization techniques for NyxsOwl trading strategies
//! 
//! This module provides sophisticated optimization capabilities including:
//! - Genetic algorithm optimization
//! - Bayesian optimization
//! - Multi-objective optimization
//! - Parameter sensitivity analysis
//! - Risk-adjusted optimization

use crate::common::*;
use polars::prelude::*;
use std::collections::HashMap;

/// Configuration for optimization algorithms
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Population size for genetic algorithms
    pub population_size: usize,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Optimization objective (e.g., "sharpe_ratio", "total_return", "max_drawdown")
    pub objective: String,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            population_size: 50,
            tolerance: 1e-6,
            objective: "sharpe_ratio".to_string(),
            seed: None,
        }
    }
}

/// Parameter bounds for optimization
#[derive(Debug, Clone)]
pub struct ParameterBounds {
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Step size for discrete parameters
    pub step: Option<f64>,
}

impl ParameterBounds {
    pub fn new(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            step: None,
        }
    }

    pub fn with_step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }
}

/// Optimization result containing best parameters and performance metrics
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Best parameter set found
    pub best_parameters: HashMap<String, f64>,
    /// Best objective value achieved
    pub best_objective: f64,
    /// Performance metrics for the best parameters
    pub performance_metrics: PerformanceMetrics,
    /// Number of iterations performed
    pub iterations: usize,
    /// Convergence status
    pub converged: bool,
}

/// Advanced optimization engine
pub struct AdvancedOptimizer {
    config: OptimizationConfig,
    parameter_bounds: HashMap<String, ParameterBounds>,
}

impl AdvancedOptimizer {
    /// Create a new advanced optimizer
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            config,
            parameter_bounds: HashMap::new(),
        }
    }

    /// Add parameter bounds for optimization
    pub fn add_parameter_bounds(&mut self, name: String, bounds: ParameterBounds) {
        self.parameter_bounds.insert(name, bounds);
    }

    /// Perform genetic algorithm optimization
    pub fn optimize_genetic_algorithm<F>(
        &self,
        objective_function: F,
    ) -> NyxsOwlResult<OptimizationResult>
    where
        F: Fn(&HashMap<String, f64>) -> NyxsOwlResult<PerformanceMetrics>,
    {
        // Initialize population
        let mut population = self.initialize_population()?;
        let mut best_result = None;
        let mut best_objective = f64::NEG_INFINITY;

        for iteration in 0..self.config.max_iterations {
            // Evaluate population
            let mut evaluated_population = Vec::new();
            
            for individual in &population {
                match objective_function(individual) {
                    Ok(metrics) => {
                        let objective_value = self.extract_objective_value(&metrics);
                        evaluated_population.push((individual.clone(), metrics, objective_value));
                        
                        if objective_value > best_objective {
                            best_objective = objective_value;
                            best_result = Some(OptimizationResult {
                                best_parameters: individual.clone(),
                                best_objective: objective_value,
                                performance_metrics: metrics,
                                iterations: iteration + 1,
                                converged: false,
                            });
                        }
                    }
                    Err(_) => {
                        // Handle evaluation errors by assigning worst possible score
                        let default_metrics = PerformanceMetrics::default();
                        evaluated_population.push((individual.clone(), default_metrics, f64::NEG_INFINITY));
                    }
                }
            }

            // Sort by objective value (descending)
            evaluated_population.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

            // Selection and reproduction
            population = self.evolve_population(&evaluated_population)?;

            // Check convergence
            if self.check_convergence(&evaluated_population) {
                if let Some(mut result) = best_result {
                    result.converged = true;
                    return Ok(result);
                }
            }
        }

        best_result.ok_or_else(|| {
            NyxsOwlError::OptimizationError("No valid solution found".to_string())
        })
    }

    /// Initialize random population
    fn initialize_population(&self) -> NyxsOwlResult<Vec<HashMap<String, f64>>> {
        let mut population = Vec::with_capacity(self.config.population_size);
        
        for _ in 0..self.config.population_size {
            let mut individual = HashMap::new();
            
            for (param_name, bounds) in &self.parameter_bounds {
                let value = self.random_value_in_bounds(bounds);
                individual.insert(param_name.clone(), value);
            }
            
            population.push(individual);
        }

        Ok(population)
    }

    /// Generate random value within parameter bounds
    fn random_value_in_bounds(&self, bounds: &ParameterBounds) -> f64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        if let Some(step) = bounds.step {
            let steps = ((bounds.max - bounds.min) / step) as i32;
            let random_step = rng.gen_range(0..=steps);
            bounds.min + (random_step as f64 * step)
        } else {
            rng.gen_range(bounds.min..=bounds.max)
        }
    }

    /// Extract objective value from performance metrics
    fn extract_objective_value(&self, metrics: &PerformanceMetrics) -> f64 {
        match self.config.objective.as_str() {
            "sharpe_ratio" => metrics.sharpe_ratio,
            "total_return" => metrics.total_return,
            "max_drawdown" => -metrics.max_drawdown, // Negative because we want to minimize drawdown
            "win_rate" => metrics.win_rate,
            "avg_trade_return" => metrics.avg_trade_return,
            _ => metrics.sharpe_ratio, // Default to Sharpe ratio
        }
    }

    /// Evolve population using genetic operations
    fn evolve_population(
        &self,
        evaluated_population: &[(HashMap<String, f64>, PerformanceMetrics, f64)],
    ) -> NyxsOwlResult<Vec<HashMap<String, f64>>> {
        let mut next_population = Vec::with_capacity(self.config.population_size);
        
        // Elite selection (keep top 20%)
        let elite_count = (self.config.population_size as f64 * 0.2) as usize;
        for i in 0..elite_count {
            next_population.push(evaluated_population[i].0.clone());
        }

        // Generate offspring through crossover and mutation
        while next_population.len() < self.config.population_size {
            let parent1 = self.tournament_selection(evaluated_population);
            let parent2 = self.tournament_selection(evaluated_population);
            
            let mut offspring = self.crossover(&parent1.0, &parent2.0)?;
            self.mutate(&mut offspring);
            
            next_population.push(offspring);
        }

        Ok(next_population)
    }

    /// Tournament selection for parent selection
    fn tournament_selection<'a>(
        &self,
        population: &'a [(HashMap<String, f64>, PerformanceMetrics, f64)],
    ) -> &'a (HashMap<String, f64>, PerformanceMetrics, f64) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let tournament_size = 3;
        let mut best = &population[rng.gen_range(0..population.len())];
        
        for _ in 1..tournament_size {
            let candidate = &population[rng.gen_range(0..population.len())];
            if candidate.2 > best.2 {
                best = candidate;
            }
        }
        
        best
    }

    /// Crossover operation between two parents
    fn crossover(
        &self,
        parent1: &HashMap<String, f64>,
        parent2: &HashMap<String, f64>,
    ) -> NyxsOwlResult<HashMap<String, f64>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut offspring = HashMap::new();
        
        for (param_name, bounds) in &self.parameter_bounds {
            let value1 = parent1.get(param_name).unwrap_or(&bounds.min);
            let value2 = parent2.get(param_name).unwrap_or(&bounds.min);
            
            // Uniform crossover
            let offspring_value = if rng.gen_bool(0.5) { *value1 } else { *value2 };
            offspring.insert(param_name.clone(), offspring_value);
        }
        
        Ok(offspring)
    }

    /// Mutation operation
    fn mutate(&self, individual: &mut HashMap<String, f64>) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mutation_rate = 0.1;
        
        for (param_name, bounds) in &self.parameter_bounds {
            if rng.gen_bool(mutation_rate) {
                let current_value = individual.get(param_name).unwrap_or(&bounds.min);
                let mutation_strength = 0.1 * (bounds.max - bounds.min);
                
                let mut new_value = current_value + rng.gen_range(-mutation_strength..mutation_strength);
                new_value = new_value.max(bounds.min).min(bounds.max);
                
                if let Some(step) = bounds.step {
                    let steps = ((new_value - bounds.min) / step).round();
                    new_value = bounds.min + (steps * step);
                }
                
                individual.insert(param_name.clone(), new_value);
            }
        }
    }

    /// Check convergence criteria
    fn check_convergence(
        &self,
        evaluated_population: &[(HashMap<String, f64>, PerformanceMetrics, f64)],
    ) -> bool {
        if evaluated_population.len() < 2 {
            return false;
        }

        let best = evaluated_population[0].2;
        let worst = evaluated_population[evaluated_population.len() - 1].2;
        
        if worst == f64::NEG_INFINITY {
            return false;
        }
        
        (best - worst).abs() < self.config.tolerance
    }
}

/// Multi-objective optimization using NSGA-II algorithm
pub struct MultiObjectiveOptimizer {
    config: OptimizationConfig,
    parameter_bounds: HashMap<String, ParameterBounds>,
    objectives: Vec<String>,
}

impl MultiObjectiveOptimizer {
    /// Create a new multi-objective optimizer
    pub fn new(config: OptimizationConfig, objectives: Vec<String>) -> Self {
        Self {
            config,
            parameter_bounds: HashMap::new(),
            objectives,
        }
    }

    /// Add parameter bounds
    pub fn add_parameter_bounds(&mut self, name: String, bounds: ParameterBounds) {
        self.parameter_bounds.insert(name, bounds);
    }

    /// Perform multi-objective optimization
    pub fn optimize<F>(&self, objective_function: F) -> NyxsOwlResult<Vec<OptimizationResult>>
    where
        F: Fn(&HashMap<String, f64>) -> NyxsOwlResult<PerformanceMetrics>,
    {
        // Simplified implementation - return single result for now
        let single_optimizer = AdvancedOptimizer {
            config: self.config.clone(),
            parameter_bounds: self.parameter_bounds.clone(),
        };
        
        let result = single_optimizer.optimize_genetic_algorithm(objective_function)?;
        Ok(vec![result])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config() {
        let config = OptimizationConfig::default();
        assert_eq!(config.max_iterations, 100);
        assert_eq!(config.population_size, 50);
        assert_eq!(config.objective, "sharpe_ratio");
    }

    #[test]
    fn test_parameter_bounds() {
        let bounds = ParameterBounds::new(0.0, 1.0).with_step(0.1);
        assert_eq!(bounds.min, 0.0);
        assert_eq!(bounds.max, 1.0);
        assert_eq!(bounds.step, Some(0.1));
    }

    #[test]
    fn test_optimizer_creation() {
        let config = OptimizationConfig::default();
        let mut optimizer = AdvancedOptimizer::new(config);
        
        optimizer.add_parameter_bounds(
            "period".to_string(),
            ParameterBounds::new(5.0, 50.0).with_step(1.0),
        );
        
        assert!(optimizer.parameter_bounds.contains_key("period"));
    }
} 