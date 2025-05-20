//! Forecasting implementations for time series prediction
//!
//! Contains implementations of various forecasting methods:
//! - Linear Regression
//! - Exponential Smoothing
//! - ARIMA Model (basic implementation)

use crate::{MathError, Result};
use std::collections::VecDeque;

/// Linear Regression implementation for trend forecasting
#[derive(Debug, Clone)]
pub struct LinearRegression {
    period: usize,
    values: VecDeque<f64>,
    slope: Option<f64>,
    intercept: Option<f64>,
}

impl LinearRegression {
    /// Create a new Linear Regression with the specified period
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(MathError::InvalidInput(
                "Period must be at least 2 for linear regression".to_string(),
            ));
        }

        Ok(Self {
            period,
            values: VecDeque::with_capacity(period),
            slope: None,
            intercept: None,
        })
    }

    /// Update the Linear Regression with a new value
    pub fn update(&mut self, value: f64) -> Result<()> {
        // Add new value
        self.values.push_back(value);

        // Keep only the required number of values
        if self.values.len() > self.period {
            self.values.pop_front();
        }

        // Calculate regression if we have enough data
        if self.values.len() >= 2 {
            self.calculate_regression()?;
        }

        Ok(())
    }

    /// Calculate the linear regression parameters (slope and intercept)
    fn calculate_regression(&mut self) -> Result<()> {
        let n = self.values.len() as f64;

        // Calculate means
        let x_mean = (0..self.values.len()).map(|i| i as f64).sum::<f64>() / n;
        let y_mean = self.values.iter().sum::<f64>() / n;

        // Calculate the slope (m)
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in self.values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean) * (x - x_mean);
        }

        if denominator.abs() < 1e-10 {
            return Err(MathError::CalculationError(
                "Cannot calculate slope: x values are too similar".to_string(),
            ));
        }

        let slope = numerator / denominator;

        // Calculate the intercept (b)
        let intercept = y_mean - slope * x_mean;

        self.slope = Some(slope);
        self.intercept = Some(intercept);

        Ok(())
    }

    /// Predict the value n periods into the future
    pub fn forecast(&self, periods_ahead: usize) -> Result<f64> {
        if self.values.len() < 2 {
            return Err(MathError::InsufficientData(
                "Not enough data for forecasting. Need at least 2 points.".to_string(),
            ));
        }

        match (self.slope, self.intercept) {
            (Some(slope), Some(intercept)) => {
                let x = (self.values.len() + periods_ahead - 1) as f64;
                Ok(slope * x + intercept)
            }
            _ => Err(MathError::CalculationError(
                "Regression parameters not calculated".to_string(),
            )),
        }
    }

    /// Get the current slope (trend direction and strength)
    pub fn slope(&self) -> Result<f64> {
        match self.slope {
            Some(slope) => Ok(slope),
            None => Err(MathError::InsufficientData(
                "Not enough data to calculate slope".to_string(),
            )),
        }
    }

    /// Get the current intercept
    pub fn intercept(&self) -> Result<f64> {
        match self.intercept {
            Some(intercept) => Ok(intercept),
            None => Err(MathError::InsufficientData(
                "Not enough data to calculate intercept".to_string(),
            )),
        }
    }

    /// Get the R-squared value (coefficient of determination)
    pub fn r_squared(&self) -> Result<f64> {
        if self.values.len() < 2 {
            return Err(MathError::InsufficientData(
                "Not enough data to calculate R-squared. Need at least 2 points.".to_string(),
            ));
        }

        match (self.slope, self.intercept) {
            (Some(slope), Some(intercept)) => {
                let y_mean = self.values.iter().sum::<f64>() / self.values.len() as f64;

                let mut ss_total = 0.0; // total sum of squares
                let mut ss_residual = 0.0; // residual sum of squares

                for (i, &y) in self.values.iter().enumerate() {
                    let x = i as f64;
                    let y_pred = slope * x + intercept;

                    ss_total += (y - y_mean).powi(2);
                    ss_residual += (y - y_pred).powi(2);
                }

                if ss_total.abs() < 1e-10 {
                    return Err(MathError::CalculationError(
                        "Cannot calculate R-squared: total sum of squares is too small".to_string(),
                    ));
                }

                Ok(1.0 - (ss_residual / ss_total))
            }
            _ => Err(MathError::CalculationError(
                "Regression parameters not calculated".to_string(),
            )),
        }
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Reset the Linear Regression, clearing all values
    pub fn reset(&mut self) {
        self.values.clear();
        self.slope = None;
        self.intercept = None;
    }
}

/// Exponential Smoothing implementation
#[derive(Debug, Clone)]
pub struct ExponentialSmoothing {
    alpha: f64,
    level: Option<f64>,
    values_seen: usize,
}

impl ExponentialSmoothing {
    /// Create a new Exponential Smoothing with the specified alpha (smoothing factor)
    pub fn new(alpha: f64) -> Result<Self> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(MathError::InvalidInput(
                "Alpha must be between 0 and 1 (exclusive)".to_string(),
            ));
        }

        Ok(Self {
            alpha,
            level: None,
            values_seen: 0,
        })
    }

    /// Update the Exponential Smoothing with a new value
    pub fn update(&mut self, value: f64) -> Result<()> {
        self.values_seen += 1;

        match self.level {
            None => {
                // First value, just use it as initial level
                self.level = Some(value);
            }
            Some(current_level) => {
                // Calculate new level: level = alpha * value + (1 - alpha) * previous_level
                let new_level = self.alpha * value + (1.0 - self.alpha) * current_level;
                self.level = Some(new_level);
            }
        }

        Ok(())
    }

    /// Get the current smoothed value
    pub fn value(&self) -> Result<f64> {
        match self.level {
            Some(level) => Ok(level),
            None => Err(MathError::InsufficientData(
                "No data available for exponential smoothing".to_string(),
            )),
        }
    }

    /// Forecast the next value (in simple exponential smoothing, the forecast equals the last level)
    pub fn forecast(&self) -> Result<f64> {
        self.value()
    }

    /// Get the current alpha value
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Reset the Exponential Smoothing, clearing all values
    pub fn reset(&mut self) {
        self.level = None;
        self.values_seen = 0;
    }
}

/// Double Exponential Smoothing (Holt's Method) implementation
#[derive(Debug, Clone)]
pub struct DoubleExponentialSmoothing {
    alpha: f64,
    beta: f64,
    level: Option<f64>,
    trend: Option<f64>,
    values_seen: usize,
}

impl DoubleExponentialSmoothing {
    /// Create a new Double Exponential Smoothing with the specified parameters
    pub fn new(alpha: f64, beta: f64) -> Result<Self> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(MathError::InvalidInput(
                "Alpha must be between 0 and 1 (exclusive)".to_string(),
            ));
        }
        if beta <= 0.0 || beta >= 1.0 {
            return Err(MathError::InvalidInput(
                "Beta must be between 0 and 1 (exclusive)".to_string(),
            ));
        }

        Ok(Self {
            alpha,
            beta,
            level: None,
            trend: None,
            values_seen: 0,
        })
    }

    /// Update the Double Exponential Smoothing with a new value
    pub fn update(&mut self, value: f64) -> Result<()> {
        self.values_seen += 1;

        match (self.level, self.trend) {
            (None, None) => {
                // First value, just use it as initial level and zero trend
                self.level = Some(value);
                self.trend = Some(0.0);
            }
            (Some(prev_level), Some(prev_trend)) => {
                // Calculate new level and trend
                let new_level = self.alpha * value + (1.0 - self.alpha) * (prev_level + prev_trend);
                let new_trend =
                    self.beta * (new_level - prev_level) + (1.0 - self.beta) * prev_trend;

                self.level = Some(new_level);
                self.trend = Some(new_trend);
            }
            _ => {
                return Err(MathError::CalculationError(
                    "Inconsistent state: level and trend should both be Some or None".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get the current smoothed value
    pub fn value(&self) -> Result<f64> {
        match self.level {
            Some(level) => Ok(level),
            None => Err(MathError::InsufficientData(
                "No data available for double exponential smoothing".to_string(),
            )),
        }
    }

    /// Forecast h steps ahead
    pub fn forecast(&self, h: usize) -> Result<f64> {
        match (self.level, self.trend) {
            (Some(level), Some(trend)) => Ok(level + (h as f64) * trend),
            _ => Err(MathError::InsufficientData(
                "Not enough data to make a forecast".to_string(),
            )),
        }
    }

    /// Get the current level
    pub fn level(&self) -> Result<f64> {
        match self.level {
            Some(level) => Ok(level),
            None => Err(MathError::InsufficientData(
                "Level not calculated yet".to_string(),
            )),
        }
    }

    /// Get the current trend
    pub fn trend(&self) -> Result<f64> {
        match self.trend {
            Some(trend) => Ok(trend),
            None => Err(MathError::InsufficientData(
                "Trend not calculated yet".to_string(),
            )),
        }
    }

    /// Reset the Double Exponential Smoothing, clearing all values
    pub fn reset(&mut self) {
        self.level = None;
        self.trend = None;
        self.values_seen = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression() {
        let mut lr = LinearRegression::new(3).unwrap();

        // Add test data with perfect linear relationship
        lr.update(10.0).unwrap();
        lr.update(20.0).unwrap();
        lr.update(30.0).unwrap();

        // Test slope (should be 10)
        assert!((lr.slope().unwrap() - 10.0).abs() < 0.001);

        // Test forecast
        let forecast = lr.forecast(1).unwrap();
        assert!((forecast - 40.0).abs() < 0.001);

        // Test R-squared (should be 1.0 for perfect linear data)
        assert!(lr.r_squared().unwrap() > 0.999);
    }

    #[test]
    fn test_exponential_smoothing() {
        let mut es = ExponentialSmoothing::new(0.3).unwrap();

        // Add test data
        es.update(10.0).unwrap(); // Initial level = 10
        assert!((es.value().unwrap() - 10.0).abs() < 0.001);

        es.update(20.0).unwrap(); // New level = 0.3*20 + 0.7*10 = 13
        assert!((es.value().unwrap() - 13.0).abs() < 0.001);

        // Test forecast (should equal current level)
        assert!((es.forecast().unwrap() - 13.0).abs() < 0.001);
    }

    #[test]
    fn test_double_exponential_smoothing() {
        let mut des = DoubleExponentialSmoothing::new(0.4, 0.3).unwrap();

        // Add test data
        des.update(10.0).unwrap(); // Initial level = 10, trend = 0
        des.update(20.0).unwrap();
        des.update(30.0).unwrap();

        // Get current level and trend
        let level = des.level().unwrap();
        let trend = des.trend().unwrap();

        assert!(level > 20.0); // Level should be increasing
        assert!(trend > 0.0); // Trend should be positive

        // Test forecast 2 steps ahead
        let forecast = des.forecast(2).unwrap();
        assert!(forecast > level); // Forecast should be higher than current level
    }
}
