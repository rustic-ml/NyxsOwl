//! Volatility forecasting and analysis

use crate::error::Result;
use crate::models::garch::GarchModel;
use crate::models::ForecastModel;

/// Calculate historical volatility over a time period
pub fn historical_volatility(returns: &[f64], window: usize) -> Vec<f64> {
    if returns.len() < window || window == 0 {
        return vec![0.0; returns.len()];
    }
    
    let mut volatility = vec![0.0; returns.len()];
    
    for i in window..returns.len() {
        // Calculate variance in the window
        let window_data = &returns[i - window..i];
        let mean = window_data.iter().sum::<f64>() / window as f64;
        let variance = window_data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / window as f64;
        
        volatility[i] = variance;
    }
    
    volatility
}

/// Calculate returns from a price series
pub fn calculate_returns(prices: &[f64]) -> Vec<f64> {
    if prices.len() < 2 {
        return Vec::new();
    }
    
    prices.windows(2)
        .map(|w| (w[1] / w[0]) - 1.0)
        .collect()
}

/// Forecast volatility using GARCH(1,1)
pub fn forecast_volatility(prices: &[f64], forecast_horizon: usize) -> Result<Vec<f64>> {
    // Calculate returns
    let returns = calculate_returns(prices);
    
    // Fit GARCH(1,1) model
    let mut model = GarchModel::new(1, 1);
    model.fit(&returns)?;
    
    // Forecast volatility
    let forecast = model.forecast(forecast_horizon)?;
    
    Ok(forecast.values)
}

/// Annualize daily volatility (standard deviation)
pub fn annualize_daily_volatility(daily_volatility: f64) -> f64 {
    daily_volatility.sqrt() * (252_f64).sqrt()
}

/// Calculate exponentially weighted volatility
pub fn ewma_volatility(returns: &[f64], lambda: f64) -> Vec<f64> {
    if returns.is_empty() {
        return Vec::new();
    }
    
    let mut volatility = vec![0.0; returns.len()];
    
    // Initialize with squared first return
    volatility[0] = returns[0].powi(2);
    
    // Update with EWMA formula
    for i in 1..returns.len() {
        volatility[i] = lambda * volatility[i-1] + (1.0 - lambda) * returns[i].powi(2);
    }
    
    volatility
} 