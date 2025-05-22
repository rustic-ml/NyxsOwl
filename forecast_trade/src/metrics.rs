//! Metrics for evaluating forecast performance

use crate::error::{ForecastError, Result};
use crate::models::ForecastModel;
use crate::utils::forecast_accuracy;
use day_trade::Signal;

/// Evaluate forecast accuracy against actual values
pub fn evaluate_forecast(forecast: &[f64], actual: &[f64]) -> Result<ForecastMetrics> {
    if forecast.len() != actual.len() || forecast.is_empty() {
        return Err(ForecastError::ValidationError(
            "Forecast and actual values must have the same non-zero length".to_string(),
        ));
    }
    
    // Calculate accuracy metrics
    let accuracy = forecast_accuracy(forecast, actual)?;
    
    // Calculate direction accuracy
    let direction_correct = forecast.windows(2)
        .zip(actual.windows(2))
        .filter(|(f, a)| (f[1] - f[0]).abs() > 1e-10 && (a[1] - a[0]).abs() > 1e-10)
        .map(|(f, a)| (f[1] > f[0]) == (a[1] > a[0]))
        .filter(|&correct| correct)
        .count();
    
    let direction_total = forecast.windows(2)
        .zip(actual.windows(2))
        .filter(|(f, a)| (f[1] - f[0]).abs() > 1e-10 && (a[1] - a[0]).abs() > 1e-10)
        .count();
    
    let direction_accuracy = if direction_total > 0 {
        direction_correct as f64 / direction_total as f64 * 100.0
    } else {
        0.0
    };
    
    Ok(ForecastMetrics {
        mae: accuracy.mae,
        mse: accuracy.mse,
        rmse: accuracy.rmse,
        mape: accuracy.mape,
        smape: accuracy.smape,
        direction_accuracy,
    })
}

/// Evaluate forecast model on a training and test set
pub fn evaluate_model<M: ForecastModel>(
    model: &mut M, 
    train_data: &[f64], 
    test_data: &[f64], 
    horizon: usize
) -> Result<ForecastMetrics> {
    // Fit model to training data
    model.fit(train_data)?;
    
    // Generate forecast
    let forecast = model.forecast(horizon)?;
    
    // Evaluate forecast against test data (limiting to min length)
    let min_len = forecast.values.len().min(test_data.len());
    
    evaluate_forecast(&forecast.values[..min_len], &test_data[..min_len])
}

/// Evaluate strategy performance based on signals
pub fn evaluate_strategy(signals: &[Signal], prices: &[f64]) -> Result<StrategyMetrics> {
    if signals.len() != prices.len() || signals.is_empty() {
        return Err(ForecastError::ValidationError(
            "Signals and prices must have the same non-zero length".to_string(),
        ));
    }
    
    // Calculate simple returns based on signals
    let mut returns = Vec::with_capacity(signals.len() - 1);
    let mut positions = Vec::with_capacity(signals.len());
    let mut position = 0; // -1 = short, 0 = flat, 1 = long
    
    positions.push(position);
    
    for i in 0..signals.len() - 1 {
        // Update position based on signal
        match signals[i] {
            Signal::Buy => position = 1,
            Signal::Sell => position = -1,
            Signal::Hold => (), // Keep current position
        }
        
        positions.push(position);
        
        // Calculate return based on position
        let price_return = (prices[i + 1] - prices[i]) / prices[i];
        let strategy_return = position as f64 * price_return;
        returns.push(strategy_return);
    }
    
    // Calculate metrics
    let total_return: f64 = returns.iter().sum();
    let mean_return = total_return / returns.len() as f64;
    
    // Calculate standard deviation of returns
    let variance = returns.iter()
        .map(|&r| (r - mean_return).powi(2))
        .sum::<f64>() / returns.len() as f64;
    let std_dev = variance.sqrt();
    
    // Calculate Sharpe ratio (assuming risk-free rate of 0)
    let sharpe = if std_dev > 0.0 {
        mean_return / std_dev * (252_f64).sqrt() // Annualized Sharpe
    } else {
        0.0
    };
    
    // Calculate drawdown
    let mut max_value = 1.0;
    let mut current_value = 1.0;
    let mut max_drawdown = 0.0;
    
    for &ret in &returns {
        current_value *= 1.0 + ret;
        if current_value > max_value {
            max_value = current_value;
        }
        let drawdown = (max_value - current_value) / max_value;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }
    
    Ok(StrategyMetrics {
        total_return,
        sharpe_ratio: sharpe,
        max_drawdown,
        win_rate: calculate_win_rate(&returns),
    })
}

/// Calculate win rate from returns
fn calculate_win_rate(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    
    let wins = returns.iter().filter(|&&r| r > 0.0).count();
    wins as f64 / returns.len() as f64 * 100.0
}

/// Forecast performance metrics
#[derive(Debug, Clone)]
pub struct ForecastMetrics {
    /// Mean Absolute Error
    pub mae: f64,
    /// Mean Squared Error
    pub mse: f64,
    /// Root Mean Squared Error
    pub rmse: f64,
    /// Mean Absolute Percentage Error
    pub mape: f64,
    /// Symmetric Mean Absolute Percentage Error
    pub smape: f64,
    /// Direction accuracy percentage
    pub direction_accuracy: f64,
}

impl std::fmt::Display for ForecastMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Forecast Performance Metrics:")?;
        writeln!(f, "  MAE:     {:.4}", self.mae)?;
        writeln!(f, "  MSE:     {:.4}", self.mse)?;
        writeln!(f, "  RMSE:    {:.4}", self.rmse)?;
        writeln!(f, "  MAPE:    {:.4}%", self.mape)?;
        writeln!(f, "  SMAPE:   {:.4}%", self.smape)?;
        writeln!(f, "  Direction: {:.2}%", self.direction_accuracy)?;
        Ok(())
    }
}

/// Strategy performance metrics
#[derive(Debug, Clone)]
pub struct StrategyMetrics {
    /// Total return
    pub total_return: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Win rate percentage
    pub win_rate: f64,
}

impl std::fmt::Display for StrategyMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Strategy Performance Metrics:")?;
        writeln!(f, "  Total Return:  {:.2}%", self.total_return * 100.0)?;
        writeln!(f, "  Sharpe Ratio:  {:.2}", self.sharpe_ratio)?;
        writeln!(f, "  Max Drawdown:  {:.2}%", self.max_drawdown * 100.0)?;
        writeln!(f, "  Win Rate:      {:.2}%", self.win_rate)?;
        Ok(())
    }
}

/// Calculate maximum drawdown on a series of prices or returns
pub fn maximum_drawdown(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    
    let mut max_value: f64 = 1.0;
    let mut current_value = 1.0;
    let mut max_drawdown: f64 = 0.0;
    
    for &val in values {
        // If values are returns, convert to price movement
        let price_movement = if val > -1.0 { 1.0 + val } else { 0.0 };
        
        // Update current value
        current_value *= price_movement;
        
        // Update maximum value seen so far
        if current_value > max_value {
            max_value = current_value;
        }
        
        // Calculate drawdown and update max drawdown
        let drawdown = 1.0 - (current_value / max_value);
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }
    
    max_drawdown
} 