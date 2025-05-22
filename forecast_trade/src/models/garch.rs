//! GARCH models for time series forecasting

use crate::error::{ForecastError, Result};
use crate::models::{ForecastModel, ForecastResult};

/// GARCH model implementation
pub struct GarchModel {
    /// Model name
    name: String,
    /// GARCH order (p)
    p: usize,
    /// ARCH order (q)
    q: usize,
    /// Historical returns
    returns: Vec<f64>,
    /// Volatility estimates
    volatility: Vec<f64>,
    /// ARCH parameters
    alpha: Vec<f64>,
    /// GARCH parameters
    beta: Vec<f64>,
    /// Long run variance
    omega: f64,
}

impl GarchModel {
    /// Create a new GARCH model
    pub fn new(p: usize, q: usize) -> Self {
        Self {
            name: format!("GARCH({},{})", p, q),
            p,
            q,
            returns: Vec::new(),
            volatility: Vec::new(),
            alpha: vec![0.1; q], // Simple default values
            beta: vec![0.8; p],  // Simple default values
            omega: 0.05,         // Simple default value
        }
    }
    
    /// Calculate returns from price series
    fn calculate_returns(&self, prices: &[f64]) -> Vec<f64> {
        if prices.len() <= 1 {
            return Vec::new();
        }
        
        prices.windows(2)
            .map(|w| (w[1] / w[0]) - 1.0)
            .collect()
    }
    
    /// Estimate simple GARCH parameters
    fn estimate_parameters(&mut self, returns: &[f64]) {
        // This is a very simplified parameter estimation
        // In reality, GARCH parameters should be estimated via maximum likelihood
        
        // Calculate sample variance for returns
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|&r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        
        // Set omega to a portion of the variance
        self.omega = 0.05 * variance;
        
        // Alpha parameters sum to about 0.15 in typical financial data
        if !self.alpha.is_empty() {
            let alpha_sum = 0.15;
            let alpha_i = alpha_sum / self.alpha.len() as f64;
            for a in &mut self.alpha {
                *a = alpha_i;
            }
        }
        
        // Beta parameters typically sum to about 0.8
        if !self.beta.is_empty() {
            let beta_sum = 0.8;
            let beta_i = beta_sum / self.beta.len() as f64;
            for b in &mut self.beta {
                *b = beta_i;
            }
        }
    }
    
    /// Forecast volatility for the next periods
    fn forecast_volatility(&self, horizon: usize) -> Vec<f64> {
        if self.volatility.is_empty() {
            return vec![0.0; horizon];
        }
        
        let current_var = *self.volatility.last().unwrap();
        
        // For GARCH, volatility tends toward the long-run average
        let alpha_sum: f64 = self.alpha.iter().sum();
        let beta_sum: f64 = self.beta.iter().sum();
        let persistence = alpha_sum + beta_sum;
        
        // Long-run variance
        let long_run_var = if persistence < 1.0 {
            self.omega / (1.0 - persistence)
        } else {
            current_var // If persistence >= 1, model is non-stationary
        };
        
        let mut forecasts = Vec::with_capacity(horizon);
        let mut forecast_var = current_var;
        
        for _ in 0..horizon {
            // Simple GARCH variance forecast formula
            forecast_var = self.omega + persistence * (forecast_var - long_run_var) + long_run_var;
            forecasts.push(forecast_var.sqrt()); // Convert variance to volatility
        }
        
        forecasts
    }
}

impl ForecastModel for GarchModel {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn fit(&mut self, data: &[f64]) -> Result<()> {
        if data.len() < self.p + self.q + 1 {
            return Err(ForecastError::ValidationError(format!(
                "Insufficient data for GARCH({},{}). Need at least {} observations.",
                self.p, self.q, self.p + self.q + 1
            )));
        }
        
        // Calculate returns from prices
        let returns = self.calculate_returns(data);
        
        if returns.is_empty() {
            return Err(ForecastError::ValidationError("Could not calculate returns from provided data".to_string()));
        }
        
        // Store returns for later use
        self.returns = returns.clone();
        
        // Estimate GARCH parameters
        self.estimate_parameters(&returns);
        
        // Calculate historical volatility estimates
        let squared_returns: Vec<f64> = self.returns.iter().map(|r| r.powi(2)).collect();
        
        // Initialize volatility with unconditional variance
        let mean_sq_return = squared_returns.iter().sum::<f64>() / squared_returns.len() as f64;
        self.volatility = vec![mean_sq_return; self.returns.len()];
        
        // Run the GARCH filter
        for t in (self.q + self.p)..self.returns.len() {
            let mut arch_component = self.omega;
            
            // ARCH terms
            for i in 0..self.q {
                arch_component += self.alpha[i] * squared_returns[t - i - 1];
            }
            
            // GARCH terms
            for i in 0..self.p {
                arch_component += self.beta[i] * self.volatility[t - i - 1];
            }
            
            self.volatility[t] = arch_component;
        }
        
        Ok(())
    }
    
    fn forecast(&self, horizon: usize) -> Result<ForecastResult> {
        if self.returns.is_empty() || self.volatility.is_empty() {
            return Err(ForecastError::ForecastingError("Model has not been fitted to data".to_string()));
        }
        
        // Get volatility forecasts
        let values = self.forecast_volatility(horizon);
        
        Ok(ForecastResult {
            values,
            intervals: None,
            timestamps: None,
        })
    }
} 